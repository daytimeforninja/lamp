use std::hash::Hasher;

use chrono::{NaiveDate, NaiveDateTime};
use uuid::Uuid;

use super::ical::*;
use crate::core::recurrence::Recurrence;
use crate::core::task::{Priority, Task, TaskState};

/// Generate a full VCALENDAR string containing a VTODO for the given task.
pub fn task_to_vcalendar(task: &Task) -> String {
    let mut lines = Vec::new();
    lines.push("BEGIN:VCALENDAR".to_string());
    lines.push("VERSION:2.0".to_string());
    lines.push("PRODID:-//Lamp GTD//EN".to_string());
    lines.push("BEGIN:VTODO".to_string());

    // UID — use original CalDAV UID if available (preserves case).
    // Fall back to extracting from sync_href filename (e.g. .../ABCD-1234.ics -> ABCD-1234).
    let id_string = task.id.to_string();
    let uid_str = task.sync_uid.as_deref().unwrap_or_else(|| {
        task.sync_href.as_deref()
            .and_then(|href| href.rsplit('/').next())
            .and_then(|f| f.strip_suffix(".ics"))
            .unwrap_or(&id_string)
    });
    lines.push(format!("UID:{}", uid_str));

    // SUMMARY
    lines.push(format!("SUMMARY:{}", fold_line(&escape_text(&task.title))));

    // STATUS
    let status = match task.state {
        TaskState::Done => "COMPLETED",
        TaskState::Cancelled => "CANCELLED",
        _ => "NEEDS-ACTION",
    };
    lines.push(format!("STATUS:{}", status));

    // PERCENT-COMPLETE
    if task.state.is_done() {
        lines.push("PERCENT-COMPLETE:100".to_string());
    }

    // PRIORITY: A→1, B→5, C→9, None→0
    let priority = match task.priority {
        Some(Priority::A) => 1,
        Some(Priority::B) => 5,
        Some(Priority::C) => 9,
        None => 0,
    };
    lines.push(format!("PRIORITY:{}", priority));

    // CATEGORIES
    if !task.contexts.is_empty() {
        lines.push(format!("CATEGORIES:{}", task.contexts.join(",")));
    }

    // DTSTART (scheduled)
    if let Some(scheduled) = task.scheduled {
        lines.push(format!("DTSTART;VALUE=DATE:{}", format_date(scheduled)));
    }

    // DUE (deadline)
    if let Some(deadline) = task.deadline {
        lines.push(format!("DUE;VALUE=DATE:{}", format_date(deadline)));
    }

    // DESCRIPTION (notes)
    if !task.notes.is_empty() {
        lines.push(format!(
            "DESCRIPTION:{}",
            fold_line(&escape_text(&task.notes))
        ));
    }

    // CREATED
    lines.push(format!("CREATED:{}", format_datetime(task.created)));

    // COMPLETED
    if let Some(completed) = task.completed {
        lines.push(format!("COMPLETED:{}", format_datetime(completed)));
    }

    // LAST-MODIFIED
    lines.push(format!(
        "LAST-MODIFIED:{}",
        format_datetime(chrono::Local::now().naive_local())
    ));

    // X-LAMP-STATE (GTD-specific state)
    lines.push(format!("X-LAMP-STATE:{}", task.state.as_keyword()));

    // X-LAMP-PROJECT
    if let Some(ref project) = task.project {
        lines.push(format!("X-LAMP-PROJECT:{}", escape_text(project)));
    }

    // X-LAMP-WAITING-FOR
    if let Some(ref wf) = task.waiting_for {
        lines.push(format!("X-LAMP-WAITING-FOR:{}", escape_text(wf)));
    }

    // X-LAMP-ESC
    if let Some(esc) = task.esc {
        lines.push(format!("X-LAMP-ESC:{}", esc));
    }

    // X-LAMP-DELEGATED
    if let Some(delegated) = task.delegated {
        lines.push(format!("X-LAMP-DELEGATED:{}", format_date(delegated)));
    }

    // X-LAMP-FOLLOW-UP
    if let Some(follow_up) = task.follow_up {
        lines.push(format!("X-LAMP-FOLLOW-UP:{}", format_date(follow_up)));
    }

    // X-LAMP-RECURRENCE (org-mode format)
    if let Some(ref recurrence) = task.recurrence {
        lines.push(format!("X-LAMP-RECURRENCE:{}", recurrence));
    }

    // X-LAMP-TAGS (non-context tags like "habit")
    if !task.extra_tags.is_empty() {
        lines.push(format!(
            "X-LAMP-TAGS:{}",
            task.extra_tags.iter().map(|t| escape_text(t)).collect::<Vec<_>>().join(",")
        ));
    }

    // X-LAMP-DAYPLAN (day plan date, optionally with budget)
    if let Some(dayplan) = task.dayplan_date {
        if let Some(budget) = task.dayplan_budget {
            lines.push(format!("X-LAMP-DAYPLAN:{};BUDGET={}", format_date(dayplan), budget));
        } else {
            lines.push(format!("X-LAMP-DAYPLAN:{}", format_date(dayplan)));
        }
    }

    // X-LAMP-LOGBOOK (habit completion timestamps)
    if !task.logbook_entries.is_empty() {
        let entries: Vec<String> = task
            .logbook_entries
            .iter()
            .map(|dt| format_datetime(*dt))
            .collect();
        lines.push(format!("X-LAMP-LOGBOOK:{}", entries.join(",")));
    }

    // X-LAMP-CLOCK (work timer sessions: start/end pairs)
    if !task.clock_entries.is_empty() {
        let entries: Vec<String> = task
            .clock_entries
            .iter()
            .map(|(s, e)| format!("{}/{}", format_datetime(*s), format_datetime(*e)))
            .collect();
        lines.push(format!("X-LAMP-CLOCK:{}", entries.join(",")));
    }

    lines.push("END:VTODO".to_string());
    lines.push("END:VCALENDAR".to_string());

    lines.join("\r\n") + "\r\n"
}

/// Parse a VCALENDAR string and extract a Task from the VTODO component.
pub fn vcalendar_to_task(ical: &str) -> Option<Task> {
    // Unfold lines (RFC 5545: lines starting with space/tab are continuations)
    let unfolded = unfold_lines(ical);

    let mut in_vtodo = false;
    let mut uid: Option<Uuid> = None;
    let mut uid_raw: Option<String> = None;
    let mut summary = String::new();
    let mut status: Option<String> = None;
    let mut priority_val: Option<i32> = None;
    let mut categories: Vec<String> = Vec::new();
    let mut dtstart: Option<NaiveDate> = None;
    let mut due: Option<NaiveDate> = None;
    let mut description = String::new();
    let mut created: Option<NaiveDateTime> = None;
    let mut completed: Option<NaiveDateTime> = None;

    // X-LAMP properties
    let mut lamp_state: Option<String> = None;
    let mut lamp_project: Option<String> = None;
    let mut lamp_waiting_for: Option<String> = None;
    let mut lamp_esc: Option<u32> = None;
    let mut lamp_delegated: Option<NaiveDate> = None;
    let mut lamp_follow_up: Option<NaiveDate> = None;
    let mut lamp_recurrence: Option<String> = None;
    let mut lamp_logbook: Option<String> = None;
    let mut lamp_clock: Option<String> = None;
    let mut lamp_tags: Option<String> = None;
    let mut lamp_dayplan: Option<NaiveDate> = None;
    let mut lamp_dayplan_budget: Option<u32> = None;

    for line in unfolded.lines() {
        let line = line.trim_end();
        if line == "BEGIN:VTODO" {
            in_vtodo = true;
            continue;
        }
        if line == "END:VTODO" {
            break;
        }
        if !in_vtodo {
            continue;
        }

        if let Some((key, value)) = parse_ical_line(line) {
            match key {
                "UID" => {
                    uid_raw = Some(value.to_string());
                    uid = Some(
                        Uuid::parse_str(value)
                            .unwrap_or_else(|_| Uuid::new_v5(&CALDAV_UUID_NAMESPACE, value.as_bytes())),
                    );
                }
                "SUMMARY" => summary = unescape_text(value),
                "STATUS" => status = Some(value.to_string()),
                "PRIORITY" => priority_val = value.parse().ok(),
                "CATEGORIES" => {
                    categories = value.split(',').map(|s| s.trim().to_string()).collect();
                }
                "DTSTART" => dtstart = parse_ical_date(value),
                "DUE" => due = parse_ical_date(value),
                "DESCRIPTION" => description = unescape_text(value),
                "CREATED" => created = parse_ical_datetime(value),
                "COMPLETED" => completed = parse_ical_datetime(value),
                "X-LAMP-STATE" => lamp_state = Some(value.to_string()),
                "X-LAMP-PROJECT" => lamp_project = Some(unescape_text(value)),
                "X-LAMP-WAITING-FOR" => lamp_waiting_for = Some(unescape_text(value)),
                "X-LAMP-ESC" => lamp_esc = value.parse().ok(),
                "X-LAMP-DELEGATED" => lamp_delegated = parse_ical_date(value),
                "X-LAMP-FOLLOW-UP" => lamp_follow_up = parse_ical_date(value),
                "X-LAMP-RECURRENCE" => lamp_recurrence = Some(value.to_string()),
                "X-LAMP-LOGBOOK" => lamp_logbook = Some(value.to_string()),
                "X-LAMP-CLOCK" => lamp_clock = Some(value.to_string()),
                "X-LAMP-TAGS" => lamp_tags = Some(value.to_string()),
                "X-LAMP-DAYPLAN" => {
                    // Format: "20260324" or "20260324;BUDGET=50"
                    if let Some((date_part, params)) = value.split_once(';') {
                        lamp_dayplan = parse_ical_date(date_part);
                        if let Some(budget_str) = params.strip_prefix("BUDGET=") {
                            lamp_dayplan_budget = budget_str.parse().ok();
                        }
                    } else {
                        lamp_dayplan = parse_ical_date(value);
                    }
                }
                _ => {}
            }
        }
    }

    // Skip VTODOs with no title — they're not useful tasks
    if summary.is_empty() {
        return None;
    }

    let id = uid.unwrap_or_else(Uuid::new_v4);

    // Determine state: prefer X-LAMP-STATE, fall back to STATUS
    let state = lamp_state
        .as_deref()
        .and_then(TaskState::from_keyword)
        .unwrap_or_else(|| match status.as_deref() {
            Some("COMPLETED") => TaskState::Done,
            Some("CANCELLED") => TaskState::Cancelled,
            _ => TaskState::Todo,
        });

    // Map priority: 1→A, 2-5→B, 6-9→C, 0→None
    let priority = match priority_val {
        Some(1) => Some(Priority::A),
        Some(2..=5) => Some(Priority::B),
        Some(6..=9) => Some(Priority::C),
        _ => None,
    };

    let recurrence = lamp_recurrence.as_deref().and_then(Recurrence::parse);

    Some(Task {
        id,
        title: summary,
        state,
        priority,
        contexts: categories,
        scheduled: dtstart,
        deadline: due,
        recurrence,
        notes: description,
        created: created.unwrap_or_else(|| chrono::Local::now().naive_local()),
        completed,
        project: lamp_project.filter(|s| !s.is_empty()),
        waiting_for: lamp_waiting_for.filter(|s| !s.is_empty()),
        esc: lamp_esc,
        delegated: lamp_delegated,
        follow_up: lamp_follow_up,
        extra_tags: lamp_tags
            .map(|s| s.split(',').map(|t| unescape_text(t.trim())).collect())
            .unwrap_or_default(),
        scheduled_time: None,
        deadline_time: None,
        dayplan_date: lamp_dayplan,
        dayplan_budget: lamp_dayplan_budget,
        logbook_entries: lamp_logbook
            .map(|s| {
                s.split(',')
                    .filter_map(|e| parse_ical_datetime(e.trim()))
                    .collect()
            })
            .unwrap_or_default(),
        clock_entries: lamp_clock
            .map(|s| {
                s.split(',')
                    .filter_map(|pair| {
                        let parts: Vec<&str> = pair.trim().splitn(2, '/').collect();
                        if parts.len() == 2 {
                            let start = parse_ical_datetime(parts[0])?;
                            let end = parse_ical_datetime(parts[1])?;
                            Some((start, end))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default(),
        sync_href: None,
        sync_hash: None,
        sync_uid: uid_raw,
        sync_etag: None,
    })
}

/// Compute a deterministic content hash for a task (for change detection).
/// Uses raw byte writes (no length prefixes) to match the Kotlin mobile StableHasher API.
pub fn task_content_hash(task: &Task) -> u64 {
    use std::hash::Hasher;

    let mut hasher = StableHasher::new();
    // Match Kotlin's writeString: just raw UTF-8 bytes
    hasher.write(task.title.as_bytes());
    hasher.write(task.state.as_keyword().as_bytes());
    // Match Kotlin's writeOptionalString: 0x00 for None, 0x01 + bytes for Some
    write_optional_string(&mut hasher, task.priority.map(|p| p.as_org().to_string()).as_deref());
    for ctx in &task.contexts {
        hasher.write(ctx.as_bytes());
    }
    write_optional_string(&mut hasher, task.scheduled.map(|d| d.to_string()).as_deref());
    write_optional_string(&mut hasher, task.deadline.map(|d| d.to_string()).as_deref());
    hasher.write(task.notes.as_bytes());
    write_optional_string(&mut hasher, task.project.as_deref());
    write_optional_string(&mut hasher, task.waiting_for.as_deref());
    write_optional_int(&mut hasher, task.esc);
    write_optional_string(&mut hasher, task.delegated.map(|d| d.to_string()).as_deref());
    write_optional_string(&mut hasher, task.follow_up.map(|d| d.to_string()).as_deref());
    write_optional_string(&mut hasher, task.recurrence.as_ref().map(|r| r.to_string()).as_deref());
    for tag in &task.extra_tags {
        hasher.write(tag.as_bytes());
    }
    for entry in &task.logbook_entries {
        hasher.write(entry.format("%Y-%m-%dT%H:%M:%S").to_string().as_bytes());
    }
    for (start, end) in &task.clock_entries {
        hasher.write(start.format("%Y-%m-%dT%H:%M:%S").to_string().as_bytes());
        hasher.write(end.format("%Y-%m-%dT%H:%M:%S").to_string().as_bytes());
    }
    // Include dayplan fields so confirming/unconfirming a task triggers sync push
    write_optional_string(&mut hasher, task.dayplan_date.map(|d| d.to_string()).as_deref());
    write_optional_int(&mut hasher, task.dayplan_budget);
    // Include completed so completion timestamp changes trigger sync
    write_optional_string(&mut hasher, task.completed.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()).as_deref());
    hasher.finish()
}

fn write_optional_string(hasher: &mut StableHasher, s: Option<&str>) {
    match s {
        Some(s) => {
            hasher.write(&[1]);
            hasher.write(s.as_bytes());
        }
        None => {
            hasher.write(&[0]);
        }
    }
}

fn write_optional_int(hasher: &mut StableHasher, v: Option<u32>) {
    match v {
        Some(v) => {
            hasher.write(&[1]);
            hasher.write(v.to_string().as_bytes());
        }
        None => {
            hasher.write(&[0]);
        }
    }
}

/// A simple FNV-1a hasher that produces deterministic results across Rust versions.
pub struct StableHasher {
    hash: u64,
}

impl StableHasher {
    pub fn new() -> Self {
        Self {
            hash: 0xcbf29ce484222325, // FNV offset basis
        }
    }
}

impl std::hash::Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash ^= byte as u64;
            self.hash = self.hash.wrapping_mul(0x100000001b3); // FNV prime
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::task::{Priority, Task, TaskState};
    use chrono::NaiveDate;

    #[test]
    fn roundtrip_simple_task() {
        let task = Task {
            title: "Fix the faucet".to_string(),
            state: TaskState::Next,
            priority: Some(Priority::A),
            contexts: vec!["@home".to_string()],
            scheduled: Some(NaiveDate::from_ymd_opt(2026, 2, 24).unwrap()),
            deadline: Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()),
            notes: "Call the plumber first".to_string(),
            esc: Some(20),
            ..Task::new("unused")
        };
        let id = task.id;
        let ical = task_to_vcalendar(&task);
        let parsed = vcalendar_to_task(&ical).unwrap();
        assert_eq!(parsed.id, id);
        assert_eq!(parsed.title, "Fix the faucet");
        assert_eq!(parsed.state, TaskState::Next);
        assert_eq!(parsed.priority, Some(Priority::A));
        assert_eq!(parsed.contexts, vec!["@home"]);
        assert_eq!(
            parsed.scheduled,
            Some(NaiveDate::from_ymd_opt(2026, 2, 24).unwrap())
        );
        assert_eq!(
            parsed.deadline,
            Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap())
        );
        assert_eq!(parsed.notes, "Call the plumber first");
        assert_eq!(parsed.esc, Some(20));
    }

    #[test]
    fn roundtrip_waiting_task() {
        let mut task = Task::new("Review PR");
        task.state = TaskState::Waiting;
        task.waiting_for = Some("Alice".to_string());
        task.delegated = Some(NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
        task.follow_up = Some(NaiveDate::from_ymd_opt(2026, 2, 27).unwrap());
        task.project = Some("Launch v2".to_string());

        let ical = task_to_vcalendar(&task);
        let parsed = vcalendar_to_task(&ical).unwrap();
        assert_eq!(parsed.state, TaskState::Waiting);
        assert_eq!(parsed.waiting_for, Some("Alice".to_string()));
        assert_eq!(
            parsed.delegated,
            Some(NaiveDate::from_ymd_opt(2026, 2, 20).unwrap())
        );
        assert_eq!(
            parsed.follow_up,
            Some(NaiveDate::from_ymd_opt(2026, 2, 27).unwrap())
        );
        assert_eq!(parsed.project, Some("Launch v2".to_string()));
    }

    #[test]
    fn roundtrip_done_task() {
        let mut task = Task::new("Old thing");
        task.complete();
        let ical = task_to_vcalendar(&task);
        let parsed = vcalendar_to_task(&ical).unwrap();
        assert_eq!(parsed.state, TaskState::Done);
        assert!(parsed.completed.is_some());
    }

    #[test]
    fn content_hash_changes() {
        let task1 = Task::new("Test");
        let mut task2 = task1.clone();
        task2.title = "Changed".to_string();
        assert_ne!(task_content_hash(&task1), task_content_hash(&task2));
    }

    #[test]
    fn content_hash_stable() {
        let task = Task::new("Stable");
        assert_eq!(task_content_hash(&task), task_content_hash(&task));
    }

    /// Cross-platform hash test vectors.
    /// Mobile (Kotlin) must produce identical hashes for the same inputs.
    /// If this test breaks, update BOTH platforms' test vectors together.
    #[test]
    fn cross_platform_hash_minimal() {
        let task = Task {
            title: "Buy milk".to_string(),
            state: TaskState::Todo,
            priority: None,
            contexts: vec![],
            scheduled: None,
            deadline: None,
            notes: "".to_string(),
            project: None,
            waiting_for: None,
            esc: None,
            delegated: None,
            follow_up: None,
            recurrence: None,
            extra_tags: vec![],
            logbook_entries: vec![],
            clock_entries: vec![],
            dayplan_date: None,
            dayplan_budget: None,
            completed: None,
            ..Task::new("unused")
        };
        assert_eq!(task_content_hash(&task), 12459385973532360430);
    }

    #[test]
    fn cross_platform_hash_full() {
        let task = Task {
            title: "Review quarterly report".to_string(),
            state: TaskState::Next,
            priority: Some(Priority::B),
            contexts: vec!["@office".to_string(), "@computer".to_string()],
            scheduled: Some(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()),
            deadline: Some(NaiveDate::from_ymd_opt(2026, 3, 20).unwrap()),
            notes: "Check figures in section 3".to_string(),
            project: Some("Q1 Review".to_string()),
            waiting_for: Some("Alice".to_string()),
            esc: Some(40),
            delegated: Some(NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
            follow_up: Some(NaiveDate::from_ymd_opt(2026, 3, 18).unwrap()),
            recurrence: None,
            extra_tags: vec!["urgent".to_string()],
            logbook_entries: vec![
                NaiveDate::from_ymd_opt(2026, 3, 1).unwrap().and_hms_opt(9, 0, 0).unwrap(),
            ],
            clock_entries: vec![(
                NaiveDate::from_ymd_opt(2026, 3, 14).unwrap().and_hms_opt(10, 0, 0).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 14).unwrap().and_hms_opt(11, 30, 0).unwrap(),
            )],
            dayplan_date: Some(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()),
            dayplan_budget: Some(80),
            completed: Some(NaiveDate::from_ymd_opt(2026, 3, 20).unwrap().and_hms_opt(17, 0, 0).unwrap()),
            ..Task::new("unused")
        };
        assert_eq!(task_content_hash(&task), 14390201416720652008);
    }

    #[test]
    fn parse_non_lamp_vtodo() {
        let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTODO\r\nUID:abc-123\r\nSUMMARY:External task\r\nSTATUS:NEEDS-ACTION\r\nPRIORITY:5\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";
        let task = vcalendar_to_task(ical).unwrap();
        assert_eq!(task.title, "External task");
        assert_eq!(task.state, TaskState::Todo); // Falls back from NEEDS-ACTION
        assert_eq!(task.priority, Some(Priority::B)); // 5 → B
    }
}
