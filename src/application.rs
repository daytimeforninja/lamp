use std::collections::{HashMap, HashSet};

use chrono::Timelike;

use cosmic::app::{Core, Task as CosmicTask, context_drawer};
use cosmic::iced::Length;
use cosmic::widget::{button, column, container, flex_row, icon, nav_bar, row, scrollable, text, text_editor, text_input};
use cosmic::{Application, Element, executor};

use crate::config::LampConfig;
use crate::core::account::Account;
use crate::core::day_plan::DayPlan;
use crate::core::event::{self, CalendarEvent};
use crate::core::habit::Habit;
use crate::core::link::LinkTarget;
use crate::core::list_item::ListItem;
use crate::core::note::Note;
use crate::core::project::Project;
use crate::core::task::{Priority, Task, TaskState};
use crate::message::{AccountField, ActiveView, AppMode, ContactField, ListKind, Message, NoteField, ServiceKind, SortColumn, WhatPage};
use crate::org::convert;
use crate::org::writer::OrgWriter;
use crate::pages;
use crate::components::month_calendar::MonthCalendarState;
use crate::sync::caldav::{CalDavClient, CalendarInfo};
use crate::sync::carddav::Contact;
use crate::sync::imap::ImapEmail;
use crate::sync::{SyncConflict, SyncStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDrawerState {
    NewTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Normal,
    Capture,
    Today,
}

pub struct NewTaskForm {
    pub title: String,
    pub state: TaskState,
    pub priority: Option<Priority>,
    pub esc: Option<u32>,
    pub contexts: Vec<String>,
    pub project: Option<String>,
    pub scheduled: String,
    pub deadline: String,
    pub notes: String,
}

impl Default for NewTaskForm {
    fn default() -> Self {
        Self {
            title: String::new(),
            state: TaskState::Todo,
            priority: None,
            esc: None,
            contexts: Vec::new(),
            project: None,
            scheduled: String::new(),
            deadline: String::new(),
            notes: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct EventForm {
    pub editing: Option<uuid::Uuid>,
    pub title: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub all_day: bool,
    pub location: String,
    pub description: String,
    pub calendar_href: String,
}

impl EventForm {
    fn from_event(event: &CalendarEvent) -> Self {
        Self {
            editing: Some(event.id),
            title: event.title.clone(),
            start_date: event.start.format("%Y-%m-%d").to_string(),
            start_time: event.start.format("%H:%M").to_string(),
            end_date: event.end.format("%Y-%m-%d").to_string(),
            end_time: event.end.format("%H:%M").to_string(),
            all_day: event.all_day,
            location: event.location.clone(),
            description: event.description.clone(),
            calendar_href: event.calendar_href.clone(),
        }
    }
}

/// Buffered edit state for note fields — committed on Done, avoids re-renders while typing.
pub struct NoteEditBuffer {
    pub id: uuid::Uuid,
    pub title: String,
    pub tags: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub enum EmailSuggestionState {
    Suggested(crate::sync::anthropic::BatchEmailSuggestion),
    NoAction,
    Dismissed,
}

pub struct Lamp {
    core: Core,
    nav_model: nav_bar::Model,
    config: LampConfig,
    cosmic_config: cosmic::cosmic_config::Config,
    active_view: ActiveView,
    app_mode: AppMode,

    // Data
    inbox_tasks: Vec<Task>,
    next_tasks: Vec<Task>,
    waiting_tasks: Vec<Task>,
    someday_tasks: Vec<Task>,
    projects: Vec<Project>,
    habits: Vec<Habit>,

    // Cached for today view (rebuilt on data changes)
    all_tasks_cache: Vec<Task>,

    // List items
    media_items: Vec<ListItem>,
    shopping_items: Vec<ListItem>,

    // Day plan
    day_plan: Option<DayPlan>,
    rejected_suggestions: HashSet<uuid::Uuid>,

    // All Tasks sort
    all_tasks_sort: Option<(SortColumn, bool)>, // (column, ascending)

    // Drawer & capture
    context_drawer_state: Option<ContextDrawerState>,
    new_task_form: NewTaskForm,
    launch_mode: LaunchMode,

    // UI state
    inbox_input: String,
    project_input: String,
    project_task_inputs: HashMap<String, String>,
    habit_input: String,
    media_input: String,
    shopping_input: String,
    search_query: String,
    settings_context_input: String,
    expanded_task: Option<uuid::Uuid>,
    note_inputs: HashMap<uuid::Uuid, String>,
    flipped_list_items: HashSet<uuid::Uuid>,
    pending_delete_list_item: Option<(ListKind, uuid::Uuid)>,
    waiting_for_inputs: HashMap<uuid::Uuid, String>,

    // Review checklist (ephemeral — resets on nav away)
    review_checked: HashSet<usize>,

    // Events
    events: Vec<CalendarEvent>,
    event_form: Option<EventForm>,

    // Contacts
    contacts: Vec<Contact>,
    contact_input: String,
    flipped_contacts: HashSet<usize>,
    editing_contact: Option<usize>,
    pending_delete_contact: Option<usize>,

    // Accounts
    accounts: Vec<Account>,
    account_input: String,
    expanded_account: Option<usize>,
    pending_delete_account: Option<usize>,

    // Notes
    notes: Vec<Note>,
    note_input: String,
    flipped_notes: HashSet<uuid::Uuid>,
    editing_note: Option<uuid::Uuid>,
    pending_delete_note: Option<uuid::Uuid>,
    note_editor_content: Option<(uuid::Uuid, text_editor::Content)>,
    note_link_search: String,
    /// Buffered edit fields — only committed to the note on Done.
    note_edit_buffer: Option<NoteEditBuffer>,
    backlink_index: HashMap<LinkTarget, Vec<uuid::Uuid>>,

    // Sync
    sync_status: SyncStatus,
    /// Discovered calendars from CalDAV test connection
    discovered_calendars: Vec<CalendarInfo>,
    /// Password inputs for [Calendars, Contacts, Notes, Imap]
    service_passwords: [String; 4],
    /// Test connection results for [Calendars, Contacts, Notes, Imap]
    service_test_status: [Option<Result<String, String>>; 4],

    // IMAP emails
    imap_emails: Vec<ImapEmail>,

    // AI batch email suggestions
    anthropic_api_key_input: String,
    anthropic_test_status: Option<Result<String, String>>,
    email_suggestions: HashMap<u32, EmailSuggestionState>,
    ai_batch_processing: bool,
    /// UIDs of emails archived this session — filtered out on re-fetch
    archived_email_uids: HashSet<u32>,

    // Pending sync completions: (sync_href, vcalendar_body) to push on next sync
    pending_completions: Vec<(String, String)>,

    // Sync conflicts awaiting user resolution
    sync_conflicts: Vec<SyncConflict>,

    // Pending remote deletions: hrefs to delete on next sync
    pending_deletions: Vec<String>,

    // Month calendar
    month_calendar: MonthCalendarState,
}

pub struct Flags {
    pub config: LampConfig,
    pub cosmic_config: cosmic::cosmic_config::Config,
    pub launch_mode: LaunchMode,
}

impl Application for Lamp {
    type Executor = executor::Default;
    type Flags = Flags;
    type Message = Message;

    const APP_ID: &'static str = "dev.lamp.app";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Self::Flags) -> (Self, CosmicTask<Self::Message>) {
        let config = flags.config;
        let cosmic_config = flags.cosmic_config;
        let launch_mode = flags.launch_mode;

        // Ensure org files exist
        if let Err(e) = config.ensure_files() {
            log::error!("Failed to create org directory: {}", e);
        }

        // Build sidebar navigation model with section dividers
        let mut nav_model = nav_bar::Model::default();
        for page in WhatPage::ALL {
            let mut item = nav_model.insert();
            item = item
                .text(page.title())
                .icon(icon::from_name(page.icon_name()).icon())
                .data(*page);
            if WhatPage::SECTION_STARTS.contains(page) {
                item.divider_above(true);
            }
        }

        // Load tasks from org files
        let inbox_tasks = load_tasks(&config.inbox_path());
        let next_tasks = load_tasks(&config.next_path());
        let waiting_tasks = load_tasks(&config.waiting_path());
        let someday_tasks = load_tasks(&config.someday_path());
        let projects = load_projects(&config.projects_path());
        let habits = load_habits(&config.habits_path());
        let media_items = load_list_items(&config.media_path());
        let shopping_items = load_list_items(&config.shopping_path());

        // Load day plan, clear if stale
        let today = chrono::Local::now().date_naive();
        let day_plan = load_day_plan(&config.dayplan_path())
            .filter(|dp| !dp.is_stale(today));

        // Load cached contacts, events, and accounts
        let contacts = crate::sync::carddav::load_contacts(&config.contacts_path());
        let events = event::load_events(&config.events_cache_path());
        let accounts = load_accounts(&config.accounts_path());
        let notes = load_notes_dir(&config.notes_dir(), &config.notes_path());

        // Set initial state based on launch mode
        let (app_mode, context_drawer_state) = match launch_mode {
            LaunchMode::Capture => {
                core.window.show_context = true;
                (AppMode::Plan, Some(ContextDrawerState::NewTask))
            }
            LaunchMode::Today => (AppMode::Do, None),
            LaunchMode::Normal => (AppMode::Plan, None),
        };

        let mut app = Self {
            core,
            nav_model,
            config,
            cosmic_config,
            active_view: ActiveView::What(WhatPage::DailyPlanning),
            app_mode,
            inbox_tasks,
            next_tasks,
            waiting_tasks,
            someday_tasks,
            projects,
            habits,
            media_items,
            shopping_items,
            day_plan,
            rejected_suggestions: HashSet::new(),
            all_tasks_cache: Vec::new(),
            all_tasks_sort: None,
            context_drawer_state,
            new_task_form: NewTaskForm::default(),
            launch_mode,
            inbox_input: String::new(),
            project_input: String::new(),
            project_task_inputs: HashMap::new(),
            habit_input: String::new(),
            media_input: String::new(),
            shopping_input: String::new(),
            search_query: String::new(),
            settings_context_input: String::new(),
            expanded_task: None,
            note_inputs: HashMap::new(),
            flipped_list_items: HashSet::new(),
            pending_delete_list_item: None,
            waiting_for_inputs: HashMap::new(),
            contacts,
            contact_input: String::new(),
            flipped_contacts: HashSet::new(),
            editing_contact: None,
            pending_delete_contact: None,
            accounts,
            account_input: String::new(),
            expanded_account: None,
            pending_delete_account: None,
            backlink_index: build_backlink_index(&notes),
            notes,
            note_input: String::new(),
            flipped_notes: HashSet::new(),
            editing_note: None,
            pending_delete_note: None,
            note_editor_content: None,
            note_link_search: String::new(),
            note_edit_buffer: None,
            events,
            event_form: None,
            review_checked: HashSet::new(),
            sync_status: SyncStatus::default(),
            discovered_calendars: Vec::new(),
            service_passwords: [String::new(), String::new(), String::new(), String::new()],
            service_test_status: [None, None, None, None],
            imap_emails: Vec::new(),
            anthropic_api_key_input: String::new(),
            anthropic_test_status: None,
            email_suggestions: HashMap::new(),
            ai_batch_processing: false,
            archived_email_uids: HashSet::new(),
            pending_completions: Vec::new(),
            sync_conflicts: Vec::new(),
            pending_deletions: Vec::new(),
            month_calendar: MonthCalendarState::default(),
        };
        app.rebuild_cache();

        (app, CosmicTask::none())
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        if self.launch_mode == LaunchMode::Today {
            return None;
        }
        match self.app_mode {
            AppMode::Plan => Some(&self.nav_model),
            AppMode::Do => None,
        }
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> CosmicTask<Message> {
        if let Some(page) = self.nav_model.data::<WhatPage>(id).cloned() {
            // Reset review checklist when navigating away from Review
            if let ActiveView::What(WhatPage::Review) = self.active_view {
                if page != WhatPage::Review {
                    self.review_checked.clear();
                }
            }
            self.active_view = ActiveView::What(page);
            self.search_query.clear();
            self.nav_model.activate(id);
        }
        CosmicTask::none()
    }

    fn header_center(&self) -> Vec<Element<'_, Message>> {
        if self.launch_mode == LaunchMode::Today {
            return vec![text::title4("Today").into()];
        }

        let plan_btn = if self.app_mode == AppMode::Plan {
            button::suggested("Plan")
        } else {
            button::standard("Plan")
        }
        .on_press(Message::SetMode(AppMode::Plan));

        let do_btn = if self.app_mode == AppMode::Do {
            button::suggested("Do")
        } else {
            button::standard("Do")
        }
        .on_press(Message::SetMode(AppMode::Do));

        vec![
            row()
                .spacing(4)
                .push(plan_btn)
                .push(do_btn)
                .into(),
        ]
    }

    fn update(&mut self, message: Message) -> CosmicTask<Message> {
        match message {
            Message::SetMode(mode) => {
                self.app_mode = mode;
            }

            Message::SearchQueryChanged(q) => {
                self.search_query = q;
            }

            Message::SelectWhen(_) => {
                // When pages removed — no-op for compatibility
            }

            Message::InboxInputChanged(value) => {
                self.inbox_input = value;
            }

            Message::InboxSubmit => {
                let title = sentence_case(&self.inbox_input);
                if !title.is_empty() {
                    let task = Task::new(title);
                    self.inbox_tasks.push(task);
                    self.inbox_input.clear();
                    self.save_inbox();
                }
            }

            Message::AddTask(title) => {
                let task = Task::new(sentence_case(&title));
                self.inbox_tasks.push(task);
                self.save_inbox();
            }

            Message::UpdateTaskTitle(id, ref title) => {
                let new_title = title.clone();
                self.modify_task(id, |task| {
                    task.title = new_title;
                });
            }

            Message::ToggleTaskDone(id) => {
                self.toggle_done(id);
            }

            Message::SetTaskState(id, state) => {
                self.set_task_state(id, state);
            }

            Message::SetTaskPriority(id, priority) => {
                self.set_task_priority(id, priority);
            }

            Message::SetTaskEsc(id, esc) => {
                self.modify_task(id, |task| {
                    task.esc = esc;
                });
            }

            Message::WaitingForInputChanged(id, value) => {
                self.waiting_for_inputs.insert(id, value);
            }

            Message::SetFollowUp(id, date) => {
                self.modify_task(id, |task| {
                    task.follow_up = date;
                });
            }

            Message::SetWaitingFor(id, ref value) => {
                // Use the stored input value if the message value is empty (from on_submit)
                let effective = if value.is_empty() {
                    self.waiting_for_inputs.get(&id).cloned().unwrap_or_default()
                } else {
                    value.clone()
                };
                let wf = if effective.trim().is_empty() { None } else { Some(effective.trim().to_string()) };
                self.modify_task(id, |task| {
                    task.waiting_for = wf;
                });
                self.waiting_for_inputs.remove(&id);
            }

            Message::DeleteTask(id) => {
                // Queue server-side deletion if task has a sync_href
                if let Some(task) = self.remove_task(id) {
                    if let Some(href) = task.sync_href {
                        self.pending_deletions.push(href);
                    }
                }
                // Also remove from projects
                for project in &mut self.projects {
                    if let Some(pos) = project.tasks.iter().position(|t| t.id == id) {
                        if let Some(href) = project.tasks[pos].sync_href.clone() {
                            self.pending_deletions.push(href);
                        }
                        project.tasks.remove(pos);
                    }
                }
                self.save_all();
            }

            Message::MoveToProject(id, ref project_name) => {
                if let Some(mut task) = self.remove_task(id) {
                    task.project = Some(project_name.clone());
                    if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                        project.tasks.push(task);
                    }
                    self.save_all();
                }
            }

            Message::AddContext(id, ref ctx) => {
                self.modify_task(id, |task| {
                    if !task.contexts.contains(ctx) {
                        task.contexts.push(ctx.clone());
                    }
                });
            }

            Message::RemoveContext(id, ref ctx) => {
                self.modify_task(id, |task| {
                    task.contexts.retain(|c| c != ctx);
                });
            }

            Message::SetScheduled(id, date) => {
                self.modify_task(id, |task| {
                    task.scheduled = date;
                });
            }

            Message::SetDeadline(id, date) => {
                self.modify_task(id, |task| {
                    task.deadline = date;
                });
            }

            Message::OpenSettings => {
                self.active_view = ActiveView::What(WhatPage::Settings);
                // Activate the corresponding sidebar nav item
                let target = self.nav_model.iter()
                    .find(|&id| self.nav_model.data::<WhatPage>(id) == Some(&WhatPage::Settings));
                if let Some(id) = target {
                    self.nav_model.activate(id);
                }
                self.search_query.clear();
            }

            Message::SettingsContextInput(value) => {
                self.settings_context_input = value;
            }

            Message::SettingsAddContext => {
                let ctx = self.settings_context_input.trim().to_string();
                if !ctx.is_empty() && !self.config.contexts.contains(&ctx) {
                    let ctx = if ctx.starts_with('@') { ctx } else { format!("@{}", ctx) };
                    if !self.config.contexts.contains(&ctx) {
                        self.config.contexts.push(ctx);
                    }
                    self.settings_context_input.clear();
                    self.save_config();
                }
            }

            Message::SettingsRemoveContext(idx) => {
                if idx < self.config.contexts.len() {
                    self.config.contexts.remove(idx);
                    self.save_config();
                }
            }

            Message::SetBrowserCommand(value) => {
                self.config.browser_command = value;
                self.save_config();
            }

            Message::ToggleDebugLogging => {
                self.config.debug_logging = !self.config.debug_logging;
                lamp::set_debug_logging(self.config.debug_logging);
                self.save_config();
            }

            Message::ProjectInputChanged(value) => {
                self.project_input = value;
            }

            Message::ProjectSubmit => {
                let name = self.project_input.trim().to_string();
                if !name.is_empty() && !self.projects.iter().any(|p| p.name == name) {
                    self.projects.push(crate::core::project::Project::new(name));
                    self.project_input.clear();
                    self.save_projects();
                }
            }

            Message::CreateProject(name) => {
                if !name.is_empty() && !self.projects.iter().any(|p| p.name == name) {
                    self.projects.push(crate::core::project::Project::new(name));
                    self.save_projects();
                }
            }

            Message::DeleteProject(name) => {
                self.projects.retain(|p| p.name != name);
                self.save_projects();
                self.rebuild_cache();
            }

            Message::ProjectTaskInputChanged(ref project_name, ref value) => {
                self.project_task_inputs
                    .insert(project_name.clone(), value.clone());
            }

            Message::AddTaskToProject(ref project_name) => {
                let input = self
                    .project_task_inputs
                    .get(project_name)
                    .cloned()
                    .unwrap_or_default();
                let title = sentence_case(&input);
                if !title.is_empty() {
                    let mut task = Task::new(title);
                    task.project = Some(project_name.clone());
                    if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                        project.tasks.push(task);
                    }
                    self.project_task_inputs.insert(project_name.clone(), String::new());
                    self.save_all();
                }
            }

            Message::SetProjectPurpose(ref project_name, ref value) => {
                if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                    project.purpose = value.clone();
                    self.save_projects();
                }
            }

            Message::SetProjectOutcome(ref project_name, ref value) => {
                if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                    project.outcome = value.clone();
                    self.save_projects();
                }
            }

            Message::SetProjectBrainstorm(ref project_name, ref value) => {
                if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                    project.brainstorm = value.clone();
                    self.save_projects();
                }
            }

            Message::ReorderProjectTask(ref project_name, task_id, direction) => {
                if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                    if let Some(pos) = project.tasks.iter().position(|t| t.id == task_id) {
                        let new_pos = pos as isize + direction;
                        if new_pos >= 0 && (new_pos as usize) < project.tasks.len() {
                            project.tasks.swap(pos, new_pos as usize);
                            self.save_projects();
                            self.rebuild_cache();
                        }
                    }
                }
            }

            Message::ToggleReviewStep(idx) => {
                if self.review_checked.contains(&idx) {
                    self.review_checked.remove(&idx);
                } else {
                    self.review_checked.insert(idx);
                }
            }

            Message::CompleteHabit(id) => {
                let today = chrono::Local::now().date_naive();
                if let Some(habit) = self.habits.iter_mut().find(|h| h.task.id == id) {
                    if habit.is_due(today) {
                        habit
                            .completions
                            .push(chrono::Local::now().naive_local());
                        habit.recalculate_streak(today);
                    }
                }
                self.save_habits();
            }

            Message::DeleteHabit(id) => {
                self.habits.retain(|h| h.task.id != id);
                self.save_habits();
            }

            Message::HabitInputChanged(value) => {
                self.habit_input = value;
            }

            Message::HabitSubmit => {
                let title = self.habit_input.trim().to_string();
                if !title.is_empty() {
                    use crate::core::recurrence::{Recurrence, RecurrenceInterval, RecurrenceUnit};

                    let today = chrono::Local::now().date_naive();
                    let mut task = Task::new(title);
                    task.scheduled = Some(today);
                    task.recurrence = Some(Recurrence::Relative(RecurrenceInterval {
                        count: 1,
                        unit: RecurrenceUnit::Day,
                    }));
                    task.contexts.push("habit".to_string());
                    let habit = crate::core::habit::Habit::new(task);
                    self.habits.push(habit);
                    self.habit_input.clear();
                    self.save_habits();
                }
            }

            Message::ToggleTaskExpand(id) => {
                if self.expanded_task == Some(id) {
                    // Collapsing — apply sentence case to title
                    self.modify_task(id, |task| {
                        task.title = sentence_case(&task.title);
                    });
                    self.expanded_task = None;
                } else {
                    self.expanded_task = Some(id);
                }
            }

            Message::NoteInputChanged(id, value) => {
                self.note_inputs.insert(id, value);
            }

            Message::AppendNote(id) => {
                let input = self.note_inputs.get(&id).cloned().unwrap_or_default();
                let text = input.trim().to_string();
                if !text.is_empty() {
                    let now = chrono::Local::now();
                    let stamp = now.format("[%Y-%m-%d %a %H:%M]").to_string();
                    let line = format!("{} {}", stamp, text);

                    // Try list items first, then fall through to tasks
                    if let Some(item) = self.media_items.iter_mut().find(|i| i.id == id) {
                        if item.notes.is_empty() {
                            item.notes = line;
                        } else {
                            item.notes.push('\n');
                            item.notes.push_str(&line);
                        }
                        self.note_inputs.insert(id, String::new());
                        self.save_media();
                    } else if let Some(item) = self.shopping_items.iter_mut().find(|i| i.id == id) {
                        if item.notes.is_empty() {
                            item.notes = line;
                        } else {
                            item.notes.push('\n');
                            item.notes.push_str(&line);
                        }
                        self.note_inputs.insert(id, String::new());
                        self.save_shopping();
                    } else {
                        self.modify_task(id, |task| {
                            if task.notes.is_empty() {
                                task.notes = line;
                            } else {
                                task.notes.push('\n');
                                task.notes.push_str(&line);
                            }
                        });
                        self.note_inputs.insert(id, String::new());
                    }
                }
            }

            Message::ListInputChanged(kind, value) => {
                match kind {
                    ListKind::Media => self.media_input = value,
                    ListKind::Shopping => self.shopping_input = value,
                }
            }

            Message::ListSubmit(kind) => {
                match kind {
                    ListKind::Media => {
                        let title = self.media_input.trim().to_string();
                        if !title.is_empty() {
                            self.media_items.push(ListItem::new(title));
                            self.media_input.clear();
                            self.save_media();
                        }
                    }
                    ListKind::Shopping => {
                        let title = self.shopping_input.trim().to_string();
                        if !title.is_empty() {
                            self.shopping_items.push(ListItem::new(title));
                            self.shopping_input.clear();
                            self.save_shopping();
                        }
                    }
                }
            }

            Message::DeleteListItem(kind, id) => {
                match kind {
                    ListKind::Media => {
                        if let Some(item) = self.media_items.iter().find(|i| i.id == id) {
                            if let Err(e) = OrgWriter::append_list_item_to_file(&self.config.consumed_path(), item) {
                                log::error!("Failed to archive media item: {}", e);
                            }
                        }
                        self.media_items.retain(|i| i.id != id);
                        self.save_media();
                    }
                    ListKind::Shopping => {
                        if let Some(item) = self.shopping_items.iter().find(|i| i.id == id) {
                            if let Err(e) = OrgWriter::append_list_item_to_file(&self.config.bought_path(), item) {
                                log::error!("Failed to archive shopping item: {}", e);
                            }
                        }
                        self.shopping_items.retain(|i| i.id != id);
                        self.save_shopping();
                    }
                }
                self.flipped_list_items.remove(&id);
                self.pending_delete_list_item = None;
            }

            Message::ToggleListItemDone(kind, id) => {
                match kind {
                    ListKind::Media => {
                        if let Some(item) = self.media_items.iter_mut().find(|i| i.id == id) {
                            item.done = !item.done;
                        }
                        self.save_media();
                    }
                    ListKind::Shopping => {
                        if let Some(item) = self.shopping_items.iter_mut().find(|i| i.id == id) {
                            item.done = !item.done;
                        }
                        self.save_shopping();
                    }
                }
            }

            Message::FlipListItem(id) => {
                if !self.flipped_list_items.remove(&id) {
                    self.flipped_list_items.insert(id);
                }
            }

            Message::ConfirmDeleteListItem(kind, id) => {
                self.pending_delete_list_item = Some((kind, id));
            }

            Message::CancelDeleteListItem => {
                self.pending_delete_list_item = None;
            }

            // Contacts CRUD
            Message::ContactInputChanged(value) => {
                self.contact_input = value;
            }

            Message::ContactSubmit => {
                let name = self.contact_input.trim().to_string();
                if !name.is_empty() {
                    self.contacts.push(Contact::new(name));
                    self.contact_input.clear();
                    self.contacts.sort_by(|a, b| a.name.cmp(&b.name));
                    self.save_contacts();
                }
            }

            Message::ConfirmDeleteContact(idx) => {
                self.pending_delete_contact = Some(idx);
            }

            Message::CancelDeleteContact => {
                self.pending_delete_contact = None;
            }

            Message::DeleteContact(idx) => {
                self.pending_delete_contact = None;
                if idx < self.contacts.len() {
                    let removed = self.contacts.remove(idx);
                    self.flipped_contacts.remove(&idx);
                    if self.editing_contact == Some(idx) {
                        self.editing_contact = None;
                    }
                    self.save_contacts();

                    // Delete from CardDAV server if we have a sync_href
                    if let Some(href) = removed.sync_href {
                        let contacts_url = self.config.contacts.url.trim().to_string();
                        if !contacts_url.is_empty() {
                            return CosmicTask::perform(
                                async move {
                                    let (username, pw) = match crate::sync::keyring::load_credentials(&contacts_url).await {
                                        Ok(Some(creds)) => creds,
                                        _ => return Err("No CardDAV credentials".to_string()),
                                    };
                                    let client = crate::sync::carddav::CardDavClient::new(
                                        &contacts_url, &username, &pw,
                                    )?;
                                    client.delete_contact(&href).await
                                },
                                |result| cosmic::Action::App(Message::ContactDeleted(result)),
                            );
                        }
                    }
                }
            }

            Message::SetContactCategory(idx, cat) => {
                if let Some(c) = self.contacts.get_mut(idx) {
                    c.category = cat;
                    self.save_contacts();
                }
            }

            Message::SetContactField(idx, ref field, ref value) => {
                if let Some(c) = self.contacts.get_mut(idx) {
                    let val = if value.is_empty() { None } else { Some(value.clone()) };
                    match field {
                        ContactField::Email => c.email = val,
                        ContactField::Phone => c.phone = val,
                        ContactField::Website => c.website = val,
                        ContactField::Signal => c.signal = val,
                        ContactField::PreferredMethod => c.preferred_method = val,
                    }
                    self.save_contacts();
                }
            }

            Message::MarkContacted(idx) => {
                if let Some(c) = self.contacts.get_mut(idx) {
                    c.last_contacted = Some(chrono::Local::now().date_naive());
                    self.save_contacts();
                }
            }

            Message::FlipContact(idx) => {
                if !self.flipped_contacts.remove(&idx) {
                    self.flipped_contacts.insert(idx);
                }
                if self.editing_contact == Some(idx) {
                    self.editing_contact = None;
                }
            }

            Message::EditContact(idx) => {
                self.flipped_contacts.insert(idx);
                self.editing_contact = Some(idx);
            }

            // Accounts
            Message::AccountInputChanged(value) => {
                self.account_input = value;
            }

            Message::AccountSubmit => {
                let name = self.account_input.trim().to_string();
                if !name.is_empty() {
                    self.accounts.push(Account::new(name));
                    self.account_input.clear();
                    self.save_accounts();
                }
            }

            Message::ConfirmDeleteAccount(idx) => {
                self.pending_delete_account = Some(idx);
            }

            Message::CancelDeleteAccount => {
                self.pending_delete_account = None;
            }

            Message::DeleteAccount(idx) => {
                self.pending_delete_account = None;
                if idx < self.accounts.len() {
                    let removed = self.accounts.remove(idx);
                    self.expanded_account = None;
                    // Archive to closed_accounts.org
                    if let Err(e) = OrgWriter::append_account_to_file(
                        &self.config.closed_accounts_path(),
                        &removed,
                    ) {
                        log::error!("Failed to archive account: {}", e);
                    }
                    self.save_accounts();
                }
            }

            Message::SetAccountFieldValue(idx, ref field, ref value) => {
                if let Some(a) = self.accounts.get_mut(idx) {
                    match field {
                        AccountField::Name => a.name = value.clone(),
                        AccountField::Url => a.url = value.clone(),
                        AccountField::Notes => a.notes = value.clone(),
                    }
                    self.save_accounts();
                }
            }

            Message::MarkAccountChecked(idx) => {
                if let Some(a) = self.accounts.get_mut(idx) {
                    a.last_checked = Some(chrono::Local::now().date_naive());
                    self.save_accounts();
                }
            }

            Message::OpenAccountUrl(idx) => {
                if let Some(a) = self.accounts.get(idx) {
                    if !a.url.is_empty() {
                        if let Err(e) = std::process::Command::new(&self.config.browser_command)
                            .arg(&a.url)
                            .spawn()
                        {
                            log::error!("Failed to open URL: {}", e);
                        }
                    }
                }
            }

            Message::ToggleAccountExpand(idx) => {
                if self.expanded_account == Some(idx) {
                    self.expanded_account = None;
                } else {
                    self.expanded_account = Some(idx);
                }
            }

            // Notes CRUD
            Message::ZettelInputChanged(value) => {
                self.note_input = value;
            }

            Message::ZettelSubmit => {
                let title = self.note_input.trim().to_string();
                if !title.is_empty() {
                    let note = Note::new(title);
                    self.save_note(&note);
                    self.notes.push(note);
                    self.note_input.clear();
                    self.notes.sort_by(|a, b| a.title.cmp(&b.title));
                }
            }

            Message::FlipNote(id) => {
                // If exiting edit mode, commit all pending edits and save
                if self.editing_note == Some(id) && self.flipped_notes.contains(&id) {
                    // Commit edit buffer (title, tags, source)
                    if let Some(buf) = self.note_edit_buffer.take() {
                        if buf.id == id {
                            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                                note.title = buf.title;
                                note.tags = buf.tags
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                note.source = if buf.source.is_empty() { None } else { Some(buf.source) };
                            }
                        }
                    }

                    // Commit text_editor body
                    if let Some((eid, ref content)) = self.note_editor_content {
                        if eid == id {
                            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                                note.body = content.text();
                                if note.body.ends_with('\n') {
                                    note.body.pop();
                                }
                            }
                        }
                    }
                    self.note_editor_content = None;

                    // Update modified timestamp
                    if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                        note.modified = chrono::Local::now().naive_local();
                    }

                    self.editing_note = None;
                    self.note_link_search.clear();
                    if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                        self.save_note(note);
                    }
                } else if self.flipped_notes.contains(&id) {
                    self.flipped_notes.remove(&id);
                } else {
                    self.flipped_notes.insert(id);
                }
            }

            Message::EditNote(id) => {
                self.editing_note = Some(id);
                self.flipped_notes.insert(id);
                if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                    self.note_editor_content =
                        Some((id, text_editor::Content::with_text(&note.body)));
                    self.note_edit_buffer = Some(NoteEditBuffer {
                        id,
                        title: note.title.clone(),
                        tags: note.tags.join(", "),
                        source: note.source.clone().unwrap_or_default(),
                    });
                }
            }

            Message::NoteEditorAction(action) => {
                if let Some((_, ref mut content)) = self.note_editor_content {
                    content.perform(action);
                }
            }

            Message::SetNoteField(_id, field, value) => {
                // Write to edit buffer only — committed on Done (FlipNote).
                if let Some(ref mut buf) = self.note_edit_buffer {
                    match field {
                        NoteField::Title => buf.title = value,
                        NoteField::Tags => buf.tags = value,
                        NoteField::Source => buf.source = value,
                        NoteField::Body => {} // handled by text_editor
                    }
                }
            }

            Message::ConfirmDeleteNote(id) => {
                self.pending_delete_note = Some(id);
            }

            Message::CancelDeleteNote => {
                self.pending_delete_note = None;
            }

            Message::DeleteNote(id) => {
                self.notes.retain(|n| n.id != id);
                self.pending_delete_note = None;
                self.flipped_notes.remove(&id);
                if self.editing_note == Some(id) {
                    self.editing_note = None;
                    self.note_editor_content = None;
                }
                self.backlink_index = build_backlink_index(&self.notes);
                self.delete_note_file(id);
            }

            Message::AddNoteLink(note_id, ref target) => {
                let target = target.clone();
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
                    if !note.links.contains(&target) {
                        note.links.push(target);
                        note.modified = chrono::Local::now().naive_local();
                    }
                }
                self.backlink_index = build_backlink_index(&self.notes);
                if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
                    self.save_note(note);
                }
            }

            Message::RemoveNoteLink(note_id, ref target) => {
                let target = target.clone();
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
                    note.links.retain(|l| l != &target);
                    note.modified = chrono::Local::now().naive_local();
                }
                self.backlink_index = build_backlink_index(&self.notes);
                if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
                    self.save_note(note);
                }
            }

            Message::OpenNoteInEditor(id) => {
                if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                    let tmp_dir = std::env::temp_dir();
                    let tmp_path = tmp_dir.join(format!("lamp-note-{}.md", note.id));
                    if let Err(e) = std::fs::write(&tmp_path, &note.body) {
                        log::error!("Failed to write temp note file: {}", e);
                    } else {
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                        if let Err(e) = std::process::Command::new(&editor)
                            .arg(&tmp_path)
                            .spawn()
                        {
                            log::error!("Failed to open editor: {}", e);
                        }
                    }
                }
            }

            Message::NoteLinkSearchChanged(value) => {
                self.note_link_search = value;
            }

            // Daily Planning messages
            Message::SetSpoonBudget(budget) => {
                let plan = self.ensure_day_plan();
                plan.spoon_budget = budget;
                self.save_day_plan();
            }

            Message::TogglePlanContext(ref ctx) => {
                let plan = self.ensure_day_plan();
                if let Some(pos) = plan.active_contexts.iter().position(|c| c == ctx) {
                    plan.active_contexts.remove(pos);
                } else {
                    plan.active_contexts.push(ctx.clone());
                }
                self.save_day_plan();
            }

            Message::ConfirmTask(id) => {
                let plan = self.ensure_day_plan();
                if !plan.confirmed_task_ids.contains(&id) {
                    plan.confirmed_task_ids.push(id);
                }
                self.save_day_plan();
            }

            Message::UnconfirmTask(id) => {
                if let Some(ref mut plan) = self.day_plan {
                    plan.confirmed_task_ids.retain(|i| *i != id);
                    self.save_day_plan();
                }
            }

            Message::RejectSuggestion(id) => {
                self.rejected_suggestions.insert(id);
            }

            Message::PickMediaItem(id) => {
                let plan = self.ensure_day_plan();
                if !plan.picked_media_ids.contains(&id) {
                    plan.picked_media_ids.push(id);
                }
                self.save_day_plan();
            }

            Message::UnpickMediaItem(id) => {
                if let Some(ref mut plan) = self.day_plan {
                    plan.picked_media_ids.retain(|i| *i != id);
                    self.save_day_plan();
                }
            }

            Message::PickShoppingItem(id) => {
                let plan = self.ensure_day_plan();
                if !plan.picked_shopping_ids.contains(&id) {
                    plan.picked_shopping_ids.push(id);
                }
                self.save_day_plan();
            }

            Message::UnpickShoppingItem(id) => {
                if let Some(ref mut plan) = self.day_plan {
                    plan.picked_shopping_ids.retain(|i| *i != id);
                    self.save_day_plan();
                }
            }

            // Do mode messages
            Message::DoMarkDone(id) => {
                let is_completed = self.day_plan.as_ref()
                    .map(|p| p.completed_tasks.iter().any(|ct| ct.id == id))
                    .unwrap_or(false);

                if is_completed {
                    // Un-complete: restore to previous state
                    if let Some(plan) = &mut self.day_plan {
                        plan.uncomplete_task(id);
                    }
                    self.modify_task(id, |task| {
                        task.state = TaskState::Next;
                        task.completed = None;
                    });
                } else {
                    // Complete: mark done in place (no archive)
                    if let Some(plan) = &mut self.day_plan {
                        let task_info = self.all_tasks_cache.iter().find(|t| t.id == id);
                        let title = task_info.map(|t| t.title.clone()).unwrap_or_default();
                        let esc = task_info.and_then(|t| t.esc);
                        plan.complete_task(id, title, esc);
                    }
                    self.modify_task(id, |task| {
                        task.state = TaskState::Done;
                        task.completed = Some(chrono::Local::now().naive_local());
                    });
                }
                self.save_day_plan();
                self.save_all();
                self.rebuild_cache();
            }

            Message::DoMarkListItemDone(id) => {
                // Archive and remove from master list + day plan
                if let Some(item) = self.media_items.iter().find(|i| i.id == id) {
                    if let Err(e) = OrgWriter::append_list_item_to_file(&self.config.consumed_path(), item) {
                        log::error!("Failed to archive media item: {}", e);
                    }
                    self.media_items.retain(|i| i.id != id);
                    self.save_media();
                }
                if let Some(item) = self.shopping_items.iter().find(|i| i.id == id) {
                    if let Err(e) = OrgWriter::append_list_item_to_file(&self.config.bought_path(), item) {
                        log::error!("Failed to archive shopping item: {}", e);
                    }
                    self.shopping_items.retain(|i| i.id != id);
                    self.save_shopping();
                }
                if let Some(ref mut plan) = self.day_plan {
                    plan.picked_media_ids.retain(|i| *i != id);
                    plan.picked_shopping_ids.retain(|i| *i != id);
                    self.save_day_plan();
                }
            }

            Message::SetAllTasksSort(col) => {
                self.all_tasks_sort = Some(match self.all_tasks_sort {
                    Some((c, asc)) if c == col => (col, !asc),
                    _ => (col, true),
                });
            }

            Message::Save => {
                self.save_all();
            }

            Message::OpenNewTaskForm => {
                self.new_task_form = NewTaskForm::default();
                self.context_drawer_state = Some(ContextDrawerState::NewTask);
                self.core.window.show_context = true;
            }

            Message::CloseNewTaskForm => {
                self.context_drawer_state = None;
                self.core.window.show_context = false;
                if self.launch_mode == LaunchMode::Capture {
                    std::process::exit(0);
                }
            }

            Message::CaptureFormTitle(value) => {
                self.new_task_form.title = value;
            }

            Message::CaptureFormState(state) => {
                self.new_task_form.state = state;
            }

            Message::CaptureFormPriority(priority) => {
                self.new_task_form.priority = priority;
            }

            Message::CaptureFormEsc(esc) => {
                self.new_task_form.esc = esc;
            }

            Message::CaptureFormToggleContext(ref ctx) => {
                if let Some(pos) = self.new_task_form.contexts.iter().position(|c| c == ctx) {
                    self.new_task_form.contexts.remove(pos);
                } else {
                    self.new_task_form.contexts.push(ctx.clone());
                }
            }

            Message::CaptureFormProject(project) => {
                self.new_task_form.project = project;
            }

            Message::CaptureFormScheduled(value) => {
                self.new_task_form.scheduled = value;
            }

            Message::CaptureFormDeadline(value) => {
                self.new_task_form.deadline = value;
            }

            Message::CaptureFormNotes(value) => {
                self.new_task_form.notes = value;
            }

            Message::CaptureFormSubmit => {
                let form = &self.new_task_form;
                let title = sentence_case(&form.title);
                if !title.is_empty() {
                    let mut task = Task::new(title);
                    task.state = form.state.clone();
                    task.priority = form.priority;
                    task.esc = form.esc;
                    task.contexts = form.contexts.clone();
                    task.project = form.project.clone();
                    task.scheduled = chrono::NaiveDate::parse_from_str(
                        form.scheduled.trim(), "%Y-%m-%d"
                    ).ok();
                    task.deadline = chrono::NaiveDate::parse_from_str(
                        form.deadline.trim(), "%Y-%m-%d"
                    ).ok();
                    task.notes = form.notes.trim().to_string();

                    if let Some(ref project_name) = task.project {
                        let project_name = project_name.clone();
                        if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                            project.tasks.push(task);
                        } else {
                            // Project doesn't exist, fall back to state-based routing
                            self.route_task_by_state(task);
                        }
                    } else {
                        self.route_task_by_state(task);
                    }
                    self.save_all();

                    // Close the drawer
                    self.context_drawer_state = None;
                    self.core.window.show_context = false;
                    if self.launch_mode == LaunchMode::Capture {
                        std::process::exit(0);
                    }
                }
            }

            // --- Sync messages (multi-account) ---
            Message::SyncNow => {
                if self.sync_status == SyncStatus::Syncing {
                    return CosmicTask::none();
                }
                self.sync_status = SyncStatus::Syncing;

                let mut batch: Vec<CosmicTask<Message>> = Vec::new();

                // CalDAV task/event sync (only if configured)
                if self.config.sync_ready() {
                    let tasks: Vec<Task> = self.all_active_tasks();
                    let events = self.events.clone();
                    let caldav_url = self.config.calendars.url.clone();
                    let task_cals = self.config.task_calendar_hrefs();
                    let event_cals = self.config.event_calendar_hrefs();
                    let sync_tokens = self.config.sync_tokens.clone();
                    let completions = std::mem::take(&mut self.pending_completions);
                    let deletions = std::mem::take(&mut self.pending_deletions);

                    batch.push(CosmicTask::perform(
                        async move {
                            let (username, password) = match crate::sync::keyring::load_credentials(&caldav_url).await {
                                Ok(Some(creds)) => creds,
                                Ok(None) => return Err("No CalDAV credentials stored".to_string()),
                                Err(e) => return Err(format!("Keyring error: {}", e)),
                            };
                            crate::sync::sync_all(
                                &caldav_url,
                                &username,
                                &password,
                                &tasks,
                                &events,
                                &task_cals,
                                &event_cals,
                                &sync_tokens,
                                &completions,
                                &deletions,
                            )
                            .await
                        },
                        |result| cosmic::Action::App(Message::SyncCompleted(result)),
                    ));
                }

                // WebDAV notes sync
                let notes_url = self.config.notes_sync.url.trim().to_string();
                if !notes_url.is_empty() {
                    let local_notes = self.notes.clone();
                    let notes_dir = self.config.notes_dir();
                    batch.push(CosmicTask::perform(
                        async move {
                            let (username, pw) = match crate::sync::keyring::load_credentials(&notes_url).await {
                                Ok(Some(creds)) => creds,
                                Ok(None) => return Err("No WebDAV credentials stored".to_string()),
                                Err(e) => return Err(format!("Keyring error: {}", e)),
                            };
                            let client = crate::sync::webdav::WebDavClient::new(
                                &notes_url, &username, &pw,
                            )?;
                            crate::sync::webdav::sync_notes(&client, &local_notes, &notes_dir).await
                        },
                        |result| cosmic::Action::App(Message::SyncNotesCompleted(result)),
                    ));
                }

                // CardDAV contacts sync
                let contacts_url = self.config.contacts.url.trim().to_string();
                if !contacts_url.is_empty() {
                    batch.push(CosmicTask::perform(
                        async move {
                            let (username, pw) = match crate::sync::keyring::load_credentials(&contacts_url).await {
                                Ok(Some((u, pw))) => (u, pw),
                                _ => return Err("No CardDAV credentials stored".to_string()),
                            };
                            let client = crate::sync::carddav::CardDavClient::new(
                                &contacts_url, &username, &pw,
                            )?;
                            client.fetch_contacts().await
                        },
                        |result| cosmic::Action::App(Message::ContactsFetched(result)),
                    ));
                }

                // IMAP email fetch
                let imap_host = self.config.imap.host.trim().to_string();
                if !imap_host.is_empty() {
                    let folder = if self.config.imap.folder.is_empty() {
                        "flup".to_string()
                    } else {
                        self.config.imap.folder.clone()
                    };
                    batch.push(CosmicTask::perform(
                        async move {
                            let keyring_key = format!("imap://{}", imap_host);
                            let (username, pw) = match crate::sync::keyring::load_credentials(&keyring_key).await {
                                Ok(Some(creds)) => creds,
                                _ => return Err("No IMAP credentials stored".to_string()),
                            };
                            crate::sync::imap::fetch_emails(&imap_host, &username, &pw, &folder).await
                        },
                        |result| cosmic::Action::App(Message::ImapFetched(result)),
                    ));
                }

                if batch.is_empty() {
                    self.sync_status = SyncStatus::default();
                    return CosmicTask::none();
                }
                return CosmicTask::batch(batch);
            }

            Message::SyncCompleted(result) => {
                match result {
                    Ok(sync_result) => {
                        // Update sync tokens
                        for (href, token) in &sync_result.new_sync_tokens {
                            self.config.set_sync_token(href, token);
                        }

                        // Remove deleted tasks
                        for id in &sync_result.deleted_local {
                            self.inbox_tasks.retain(|t| t.id != *id);
                            self.next_tasks.retain(|t| t.id != *id);
                            self.waiting_tasks.retain(|t| t.id != *id);
                            self.someday_tasks.retain(|t| t.id != *id);
                            for project in &mut self.projects {
                                project.tasks.retain(|t| t.id != *id);
                            }
                        }

                        // Apply pulled tasks (new + updated)
                        for pulled in &sync_result.pulled {
                            let _existing = self.remove_task(pulled.id);
                            if let Some(ref project_name) = pulled.project {
                                let project_name = project_name.clone();
                                if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                                    project.tasks.push(pulled.clone());
                                    continue;
                                }
                            }
                            self.route_task_by_state(pulled.clone());
                        }

                        // Remove deleted events
                        for id in &sync_result.deleted_events {
                            self.events.retain(|e| e.id != *id);
                        }

                        // Apply pulled events
                        for pulled_event in &sync_result.pulled_events {
                            self.events.retain(|e| e.id != pulled_event.id);
                            self.events.push(pulled_event.clone());
                        }

                        self.save_events();
                        self.save_all();
                        self.save_config();

                        self.sync_conflicts = sync_result.conflicts;

                        let now = chrono::Local::now().format("%H:%M").to_string();
                        self.sync_status = SyncStatus::LastSynced(now);

                        if !sync_result.errors.is_empty() {
                            log::warn!("Sync completed with errors: {:?}", sync_result.errors);
                        }
                    }
                    Err(e) => {
                        log::error!("Sync failed: {}", e);
                        self.sync_status = SyncStatus::Error(e);
                    }
                }
            }

            Message::SetServiceUrl(kind, url) => {
                match kind {
                    ServiceKind::Calendars => self.config.calendars.url = url,
                    ServiceKind::Contacts => self.config.contacts.url = url,
                    ServiceKind::Notes => self.config.notes_sync.url = url,
                    ServiceKind::Imap => self.config.imap.host = url,
                }
                self.save_config();
            }

            Message::SetServiceUsername(kind, username) => {
                match kind {
                    ServiceKind::Calendars => self.config.calendars.username = username,
                    ServiceKind::Contacts => self.config.contacts.username = username,
                    ServiceKind::Notes => self.config.notes_sync.username = username,
                    ServiceKind::Imap => self.config.imap.username = username,
                }
                self.save_config();
            }

            Message::SetServicePassword(kind, password) => {
                let idx = kind as usize;
                self.service_passwords[idx] = password;
            }

            Message::TestServiceConnection(kind) => {
                let idx = kind as usize;

                if kind == ServiceKind::Imap {
                    // IMAP uses host, not URL
                    let host = self.config.imap.host.trim().to_string();
                    if host.is_empty() {
                        self.service_test_status[idx] = Some(Err("IMAP host is required".to_string()));
                        return CosmicTask::none();
                    }
                    let username = self.config.imap.username.clone();
                    if username.is_empty() {
                        self.service_test_status[idx] = Some(Err("Username is required".to_string()));
                        return CosmicTask::none();
                    }
                    let password = self.service_passwords[idx].clone();
                    self.service_test_status[idx] = None;

                    return CosmicTask::perform(
                        async move {
                            let keyring_key = format!("imap://{}", host);
                            if !password.is_empty() {
                                let _ = crate::sync::keyring::store_credentials(
                                    &keyring_key, &username, &password,
                                ).await;
                            }
                            let pw = if !password.is_empty() {
                                password
                            } else {
                                match crate::sync::keyring::load_credentials(&keyring_key).await {
                                    Ok(Some((_, pw))) => pw,
                                    _ => return Err((kind, "No password available — enter an app password".to_string())),
                                }
                            };
                            let msg = crate::sync::imap::test_connection(&host, &username, &pw)
                                .await
                                .map_err(|e| (kind, e))?;
                            Ok((kind, msg, Vec::new()))
                        },
                        |result| {
                            match result {
                                Ok((kind, msg, cals)) => {
                                    cosmic::Action::App(Message::ServiceConnectionTested(kind, Ok(msg), cals))
                                }
                                Err((kind, e)) => {
                                    cosmic::Action::App(Message::ServiceConnectionTested(kind, Err(e), Vec::new()))
                                }
                            }
                        },
                    );
                }

                let svc = match kind {
                    ServiceKind::Calendars => &self.config.calendars,
                    ServiceKind::Contacts => &self.config.contacts,
                    ServiceKind::Notes => &self.config.notes_sync,
                    ServiceKind::Imap => unreachable!(),
                };
                let url = svc.url.trim().to_string();
                if url.is_empty() || !url.starts_with("https://") {
                    self.service_test_status[idx] = Some(Err("URL must start with https://".to_string()));
                    return CosmicTask::none();
                }
                let username = svc.username.clone();
                if username.is_empty() {
                    self.service_test_status[idx] = Some(Err("Username is required".to_string()));
                    return CosmicTask::none();
                }
                let password = self.service_passwords[idx].clone();
                self.service_test_status[idx] = None;

                return CosmicTask::perform(
                    async move {
                        // Store credentials if password provided
                        if !password.is_empty() {
                            let _ = crate::sync::keyring::store_credentials(
                                &url, &username, &password,
                            ).await;
                        }
                        let pw = if !password.is_empty() {
                            password
                        } else {
                            match crate::sync::keyring::load_credentials(&url).await {
                                Ok(Some((_, pw))) => pw,
                                _ => return Err((kind, "No password available — enter an app password".to_string())),
                            }
                        };

                        match kind {
                            ServiceKind::Calendars => {
                                let client = CalDavClient::new(&url, &username, &pw).map_err(|e| (kind, e))?;
                                let cals = client.discover_calendars().await.map_err(|e| (kind, e))?;
                                let msg = format!("Found {} calendars", cals.len());
                                Ok((kind, msg, cals))
                            }
                            ServiceKind::Contacts => {
                                let client = crate::sync::carddav::CardDavClient::new(&url, &username, &pw).map_err(|e| (kind, e))?;
                                let contacts = client.fetch_contacts().await.map_err(|e| (kind, e))?;
                                Ok((kind, format!("Connected ({} contacts)", contacts.len()), Vec::new()))
                            }
                            ServiceKind::Notes => {
                                let client = crate::sync::webdav::WebDavClient::new(&url, &username, &pw).map_err(|e| (kind, e))?;
                                client.ensure_collection().await.map_err(|e| (kind, format!("Collection error: {}", e)))?;
                                let files = client.list_files().await.map_err(|e| (kind, e))?;
                                Ok((kind, format!("Connected ({} files)", files.len()), Vec::new()))
                            }
                            ServiceKind::Imap => unreachable!(),
                        }
                    },
                    |result| {
                        match result {
                            Ok((kind, msg, cals)) => {
                                cosmic::Action::App(Message::ServiceConnectionTested(kind, Ok(msg), cals))
                            }
                            Err((kind, e)) => {
                                cosmic::Action::App(Message::ServiceConnectionTested(kind, Err(e), Vec::new()))
                            }
                        }
                    },
                );
            }

            Message::ServiceConnectionTested(kind, ref result, ref cals) => {
                let idx = kind as usize;
                self.service_test_status[idx] = Some(result.clone());

                if kind == ServiceKind::Calendars {
                    self.discovered_calendars = cals.clone();
                }
            }

            Message::SetCalendarPurpose(ref href, ref purpose) => {
                // Remove existing assignment for this href
                self.config
                    .calendar_assignments
                    .retain(|a| a.calendar_href != *href);
                // Add new assignment
                self.config
                    .calendar_assignments
                    .push(crate::config::CalendarAssignment {
                        calendar_href: href.clone(),
                        purpose: purpose.clone(),
                    });
                self.save_config();
            }

            Message::SyncNotesCompleted(result) => {
                match result {
                    Ok(sync_result) => {
                        // Apply pulled notes (new + updated from remote)
                        for pulled in &sync_result.pulled {
                            self.notes.retain(|n| n.id != pulled.id);
                            self.notes.push(pulled.clone());
                        }
                        self.notes.sort_by(|a, b| a.title.cmp(&b.title));
                        self.backlink_index = build_backlink_index(&self.notes);
                        // Save all notes (etags may have been updated)
                        self.save_all_notes();

                        log::info!(
                            "Notes sync: {} pulled, {} pushed",
                            sync_result.pulled.len(),
                            sync_result.pushed,
                        );
                        if !sync_result.errors.is_empty() {
                            log::warn!("Notes sync errors: {:?}", sync_result.errors);
                        }
                    }
                    Err(e) => {
                        log::error!("Notes sync failed: {}", e);
                    }
                }
            }

            Message::ContactsFetched(result) => {
                match result {
                    Ok(remote) => {
                        crate::sync::carddav::merge_contacts(&mut self.contacts, remote);
                        let _ = crate::sync::carddav::save_contacts(
                            &self.config.contacts_path(),
                            &self.contacts,
                        );
                    }
                    Err(e) => {
                        log::error!("Contact fetch failed: {}", e);
                    }
                }
            }

            Message::ContactDeleted(result) => {
                if let Err(e) = result {
                    log::error!("Failed to delete contact from server: {}", e);
                }
            }

            // --- IMAP email integration ---
            Message::ImapFetched(result) => {
                match result {
                    Ok(emails) => {
                        log::info!("IMAP: fetched {} emails", emails.len());
                        if self.archived_email_uids.is_empty() {
                            self.imap_emails = emails;
                        } else {
                            self.imap_emails = emails
                                .into_iter()
                                .filter(|e| !self.archived_email_uids.contains(&e.uid))
                                .collect();
                        }
                        // Clear old suggestions and auto-trigger batch analysis
                        self.email_suggestions.clear();
                        if !self.imap_emails.is_empty() {
                            return self.update(Message::SuggestEmailTasks);
                        }
                    }
                    Err(e) => {
                        log::error!("IMAP fetch failed: {}", e);
                    }
                }
                // Update sync status if CalDAV isn't also running
                if self.sync_status == SyncStatus::Syncing && !self.config.sync_ready() {
                    let now = chrono::Local::now().format("%H:%M").to_string();
                    self.sync_status = SyncStatus::LastSynced(now);
                }
            }

            Message::SuggestEmailTasks => {
                if self.imap_emails.is_empty() || self.ai_batch_processing {
                    return CosmicTask::none();
                }

                let emails: Vec<(u32, String, String, Option<String>, String)> = self
                    .imap_emails
                    .iter()
                    .map(|e| {
                        (
                            e.uid,
                            e.subject.clone(),
                            e.from.clone(),
                            e.date.map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
                            e.body_full.clone(),
                        )
                    })
                    .collect();
                let contexts = self.config.contexts.clone();
                let project_names: Vec<String> =
                    self.projects.iter().map(|p| p.name.clone()).collect();
                let existing_titles: Vec<String> =
                    self.all_tasks_cache.iter().map(|t| t.title.clone()).collect();
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();

                self.ai_batch_processing = true;

                return CosmicTask::perform(
                    async move {
                        let api_key = match crate::sync::anthropic::load_api_key().await {
                            Ok(Some(key)) => key,
                            _ => return Err("No Anthropic API key configured".to_string()),
                        };
                        crate::sync::anthropic::extract_tasks_from_emails_batch(
                            &api_key,
                            emails,
                            &contexts,
                            &project_names,
                            &existing_titles,
                            &today,
                        )
                        .await
                    },
                    |result| cosmic::Action::App(Message::BatchSuggestionsReady(result)),
                );
            }

            Message::BatchSuggestionsReady(result) => {
                self.ai_batch_processing = false;
                match result {
                    Ok(suggestions) => {
                        for (uid, suggestion) in suggestions {
                            if !suggestion.action_needed || suggestion.is_duplicate == Some(true) {
                                self.email_suggestions.insert(uid, EmailSuggestionState::NoAction);
                            } else {
                                self.email_suggestions
                                    .insert(uid, EmailSuggestionState::Suggested(suggestion));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Batch AI suggestion failed: {}", e);
                    }
                }
            }

            Message::ApproveSuggestion(uid) => {
                if let Some(EmailSuggestionState::Suggested(suggestion)) =
                    self.email_suggestions.remove(&uid)
                {
                    let title = suggestion
                        .title
                        .unwrap_or_else(|| "Untitled task".to_string());
                    let mut task = Task::new(&title);

                    // Priority
                    if let Some(ref p) = suggestion.priority {
                        task.priority = Priority::from_org(p);
                    }

                    // Contexts — only accept ones in our configured list
                    if let Some(ref ctxs) = suggestion.contexts {
                        task.contexts = ctxs
                            .iter()
                            .filter(|c| self.config.contexts.contains(c))
                            .cloned()
                            .collect();
                    }

                    // Deadline
                    if let Some(ref d) = suggestion.deadline {
                        task.deadline = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok();
                    }

                    // Scheduled
                    if let Some(ref s) = suggestion.scheduled {
                        task.scheduled = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok();
                    }

                    // Full email body as note
                    if let Some(email) = self.imap_emails.iter().find(|e| e.uid == uid) {
                        let mut note = format!("From: {}\n", email.from);
                        if let Some(date) = email.date {
                            note.push_str(&format!(
                                "Date: {}\n",
                                date.format("%Y-%m-%d %H:%M")
                            ));
                        }
                        if !email.body_full.is_empty() {
                            note.push('\n');
                            note.push_str(&email.body_full);
                        }
                        task.notes = note;
                    }

                    // Project assignment
                    if let Some(ref proj_name) = suggestion.project {
                        if let Some(project) =
                            self.projects.iter_mut().find(|p| p.name == *proj_name)
                        {
                            task.project = Some(proj_name.clone());
                            task.state = TaskState::Next;
                            project.tasks.push(task);
                            self.save_projects();
                        } else {
                            self.inbox_tasks.push(task);
                            self.save_inbox();
                        }
                    } else {
                        self.inbox_tasks.push(task);
                        self.save_inbox();
                    }

                    self.rebuild_cache();
                    return self.update(Message::ArchiveEmail(uid));
                }
            }

            Message::DismissSuggestion(uid) => {
                self.email_suggestions.insert(uid, EmailSuggestionState::Dismissed);
            }

            Message::SetAnthropicApiKey(key) => {
                self.anthropic_api_key_input = key;
            }

            Message::TestAnthropicApiKey => {
                let key = self.anthropic_api_key_input.clone();
                if key.is_empty() {
                    self.anthropic_test_status = Some(Err("No API key entered".to_string()));
                    return CosmicTask::none();
                }
                self.anthropic_test_status = None;
                return CosmicTask::perform(
                    async move {
                        // Store the key first
                        crate::sync::anthropic::store_api_key(&key).await?;
                        // Verify with a minimal API call
                        crate::sync::anthropic::test_api_key(&key).await
                    },
                    |result| cosmic::Action::App(Message::AnthropicKeyTested(result)),
                );
            }

            Message::AnthropicKeyTested(result) => {
                self.anthropic_test_status = Some(result);
            }

            Message::ArchiveEmail(uid) => {
                let host = self.config.imap.host.trim().to_string();
                let folder = if self.config.imap.folder.is_empty() {
                    "flup".to_string()
                } else {
                    self.config.imap.folder.clone()
                };
                return CosmicTask::perform(
                    async move {
                        let keyring_key = format!("imap://{}", host);
                        let (username, pw) = match crate::sync::keyring::load_credentials(&keyring_key).await {
                            Ok(Some(creds)) => creds,
                            _ => return Err("No IMAP credentials stored".to_string()),
                        };
                        crate::sync::imap::archive_email(&host, &username, &pw, &folder, uid).await
                    },
                    |result| cosmic::Action::App(Message::EmailArchived(result)),
                );
            }

            Message::EmailArchived(result) => {
                match result {
                    Ok(uid) => {
                        self.imap_emails.retain(|e| e.uid != uid);
                        self.archived_email_uids.insert(uid);
                        self.email_suggestions.remove(&uid);
                    }
                    Err(e) => {
                        log::error!("Failed to archive email: {}", e);
                    }
                }
            }

            Message::SetImapFolder(folder) => {
                self.config.imap.folder = folder;
                self.save_config();
            }

            // --- Event messages ---
            Message::CreateEvent => {
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                let now_time = chrono::Local::now().format("%H:%M").to_string();
                let default_cal = self
                    .config
                    .event_calendar_hrefs()
                    .first()
                    .cloned()
                    .unwrap_or_default();
                self.event_form = Some(EventForm {
                    editing: None,
                    title: String::new(),
                    start_date: today.clone(),
                    start_time: now_time.clone(),
                    end_date: today,
                    end_time: {
                        let hour = chrono::Local::now().hour();
                        format!("{:02}:00", (hour + 1) % 24)
                    },
                    all_day: false,
                    location: String::new(),
                    description: String::new(),
                    calendar_href: default_cal,
                });
            }

            Message::EditEvent(id) => {
                if let Some(ev) = self.events.iter().find(|e| e.id == id) {
                    self.event_form = Some(EventForm::from_event(ev));
                }
            }

            Message::CancelEventForm => {
                self.event_form = None;
            }

            Message::SetEventTitle(value) => {
                if let Some(ref mut form) = self.event_form {
                    form.title = value;
                }
            }

            Message::SetEventStart(value) => {
                if let Some(ref mut form) = self.event_form {
                    if let Some((d, t)) = value.split_once(' ') {
                        form.start_date = d.to_string();
                        form.start_time = t.to_string();
                    } else if value.contains(':') {
                        form.start_time = value;
                    } else {
                        form.start_date = value;
                    }
                }
            }

            Message::SetEventEnd(value) => {
                if let Some(ref mut form) = self.event_form {
                    if let Some((d, t)) = value.split_once(' ') {
                        form.end_date = d.to_string();
                        form.end_time = t.to_string();
                    } else if value.contains(':') {
                        form.end_time = value;
                    } else {
                        form.end_date = value;
                    }
                }
            }

            Message::SetEventAllDay(all_day) => {
                if let Some(ref mut form) = self.event_form {
                    form.all_day = all_day;
                }
            }

            Message::SetEventLocation(value) => {
                if let Some(ref mut form) = self.event_form {
                    form.location = value;
                }
            }

            Message::SetEventDescription(value) => {
                if let Some(ref mut form) = self.event_form {
                    form.description = value;
                }
            }

            Message::SetEventCalendar(href) => {
                if let Some(ref mut form) = self.event_form {
                    form.calendar_href = href;
                }
            }

            Message::SubmitEvent => {
                if let Some(form) = self.event_form.take() {
                    let title = form.title.trim().to_string();
                    if title.is_empty() {
                        return CosmicTask::none();
                    }
                    let start = parse_form_datetime(&form.start_date, &form.start_time, form.all_day);
                    let end = parse_form_datetime(&form.end_date, &form.end_time, form.all_day);
                    if let (Some(start), Some(end)) = (start, end) {
                        let cal_name = self.all_discovered_calendars().iter()
                            .find(|c| c.href == form.calendar_href)
                            .map(|c| c.display_name.clone())
                            .unwrap_or_default();
                        let mut ev = CalendarEvent::new(title, start, end);
                        ev.all_day = form.all_day;
                        ev.location = form.location;
                        ev.description = form.description;
                        ev.calendar_href = form.calendar_href;
                        ev.calendar_name = cal_name;
                        self.events.push(ev);
                        self.save_events();
                    }
                }
            }

            Message::UpdateEvent(id) => {
                if let Some(form) = self.event_form.take() {
                    let title = form.title.trim().to_string();
                    if title.is_empty() {
                        return CosmicTask::none();
                    }
                    let start = parse_form_datetime(&form.start_date, &form.start_time, form.all_day);
                    let end = parse_form_datetime(&form.end_date, &form.end_time, form.all_day);
                    if let (Some(start), Some(end)) = (start, end) {
                        if let Some(ev) = self.events.iter_mut().find(|e| e.id == id) {
                            ev.title = title;
                            ev.start = start;
                            ev.end = end;
                            ev.all_day = form.all_day;
                            ev.location = form.location;
                            ev.description = form.description;
                            ev.calendar_href = form.calendar_href;
                        }
                        self.save_events();
                    }
                }
            }

            Message::DeleteEvent(id) => {
                // Delete from CalDAV if synced — find the account that owns this event
                if let Some(ev) = self.events.iter().find(|e| e.id == id) {
                    if let Some(ref sync_href) = ev.sync_href {
                        let caldav_url = self.config.calendars.url.clone();
                        if !caldav_url.is_empty() {
                            let href = sync_href.clone();
                            let _ = CosmicTask::perform(
                                async move {
                                    let creds =
                                        crate::sync::keyring::load_credentials(&caldav_url).await;
                                    if let Ok(Some((username, pw))) = creds {
                                        if let Ok(client) =
                                            CalDavClient::new(&caldav_url, &username, &pw)
                                        {
                                            let _ = client.delete_vtodo(&href, "").await;
                                        }
                                    }
                                    Ok::<(), String>(())
                                },
                                |_| cosmic::Action::App(Message::Save),
                            );
                        }
                    }
                }
                self.events.retain(|e| e.id != id);
                self.save_events();
            }

            // Month calendar navigation
            Message::CalendarPrevMonth => {
                self.month_calendar.prev_month();
            }

            Message::CalendarNextMonth => {
                self.month_calendar.next_month();
            }

            Message::CalendarSelectDay(date) => {
                self.month_calendar.select_day(date);
            }

            // Conflict resolution
            Message::ImportConflictTask(idx) => {
                if idx < self.sync_conflicts.len() {
                    if let SyncConflict::RemoteOnly { task, .. } =
                        self.sync_conflicts.remove(idx)
                    {
                        self.inbox_tasks.push(task);
                        self.save_all();
                        self.rebuild_cache();
                    }
                }
            }

            Message::DeleteConflict(idx) => {
                if idx < self.sync_conflicts.len() {
                    match self.sync_conflicts.remove(idx) {
                        SyncConflict::RemoteOnly { href, .. } => {
                            // Queue deletion from server on next sync
                            self.pending_deletions.push(href);
                        }
                        SyncConflict::LocalOnly { task_id, .. } => {
                            // Delete the local task
                            self.remove_task(task_id);
                            self.save_all();
                            self.rebuild_cache();
                        }
                        SyncConflict::StateMismatch { .. } => {}
                    }
                }
            }

            Message::AcceptRemoteState(idx) => {
                if idx < self.sync_conflicts.len() {
                    if let SyncConflict::StateMismatch {
                        task_id,
                        remote_state,
                        ..
                    } = self.sync_conflicts.remove(idx)
                    {
                        if let Some(new_state) =
                            crate::core::task::TaskState::from_keyword(&remote_state)
                        {
                            if let Some(mut task) = self.remove_task(task_id) {
                                task.state = new_state;
                                self.route_task_by_state(task);
                                self.save_all();
                                self.rebuild_cache();
                            }
                        }
                    }
                }
            }

            Message::AcceptLocalState(idx) => {
                if idx < self.sync_conflicts.len() {
                    if let SyncConflict::StateMismatch {
                        task_id, href, ..
                    } = self.sync_conflicts.remove(idx)
                    {
                        // Find the local task and queue its current state for push on next sync
                        let all = self.all_active_tasks();
                        if let Some(task) = all.iter().find(|t| t.id == task_id) {
                            let ical =
                                crate::sync::vtodo::task_to_vcalendar(task);
                            self.pending_completions.push((href, ical));
                        }
                    }
                }
            }

            _ => {}
        }

        CosmicTask::none()
    }

    fn header_end(&self) -> Vec<Element<'_, Message>> {
        let mut header_row = row()
            .spacing(4)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::OpenNewTaskForm),
            );

        // Sync button (only if sync is configured)
        if self.config.sync_ready() {
            let sync_icon = match self.sync_status {
                SyncStatus::Syncing => "emblem-synchronizing-symbolic",
                SyncStatus::Error(_) => "dialog-warning-symbolic",
                _ => "emblem-synchronizing-symbolic",
            };
            let sync_btn = button::icon(icon::from_name(sync_icon))
                .on_press(Message::SyncNow);
            header_row = header_row.push(sync_btn);
        }

        header_row = header_row.push(
            button::icon(icon::from_name("emblem-system-symbolic"))
                .on_press(Message::OpenSettings),
        );

        vec![header_row.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Message>> {
        let drawer_state = self.context_drawer_state?;

        match drawer_state {
            ContextDrawerState::NewTask => {
                Some(context_drawer::context_drawer(
                    container(scrollable(self.capture_form_view().padding(16)))
                        .width(Length::Fill),
                    Message::CloseNewTaskForm,
                ).title("New Task"))
            }
        }
    }

    fn on_escape(&mut self) -> CosmicTask<Message> {
        if self.context_drawer_state == Some(ContextDrawerState::NewTask) {
            self.context_drawer_state = None;
            self.core.window.show_context = false;
            if self.launch_mode == LaunchMode::Capture {
                std::process::exit(0);
            }
        }
        CosmicTask::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        cosmic::iced::event::listen_with(|event, _status, _id| {
            match event {
                cosmic::iced::Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
                    key: cosmic::iced::keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                }) if c.as_str() == "n" && modifiers.control() => {
                    Some(Message::OpenNewTaskForm)
                }
                _ => None,
            }
        })
    }

    fn view(&self) -> Element<'_, Message> {
        match self.app_mode {
            AppMode::Plan => self.plan_view(),
            AppMode::Do => self.do_view(),
        }
    }
}

impl Lamp {
    fn capture_form_view(&self) -> column::Column<'_, Message> {
        let form = &self.new_task_form;
        let mut content = column().spacing(16);

        // Title
        content = content.push(text::title4("Title"));
        content = content.push(
            text_input::text_input("Task title...", &form.title)
                .on_input(Message::CaptureFormTitle)
                .on_submit(|_| Message::CaptureFormSubmit)
                .width(Length::Fill),
        );

        // State
        content = content.push(text::title4("State"));
        let state_row = row()
            .spacing(4)
            .push(state_button("Todo", TaskState::Todo, &form.state))
            .push(state_button("Next", TaskState::Next, &form.state))
            .push(state_button("Waiting", TaskState::Waiting, &form.state))
            .push(state_button("Someday", TaskState::Someday, &form.state));
        content = content.push(state_row);

        // Priority
        content = content.push(text::title4("Priority"));
        let priority_row = row()
            .spacing(4)
            .push(priority_button("-", None, &form.priority))
            .push(priority_button("A", Some(Priority::A), &form.priority))
            .push(priority_button("B", Some(Priority::B), &form.priority))
            .push(priority_button("C", Some(Priority::C), &form.priority));
        content = content.push(priority_row);

        // ESC (energy/spoon cost)
        content = content.push(text::title4("Energy Cost"));
        let mut esc_items: Vec<Element<'_, Message>> = vec![esc_button("-", None, &form.esc)];
        for val in [5, 10, 15, 20, 25, 30, 40, 50, 75, 100] {
            esc_items.push(esc_button(&val.to_string(), Some(val), &form.esc));
        }
        content = content.push(flex_row(esc_items).row_spacing(4).column_spacing(4));

        // Contexts
        if !self.config.contexts.is_empty() {
            content = content.push(text::title4("Contexts"));
            let mut ctx_items: Vec<Element<'_, Message>> = Vec::new();
            for ctx in &self.config.contexts {
                let active = form.contexts.contains(ctx);
                let btn: Element<'_, Message> = if active {
                    button::suggested(ctx.as_str())
                        .on_press(Message::CaptureFormToggleContext(ctx.clone()))
                        .into()
                } else {
                    button::standard(ctx.as_str())
                        .on_press(Message::CaptureFormToggleContext(ctx.clone()))
                        .into()
                };
                ctx_items.push(btn);
            }
            content = content.push(flex_row(ctx_items).row_spacing(4).column_spacing(4));
        }

        // Project
        let project_names: Vec<String> = self.projects.iter().map(|p| p.name.clone()).collect();
        if !project_names.is_empty() {
            content = content.push(text::title4("Project"));
            let mut proj_items: Vec<Element<'_, Message>> = Vec::new();
            let none_btn: Element<'_, Message> = if form.project.is_none() {
                button::suggested("None")
                    .on_press(Message::CaptureFormProject(None))
                    .into()
            } else {
                button::standard("None")
                    .on_press(Message::CaptureFormProject(None))
                    .into()
            };
            proj_items.push(none_btn);
            for name in &project_names {
                let active = form.project.as_ref() == Some(name);
                let btn: Element<'_, Message> = if active {
                    button::suggested(name.clone())
                        .on_press(Message::CaptureFormProject(Some(name.clone())))
                        .into()
                } else {
                    button::standard(name.clone())
                        .on_press(Message::CaptureFormProject(Some(name.clone())))
                        .into()
                };
                proj_items.push(btn);
            }
            content = content.push(flex_row(proj_items).row_spacing(4).column_spacing(4));
        }

        // Scheduled
        content = content.push(text::title4("Scheduled"));
        content = content.push(
            text_input::text_input("YYYY-MM-DD", &form.scheduled)
                .on_input(Message::CaptureFormScheduled)
                .width(Length::Fill),
        );

        // Deadline
        content = content.push(text::title4("Deadline"));
        content = content.push(
            text_input::text_input("YYYY-MM-DD", &form.deadline)
                .on_input(Message::CaptureFormDeadline)
                .width(Length::Fill),
        );

        // Notes
        content = content.push(text::title4("Notes"));
        content = content.push(
            text_input::text_input("Notes...", &form.notes)
                .on_input(Message::CaptureFormNotes)
                .width(Length::Fill),
        );

        // Submit button
        content = content.push(
            button::suggested("Create Task")
                .on_press(Message::CaptureFormSubmit)
                .width(Length::Fill),
        );

        content
    }

    fn route_task_by_state(&mut self, task: Task) {
        match task.state {
            TaskState::Todo => self.inbox_tasks.push(task),
            TaskState::Next => self.next_tasks.push(task),
            TaskState::Waiting => self.waiting_tasks.push(task),
            TaskState::Someday => self.someday_tasks.push(task),
            _ => self.inbox_tasks.push(task),
        }
    }

    fn plan_view(&self) -> Element<'_, Message> {
        let project_names: Vec<String> = self.projects.iter().map(|p| p.name.clone()).collect();
        let row_ctx = crate::components::task_row::TaskRowCtx {
            contexts: &self.config.contexts,
            project_names: &project_names,
            expanded_task: self.expanded_task,
            note_inputs: &self.note_inputs,
            waiting_for_inputs: &self.waiting_for_inputs,
            contacts: &self.contacts,
        };

        let what = match self.active_view {
            ActiveView::What(w) => w,
            ActiveView::When(_) => WhatPage::DailyPlanning,
        };

        let searchable = matches!(
            what,
            WhatPage::Inbox
                | WhatPage::AllTasks
                | WhatPage::NextActions
                | WhatPage::Projects
                | WhatPage::Waiting
                | WhatPage::Someday
                | WhatPage::Habits
                | WhatPage::Media
                | WhatPage::Shopping
                | WhatPage::Contacts
                | WhatPage::Accounts
                | WhatPage::Notes
        );

        // Pre-filter data when search is active
        let q = &self.search_query;
        let filtered_tasks: Vec<Task>;
        let tasks: &[Task] = if !q.is_empty() && searchable {
            let lq = q.to_lowercase();
            filtered_tasks = self
                .all_tasks_cache
                .iter()
                .filter(|t| t.title.to_lowercase().contains(&lq))
                .cloned()
                .collect();
            &filtered_tasks
        } else {
            &self.all_tasks_cache
        };

        let filtered_projects: Vec<Project>;
        let projects: &[Project] = if !q.is_empty() && what == WhatPage::Projects {
            let lq = q.to_lowercase();
            filtered_projects = self
                .projects
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&lq)
                        || p.tasks.iter().any(|t| t.title.to_lowercase().contains(&lq))
                })
                .cloned()
                .collect();
            &filtered_projects
        } else {
            &self.projects
        };

        let filtered_habits: Vec<Habit>;
        let habits: &[Habit] = if !q.is_empty() && what == WhatPage::Habits {
            let lq = q.to_lowercase();
            filtered_habits = self
                .habits
                .iter()
                .filter(|h| h.task.title.to_lowercase().contains(&lq))
                .cloned()
                .collect();
            &filtered_habits
        } else {
            &self.habits
        };

        let filtered_media: Vec<ListItem>;
        let media: &[ListItem] = if !q.is_empty() && what == WhatPage::Media {
            let lq = q.to_lowercase();
            filtered_media = self
                .media_items
                .iter()
                .filter(|i| i.title.to_lowercase().contains(&lq))
                .cloned()
                .collect();
            &filtered_media
        } else {
            &self.media_items
        };

        let filtered_shopping: Vec<ListItem>;
        let shopping: &[ListItem] = if !q.is_empty() && what == WhatPage::Shopping {
            let lq = q.to_lowercase();
            filtered_shopping = self
                .shopping_items
                .iter()
                .filter(|i| i.title.to_lowercase().contains(&lq))
                .cloned()
                .collect();
            &filtered_shopping
        } else {
            &self.shopping_items
        };

        let filtered_contacts: Vec<Contact>;
        let contacts_filtered: &[Contact] = if !q.is_empty() && what == WhatPage::Contacts {
            let lq = q.to_lowercase();
            filtered_contacts = self
                .contacts
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&lq))
                .cloned()
                .collect();
            &filtered_contacts
        } else {
            &self.contacts
        };

        let filtered_accounts: Vec<Account>;
        let accounts_filtered: &[Account] = if !q.is_empty() && what == WhatPage::Accounts {
            let lq = q.to_lowercase();
            filtered_accounts = self
                .accounts
                .iter()
                .filter(|a| a.name.to_lowercase().contains(&lq))
                .cloned()
                .collect();
            &filtered_accounts
        } else {
            &self.accounts
        };

        let filtered_notes: Vec<Note>;
        let notes_filtered: &[Note] = if !q.is_empty() && what == WhatPage::Notes {
            let lq = q.to_lowercase();
            filtered_notes = self
                .notes
                .iter()
                .filter(|n| {
                    n.title.to_lowercase().contains(&lq)
                        || n.body.to_lowercase().contains(&lq)
                        || n.tags.iter().any(|t| t.to_lowercase().contains(&lq))
                })
                .cloned()
                .collect();
            &filtered_notes
        } else {
            &self.notes
        };

        let content: Element<'_, Message> = match what {
                WhatPage::DailyPlanning => {
                    pages::daily_planning::daily_planning_view(
                        &self.day_plan,
                        &self.all_tasks_cache,
                        &self.media_items,
                        &self.shopping_items,
                        &self.config.contexts,
                        &self.rejected_suggestions,
                    )
                }
                WhatPage::Inbox => {
                    pages::inbox::inbox_view(
                        tasks,
                        &self.imap_emails,
                        &self.inbox_input,
                        &row_ctx,
                        &self.email_suggestions,
                        self.ai_batch_processing,
                    )
                }
                WhatPage::AllTasks => {
                    pages::all_tasks::all_tasks_view(
                        tasks,
                        &row_ctx,
                        self.all_tasks_sort,
                    )
                }
                WhatPage::NextActions => {
                    pages::next_actions::next_actions_view(
                        tasks,
                        &row_ctx,
                    )
                }
                WhatPage::Projects => {
                    pages::projects::projects_view(
                        projects,
                        &self.project_input,
                        &self.project_task_inputs,
                        &row_ctx,
                    )
                }
                WhatPage::Waiting => {
                    pages::waiting::waiting_view(
                        tasks,
                        &row_ctx,
                    )
                }
                WhatPage::Someday => {
                    pages::someday::someday_view(
                        tasks,
                        &row_ctx,
                    )
                }
                WhatPage::Habits => {
                    pages::habits::habits_view(habits, &self.habit_input)
                }
                WhatPage::Conflicts => {
                    pages::conflicts::conflicts_view(&self.sync_conflicts)
                }
                WhatPage::Review => {
                    pages::review::review_view(
                        &self.all_tasks_cache,
                        &self.projects,
                        &self.habits,
                        &self.review_checked,
                    )
                }
                WhatPage::Tickler => {
                    let flat_cals = self.all_discovered_calendars();
                    pages::temporal::agenda_view(
                        &self.all_tasks_cache,
                        &self.habits,
                        &self.events,
                        self.event_form.as_ref(),
                        &row_ctx,
                        &flat_cals,
                        &self.month_calendar,
                    )
                }
                WhatPage::Media => {
                    pages::list::list_view(
                        media,
                        &self.media_input,
                        crate::fl!("media-placeholder"),
                        crate::fl!("media-empty"),
                        ListKind::Media,
                        &self.flipped_list_items,
                        self.pending_delete_list_item,
                        &self.note_inputs,
                    )
                }
                WhatPage::Shopping => {
                    pages::list::list_view(
                        shopping,
                        &self.shopping_input,
                        crate::fl!("shopping-placeholder"),
                        crate::fl!("shopping-empty"),
                        ListKind::Shopping,
                        &self.flipped_list_items,
                        self.pending_delete_list_item,
                        &self.note_inputs,
                    )
                }
                WhatPage::Contacts => {
                    pages::contacts::contacts_view(
                        contacts_filtered,
                        &self.contact_input,
                        &self.flipped_contacts,
                        self.editing_contact,
                        self.pending_delete_contact,
                    )
                }
                WhatPage::Accounts => {
                    pages::accounts::accounts_view(
                        accounts_filtered,
                        &self.account_input,
                        self.expanded_account,
                        self.pending_delete_account,
                    )
                }
                WhatPage::Notes => {
                    pages::notes::notes_view(
                        notes_filtered,
                        &self.notes,
                        &self.note_input,
                        &self.flipped_notes,
                        self.editing_note,
                        self.pending_delete_note,
                        &self.note_editor_content,
                        &self.note_edit_buffer,
                        &self.note_link_search,

                        &self.backlink_index,
                        &self.contacts,
                        &self.accounts,
                        &self.projects,
                        &self.all_tasks_cache,
                        &self.media_items,
                        &self.shopping_items,
                    )
                }
                WhatPage::Settings => {
                    pages::settings::settings_view(
                        &self.config,
                        &self.settings_context_input,
                        &self.service_passwords,
                        &self.service_test_status,
                        &self.discovered_calendars,
                        &self.anthropic_api_key_input,
                        &self.anthropic_test_status,
                        &self.sync_status,
                    )
                }
        };

        if searchable {
            let search_input = text_input::text_input(
                crate::fl!("search-placeholder"),
                self.search_query.clone(),
            )
            .on_input(Message::SearchQueryChanged)
            .width(Length::Fill);

            container(
                column()
                    .spacing(8)
                    .push(container(search_input).padding([0, 16]))
                    .push(content),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn do_view(&self) -> Element<'_, Message> {
        pages::do_mode::do_mode_view(
            &self.day_plan,
            &self.all_tasks_cache,
            &self.habits,
            &self.media_items,
            &self.shopping_items,
            self.expanded_task,
            &self.note_inputs,
        )
    }

    fn rebuild_cache(&mut self) {
        self.all_tasks_cache = self.all_active_tasks();
    }

    fn all_active_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        tasks.extend(self.inbox_tasks.iter().cloned());
        tasks.extend(self.next_tasks.iter().cloned());
        tasks.extend(self.waiting_tasks.iter().cloned());
        tasks.extend(self.someday_tasks.iter().cloned());
        for project in &self.projects {
            tasks.extend(project.tasks.iter().cloned());
        }
        tasks
    }

    fn toggle_done(&mut self, id: uuid::Uuid) {
        // Find the task, complete it, archive it, and remove from source
        if let Some(mut task) = self.remove_task(id) {
            if task.state.is_done() {
                // Un-completing: put back as Todo
                task.state = crate::core::task::TaskState::Todo;
                task.completed = None;
                self.route_task_by_state(task);
            } else {
                // Completing: mark done and archive
                task.complete();

                // Queue CalDAV completion push for next sync
                if let Some(ref sync_href) = task.sync_href {
                    let ical = crate::sync::vtodo::task_to_vcalendar(&task);
                    log::info!("Queuing completion for sync: {}", sync_href);
                    self.pending_completions.push((sync_href.clone(), ical));
                }

                let archive_path = self.config.archive_path();
                if OrgWriter::append_to_file(&archive_path, &task).is_err() {
                    log::error!("Failed to archive task, keeping in source list");
                    self.route_task_by_state(task);
                }

                self.save_all();
                return;
            }
            self.save_all();
        }
    }

    fn set_task_state(&mut self, id: uuid::Uuid, state: crate::core::task::TaskState) {
        use crate::core::task::TaskState;

        if let Some(mut task) = self.remove_task(id) {
            let old_state = task.state.clone();
            task.state = state.clone();

            // Auto-stamp delegated date when entering Waiting state
            if state == TaskState::Waiting && old_state != TaskState::Waiting {
                if task.delegated.is_none() {
                    task.delegated = Some(chrono::Local::now().date_naive());
                }
            }
            // Clear delegated/follow_up when leaving Waiting state
            if state != TaskState::Waiting && old_state == TaskState::Waiting {
                task.delegated = None;
                task.follow_up = None;
            }

            // If task belongs to a project, put it back there with the new state
            if let Some(ref project_name) = task.project {
                let project_name = project_name.clone();
                if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                    project.tasks.push(task);
                }
            } else {
                match state {
                    TaskState::Todo => self.inbox_tasks.push(task),
                    TaskState::Next => self.next_tasks.push(task),
                    TaskState::Waiting => self.waiting_tasks.push(task),
                    TaskState::Someday => self.someday_tasks.push(task),
                    _ => self.inbox_tasks.push(task),
                }
            }
            self.save_all();
        }
    }

    fn set_task_priority(&mut self, id: uuid::Uuid, priority: Option<crate::core::task::Priority>) {
        fn set_in_list(list: &mut [Task], id: uuid::Uuid, priority: Option<crate::core::task::Priority>) -> bool {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                task.priority = priority;
                return true;
            }
            false
        }

        let mut found = set_in_list(&mut self.inbox_tasks, id, priority)
            || set_in_list(&mut self.next_tasks, id, priority)
            || set_in_list(&mut self.waiting_tasks, id, priority)
            || set_in_list(&mut self.someday_tasks, id, priority);

        if !found {
            for project in &mut self.projects {
                if set_in_list(&mut project.tasks, id, priority) {
                    found = true;
                    break;
                }
            }
        }

        if found {
            self.save_all();
        }
    }

    /// Find a task across all lists, apply a mutation, and save.
    fn modify_task(&mut self, id: uuid::Uuid, f: impl FnOnce(&mut Task)) {
        fn find_and_modify(list: &mut [Task], id: uuid::Uuid, f: &mut Option<impl FnOnce(&mut Task)>) -> bool {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                if let Some(func) = f.take() {
                    func(task);
                }
                return true;
            }
            false
        }

        let mut f = Some(f);
        let mut found = find_and_modify(&mut self.inbox_tasks, id, &mut f)
            || find_and_modify(&mut self.next_tasks, id, &mut f)
            || find_and_modify(&mut self.waiting_tasks, id, &mut f)
            || find_and_modify(&mut self.someday_tasks, id, &mut f);

        if !found {
            for project in &mut self.projects {
                if find_and_modify(&mut project.tasks, id, &mut f) {
                    found = true;
                    break;
                }
            }
        }

        if found {
            self.save_all();
        }
    }

    fn remove_task(&mut self, id: uuid::Uuid) -> Option<Task> {
        for list in [
            &mut self.inbox_tasks,
            &mut self.next_tasks,
            &mut self.waiting_tasks,
            &mut self.someday_tasks,
        ] {
            if let Some(pos) = list.iter().position(|t| t.id == id) {
                return Some(list.remove(pos));
            }
        }
        for project in &mut self.projects {
            if let Some(pos) = project.tasks.iter().position(|t| t.id == id) {
                return Some(project.tasks.remove(pos));
            }
        }
        None
    }

    fn ensure_day_plan(&mut self) -> &mut DayPlan {
        let today = chrono::Local::now().date_naive();
        if self.day_plan.as_ref().is_none_or(|dp| dp.is_stale(today)) {
            self.day_plan = Some(DayPlan::new(today));
            self.rejected_suggestions.clear();
        }
        self.day_plan.as_mut().unwrap()
    }

    fn save_day_plan(&self) {
        if let Some(ref plan) = self.day_plan {
            let content = OrgWriter::write_day_plan(plan);
            if let Err(e) = std::fs::write(self.config.dayplan_path(), &content) {
                log::error!("Failed to save day plan: {}", e);
            }
        }
    }

    fn save_inbox(&mut self) {
        let content = OrgWriter::write_file("Inbox", &self.inbox_tasks);
        if let Err(e) = std::fs::write(self.config.inbox_path(), &content) {
            log::error!("Failed to save inbox: {}", e);
        }
        self.rebuild_cache();
    }

    fn save_all(&mut self) {
        let saves: Vec<(&str, &[Task], std::path::PathBuf)> = vec![
            ("Inbox", &self.inbox_tasks, self.config.inbox_path()),
            ("Next Actions", &self.next_tasks, self.config.next_path()),
            ("Waiting For", &self.waiting_tasks, self.config.waiting_path()),
            ("Someday/Maybe", &self.someday_tasks, self.config.someday_path()),
        ];

        for (title, tasks, path) in saves {
            let content = OrgWriter::write_file(title, tasks);
            if let Err(e) = std::fs::write(&path, &content) {
                log::error!("Failed to save {}: {}", title, e);
            }
        }
        self.save_projects();
        self.save_habits();
        self.rebuild_cache();
    }

    fn save_projects(&self) {
        let content = OrgWriter::write_projects_file(&self.projects);
        if let Err(e) = std::fs::write(self.config.projects_path(), &content) {
            log::error!("Failed to save projects: {}", e);
        }
    }

    fn save_media(&self) {
        let content = OrgWriter::write_list_items_file("Media Recommendations", &self.media_items);
        if let Err(e) = std::fs::write(self.config.media_path(), &content) {
            log::error!("Failed to save media: {}", e);
        }
    }

    fn save_shopping(&self) {
        let content = OrgWriter::write_list_items_file("Shopping", &self.shopping_items);
        if let Err(e) = std::fs::write(self.config.shopping_path(), &content) {
            log::error!("Failed to save shopping: {}", e);
        }
    }

    fn save_contacts(&self) {
        if let Err(e) = crate::sync::carddav::save_contacts(&self.config.contacts_path(), &self.contacts) {
            log::error!("Failed to save contacts: {}", e);
        }
    }

    fn save_accounts(&self) {
        let content = OrgWriter::write_accounts_file(&self.accounts);
        if let Err(e) = std::fs::write(self.config.accounts_path(), &content) {
            log::error!("Failed to save accounts: {}", e);
        }
    }

    fn save_note(&self, note: &Note) {
        let filename = format!("{}.org", note.id);
        let path = self.config.notes_dir().join(&filename);
        let content = OrgWriter::write_note_file(note);
        if let Err(e) = std::fs::write(&path, &content) {
            log::error!("Failed to save note {}: {}", filename, e);
        }
    }

    fn save_all_notes(&self) {
        for note in &self.notes {
            self.save_note(note);
        }
    }

    fn delete_note_file(&self, id: uuid::Uuid) {
        let filename = format!("{}.org", id);
        let path = self.config.notes_dir().join(&filename);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                log::error!("Failed to delete note file {}: {}", filename, e);
            }
        }
    }

    fn save_habits(&self) {
        let mut out = String::new();
        out.push_str("#+TITLE: Habits\n");
        out.push_str("#+TODO: TODO NEXT WAITING SOMEDAY | DONE CANCELLED\n\n");
        for habit in &self.habits {
            out.push_str(&OrgWriter::write_habit_task(&habit.task, &habit.completions));
            out.push('\n');
        }
        if let Err(e) = std::fs::write(self.config.habits_path(), &out) {
            log::error!("Failed to save habits: {}", e);
        }
    }

    fn save_events(&self) {
        event::save_events(&self.config.events_cache_path(), &self.events);
    }

    /// All discovered calendars.
    fn all_discovered_calendars(&self) -> Vec<CalendarInfo> {
        self.discovered_calendars.clone()
    }

    fn save_config(&self) {
        use cosmic::cosmic_config::CosmicConfigEntry;
        if let Err(e) = self.config.write_entry(&self.cosmic_config) {
            log::error!("Failed to save config: {:?}", e);
        }
    }
}

fn parse_form_datetime(
    date_str: &str,
    time_str: &str,
    all_day: bool,
) -> Option<chrono::NaiveDateTime> {
    let date = chrono::NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d").ok()?;
    if all_day {
        Some(date.and_hms_opt(0, 0, 0).unwrap())
    } else {
        let time = chrono::NaiveTime::parse_from_str(time_str.trim(), "%H:%M").ok()?;
        Some(date.and_time(time))
    }
}

/// Sentence-case: first letter uppercase, rest lowercase.
fn sentence_case(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    first + &chars.as_str().to_lowercase()
}

fn load_tasks(path: &std::path::Path) -> Vec<Task> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_tasks(&content),
        Err(_) => Vec::new(),
    }
}

fn load_habits(path: &std::path::Path) -> Vec<Habit> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_habits(&content),
        Err(_) => Vec::new(),
    }
}

fn load_projects(path: &std::path::Path) -> Vec<crate::core::project::Project> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_projects(&content),
        Err(_) => Vec::new(),
    }
}

fn load_list_items(path: &std::path::Path) -> Vec<ListItem> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_list_items(&content),
        Err(_) => Vec::new(),
    }
}

fn load_day_plan(path: &std::path::Path) -> Option<DayPlan> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| convert::parse_day_plan(&content))
}

fn load_accounts(path: &std::path::Path) -> Vec<Account> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_accounts(&content),
        Err(_) => Vec::new(),
    }
}

/// Load notes from the per-file notes directory.
/// Falls back to migrating from the old monolithic notes.org if present.
fn load_notes_dir(notes_dir: &std::path::Path, old_notes_path: &std::path::Path) -> Vec<Note> {
    let dir_has_files = notes_dir
        .read_dir()
        .ok()
        .map(|mut rd| rd.any(|e| e.is_ok_and(|e| e.path().extension().is_some_and(|ext| ext == "org"))))
        .unwrap_or(false);

    if dir_has_files {
        // Already migrated — load from dir
        load_notes_from_dir(notes_dir)
    } else if old_notes_path.exists() {
        // Migrate from old monolithic file
        let notes = match std::fs::read_to_string(old_notes_path) {
            Ok(content) => convert::parse_notes(&content),
            Err(_) => return Vec::new(),
        };
        // Write each note to its own file
        for note in &notes {
            let filename = format!("{}.org", note.id);
            let path = notes_dir.join(&filename);
            let content = OrgWriter::write_note_file(note);
            if let Err(e) = std::fs::write(&path, &content) {
                log::error!("Migration: failed to write {}: {}", path.display(), e);
            }
        }
        // Remove old monolithic file
        if let Err(e) = std::fs::remove_file(old_notes_path) {
            log::warn!("Failed to remove old notes.org: {}", e);
        }
        log::info!("Migrated {} notes to per-file storage", notes.len());
        notes
    } else {
        Vec::new()
    }
}

fn load_notes_from_dir(dir: &std::path::Path) -> Vec<Note> {
    let mut notes = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return notes,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "org") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut parsed = convert::parse_notes(&content);
                notes.append(&mut parsed);
            }
        }
    }
    notes
}

fn build_backlink_index(notes: &[Note]) -> HashMap<LinkTarget, Vec<uuid::Uuid>> {
    let mut idx: HashMap<LinkTarget, Vec<uuid::Uuid>> = HashMap::new();
    for note in notes {
        for link in &note.links {
            idx.entry(link.clone()).or_default().push(note.id);
        }
    }
    idx
}

fn state_button<'a>(label: &'a str, value: TaskState, current: &TaskState) -> Element<'a, Message> {
    let btn = if *current == value {
        button::suggested(label)
    } else {
        button::standard(label)
    };
    btn.on_press(Message::CaptureFormState(value)).into()
}

fn priority_button<'a>(label: &'a str, value: Option<Priority>, current: &Option<Priority>) -> Element<'a, Message> {
    let btn = if *current == value {
        button::suggested(label)
    } else {
        button::standard(label)
    };
    btn.on_press(Message::CaptureFormPriority(value)).into()
}

fn esc_button<'a>(label: &str, value: Option<u32>, current: &Option<u32>) -> Element<'a, Message> {
    let btn = if *current == value {
        button::suggested(label.to_string())
    } else {
        button::standard(label.to_string())
    };
    btn.on_press(Message::CaptureFormEsc(value)).into()
}
