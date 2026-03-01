pub mod anthropic;
pub mod caldav;
pub mod carddav;
pub mod ical;
pub mod imap;
pub mod keyring;
pub mod merge;
pub mod vevent;
pub mod vtodo;
pub mod webdav;

use std::collections::HashMap;
use uuid::Uuid;

use crate::core::event::CalendarEvent;
use crate::core::task::Task;
use caldav::{CalDavClient, PutCondition, SyncChange};
use vevent::{event_content_hash, event_to_vcalendar, vcalendar_to_events};
use vtodo::{task_content_hash, task_to_vcalendar, vcalendar_to_task};

/// A conflict detected during sync that needs user resolution.
#[derive(Debug, Clone)]
pub enum SyncConflict {
    StateMismatch {
        task_id: Uuid,
        title: String,
        href: String,
        local_state: String,
        remote_state: String,
    },
    RemoteOnly {
        task: Task,
        href: String,
    },
    LocalOnly {
        task_id: Uuid,
        title: String,
        local_state: String,
        href: String,
    },
}

/// Current sync status displayed in the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error(String),
    LastSynced(String), // formatted timestamp
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Result of a sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Tasks pulled from remote (new or updated).
    pub pulled: Vec<Task>,
    /// Number of tasks pushed to remote.
    pub pushed: usize,
    /// UUIDs of tasks deleted on remote.
    pub deleted_local: Vec<Uuid>,
    /// Number of field-level merges performed.
    pub merged: usize,
    /// Events pulled from remote (new or updated).
    pub pulled_events: Vec<CalendarEvent>,
    /// Number of events pushed to remote.
    pub pushed_events: usize,
    /// UUIDs of events deleted on remote.
    pub deleted_events: Vec<Uuid>,
    /// Updated sync tokens: (calendar_href, token).
    pub new_sync_tokens: Vec<(String, String)>,
    /// Non-fatal errors encountered during sync.
    pub errors: Vec<String>,
    /// Conflicts detected during sync that need user resolution.
    pub conflicts: Vec<SyncConflict>,
}

/// Performs bidirectional sync between local tasks/events and a CalDAV server.
pub struct SyncEngine {
    client: CalDavClient,
    calendar_href: String,
}

impl SyncEngine {
    pub fn new(client: CalDavClient, calendar_href: String) -> Self {
        Self {
            client,
            calendar_href,
        }
    }

    /// Run a full bidirectional sync of tasks.
    pub async fn sync_tasks(
        &self,
        tasks: &[Task],
        sync_token: Option<&str>,
    ) -> Result<SyncResult, String> {
        let mut result = new_sync_result();

        log::info!(
            "Starting task sync with {} local tasks, calendar: {}",
            tasks.len(),
            self.calendar_href
        );

        // Build a map of local tasks by UUID for fast lookup
        let mut local_by_id: HashMap<Uuid, &Task> = HashMap::new();
        for task in tasks {
            local_by_id.insert(task.id, task);
        }

        // Step 1: Get remote changes
        let (changes, new_token) = match self
            .client
            .sync_collection(&self.calendar_href, sync_token)
            .await
        {
            Ok(r) => r,
            Err(e) if e == "sync-token-expired" => {
                log::info!("Sync token expired, performing full listing");
                return self.full_sync_tasks(tasks).await;
            }
            Err(e) => return Err(e),
        };

        if let Some(t) = new_token {
            result
                .new_sync_tokens
                .push((self.calendar_href.clone(), t));
        }

        log::info!("Got {} remote changes", changes.len());

        // Track which remote hrefs we've seen in changes
        let mut seen_hrefs: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Step 2: Process remote changes
        for change in &changes {
            match change {
                SyncChange::Changed(remote_vtodo) => {
                    seen_hrefs.insert(remote_vtodo.href.clone());

                    let remote_task = match vcalendar_to_task(&remote_vtodo.ical_body) {
                        Some(t) => t,
                        None => {
                            log::debug!("Skipping unparseable VTODO: {}", remote_vtodo.href);
                            continue;
                        }
                    };

                    // Skip completed/cancelled tasks from remote
                    if remote_task.state.is_done() {
                        log::debug!("Skipping completed remote task: {}", remote_task.title);
                        continue;
                    }

                    if let Some(&local_task) = local_by_id.get(&remote_task.id) {
                        let local_hash = task_content_hash(local_task);
                        let local_changed = local_task
                            .sync_hash
                            .is_some_and(|h| h != local_hash);

                        if local_changed {
                            let merged =
                                merge::merge_tasks(local_task, &remote_task, &remote_task);
                            let mut pulled = merged;
                            pulled.sync_href = Some(remote_vtodo.href.clone());
                            pulled.sync_hash = Some(task_content_hash(&pulled));
                            result.pulled.push(pulled);
                            result.merged += 1;
                            log::info!("Merged: {}", remote_task.title);
                        } else {
                            let mut pulled = remote_task;
                            pulled.sync_href = Some(remote_vtodo.href.clone());
                            pulled.sync_hash = Some(task_content_hash(&pulled));
                            pulled.project = local_task.project.clone();
                            result.pulled.push(pulled);
                        }
                    } else {
                        log::info!("Importing new remote task: {}", remote_task.title);
                        let mut pulled = remote_task;
                        pulled.sync_href = Some(remote_vtodo.href.clone());
                        pulled.sync_hash = Some(task_content_hash(&pulled));
                        result.pulled.push(pulled);
                    }
                }
                SyncChange::Deleted(href) => {
                    seen_hrefs.insert(href.clone());
                    for task in tasks {
                        if task.sync_href.as_deref() == Some(href.as_str()) {
                            log::info!("Remote deleted: {}", task.title);
                            result.deleted_local.push(task.id);
                        }
                    }
                }
            }
        }

        // Step 3: Push local changes to remote
        self.push_local_changes(tasks, &seen_hrefs, &mut result).await;

        log::info!(
            "Task sync complete: {} pulled, {} pushed, {} deleted, {} merged",
            result.pulled.len(),
            result.pushed,
            result.deleted_local.len(),
            result.merged,
        );

        Ok(result)
    }

    /// Full sync without a token — lists all remote VTODOs and reconciles.
    async fn full_sync_tasks(
        &self,
        tasks: &[Task],
    ) -> Result<SyncResult, String> {
        let mut result = new_sync_result();

        let remote_vtodos = self.client.list_vtodos(&self.calendar_href).await?;
        log::info!("Full sync: {} remote VTODOs", remote_vtodos.len());

        let mut local_by_id: HashMap<Uuid, &Task> = HashMap::new();
        for task in tasks {
            local_by_id.insert(task.id, task);
        }

        let mut matched_local: std::collections::HashSet<Uuid> =
            std::collections::HashSet::new();

        for remote_vtodo in &remote_vtodos {
            let remote_task = match vcalendar_to_task(&remote_vtodo.ical_body) {
                Some(t) => t,
                None => {
                    log::debug!("Skipping unparseable VTODO: {}", remote_vtodo.href);
                    continue;
                }
            };

            // Skip completed/cancelled tasks from remote — we don't import done items
            if remote_task.state.is_done() {
                log::debug!("Skipping completed remote task: {}", remote_task.title);
                matched_local.insert(remote_task.id);
                continue;
            }

            matched_local.insert(remote_task.id);

            if let Some(&local_task) = local_by_id.get(&remote_task.id) {
                let local_hash = task_content_hash(local_task);
                let remote_hash = task_content_hash(&remote_task);

                if local_task.sync_hash.is_none() || local_hash != remote_hash {
                    let mut pulled = remote_task;
                    pulled.sync_href = Some(remote_vtodo.href.clone());
                    pulled.sync_hash = Some(task_content_hash(&pulled));
                    pulled.project = local_task.project.clone();
                    result.pulled.push(pulled);
                }
            } else {
                log::info!("Importing new remote task: {}", remote_task.title);
                let mut pulled = remote_task;
                pulled.sync_href = Some(remote_vtodo.href.clone());
                pulled.sync_hash = Some(task_content_hash(&pulled));
                result.pulled.push(pulled);
            }
        }

        // Push local tasks that aren't on remote
        for task in tasks {
            if task.state.is_done() {
                continue;
            }
            if !matched_local.contains(&task.id) {
                let href = task.sync_href.clone().unwrap_or_else(|| {
                    caldav::vtodo_href(&self.calendar_href, &task.id)
                });
                let ical = task_to_vcalendar(task);
                log::info!("Pushing to remote: {} -> {}", task.title, href);
                match self.client.put_vtodo(&href, PutCondition::Unconditional, &ical).await {
                    Ok(_) => {
                        let mut updated = task.clone();
                        updated.sync_href = Some(href);
                        updated.sync_hash = Some(task_content_hash(task));
                        result.pulled.push(updated);
                        result.pushed += 1;
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("Failed to push {}: {}", task.title, e));
                    }
                }
            }
        }

        // Try to get a sync token for next time
        if let Ok((_, token)) = self
            .client
            .sync_collection(&self.calendar_href, None)
            .await
        {
            if let Some(t) = token {
                result
                    .new_sync_tokens
                    .push((self.calendar_href.clone(), t));
            }
        }

        log::info!(
            "Full task sync complete: {} pulled, {} pushed, {} errors",
            result.pulled.len(),
            result.pushed,
            result.errors.len()
        );

        Ok(result)
    }

    /// Sync events from a single calendar (read-write).
    pub async fn sync_events(
        &self,
        local_events: &[CalendarEvent],
        sync_token: Option<&str>,
    ) -> Result<SyncResult, String> {
        let mut result = new_sync_result();
        let cal_href = &self.calendar_href;

        log::info!("Syncing events from calendar: {}", cal_href);

        let local_for_cal: Vec<&CalendarEvent> = local_events
            .iter()
            .filter(|e| e.calendar_href == *cal_href)
            .collect();

        match self.client.list_vevents(cal_href).await {
            Ok(remote_vevents) => {
                log::info!("Got {} remote events from {}", remote_vevents.len(), cal_href);

                let mut local_by_id: HashMap<Uuid, &CalendarEvent> = HashMap::new();
                for event in &local_for_cal {
                    local_by_id.insert(event.id, event);
                }

                let mut matched_local: std::collections::HashSet<Uuid> =
                    std::collections::HashSet::new();

                for remote in &remote_vevents {
                    let remote_instances = vcalendar_to_events(&remote.ical_body);
                    if remote_instances.is_empty() {
                        continue;
                    }

                    for mut remote_event in remote_instances {
                        remote_event.calendar_href = cal_href.clone();
                        if remote_event.sync_href.is_none() {
                            // This is an expanded recurrence instance — no sync_href
                        } else {
                            remote_event.sync_href = Some(remote.href.clone());
                        }
                        remote_event.sync_hash = Some(event_content_hash(&remote_event));

                        matched_local.insert(remote_event.id);

                        if let Some(&local) = local_by_id.get(&remote_event.id) {
                            let local_hash = event_content_hash(local);
                            let remote_hash = event_content_hash(&remote_event);
                            if local.sync_hash.is_none() || local_hash != remote_hash {
                                remote_event.calendar_name = local.calendar_name.clone();
                                result.pulled_events.push(remote_event);
                            }
                        } else {
                            result.pulled_events.push(remote_event);
                        }
                    }
                }

                // Push local events that aren't on remote
                for local_event in &local_for_cal {
                    if matched_local.contains(&local_event.id) {
                        continue;
                    }
                    let local_hash = event_content_hash(local_event);
                    let changed = local_event.sync_hash.is_some_and(|h| h != local_hash);

                    if local_event.sync_href.is_none() || changed {
                        let href = local_event.sync_href.clone().unwrap_or_else(|| {
                            caldav::vevent_href(cal_href, &local_event.id)
                        });
                        let ical = event_to_vcalendar(local_event);
                        match self.client.put_vtodo(&href, PutCondition::Unconditional, &ical).await {
                            Ok(_) => {
                                let mut updated = (*local_event).clone();
                                updated.sync_href = Some(href);
                                updated.sync_hash = Some(local_hash);
                                result.pulled_events.push(updated);
                                result.pushed_events += 1;
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to push event {}: {}",
                                    local_event.title, e
                                ));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to list events from {}: {}", cal_href, e));
            }
        }

        // Try to get a sync token for this calendar
        if let Ok((_, new_token)) = self.client.sync_collection(cal_href, sync_token).await {
            if let Some(t) = new_token {
                result
                    .new_sync_tokens
                    .push((cal_href.clone(), t));
            }
        }

        Ok(result)
    }

    /// Push locally-changed and new tasks to the remote server.
    async fn push_local_changes(
        &self,
        tasks: &[Task],
        seen_hrefs: &std::collections::HashSet<String>,
        result: &mut SyncResult,
    ) {
        for task in tasks {
            if task.state.is_done() {
                if let Some(ref href) = task.sync_href {
                    if !seen_hrefs.contains(href) {
                        if let Err(e) = self.client.delete_vtodo(href, "").await {
                            result
                                .errors
                                .push(format!("Failed to delete done task: {}", e));
                        }
                    }
                }
                continue;
            }

            if let Some(ref href) = task.sync_href {
                if !seen_hrefs.contains(href) {
                    let local_hash = task_content_hash(task);
                    let changed = task
                        .sync_hash
                        .is_some_and(|h| h != local_hash);

                    if changed {
                        let ical = task_to_vcalendar(task);
                        log::info!("Pushing update: {}", task.title);
                        match self.client.put_vtodo(href, PutCondition::Unconditional, &ical).await {
                            Ok(_) => {
                                let mut updated = task.clone();
                                updated.sync_hash = Some(local_hash);
                                result.pulled.push(updated);
                                result.pushed += 1;
                            }
                            Err(e) => {
                                result
                                    .errors
                                    .push(format!("Failed to push {}: {}", task.title, e));
                            }
                        }
                    }
                }
            } else {
                let href = caldav::vtodo_href(&self.calendar_href, &task.id);
                let ical = task_to_vcalendar(task);
                log::info!("Creating remote: {} -> {}", task.title, href);
                match self.client.put_vtodo(&href, PutCondition::CreateOnly, &ical).await {
                    Ok(_) => {
                        let mut updated = task.clone();
                        updated.sync_href = Some(href);
                        updated.sync_hash = Some(task_content_hash(task));
                        result.pulled.push(updated);
                        result.pushed += 1;
                    }
                    Err(ref e) if e.contains("412") || e.contains("403") => {
                        log::info!("Already exists, updating: {}", task.title);
                        match self.client.put_vtodo(&href, PutCondition::Unconditional, &ical).await {
                            Ok(_) => {
                                let mut updated = task.clone();
                                updated.sync_href = Some(href);
                                updated.sync_hash = Some(task_content_hash(task));
                                result.pulled.push(updated);
                                result.pushed += 1;
                            }
                            Err(e) => {
                                result
                                    .errors
                                    .push(format!("Failed to update {}: {}", task.title, e));
                            }
                        }
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("Failed to create {}: {}", task.title, e));
                    }
                }
            }
        }
    }
}

/// Flat sync orchestrator — single CalDAV account.
/// Creates one CalDavClient, syncs tasks from Task calendars
/// and events from Event calendars, merging results.
pub async fn sync_all(
    caldav_url: &str,
    username: &str,
    password: &str,
    tasks: &[Task],
    events: &[CalendarEvent],
    task_cals: &[String],
    event_cals: &[String],
    sync_tokens: &[(String, String)],
    pending_completions: &[(String, String)],
    pending_deletions: &[String],
) -> Result<SyncResult, String> {
    let mut merged_result = new_sync_result();

    let client = CalDavClient::new(caldav_url, username, password)?;

    // Push pending completions (tasks completed locally since last sync)
    for (href, ical) in pending_completions {
        log::info!("Pushing pending completion: {}", href);
        match client
            .put_vtodo(href, PutCondition::Unconditional, ical)
            .await
        {
            Ok(_) => {
                log::info!("Completion pushed successfully: {}", href);
                merged_result.pushed += 1;
            }
            Err(e) => {
                log::error!("Failed to push completion {}: {}", href, e);
                merged_result.errors.push(format!(
                    "Failed to push completion {}: {}",
                    href, e
                ));
            }
        }
    }

    // Process pending deletions (remote-only conflicts the user chose to delete)
    for href in pending_deletions {
        log::info!("Deleting remote VTODO: {}", href);
        match client.delete_vtodo(href, "").await {
            Ok(_) => log::info!("Deleted successfully: {}", href),
            Err(e) => {
                merged_result
                    .errors
                    .push(format!("Failed to delete {}: {}", href, e));
            }
        }
    }

    // Sync tasks from each task calendar
    for cal_href in task_cals {
        let token = sync_tokens
            .iter()
            .find(|(h, _)| h == cal_href)
            .map(|(_, t)| t.as_str());

        let engine = SyncEngine::new(client.clone(), cal_href.clone());
        match engine.sync_tasks(tasks, token).await {
            Ok(res) => {
                merged_result.pulled.extend(res.pulled);
                merged_result.pushed += res.pushed;
                merged_result.deleted_local.extend(res.deleted_local);
                merged_result.merged += res.merged;
                merged_result.new_sync_tokens.extend(res.new_sync_tokens);
                merged_result.errors.extend(res.errors);
            }
            Err(e) => {
                merged_result
                    .errors
                    .push(format!("Task sync error (cal {}): {}", cal_href, e));
            }
        }
    }

    // Build post-sync task list: start with originals, apply pulled/deleted
    let mut effective_tasks: Vec<Task> = tasks.to_vec();
    for id in &merged_result.deleted_local {
        effective_tasks.retain(|t| t.id != *id);
    }
    for pulled in &merged_result.pulled {
        effective_tasks.retain(|t| t.id != pulled.id);
        effective_tasks.push(pulled.clone());
    }

    // Conflict detection: compare post-sync local vs remote for each task calendar
    for cal_href in task_cals {
        let remote_vtodos = match client.list_vtodos(cal_href).await {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Conflict detection: failed to list VTODOs from {}: {}", cal_href, e);
                continue;
            }
        };

        // Build local tasks indexed by sync_href for this calendar
        let mut local_by_href: HashMap<String, &Task> = HashMap::new();
        for task in &effective_tasks {
            if let Some(ref href) = task.sync_href {
                if href.contains(cal_href.as_str()) {
                    local_by_href.insert(href.clone(), task);
                }
            }
        }

        // Build set of remote hrefs
        let mut remote_hrefs: std::collections::HashSet<String> = std::collections::HashSet::new();

        for remote_vtodo in &remote_vtodos {
            remote_hrefs.insert(remote_vtodo.href.clone());

            let remote_task = match vcalendar_to_task(&remote_vtodo.ical_body) {
                Some(t) => t,
                None => continue,
            };

            // Skip completed/cancelled remote tasks
            if remote_task.state.is_done() {
                continue;
            }

            if let Some(&local_task) = local_by_href.get(&remote_vtodo.href) {
                // Both exist — compare states
                let local_kw = local_task.state.as_keyword();
                let remote_kw = remote_task.state.as_keyword();
                if local_kw != remote_kw {
                    merged_result.conflicts.push(SyncConflict::StateMismatch {
                        task_id: local_task.id,
                        title: local_task.title.clone(),
                        href: remote_vtodo.href.clone(),
                        local_state: local_kw.to_string(),
                        remote_state: remote_kw.to_string(),
                    });
                }
            } else {
                // Remote-only
                let mut task = remote_task;
                task.sync_href = Some(remote_vtodo.href.clone());
                merged_result.conflicts.push(SyncConflict::RemoteOnly {
                    task,
                    href: remote_vtodo.href.clone(),
                });
            }
        }

        // Local-only: local tasks with sync_href in this calendar but not found on remote
        for (href, local_task) in &local_by_href {
            if !remote_hrefs.contains(href.as_str()) && !local_task.state.is_done() {
                merged_result.conflicts.push(SyncConflict::LocalOnly {
                    task_id: local_task.id,
                    title: local_task.title.clone(),
                    local_state: local_task.state.as_keyword().to_string(),
                    href: href.clone(),
                });
            }
        }
    }

    for conflict in &merged_result.conflicts {
        match conflict {
            SyncConflict::StateMismatch { title, local_state, remote_state, href, .. } => {
                log::info!("Conflict: StateMismatch '{}' local={} remote={} href={}", title, local_state, remote_state, href);
            }
            SyncConflict::RemoteOnly { task, href } => {
                log::info!("Conflict: RemoteOnly '{}' state={} href={}", task.title, task.state.as_keyword(), href);
            }
            SyncConflict::LocalOnly { title, local_state, href, .. } => {
                log::info!("Conflict: LocalOnly '{}' state={} href={}", title, local_state, href);
            }
        }
    }
    log::info!("Conflict detection: {} conflicts found", merged_result.conflicts.len());

    // Sync events from each event calendar
    for cal_href in event_cals {
        let token = sync_tokens
            .iter()
            .find(|(h, _)| h == cal_href)
            .map(|(_, t)| t.as_str());

        let engine = SyncEngine::new(client.clone(), cal_href.clone());
        match engine.sync_events(events, token).await {
            Ok(res) => {
                merged_result.pulled_events.extend(res.pulled_events);
                merged_result.pushed_events += res.pushed_events;
                merged_result.deleted_events.extend(res.deleted_events);
                merged_result.new_sync_tokens.extend(res.new_sync_tokens);
                merged_result.errors.extend(res.errors);
            }
            Err(e) => {
                merged_result
                    .errors
                    .push(format!("Event sync error (cal {}): {}", cal_href, e));
            }
        }
    }

    log::info!(
        "Sync complete: {} pulled tasks, {} pushed, {} pulled events, {} errors",
        merged_result.pulled.len(),
        merged_result.pushed,
        merged_result.pulled_events.len(),
        merged_result.errors.len()
    );

    Ok(merged_result)
}

fn new_sync_result() -> SyncResult {
    SyncResult {
        pulled: Vec::new(),
        pushed: 0,
        deleted_local: Vec::new(),
        merged: 0,
        pulled_events: Vec::new(),
        pushed_events: 0,
        deleted_events: Vec::new(),
        new_sync_tokens: Vec::new(),
        errors: Vec::new(),
        conflicts: Vec::new(),
    }
}
