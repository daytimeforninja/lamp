use std::collections::{HashMap, HashSet};

use chrono::Timelike;

use relm4::prelude::*;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::adw;
use relm4::adw::prelude::*;

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
use crate::ui;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskLocation {
    Inbox,
    Next,
    Waiting,
    Someday,
    Project(String),
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
    pub scheduled_error: Option<String>,
    pub deadline_error: Option<String>,
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
            scheduled_error: None,
            deadline_error: None,
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
    pub start_error: Option<String>,
    pub end_error: Option<String>,
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
            start_error: None,
            end_error: None,
        }
    }
}

/// Buffered edit state for note fields.
pub struct NoteEditBuffer {
    pub id: uuid::Uuid,
    pub title: String,
    pub tags: String,
    pub source: String,
}

/// Async command results from background tasks.
#[derive(Debug)]
pub enum CommandMsg {
    SyncCompleted(Result<crate::sync::SyncResult, String>),
    SyncNotesCompleted(Result<crate::sync::webdav::NoteSyncResult, String>),
    SyncAccountsCompleted(Result<crate::sync::webdav::FileSyncResult<Account>, String>),
    ContactsFetched(Result<Vec<Contact>, String>),
    ContactDeleted(Result<(), String>),
    ImapFetched(Result<Vec<ImapEmail>, String>),
    EmailArchived(Result<u32, String>),
    ServiceConnectionTested(ServiceKind, Result<String, String>, Vec<CalendarInfo>),
    EventDeleted,
}

pub struct Lamp {
    config: LampConfig,
    active_view: ActiveView,
    app_mode: AppMode,

    // Data
    inbox_tasks: Vec<Task>,
    next_tasks: Vec<Task>,
    waiting_tasks: Vec<Task>,
    someday_tasks: Vec<Task>,
    projects: Vec<Project>,
    habits: Vec<Habit>,

    all_tasks_cache: Vec<Task>,
    task_index: HashMap<uuid::Uuid, TaskLocation>,

    // List items
    media_items: Vec<ListItem>,
    shopping_tasks: Vec<Task>,

    // Day plan
    day_plan: Option<DayPlan>,
    rejected_suggestions: HashSet<uuid::Uuid>,

    // All Tasks sort
    all_tasks_sort: Option<(SortColumn, bool)>,

    // Archive browsing
    archive_tasks: Vec<Task>,
    archive_search: String,

    // Capture
    new_task_form: NewTaskForm,
    launch_mode: LaunchMode,
    show_capture_dialog: bool,

    // Time tracking
    active_timer: Option<(uuid::Uuid, chrono::NaiveDateTime)>,

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

    // Review checklist
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
    note_body_buffer: Option<(uuid::Uuid, String)>,
    note_link_search: String,
    note_edit_buffer: Option<NoteEditBuffer>,
    backlink_index: HashMap<LinkTarget, Vec<uuid::Uuid>>,

    // Sync
    sync_status: SyncStatus,
    sync_anim_frame: usize,
    discovered_calendars: Vec<CalendarInfo>,
    service_passwords: [String; 4],
    service_test_status: [Option<Result<String, String>>; 4],

    // IMAP
    imap_emails: Vec<ImapEmail>,
    archived_email_uids: HashSet<u32>,

    // Pending sync ops
    pending_completions: Vec<(String, String)>,
    sync_conflicts: Vec<SyncConflict>,
    pending_deletions: Vec<String>,

    // Month calendar
    month_calendar: MonthCalendarState,

    sync_ops_pending: usize,

    // Whether the page content needs rebuilding after update
    needs_rebuild: bool,
}

pub struct LampWidgets {
    page_content: gtk::Box,
    search_entry: gtk::SearchEntry,
    split_view: adw::NavigationSplitView,
    toast_overlay: adw::ToastOverlay,
    sync_button: gtk::Button,
    timer_source: Option<gtk::glib::SourceId>,
    sync_anim_source: Option<gtk::glib::SourceId>,
    last_sync_toast: Option<String>,
}

impl Component for Lamp {
    type Init = (LampConfig, LaunchMode);
    type Input = Message;
    type Output = ();
    type CommandOutput = CommandMsg;
    type Root = adw::ApplicationWindow;
    type Widgets = LampWidgets;

    fn init_root() -> Self::Root {
        let window = adw::ApplicationWindow::new(&relm4::main_adw_application());
        window.set_default_size(1200, 800);
        window.set_title(Some("Lamp"));
        window
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (config, launch_mode) = init;

        // Ensure org files exist
        if let Err(e) = config.ensure_files() {
            log::error!("Failed to create org directory: {}", e);
        }

        // Load data
        let inbox_tasks = load_tasks(&config.inbox_path());
        let next_tasks = load_tasks(&config.next_path());
        let waiting_tasks = load_tasks(&config.waiting_path());
        let someday_tasks = load_tasks(&config.someday_path());
        let projects = load_projects(&config.projects_path());
        let habits = load_habits(&config.habits_path());
        let media_items = load_list_items(&config.media_path());
        let shopping_tasks = load_shopping_tasks(&config.shopping_path());

        let today = chrono::Local::now().date_naive();
        let day_plan = load_day_plan(&config.dayplan_path())
            .filter(|dp| !dp.is_stale(today));

        let contacts = crate::sync::carddav::load_contacts(&config.contacts_path());
        let events = event::load_events(&config.events_cache_path());
        let accounts = load_accounts(&config.accounts_path());
        let notes = load_notes_dir(&config.notes_dir(), &config.notes_path());

        let (loaded_pending_completions, loaded_pending_deletions) = Self::load_pending_ops(&config);

        let app_mode = match launch_mode {
            LaunchMode::Today => AppMode::Do,
            _ => AppMode::Plan,
        };

        let mut model = Self {
            config,
            active_view: ActiveView::What(WhatPage::DailyPlanning),
            app_mode,
            inbox_tasks,
            next_tasks,
            waiting_tasks,
            someday_tasks,
            projects,
            habits,
            media_items,
            shopping_tasks,
            day_plan,
            rejected_suggestions: HashSet::new(),
            all_tasks_cache: Vec::new(),
            task_index: HashMap::new(),
            all_tasks_sort: None,
            archive_tasks: Vec::new(),
            archive_search: String::new(),
            new_task_form: NewTaskForm::default(),
            launch_mode,
            show_capture_dialog: launch_mode == LaunchMode::Capture,
            active_timer: None,
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
            note_body_buffer: None,
            note_link_search: String::new(),
            note_edit_buffer: None,
            events,
            event_form: None,
            review_checked: HashSet::new(),
            sync_status: SyncStatus::default(),
            sync_anim_frame: 0,
            discovered_calendars: Vec::new(),
            service_passwords: [String::new(), String::new(), String::new(), String::new()],
            service_test_status: [None, None, None, None],
            imap_emails: Vec::new(),
            archived_email_uids: HashSet::new(),
            pending_completions: loaded_pending_completions,
            sync_conflicts: Vec::new(),
            pending_deletions: loaded_pending_deletions,
            month_calendar: MonthCalendarState::default(),
            sync_ops_pending: 0,
            needs_rebuild: true,
        };

        // Purge habit/shopping tasks from task lists
        let habit_ids: HashSet<uuid::Uuid> = model.habits.iter().map(|h| h.task.id).collect();
        let shopping_ids: HashSet<uuid::Uuid> = model.shopping_tasks.iter().map(|t| t.id).collect();
        let exclude_ids: HashSet<uuid::Uuid> = habit_ids.union(&shopping_ids).copied().collect();
        if !exclude_ids.is_empty() {
            model.inbox_tasks.retain(|t| !exclude_ids.contains(&t.id));
            model.next_tasks.retain(|t| !exclude_ids.contains(&t.id));
            model.waiting_tasks.retain(|t| !exclude_ids.contains(&t.id));
            model.someday_tasks.retain(|t| !exclude_ids.contains(&t.id));
            for project in &mut model.projects {
                project.tasks.retain(|t| !exclude_ids.contains(&t.id));
            }
        }
        model.rebuild_cache();

        // --- Build UI ---

        // Sidebar
        let sidebar = gtk::ListBox::new();
        sidebar.set_selection_mode(gtk::SelectionMode::Single);
        sidebar.add_css_class("navigation-sidebar");

        // Build a mapping from row index to page (separators shift indices)
        let mut row_to_page: Vec<WhatPage> = Vec::new();
        for (i, page) in WhatPage::ALL.iter().enumerate() {
            if WhatPage::SECTION_STARTS.contains(page) && i > 0 {
                let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
                sep.set_margin_top(6);
                sep.set_margin_bottom(6);
                sidebar.append(&sep);
            }
            let row = gtk::ListBoxRow::new();
            let hbox = ui::hbox(8);
            hbox.set_margin_start(8);
            hbox.set_margin_end(8);
            hbox.set_margin_top(4);
            hbox.set_margin_bottom(4);
            let icon = gtk::Image::from_icon_name(page.icon_name());
            let label = gtk::Label::new(Some(&page.title()));
            label.set_hexpand(true);
            label.set_xalign(0.0);
            hbox.append(&icon);
            hbox.append(&label);
            row.set_child(Some(&hbox));
            sidebar.append(&row);
            row_to_page.push(*page);
        }

        {
            let s = sender.input_sender().clone();
            sidebar.connect_row_selected(move |listbox, row| {
                if let Some(row) = row {
                    // Count only selectable rows (skip separators) up to this one
                    let mut page_idx = 0;
                    let mut i = 0;
                    while let Some(r) = listbox.row_at_index(i) {
                        if r == *row { break; }
                        if r.is_selectable() { page_idx += 1; }
                        i += 1;
                    }
                    if page_idx < row_to_page.len() {
                        s.emit(Message::NavigateTo(row_to_page[page_idx]));
                    }
                }
            });
        }

        let sidebar_scroll = ui::scrolled(&sidebar);
        let sidebar_page = adw::NavigationPage::new(&sidebar_scroll, "Navigation");

        // --- Header bar (CSD) ---
        let header_bar = adw::HeaderBar::new();

        // Mode toggle — icon-only with tooltips per HIG
        let mode_plan_btn = gtk::ToggleButton::new();
        mode_plan_btn.set_icon_name("view-list-symbolic");
        mode_plan_btn.set_tooltip_text(Some("Plan"));
        let mode_do_btn = gtk::ToggleButton::new();
        mode_do_btn.set_icon_name("media-playback-start-symbolic");
        mode_do_btn.set_tooltip_text(Some("Do"));
        mode_do_btn.set_group(Some(&mode_plan_btn));
        if model.app_mode == AppMode::Plan {
            mode_plan_btn.set_active(true);
        } else {
            mode_do_btn.set_active(true);
        }
        {
            let s = sender.input_sender().clone();
            mode_plan_btn.connect_toggled(move |btn| {
                if btn.is_active() {
                    s.emit(Message::SetMode(AppMode::Plan));
                }
            });
        }
        {
            let s = sender.input_sender().clone();
            mode_do_btn.connect_toggled(move |btn| {
                if btn.is_active() {
                    s.emit(Message::SetMode(AppMode::Do));
                }
            });
        }
        // Place Plan/Do in center as title widget (view switcher pattern)
        let mode_box = ui::hbox(0);
        mode_box.add_css_class("linked");
        mode_box.append(&mode_plan_btn);
        mode_box.append(&mode_do_btn);
        header_bar.set_title_widget(Some(&mode_box));

        // Header end buttons — icon-only with tooltips per HIG
        let new_task_btn = ui::icon_button("list-add-symbolic");
        new_task_btn.set_tooltip_text(Some("New Task (Ctrl+N)"));
        {
            let s = sender.input_sender().clone();
            new_task_btn.connect_clicked(move |_| {
                s.emit(Message::OpenNewTaskForm);
            });
        }
        header_bar.pack_end(&new_task_btn);

        let sync_button = ui::icon_button("media-playlist-repeat-symbolic");
        sync_button.set_tooltip_text(Some("Sync"));
        {
            let s = sender.input_sender().clone();
            sync_button.connect_clicked(move |_| {
                s.emit(Message::SyncNow);
            });
        }
        header_bar.pack_end(&sync_button);

        let settings_btn = ui::icon_button("emblem-system-symbolic");
        settings_btn.set_tooltip_text(Some("Settings"));
        {
            let s = sender.input_sender().clone();
            settings_btn.connect_clicked(move |_| {
                s.emit(Message::OpenSettings);
            });
        }
        header_bar.pack_end(&settings_btn);

        // --- Content pane (inside split view) ---
        let content_inner = ui::vbox(0);

        // Search entry
        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some(&crate::fl!("search-placeholder")));
        search_entry.set_margin_start(16);
        search_entry.set_margin_end(16);
        search_entry.set_margin_top(4);
        {
            let s = sender.input_sender().clone();
            search_entry.connect_search_changed(move |e| {
                s.emit(Message::SearchQueryChanged(e.text().to_string()));
            });
        }
        content_inner.append(&search_entry);

        // Page content area — wrapped in AdwClamp for comfortable reading width
        let page_content = ui::vbox(0);
        page_content.set_vexpand(true);
        page_content.set_hexpand(true);
        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(900);
        clamp.set_child(Some(&page_content));
        let page_scroll = ui::scrolled(&clamp);
        content_inner.append(&page_scroll);

        // Split view
        let split_view = adw::NavigationSplitView::new();
        split_view.set_sidebar(Some(&sidebar_page));
        let content_page = adw::NavigationPage::new(&content_inner, "Content");
        split_view.set_content(Some(&content_page));

        if model.app_mode == AppMode::Do || model.launch_mode == LaunchMode::Today {
            split_view.set_show_content(true);
            split_view.set_collapsed(true);
        }

        // Toast overlay wraps the split view for transient notifications
        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&split_view));

        // ToolbarView wraps everything — header bar on top, toast overlay as content
        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&toast_overlay));

        root.set_content(Some(&toolbar_view));

        // Keyboard shortcut: Ctrl+N for new task
        let controller = gtk::EventControllerKey::new();
        {
            let s = sender.input_sender().clone();
            controller.connect_key_pressed(move |_, key, _, modifiers| {
                if key == gtk::gdk::Key::n && modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    s.emit(Message::OpenNewTaskForm);
                    return gtk::glib::Propagation::Stop;
                }
                gtk::glib::Propagation::Proceed
            });
        }
        root.add_controller(controller);

        let widgets = LampWidgets {
            page_content,
            search_entry,
            split_view,
            toast_overlay,
            sync_button,
            timer_source: None,
            sync_anim_source: None,
            last_sync_toast: None,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Message, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.needs_rebuild = false;

        match message {
            Message::NavigateTo(page) => {
                if let ActiveView::What(WhatPage::Review) = self.active_view {
                    if page != WhatPage::Review {
                        self.review_checked.clear();
                    }
                }
                self.active_view = ActiveView::What(page);
                self.search_query.clear();
                if page == WhatPage::Archive {
                    self.archive_tasks = load_tasks(&self.config.archive_path());
                    self.archive_search.clear();
                }
                self.needs_rebuild = true;
            }

            Message::SetMode(mode) => {
                if let Some((active_id, start)) = self.active_timer.take() {
                    let now = chrono::Local::now().naive_local();
                    self.modify_task(active_id, |task| {
                        task.clock_entries.push((start, now));
                    });
                    self.save_all();
                }
                self.app_mode = mode;
                self.needs_rebuild = true;
            }

            Message::SearchQueryChanged(q) => {
                self.search_query = q;
                self.needs_rebuild = true;
            }

            Message::ArchiveSearchChanged(q) => {
                self.archive_search = q;
                self.needs_rebuild = true;
            }

            Message::SelectWhen(_) => {}

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
                    self.needs_rebuild = true;
                }
            }

            Message::AddTask(title) => {
                let task = Task::new(sentence_case(&title));
                self.inbox_tasks.push(task);
                self.save_inbox();
                self.needs_rebuild = true;
            }

            Message::UpdateTaskTitle(id, ref title) => {
                let new_title = title.clone();
                self.modify_task(id, |task| {
                    task.title = new_title;
                });
            }

            Message::ToggleTaskDone(id) => {
                self.toggle_done(id);
                self.needs_rebuild = true;
            }

            Message::SetTaskState(id, state) => {
                self.set_task_state(id, state);
                self.needs_rebuild = true;
            }

            Message::SetTaskPriority(id, priority) => {
                self.set_task_priority(id, priority);
                self.needs_rebuild = true;
            }

            Message::SetTaskEsc(id, esc) => {
                self.modify_task(id, |task| {
                    task.esc = esc;
                });
                self.needs_rebuild = true;
            }

            Message::WaitingForInputChanged(id, value) => {
                self.waiting_for_inputs.insert(id, value);
            }

            Message::SetFollowUp(id, date) => {
                self.modify_task(id, |task| {
                    task.follow_up = date;
                });
                self.needs_rebuild = true;
            }

            Message::SetWaitingFor(id, ref value) => {
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
                self.needs_rebuild = true;
            }

            Message::DeleteTask(id) => {
                let mut any_pending = false;
                if let Some(task) = self.remove_task(id) {
                    if let Some(href) = task.sync_href {
                        self.pending_deletions.push(href);
                        any_pending = true;
                    }
                }
                for project in &mut self.projects {
                    if let Some(pos) = project.tasks.iter().position(|t| t.id == id) {
                        if let Some(href) = project.tasks[pos].sync_href.clone() {
                            self.pending_deletions.push(href);
                            any_pending = true;
                        }
                        project.tasks.remove(pos);
                    }
                }
                if any_pending { self.save_pending_ops(); }
                self.save_all();
                self.needs_rebuild = true;
            }

            Message::MoveToProject(id, ref project_name) => {
                if let Some(mut task) = self.remove_task(id) {
                    task.project = Some(project_name.clone());
                    if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                        project.tasks.push(task);
                    } else {
                        log::warn!("Project '{}' not found, routing task by state", project_name);
                        self.route_task_by_state(task);
                    }
                    self.save_all();
                    self.needs_rebuild = true;
                }
            }

            Message::AddContext(id, ref ctx) => {
                self.modify_task(id, |task| {
                    if !task.contexts.contains(ctx) {
                        task.contexts.push(ctx.clone());
                    }
                });
                self.needs_rebuild = true;
            }

            Message::RemoveContext(id, ref ctx) => {
                self.modify_task(id, |task| {
                    task.contexts.retain(|c| c != ctx);
                });
                self.needs_rebuild = true;
            }

            Message::SetScheduled(id, date) => {
                self.modify_task(id, |task| {
                    task.scheduled = date;
                });
                self.needs_rebuild = true;
            }

            Message::SetDeadline(id, date) => {
                self.modify_task(id, |task| {
                    task.deadline = date;
                });
                self.needs_rebuild = true;
            }

            Message::OpenSettings => {
                self.active_view = ActiveView::What(WhatPage::Settings);
                self.search_query.clear();
                self.needs_rebuild = true;
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
                    self.needs_rebuild = true;
                }
            }

            Message::SettingsRemoveContext(idx) => {
                if idx < self.config.contexts.len() {
                    self.config.contexts.remove(idx);
                    self.save_config();
                    self.needs_rebuild = true;
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
                self.needs_rebuild = true;
            }

            Message::ProjectInputChanged(value) => {
                self.project_input = value;
            }

            Message::ProjectSubmit => {
                let name = self.project_input.trim().to_string();
                if !name.is_empty() && !self.projects.iter().any(|p| p.name == name) {
                    self.projects.push(Project::new(name));
                    self.project_input.clear();
                    self.save_projects();
                    self.needs_rebuild = true;
                }
            }

            Message::CreateProject(name) => {
                if !name.is_empty() && !self.projects.iter().any(|p| p.name == name) {
                    self.projects.push(Project::new(name));
                    self.save_projects();
                    self.needs_rebuild = true;
                }
            }

            Message::DeleteProject(name) => {
                if let Some(pos) = self.projects.iter().position(|p| p.name == name) {
                    let project = self.projects.remove(pos);
                    for mut task in project.tasks {
                        task.project = None;
                        self.route_task_by_state(task);
                    }
                }
                self.save_all();
                self.rebuild_cache();
                self.needs_rebuild = true;
            }

            Message::ProjectTaskInputChanged(ref project_name, ref value) => {
                self.project_task_inputs.insert(project_name.clone(), value.clone());
            }

            Message::AddTaskToProject(ref project_name) => {
                let input = self.project_task_inputs.get(project_name).cloned().unwrap_or_default();
                let title = sentence_case(&input);
                if !title.is_empty() {
                    let mut task = Task::new(title);
                    task.project = Some(project_name.clone());
                    if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                        project.tasks.push(task);
                    }
                    self.project_task_inputs.insert(project_name.clone(), String::new());
                    self.save_all();
                    self.needs_rebuild = true;
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
                            self.needs_rebuild = true;
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
                self.needs_rebuild = true;
            }

            Message::CompleteHabit(id) => {
                let today = chrono::Local::now().date_naive();
                if let Some(habit) = self.habits.iter_mut().find(|h| h.task.id == id) {
                    if habit.is_due(today) {
                        habit.completions.push(chrono::Local::now().naive_local());
                        habit.recalculate_streak(today);
                    }
                }
                self.save_habits();
                self.needs_rebuild = true;
            }

            Message::DeleteHabit(id) => {
                self.habits.retain(|h| h.task.id != id);
                self.save_habits();
                self.needs_rebuild = true;
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
                    let habit = Habit::new(task);
                    self.habits.push(habit);
                    self.habit_input.clear();
                    self.save_habits();
                    self.needs_rebuild = true;
                }
            }

            Message::ToggleTaskExpand(id) => {
                if self.expanded_task == Some(id) {
                    self.modify_task(id, |task| {
                        task.title = sentence_case(&task.title);
                    });
                    self.expanded_task = None;
                } else {
                    self.expanded_task = Some(id);
                }
                self.needs_rebuild = true;
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

                    if let Some(item) = self.media_items.iter_mut().find(|i| i.id == id) {
                        if item.notes.is_empty() { item.notes = line; }
                        else { item.notes.push('\n'); item.notes.push_str(&line); }
                        self.note_inputs.insert(id, String::new());
                        self.save_media();
                    } else if let Some(task) = self.shopping_tasks.iter_mut().find(|t| t.id == id) {
                        if task.notes.is_empty() { task.notes = line; }
                        else { task.notes.push('\n'); task.notes.push_str(&line); }
                        self.note_inputs.insert(id, String::new());
                        self.save_shopping();
                    } else {
                        self.modify_task(id, |task| {
                            if task.notes.is_empty() { task.notes = line; }
                            else { task.notes.push('\n'); task.notes.push_str(&line); }
                        });
                        self.note_inputs.insert(id, String::new());
                    }
                    self.needs_rebuild = true;
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
                            let mut task = Task::new(title);
                            task.extra_tags = vec!["shopping".to_string()];
                            self.shopping_tasks.push(task);
                            self.shopping_input.clear();
                            self.save_shopping();
                        }
                    }
                }
                self.needs_rebuild = true;
            }

            Message::DeleteListItem(kind, id) => {
                match kind {
                    ListKind::Media => {
                        if let Some(item) = self.media_items.iter().find(|i| i.id == id) {
                            let _ = OrgWriter::append_list_item_to_file(&self.config.consumed_path(), item);
                        }
                        self.media_items.retain(|i| i.id != id);
                        self.save_media();
                    }
                    ListKind::Shopping => {
                        if let Some(task) = self.shopping_tasks.iter().find(|t| t.id == id) {
                            let _ = OrgWriter::append_to_file(&self.config.bought_path(), task);
                        }
                        self.shopping_tasks.retain(|t| t.id != id);
                        self.save_shopping();
                    }
                }
                self.flipped_list_items.remove(&id);
                self.pending_delete_list_item = None;
                self.needs_rebuild = true;
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
                        if let Some(task) = self.shopping_tasks.iter_mut().find(|t| t.id == id) {
                            if task.state.is_done() {
                                task.state = TaskState::Todo;
                                task.completed = None;
                            } else {
                                task.complete();
                            }
                        }
                        self.save_shopping();
                    }
                }
                self.needs_rebuild = true;
            }

            Message::FlipListItem(id) => {
                if !self.flipped_list_items.remove(&id) {
                    self.flipped_list_items.insert(id);
                }
                self.needs_rebuild = true;
            }

            Message::ConfirmDeleteListItem(kind, id) => {
                self.pending_delete_list_item = Some((kind, id));
                self.needs_rebuild = true;
            }

            Message::CancelDeleteListItem => {
                self.pending_delete_list_item = None;
                self.needs_rebuild = true;
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
                    self.needs_rebuild = true;
                }
            }

            Message::ConfirmDeleteContact(idx) => {
                self.pending_delete_contact = Some(idx);
                self.needs_rebuild = true;
            }

            Message::CancelDeleteContact => {
                self.pending_delete_contact = None;
                self.needs_rebuild = true;
            }

            Message::DeleteContact(idx) => {
                self.pending_delete_contact = None;
                if idx < self.contacts.len() {
                    let removed = self.contacts.remove(idx);
                    self.flipped_contacts = self.flipped_contacts.iter()
                        .filter(|&&i| i != idx)
                        .map(|&i| if i > idx { i - 1 } else { i })
                        .collect();
                    self.editing_contact = match self.editing_contact {
                        Some(i) if i == idx => None,
                        Some(i) if i > idx => Some(i - 1),
                        other => other,
                    };
                    self.save_contacts();

                    if let Some(href) = removed.sync_href {
                        let contacts_url = self.config.contacts.url.trim().to_string();
                        if !contacts_url.is_empty() {
                            sender.command(|out, _| async move {
                                let (username, pw) = match crate::sync::keyring::load_credentials(&contacts_url).await {
                                    Ok(Some(creds)) => creds,
                                    _ => return out.emit(CommandMsg::ContactDeleted(Err("No CardDAV credentials".to_string()))),
                                };
                                let client = match crate::sync::carddav::CardDavClient::new(&contacts_url, &username, &pw) {
                                    Ok(c) => c,
                                    Err(e) => return out.emit(CommandMsg::ContactDeleted(Err(e))),
                                };
                                let result = client.delete_contact(&href).await;
                                out.emit(CommandMsg::ContactDeleted(result));
                            });
                        }
                    }
                    self.needs_rebuild = true;
                }
            }

            Message::SetContactGroups(idx, ref groups) => {
                if let Some(c) = self.contacts.get_mut(idx) {
                    c.groups = groups.clone();
                    self.save_contacts();
                    self.needs_rebuild = true;
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
                    self.needs_rebuild = true;
                }
            }

            Message::FlipContact(idx) => {
                if !self.flipped_contacts.remove(&idx) {
                    self.flipped_contacts.insert(idx);
                }
                if self.editing_contact == Some(idx) {
                    self.editing_contact = None;
                }
                self.needs_rebuild = true;
            }

            Message::EditContact(idx) => {
                self.flipped_contacts.insert(idx);
                self.editing_contact = Some(idx);
                self.needs_rebuild = true;
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
                    self.needs_rebuild = true;
                }
            }

            Message::ConfirmDeleteAccount(idx) => {
                self.pending_delete_account = Some(idx);
                self.needs_rebuild = true;
            }

            Message::CancelDeleteAccount => {
                self.pending_delete_account = None;
                self.needs_rebuild = true;
            }

            Message::DeleteAccount(idx) => {
                self.pending_delete_account = None;
                if idx < self.accounts.len() {
                    let removed = self.accounts.remove(idx);
                    self.expanded_account = None;
                    let _ = OrgWriter::append_account_to_file(&self.config.closed_accounts_path(), &removed);
                    self.save_accounts();
                    self.needs_rebuild = true;
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
                    self.needs_rebuild = true;
                }
            }

            Message::OpenAccountUrl(idx) => {
                if let Some(a) = self.accounts.get(idx) {
                    if !a.url.is_empty() {
                        let _ = std::process::Command::new(&self.config.browser_command)
                            .arg(&a.url)
                            .spawn();
                    }
                }
            }

            Message::ToggleAccountExpand(idx) => {
                if self.expanded_account == Some(idx) {
                    self.expanded_account = None;
                } else {
                    self.expanded_account = Some(idx);
                }
                self.needs_rebuild = true;
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
                    self.needs_rebuild = true;
                }
            }

            Message::FlipNote(id) => {
                if self.editing_note == Some(id) && self.flipped_notes.contains(&id) {
                    if let Some(buf) = self.note_edit_buffer.take() {
                        if buf.id == id {
                            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                                note.title = buf.title;
                                note.tags = buf.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                                note.source = if buf.source.is_empty() { None } else { Some(buf.source) };
                            }
                        }
                    }
                    if let Some((eid, ref body)) = self.note_body_buffer {
                        if eid == id {
                            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                                note.body = body.clone();
                                if note.body.ends_with('\n') { note.body.pop(); }
                            }
                        }
                    }
                    self.note_body_buffer = None;
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
                self.needs_rebuild = true;
            }

            Message::EditNote(id) => {
                self.editing_note = Some(id);
                self.flipped_notes.insert(id);
                if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                    self.note_body_buffer = Some((id, note.body.clone()));
                    self.note_edit_buffer = Some(NoteEditBuffer {
                        id,
                        title: note.title.clone(),
                        tags: note.tags.join(", "),
                        source: note.source.clone().unwrap_or_default(),
                    });
                }
                self.needs_rebuild = true;
            }

            Message::NoteBodyChanged(body) => {
                if let Some((id, _)) = &self.note_body_buffer {
                    self.note_body_buffer = Some((*id, body));
                }
            }

            Message::SetNoteField(_id, field, value) => {
                if let Some(ref mut buf) = self.note_edit_buffer {
                    match field {
                        NoteField::Title => buf.title = value,
                        NoteField::Tags => buf.tags = value,
                        NoteField::Source => buf.source = value,
                        NoteField::Body => {}
                    }
                }
            }

            Message::ConfirmDeleteNote(id) => {
                self.pending_delete_note = Some(id);
                self.needs_rebuild = true;
            }

            Message::CancelDeleteNote => {
                self.pending_delete_note = None;
                self.needs_rebuild = true;
            }

            Message::DeleteNote(id) => {
                self.notes.retain(|n| n.id != id);
                self.pending_delete_note = None;
                self.flipped_notes.remove(&id);
                if self.editing_note == Some(id) {
                    self.editing_note = None;
                    self.note_body_buffer = None;
                }
                self.backlink_index = build_backlink_index(&self.notes);
                self.delete_note_file(id);
                self.needs_rebuild = true;
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
                self.needs_rebuild = true;
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
                self.needs_rebuild = true;
            }

            Message::OpenNoteInEditor(id) => {
                if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                    let note_path = self.config.notes_dir().join(format!("{}.body.txt", note.id));
                    if let Err(e) = std::fs::write(&note_path, &note.body) {
                        log::error!("Failed to write note file: {}", e);
                    } else {
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                        let note_id = note.id;
                        let path_clone = note_path.clone();
                        let notes_dir = self.config.notes_dir();
                        match std::process::Command::new(&editor).arg(&note_path).spawn() {
                            Ok(mut child) => {
                                std::thread::spawn(move || {
                                    let _ = child.wait();
                                    if let Ok(edited_body) = std::fs::read_to_string(&path_clone) {
                                        let org_path = notes_dir.join(format!("{}.org", note_id));
                                        if let Ok(content) = std::fs::read_to_string(&org_path) {
                                            let mut notes = crate::org::convert::parse_notes(&content);
                                            if let Some(note) = notes.iter_mut().find(|n| n.id == note_id) {
                                                note.body = edited_body.trim().to_string();
                                                note.modified = chrono::Local::now().naive_local();
                                                let new_content = OrgWriter::write_note_file(note);
                                                let _ = std::fs::write(&org_path, new_content);
                                            }
                                        }
                                    }
                                    let _ = std::fs::remove_file(&path_clone);
                                });
                            }
                            Err(e) => log::error!("Failed to open editor: {}", e),
                        }
                    }
                }
            }

            Message::NoteLinkSearchChanged(value) => {
                self.note_link_search = value;
                self.needs_rebuild = true;
            }

            // Daily Planning
            Message::SetSpoonBudget(budget) => {
                let plan = self.ensure_day_plan();
                plan.spoon_budget = budget;
                self.save_day_plan();
                self.needs_rebuild = true;
            }

            Message::TogglePlanContext(ref ctx) => {
                let plan = self.ensure_day_plan();
                if let Some(pos) = plan.active_contexts.iter().position(|c| c == ctx) {
                    plan.active_contexts.remove(pos);
                } else {
                    plan.active_contexts.push(ctx.clone());
                }
                self.save_day_plan();
                self.needs_rebuild = true;
            }

            Message::ConfirmTask(id) => {
                let plan = self.ensure_day_plan();
                if !plan.confirmed_task_ids.contains(&id) {
                    plan.confirmed_task_ids.push(id);
                }
                self.save_day_plan();
                self.needs_rebuild = true;
            }

            Message::UnconfirmTask(id) => {
                if let Some(ref mut plan) = self.day_plan {
                    plan.confirmed_task_ids.retain(|i| *i != id);
                    self.save_day_plan();
                }
                self.needs_rebuild = true;
            }

            Message::RejectSuggestion(id) => {
                self.rejected_suggestions.insert(id);
                self.needs_rebuild = true;
            }

            Message::PickMediaItem(id) => {
                let plan = self.ensure_day_plan();
                if !plan.picked_media_ids.contains(&id) { plan.picked_media_ids.push(id); }
                self.save_day_plan();
                self.needs_rebuild = true;
            }

            Message::UnpickMediaItem(id) => {
                if let Some(ref mut plan) = self.day_plan { plan.picked_media_ids.retain(|i| *i != id); self.save_day_plan(); }
                self.needs_rebuild = true;
            }

            Message::PickShoppingItem(id) => {
                let plan = self.ensure_day_plan();
                if !plan.picked_shopping_ids.contains(&id) { plan.picked_shopping_ids.push(id); }
                self.save_day_plan();
                self.needs_rebuild = true;
            }

            Message::UnpickShoppingItem(id) => {
                if let Some(ref mut plan) = self.day_plan { plan.picked_shopping_ids.retain(|i| *i != id); self.save_day_plan(); }
                self.needs_rebuild = true;
            }

            // Do mode
            Message::DoMarkDone(id) => {
                if let Some((active_id, start)) = self.active_timer.take() {
                    let now = chrono::Local::now().naive_local();
                    self.modify_task(active_id, |task| { task.clock_entries.push((start, now)); });
                }
                let is_completed = self.day_plan.as_ref()
                    .map(|p| p.completed_tasks.iter().any(|ct| ct.id == id))
                    .unwrap_or(false);
                if is_completed {
                    if let Some(plan) = &mut self.day_plan { plan.uncomplete_task(id); }
                    self.modify_task(id, |task| { task.state = TaskState::Next; task.completed = None; });
                } else {
                    if let Some(plan) = &mut self.day_plan {
                        let task_info = self.all_tasks_cache.iter().find(|t| t.id == id);
                        let title = task_info.map(|t| t.title.clone()).unwrap_or_default();
                        let esc = task_info.and_then(|t| t.esc);
                        plan.complete_task(id, title, esc);
                    }
                    self.modify_task(id, |task| { task.state = TaskState::Done; task.completed = Some(chrono::Local::now().naive_local()); });
                }
                self.save_day_plan();
                self.save_all();
                self.rebuild_cache();
                self.needs_rebuild = true;
            }

            Message::DoMarkListItemDone(id) => {
                if let Some(item) = self.media_items.iter().find(|i| i.id == id) {
                    let _ = OrgWriter::append_list_item_to_file(&self.config.consumed_path(), item);
                    self.media_items.retain(|i| i.id != id);
                    self.save_media();
                }
                if let Some(task) = self.shopping_tasks.iter().find(|t| t.id == id) {
                    let _ = OrgWriter::append_to_file(&self.config.bought_path(), task);
                    self.shopping_tasks.retain(|t| t.id != id);
                    self.save_shopping();
                }
                if let Some(ref mut plan) = self.day_plan {
                    plan.picked_media_ids.retain(|i| *i != id);
                    plan.picked_shopping_ids.retain(|i| *i != id);
                    self.save_day_plan();
                }
                self.needs_rebuild = true;
            }

            Message::ToggleWorkTimer(id) => {
                let now = chrono::Local::now().naive_local();
                if let Some((active_id, start)) = self.active_timer.take() {
                    self.modify_task(active_id, |task| { task.clock_entries.push((start, now)); });
                    self.save_all();
                    self.rebuild_cache();
                    if active_id != id { self.active_timer = Some((id, now)); }
                } else {
                    self.active_timer = Some((id, now));
                }
                self.needs_rebuild = true;
            }

            Message::TimerTick => {
                self.needs_rebuild = true;
            }

            Message::SyncAnimTick => {
                self.sync_anim_frame = (self.sync_anim_frame + 1) % 8;
            }

            Message::SetAllTasksSort(col) => {
                self.all_tasks_sort = Some(match self.all_tasks_sort {
                    Some((c, asc)) if c == col => (col, !asc),
                    _ => (col, true),
                });
                self.needs_rebuild = true;
            }

            Message::Save => { self.save_all(); }

            Message::OpenNewTaskForm => {
                self.new_task_form = NewTaskForm::default();
                self.show_capture_dialog = true;
                self.needs_rebuild = true;
            }

            Message::CloseNewTaskForm => {
                self.show_capture_dialog = false;
                if self.launch_mode == LaunchMode::Capture {
                    std::process::exit(0);
                }
                self.needs_rebuild = true;
            }

            Message::CaptureFormTitle(value) => { self.new_task_form.title = value; }
            Message::CaptureFormState(state) => { self.new_task_form.state = state; }
            Message::CaptureFormPriority(priority) => { self.new_task_form.priority = priority; }
            Message::CaptureFormEsc(esc) => { self.new_task_form.esc = esc; }

            Message::CaptureFormToggleContext(ref ctx) => {
                if let Some(pos) = self.new_task_form.contexts.iter().position(|c| c == ctx) {
                    self.new_task_form.contexts.remove(pos);
                } else {
                    self.new_task_form.contexts.push(ctx.clone());
                }
                self.needs_rebuild = true;
            }

            Message::CaptureFormProject(project) => { self.new_task_form.project = project; self.needs_rebuild = true; }
            Message::CaptureFormScheduled(value) => { self.new_task_form.scheduled = value; self.new_task_form.scheduled_error = None; }
            Message::CaptureFormDeadline(value) => { self.new_task_form.deadline = value; self.new_task_form.deadline_error = None; }
            Message::CaptureFormNotes(value) => { self.new_task_form.notes = value; }

            Message::CaptureFormSubmit => {
                let title = sentence_case(&self.new_task_form.title);
                if !title.is_empty() {
                    let sched = parse_optional_date(&self.new_task_form.scheduled);
                    let dead = parse_optional_date(&self.new_task_form.deadline);
                    let mut has_error = false;
                    if let Err(ref e) = sched { self.new_task_form.scheduled_error = Some(e.clone()); has_error = true; }
                    if let Err(ref e) = dead { self.new_task_form.deadline_error = Some(e.clone()); has_error = true; }
                    if has_error { self.needs_rebuild = true; return; }

                    let mut task = Task::new(title);
                    task.state = self.new_task_form.state.clone();
                    task.priority = self.new_task_form.priority;
                    task.esc = self.new_task_form.esc;
                    task.contexts = self.new_task_form.contexts.clone();
                    task.project = self.new_task_form.project.clone();
                    task.scheduled = sched.unwrap();
                    task.deadline = dead.unwrap();
                    task.notes = self.new_task_form.notes.trim().to_string();

                    if let Some(ref project_name) = task.project {
                        let project_name = project_name.clone();
                        if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                            project.tasks.push(task);
                        } else {
                            self.route_task_by_state(task);
                        }
                    } else {
                        self.route_task_by_state(task);
                    }
                    self.save_all();
                    self.show_capture_dialog = false;
                    if self.launch_mode == LaunchMode::Capture { std::process::exit(0); }
                    self.needs_rebuild = true;
                }
            }

            Message::ConfigChanged => {}
            Message::Loaded(_) => {}

            // --- Sync ---
            Message::SyncNow => {
                if self.sync_status == SyncStatus::Syncing { return; }
                self.sync_status = SyncStatus::Syncing;
                let mut ops = 0;

                if self.config.sync_ready() {
                    ops += 1;
                    let mut tasks: Vec<Task> = self.all_active_tasks();
                    if let Some(ref plan) = self.day_plan {
                        for task in &mut tasks {
                            if plan.confirmed_task_ids.contains(&task.id) {
                                task.dayplan_date = Some(plan.date);
                            } else if task.dayplan_date.is_some() {
                                task.dayplan_date = None;
                            }
                        }
                    }
                    let events = self.events.clone();
                    let caldav_url = self.config.calendars.url.clone();
                    let task_cals = self.config.task_calendar_hrefs();
                    let event_cals = self.config.event_calendar_hrefs();
                    let sync_tokens = self.config.sync_tokens.clone();
                    let completions = self.pending_completions.clone();
                    let deletions = self.pending_deletions.clone();
                    sender.command(|out, _| async move {
                        let (username, password) = match crate::sync::keyring::load_credentials(&caldav_url).await {
                            Ok(Some(creds)) => creds,
                            Ok(None) => return out.emit(CommandMsg::SyncCompleted(Err("No CalDAV credentials stored".to_string()))),
                            Err(e) => return out.emit(CommandMsg::SyncCompleted(Err(format!("Keyring error: {}", e)))),
                        };
                        let result = crate::sync::sync_all(&caldav_url, &username, &password, &tasks, &events, &task_cals, &event_cals, &sync_tokens, &completions, &deletions).await;
                        out.emit(CommandMsg::SyncCompleted(result));
                    });
                }

                let notes_url = self.config.notes_sync.url.trim().to_string();
                if !notes_url.is_empty() {
                    ops += 2; // notes + accounts
                    let local_notes = self.notes.clone();
                    let notes_dir = self.config.notes_dir();
                    let notes_url1 = notes_url.clone();
                    sender.command(|out, _| async move {
                        let (username, pw) = match crate::sync::keyring::load_credentials(&notes_url1).await {
                            Ok(Some(creds)) => creds,
                            Ok(None) => return out.emit(CommandMsg::SyncNotesCompleted(Err("No WebDAV credentials".to_string()))),
                            Err(e) => return out.emit(CommandMsg::SyncNotesCompleted(Err(format!("Keyring error: {}", e)))),
                        };
                        let client = match crate::sync::webdav::WebDavClient::new(&notes_url1, &username, &pw) {
                            Ok(c) => c,
                            Err(e) => return out.emit(CommandMsg::SyncNotesCompleted(Err(e))),
                        };
                        let result = crate::sync::webdav::sync_notes(&client, &local_notes, &notes_dir).await;
                        out.emit(CommandMsg::SyncNotesCompleted(result));
                    });

                    let accounts = self.accounts.clone();
                    sender.command(|out, _| async move {
                        let (username, pw) = match crate::sync::keyring::load_credentials(&notes_url).await {
                            Ok(Some(creds)) => creds,
                            Ok(None) => return out.emit(CommandMsg::SyncAccountsCompleted(Err("No WebDAV credentials".to_string()))),
                            Err(e) => return out.emit(CommandMsg::SyncAccountsCompleted(Err(format!("Keyring error: {}", e)))),
                        };
                        let client = match crate::sync::webdav::WebDavClient::new(&notes_url, &username, &pw) {
                            Ok(c) => c,
                            Err(e) => return out.emit(CommandMsg::SyncAccountsCompleted(Err(e))),
                        };
                        let result = crate::sync::webdav::sync_accounts(&client, &accounts).await;
                        out.emit(CommandMsg::SyncAccountsCompleted(result));
                    });
                }

                let contacts_url = self.config.contacts.url.trim().to_string();
                if !contacts_url.is_empty() {
                    ops += 1;
                    sender.command(|out, _| async move {
                        let (username, pw) = match crate::sync::keyring::load_credentials(&contacts_url).await {
                            Ok(Some((u, pw))) => (u, pw),
                            _ => return out.emit(CommandMsg::ContactsFetched(Err("No CardDAV credentials".to_string()))),
                        };
                        let client = match crate::sync::carddav::CardDavClient::new(&contacts_url, &username, &pw) {
                            Ok(c) => c,
                            Err(e) => return out.emit(CommandMsg::ContactsFetched(Err(e))),
                        };
                        let result = client.fetch_contacts().await;
                        out.emit(CommandMsg::ContactsFetched(result));
                    });
                }

                let imap_host = self.config.imap.host.trim().to_string();
                if !imap_host.is_empty() {
                    ops += 1;
                    let folder = if self.config.imap.folder.is_empty() { "INBOX".to_string() } else { self.config.imap.folder.clone() };
                    sender.command(|out, _| async move {
                        let keyring_key = format!("imap://{}", imap_host);
                        let (username, pw) = match crate::sync::keyring::load_credentials(&keyring_key).await {
                            Ok(Some(creds)) => creds,
                            _ => return out.emit(CommandMsg::ImapFetched(Err("No IMAP credentials".to_string()))),
                        };
                        let result = crate::sync::imap::fetch_emails(&imap_host, &username, &pw, &folder).await;
                        out.emit(CommandMsg::ImapFetched(result));
                    });
                }

                if ops == 0 {
                    self.sync_status = SyncStatus::Error("No sync services configured. Set up CalDAV, WebDAV, or IMAP in Settings.".to_string());
                }
                self.sync_ops_pending = ops;
                self.needs_rebuild = true;
            }

            Message::SyncCompleted(_) | Message::SyncNotesCompleted(_) | Message::SyncAccountsCompleted(_)
            | Message::ContactsFetched(_) | Message::ContactDeleted(_) | Message::ImapFetched(_)
            | Message::EmailArchived(_) | Message::ServiceConnectionTested(_, _, _) => {
                // These are handled via CommandMsg now
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
                self.service_passwords[kind as usize] = password;
            }

            Message::TestServiceConnection(kind) => {
                let idx = kind as usize;
                if kind == ServiceKind::Imap {
                    let host = self.config.imap.host.trim().to_string();
                    let username = self.config.imap.username.clone();
                    let password = self.service_passwords[idx].clone();
                    self.service_test_status[idx] = None;
                    sender.command(move |out, _| async move {
                        let keyring_key = format!("imap://{}", host);
                        if !password.is_empty() {
                            let _ = crate::sync::keyring::store_credentials(&keyring_key, &username, &password).await;
                        }
                        let pw = if !password.is_empty() { password } else {
                            match crate::sync::keyring::load_credentials(&keyring_key).await {
                                Ok(Some((_, pw))) => pw,
                                _ => return out.emit(CommandMsg::ServiceConnectionTested(kind, Err("No password".to_string()), Vec::new())),
                            }
                        };
                        match crate::sync::imap::test_connection(&host, &username, &pw).await {
                            Ok(msg) => out.emit(CommandMsg::ServiceConnectionTested(kind, Ok(msg), Vec::new())),
                            Err(e) => out.emit(CommandMsg::ServiceConnectionTested(kind, Err(e), Vec::new())),
                        }
                    });
                } else {
                    let svc = match kind {
                        ServiceKind::Calendars => &self.config.calendars,
                        ServiceKind::Contacts => &self.config.contacts,
                        ServiceKind::Notes => &self.config.notes_sync,
                        ServiceKind::Imap => unreachable!(),
                    };
                    let url = svc.url.trim().to_string();
                    let username = svc.username.clone();
                    let password = self.service_passwords[idx].clone();
                    self.service_test_status[idx] = None;
                    sender.command(move |out, _| async move {
                        if !password.is_empty() {
                            let _ = crate::sync::keyring::store_credentials(&url, &username, &password).await;
                        }
                        let pw = if !password.is_empty() { password } else {
                            match crate::sync::keyring::load_credentials(&url).await {
                                Ok(Some((_, pw))) => pw,
                                _ => return out.emit(CommandMsg::ServiceConnectionTested(kind, Err("No password".to_string()), Vec::new())),
                            }
                        };
                        match kind {
                            ServiceKind::Calendars => {
                                let client = match CalDavClient::new(&url, &username, &pw) { Ok(c) => c, Err(e) => return out.emit(CommandMsg::ServiceConnectionTested(kind, Err(e), Vec::new())) };
                                match client.discover_calendars().await {
                                    Ok(cals) => out.emit(CommandMsg::ServiceConnectionTested(kind, Ok(format!("Found {} calendars", cals.len())), cals)),
                                    Err(e) => out.emit(CommandMsg::ServiceConnectionTested(kind, Err(e), Vec::new())),
                                }
                            }
                            ServiceKind::Contacts => {
                                let client = match crate::sync::carddav::CardDavClient::new(&url, &username, &pw) { Ok(c) => c, Err(e) => return out.emit(CommandMsg::ServiceConnectionTested(kind, Err(e), Vec::new())) };
                                match client.fetch_contacts().await {
                                    Ok(contacts) => out.emit(CommandMsg::ServiceConnectionTested(kind, Ok(format!("Connected ({} contacts)", contacts.len())), Vec::new())),
                                    Err(e) => out.emit(CommandMsg::ServiceConnectionTested(kind, Err(e), Vec::new())),
                                }
                            }
                            ServiceKind::Notes => {
                                let client = match crate::sync::webdav::WebDavClient::new(&url, &username, &pw) { Ok(c) => c, Err(e) => return out.emit(CommandMsg::ServiceConnectionTested(kind, Err(e), Vec::new())) };
                                match client.ensure_collection().await {
                                    Ok(_) => match client.list_files().await {
                                        Ok(files) => out.emit(CommandMsg::ServiceConnectionTested(kind, Ok(format!("Connected ({} files)", files.len())), Vec::new())),
                                        Err(e) => out.emit(CommandMsg::ServiceConnectionTested(kind, Err(e), Vec::new())),
                                    }
                                    Err(e) => out.emit(CommandMsg::ServiceConnectionTested(kind, Err(format!("Collection error: {}", e)), Vec::new())),
                                }
                            }
                            _ => {}
                        }
                    });
                }
            }

            Message::SetCalendarPurpose(ref href, ref purpose) => {
                self.config.calendar_assignments.retain(|a| a.calendar_href != *href);
                self.config.calendar_assignments.push(crate::config::CalendarAssignment {
                    calendar_href: href.clone(),
                    purpose: purpose.clone(),
                });
                self.save_config();
                self.needs_rebuild = true;
            }

            Message::SetImapFolder(folder) => {
                self.config.imap.folder = folder;
                self.save_config();
            }

            Message::ArchiveEmail(uid) => {
                let host = self.config.imap.host.trim().to_string();
                let folder = if self.config.imap.folder.is_empty() { "INBOX".to_string() } else { self.config.imap.folder.clone() };
                sender.command(move |out, _| async move {
                    let keyring_key = format!("imap://{}", host);
                    let (username, pw) = match crate::sync::keyring::load_credentials(&keyring_key).await {
                        Ok(Some(creds)) => creds,
                        _ => return out.emit(CommandMsg::EmailArchived(Err("No IMAP credentials".to_string()))),
                    };
                    let result = crate::sync::imap::archive_email(&host, &username, &pw, &folder, uid).await;
                    out.emit(CommandMsg::EmailArchived(result));
                });
            }

            Message::CreateTaskFromEmail(uid) => {
                if let Some(email) = self.imap_emails.iter().find(|e| e.uid == uid) {
                    let mut task = Task::new(&email.subject);
                    let mut note = format!("From: {}\n", email.from);
                    if let Some(date) = email.date {
                        note.push_str(&format!("Date: {}\n", date.format("%Y-%m-%d %H:%M")));
                    }
                    if !email.body_full.is_empty() {
                        note.push('\n');
                        note.push_str(&email.body_full);
                    }
                    task.notes = note;
                    self.inbox_tasks.push(task);
                    self.save_inbox();
                    self.rebuild_cache();
                    // Archive the email
                    let s = sender.input_sender().clone();
                    s.emit(Message::ArchiveEmail(uid));
                    self.needs_rebuild = true;
                }
            }

            // Events
            Message::CreateEvent => {
                let now = chrono::Local::now();
                let today = now.format("%Y-%m-%d").to_string();
                let now_time = now.format("%H:%M").to_string();
                let hour = now.hour();
                let default_cal = self.config.event_calendar_hrefs().first().cloned().unwrap_or_default();
                self.event_form = Some(EventForm {
                    editing: None, title: String::new(),
                    start_date: today.clone(), start_time: now_time,
                    end_date: if hour >= 23 { (now.date_naive() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string() } else { today },
                    end_time: format!("{:02}:00", (hour + 1) % 24),
                    all_day: false, location: String::new(), description: String::new(),
                    calendar_href: default_cal, start_error: None, end_error: None,
                });
                self.needs_rebuild = true;
            }

            Message::EditEvent(id) => {
                if let Some(ev) = self.events.iter().find(|e| e.id == id) {
                    self.event_form = Some(EventForm::from_event(ev));
                }
                self.needs_rebuild = true;
            }

            Message::CancelEventForm => { self.event_form = None; self.needs_rebuild = true; }
            Message::SetEventTitle(value) => { if let Some(ref mut f) = self.event_form { f.title = value; } }
            Message::SetEventStart(value) => { if let Some(ref mut f) = self.event_form { f.start_date = value; f.start_error = None; } }
            Message::SetEventStartTime(value) => { if let Some(ref mut f) = self.event_form { f.start_time = value; f.start_error = None; } }
            Message::SetEventEnd(value) => { if let Some(ref mut f) = self.event_form { f.end_date = value; f.end_error = None; } }
            Message::SetEventEndTime(value) => { if let Some(ref mut f) = self.event_form { f.end_time = value; f.end_error = None; } }
            Message::SetEventAllDay(v) => { if let Some(ref mut f) = self.event_form { f.all_day = v; } }
            Message::SetEventLocation(value) => { if let Some(ref mut f) = self.event_form { f.location = value; } }
            Message::SetEventDescription(value) => { if let Some(ref mut f) = self.event_form { f.description = value; } }
            Message::SetEventCalendar(href) => { if let Some(ref mut f) = self.event_form { f.calendar_href = href; } }

            Message::SubmitEvent => {
                if let Some(mut form) = self.event_form.take() {
                    let title = form.title.trim().to_string();
                    if title.is_empty() { self.event_form = Some(form); return; }
                    let start = parse_form_datetime(&form.start_date, &form.start_time, form.all_day);
                    let end = parse_form_datetime(&form.end_date, &form.end_time, form.all_day);
                    if start.is_none() { form.start_error = Some(crate::fl!("validation-invalid-date")); }
                    if end.is_none() { form.end_error = Some(crate::fl!("validation-invalid-date")); }
                    if start.is_none() || end.is_none() { self.event_form = Some(form); self.needs_rebuild = true; return; }
                    let (start, end) = (start.unwrap(), end.unwrap());
                    let cal_name = self.all_discovered_calendars().iter()
                        .find(|c| c.href == form.calendar_href).map(|c| c.display_name.clone()).unwrap_or_default();
                    let mut ev = CalendarEvent::new(title, start, end);
                    ev.all_day = form.all_day;
                    ev.location = form.location;
                    ev.description = form.description;
                    ev.calendar_href = form.calendar_href;
                    ev.calendar_name = cal_name;
                    self.events.push(ev);
                    self.save_events();
                    self.needs_rebuild = true;
                }
            }

            Message::UpdateEvent(id) => {
                if let Some(mut form) = self.event_form.take() {
                    let title = form.title.trim().to_string();
                    if title.is_empty() { self.event_form = Some(form); return; }
                    let start = parse_form_datetime(&form.start_date, &form.start_time, form.all_day);
                    let end = parse_form_datetime(&form.end_date, &form.end_time, form.all_day);
                    if start.is_none() { form.start_error = Some(crate::fl!("validation-invalid-date")); }
                    if end.is_none() { form.end_error = Some(crate::fl!("validation-invalid-date")); }
                    if start.is_none() || end.is_none() { self.event_form = Some(form); self.needs_rebuild = true; return; }
                    let (start, end) = (start.unwrap(), end.unwrap());
                    if let Some(ev) = self.events.iter_mut().find(|e| e.id == id) {
                        ev.title = title; ev.start = start; ev.end = end;
                        ev.all_day = form.all_day; ev.location = form.location;
                        ev.description = form.description; ev.calendar_href = form.calendar_href;
                    }
                    self.save_events();
                    self.needs_rebuild = true;
                }
            }

            Message::DeleteEvent(id) => {
                if let Some(ev) = self.events.iter().find(|e| e.id == id) {
                    if let Some(ref sync_href) = ev.sync_href {
                        let caldav_url = self.config.calendars.url.clone();
                        let href = sync_href.clone();
                        if !caldav_url.is_empty() {
                            sender.command(move |out, _| async move {
                                if let Ok(Some((username, pw))) = crate::sync::keyring::load_credentials(&caldav_url).await {
                                    if let Ok(client) = CalDavClient::new(&caldav_url, &username, &pw) {
                                        let _ = client.delete_vtodo(&href, "").await;
                                    }
                                }
                                out.emit(CommandMsg::EventDeleted);
                            });
                        }
                    }
                }
                self.events.retain(|e| e.id != id);
                self.save_events();
                self.needs_rebuild = true;
            }

            Message::CalendarPrevMonth => { self.month_calendar.prev_month(); self.needs_rebuild = true; }
            Message::CalendarNextMonth => { self.month_calendar.next_month(); self.needs_rebuild = true; }
            Message::CalendarSelectDay(date) => { self.month_calendar.select_day(date); self.needs_rebuild = true; }

            // Conflict resolution
            Message::ImportConflictTask(idx) => {
                if idx < self.sync_conflicts.len() {
                    if let SyncConflict::RemoteOnly { task, .. } = self.sync_conflicts.remove(idx) {
                        self.inbox_tasks.push(task);
                        self.save_all();
                        self.rebuild_cache();
                    }
                }
                self.needs_rebuild = true;
            }

            Message::DeleteConflict(idx) => {
                if idx < self.sync_conflicts.len() {
                    match self.sync_conflicts.remove(idx) {
                        SyncConflict::RemoteOnly { href, .. } => { self.pending_deletions.push(href); self.save_pending_ops(); }
                        SyncConflict::LocalOnly { task_id, .. } => {
                            self.remove_task(task_id);
                            self.save_all();
                            self.rebuild_cache();
                        }
                        SyncConflict::StateMismatch { .. } => {}
                    }
                }
                self.needs_rebuild = true;
            }

            Message::AcceptRemoteState(idx) => {
                if idx < self.sync_conflicts.len() {
                    if let SyncConflict::StateMismatch { task_id, remote_state, .. } = self.sync_conflicts.remove(idx) {
                        if let Some(new_state) = TaskState::from_keyword(&remote_state) {
                            if let Some(mut task) = self.remove_task(task_id) {
                                task.state = new_state;
                                self.route_task_by_state(task);
                                self.save_all();
                                self.rebuild_cache();
                            }
                        }
                    }
                }
                self.needs_rebuild = true;
            }

            Message::AcceptLocalState(idx) => {
                if idx < self.sync_conflicts.len() {
                    if let SyncConflict::StateMismatch { task_id, href, .. } = self.sync_conflicts.remove(idx) {
                        let all = self.all_active_tasks();
                        if let Some(task) = all.iter().find(|t| t.id == task_id) {
                            let ical = crate::sync::vtodo::task_to_vcalendar(task);
                            self.pending_completions.push((href, ical));
                            self.save_pending_ops();
                        }
                    }
                }
                self.needs_rebuild = true;
            }
        }
    }

    fn update_cmd(&mut self, msg: CommandMsg, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            CommandMsg::SyncCompleted(result) => {
                match result {
                    Ok(sync_result) => {
                        for (href, token) in &sync_result.new_sync_tokens {
                            self.config.set_sync_token(href, token);
                        }
                        let archive_path = self.config.archive_path();
                        for id in &sync_result.deleted_local {
                            let task = self.inbox_tasks.iter()
                                .chain(self.next_tasks.iter()).chain(self.waiting_tasks.iter())
                                .chain(self.someday_tasks.iter())
                                .chain(self.projects.iter().flat_map(|p| p.tasks.iter()))
                                .find(|t| t.id == *id).cloned();
                            if let Some(task) = task { let _ = OrgWriter::append_to_file(&archive_path, &task); }
                            self.inbox_tasks.retain(|t| t.id != *id);
                            self.next_tasks.retain(|t| t.id != *id);
                            self.waiting_tasks.retain(|t| t.id != *id);
                            self.someday_tasks.retain(|t| t.id != *id);
                            for project in &mut self.projects { project.tasks.retain(|t| t.id != *id); }
                        }
                        self.rebuild_task_index();
                        for pulled in &sync_result.pulled {
                            if let Some(habit) = self.habits.iter_mut().find(|h| h.task.id == pulled.id) {
                                // Merge remote logbook entries into local completions
                                for entry in &pulled.logbook_entries {
                                    if !habit.completions.contains(entry) {
                                        habit.completions.push(*entry);
                                    }
                                }
                                habit.completions.sort();
                                habit.task = pulled.clone();
                                let today = chrono::Local::now().date_naive();
                                habit.recalculate_streak(today);
                                continue;
                            }
                            if pulled.extra_tags.contains(&"shopping".to_string()) {
                                if let Some(existing) = self.shopping_tasks.iter_mut().find(|t| t.id == pulled.id) { *existing = pulled.clone(); }
                                else { self.shopping_tasks.push(pulled.clone()); }
                                continue;
                            }
                            let _existing = self.remove_task(pulled.id);
                            if let Some(ref project_name) = pulled.project {
                                let project_name = project_name.clone();
                                if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) { project.tasks.push(pulled.clone()); continue; }
                            }
                            self.route_task_by_state(pulled.clone());
                        }
                        self.save_habits(); self.save_shopping();
                        let habit_ids: HashSet<uuid::Uuid> = self.habits.iter().map(|h| h.task.id).collect();
                        let shopping_ids: HashSet<uuid::Uuid> = self.shopping_tasks.iter().map(|t| t.id).collect();
                        let exclude: HashSet<uuid::Uuid> = habit_ids.union(&shopping_ids).copied().collect();
                        self.inbox_tasks.retain(|t| !exclude.contains(&t.id));
                        self.next_tasks.retain(|t| !exclude.contains(&t.id));
                        self.waiting_tasks.retain(|t| !exclude.contains(&t.id));
                        self.someday_tasks.retain(|t| !exclude.contains(&t.id));
                        for project in &mut self.projects { project.tasks.retain(|t| !exclude.contains(&t.id)); }
                        let today = chrono::Local::now().date_naive();
                        let mut synced_plan_ids: Vec<uuid::Uuid> = Vec::new();
                        for pulled in &sync_result.pulled { if pulled.dayplan_date == Some(today) { synced_plan_ids.push(pulled.id); } }
                        if !synced_plan_ids.is_empty() {
                            let plan = self.ensure_day_plan();
                            for id in synced_plan_ids { if !plan.confirmed_task_ids.contains(&id) { plan.confirmed_task_ids.push(id); } }
                            self.save_day_plan();
                        }
                        for id in &sync_result.deleted_events { self.events.retain(|e| e.id != *id); }
                        for pulled_event in &sync_result.pulled_events { self.events.retain(|e| e.id != pulled_event.id); self.events.push(pulled_event.clone()); }
                        self.save_events(); self.save_all(); self.save_config();
                        self.pending_completions.clear(); self.pending_deletions.clear();
                        self.save_pending_ops();
                        self.sync_conflicts = sync_result.conflicts;
                    }
                    Err(e) => {
                        log::error!("Sync failed: {}", e);
                        if self.sync_status == SyncStatus::Syncing { self.sync_status = SyncStatus::Error(e); }
                    }
                }
                self.finish_sync_op();
                self.needs_rebuild = true;
            }

            CommandMsg::SyncNotesCompleted(result) => {
                if let Ok(sync_result) = result {
                    for pulled in &sync_result.pulled { self.notes.retain(|n| n.id != pulled.id); self.notes.push(pulled.clone()); }
                    self.notes.sort_by(|a, b| a.title.cmp(&b.title));
                    self.backlink_index = build_backlink_index(&self.notes);
                    self.save_all_notes();
                }
                self.finish_sync_op();
                self.needs_rebuild = true;
            }

            CommandMsg::SyncAccountsCompleted(result) => {
                if let Ok(sync_result) = result { self.accounts = sync_result.items; self.save_accounts(); }
                self.finish_sync_op();
                self.needs_rebuild = true;
            }

            CommandMsg::ContactsFetched(result) => {
                if let Ok(remote) = result {
                    crate::sync::carddav::merge_contacts(&mut self.contacts, remote);
                    let _ = crate::sync::carddav::save_contacts(&self.config.contacts_path(), &self.contacts);
                }
                self.finish_sync_op();
                self.needs_rebuild = true;
            }

            CommandMsg::ContactDeleted(result) => {
                if let Err(e) = result { log::error!("Failed to delete contact from server: {}", e); }
            }

            CommandMsg::ImapFetched(result) => {
                if let Ok(emails) = result {
                    if self.archived_email_uids.is_empty() { self.imap_emails = emails; }
                    else { self.imap_emails = emails.into_iter().filter(|e| !self.archived_email_uids.contains(&e.uid)).collect(); }
                }
                self.finish_sync_op();
                self.needs_rebuild = true;
            }

            CommandMsg::EmailArchived(result) => {
                if let Ok(uid) = result { self.imap_emails.retain(|e| e.uid != uid); self.archived_email_uids.insert(uid); }
                self.needs_rebuild = true;
            }

            CommandMsg::ServiceConnectionTested(kind, ref result, ref cals) => {
                self.service_test_status[kind as usize] = Some(result.clone());
                if kind == ServiceKind::Calendars { self.discovered_calendars = cals.clone(); }
                self.needs_rebuild = true;
            }

            CommandMsg::EventDeleted => {}
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        // Don't programmatically set toggle buttons — they maintain their own state
        // and setting them would trigger connect_toggled in a feedback loop.

        // Show/hide sidebar based on mode
        let hide_sidebar = self.app_mode == AppMode::Do || self.launch_mode == LaunchMode::Today;
        widgets.split_view.set_collapsed(hide_sidebar);
        if hide_sidebar {
            widgets.split_view.set_show_content(true);
        }

        // Update search visibility
        let what = match self.active_view {
            ActiveView::What(w) => w,
            ActiveView::When(_) => WhatPage::DailyPlanning,
        };
        let searchable = matches!(what,
            WhatPage::Inbox | WhatPage::AllTasks | WhatPage::NextActions | WhatPage::Projects
            | WhatPage::Waiting | WhatPage::Someday | WhatPage::Habits | WhatPage::Media
            | WhatPage::Shopping | WhatPage::Contacts | WhatPage::Accounts | WhatPage::Notes
        );
        widgets.search_entry.set_visible(searchable && self.app_mode == AppMode::Plan);

        // Sync button feedback — spinner while syncing, icon when idle
        match &self.sync_status {
            SyncStatus::Syncing => {
                widgets.sync_button.set_icon_name("info-outline-symbolic");
                widgets.sync_button.set_sensitive(false);
                // Show "syncing" toast once
                if widgets.last_sync_toast.as_deref() != Some("syncing") {
                    let toast = adw::Toast::new("Syncing…");
                    toast.set_timeout(1);
                    widgets.toast_overlay.add_toast(toast);
                    widgets.last_sync_toast = Some("syncing".to_string());
                }
            }
            SyncStatus::LastSynced(t) => {
                widgets.sync_button.set_icon_name("media-playlist-repeat-symbolic");
                widgets.sync_button.set_sensitive(true);
                let key = format!("done:{}", t);
                if widgets.last_sync_toast.as_deref() != Some(&key) {
                    let toast = adw::Toast::new(&format!("Synced at {}", t));
                    toast.set_timeout(2);
                    widgets.toast_overlay.add_toast(toast);
                    widgets.last_sync_toast = Some(key);
                }
            }
            SyncStatus::Error(e) => {
                widgets.sync_button.set_icon_name("dialog-error-symbolic");
                widgets.sync_button.set_sensitive(true);
                let key = format!("err:{}", e);
                if widgets.last_sync_toast.as_deref() != Some(&key) {
                    let toast = adw::Toast::new(&format!("Sync error: {}", e));
                    toast.set_timeout(5);
                    widgets.toast_overlay.add_toast(toast);
                    widgets.last_sync_toast = Some(key);
                }
            }
            SyncStatus::Idle => {
                widgets.sync_button.set_icon_name("media-playlist-repeat-symbolic");
                widgets.sync_button.set_sensitive(true);
            }
        }

        // Manage timer
        if self.active_timer.is_some() && widgets.timer_source.is_none() {
            let s = sender.input_sender().clone();
            widgets.timer_source = Some(gtk::glib::timeout_add_seconds_local(1, move || {
                s.emit(Message::TimerTick);
                gtk::glib::ControlFlow::Continue
            }));
        } else if self.active_timer.is_none() {
            if let Some(source) = widgets.timer_source.take() {
                source.remove();
            }
        }

        // Manage sync animation
        if self.sync_status == SyncStatus::Syncing && widgets.sync_anim_source.is_none() {
            let s = sender.input_sender().clone();
            widgets.sync_anim_source = Some(gtk::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                s.emit(Message::SyncAnimTick);
                gtk::glib::ControlFlow::Continue
            }));
        } else if self.sync_status != SyncStatus::Syncing {
            if let Some(source) = widgets.sync_anim_source.take() {
                source.remove();
            }
        }

        if !self.needs_rebuild {
            return;
        }

        // Rebuild page content
        let s = sender.input_sender();
        ui::clear_box(&widgets.page_content);

        if self.app_mode == AppMode::Do {
            let page = pages::do_mode::do_mode_view(
                &self.day_plan, &self.all_tasks_cache, &self.habits,
                &self.media_items, &self.shopping_tasks,
                self.expanded_task, &self.note_inputs, self.active_timer, s,
            );
            widgets.page_content.append(&page);
            return;
        }

        let project_names: Vec<String> = self.projects.iter().map(|p| p.name.clone()).collect();
        let row_ctx = crate::components::task_row::TaskRowCtx {
            contexts: self.config.contexts.clone(),
            project_names: project_names.clone(),
            expanded_task: self.expanded_task,
            note_inputs: self.note_inputs.clone(),
            waiting_for_inputs: self.waiting_for_inputs.clone(),
            contacts: self.contacts.clone(),
        };

        // Apply search filter
        let q = &self.search_query;
        let filtered_tasks: Vec<Task>;
        let tasks: &[Task] = if !q.is_empty() && searchable {
            let lq = q.to_lowercase();
            filtered_tasks = self.all_tasks_cache.iter()
                .filter(|t| t.title.to_lowercase().contains(&lq)).cloned().collect();
            &filtered_tasks
        } else {
            &self.all_tasks_cache
        };

        let page: gtk::Widget = match what {
            WhatPage::DailyPlanning => {
                let mut plan_tasks = self.all_tasks_cache.clone();
                plan_tasks.extend(self.habits.iter().map(|h| h.task.clone()));
                pages::daily_planning::daily_planning_view(
                    &self.day_plan, &plan_tasks, &self.media_items, &self.shopping_tasks,
                    &self.config.contexts, &self.rejected_suggestions, s,
                )
            }
            WhatPage::Inbox => pages::inbox::inbox_view(tasks, &self.imap_emails, &self.inbox_input, &row_ctx, s),
            WhatPage::AllTasks => pages::all_tasks::all_tasks_view(tasks, &row_ctx, self.all_tasks_sort, s),
            WhatPage::NextActions => pages::next_actions::next_actions_view(tasks, &row_ctx, s),
            WhatPage::Projects => {
                let filtered_projects: Vec<Project>;
                let projects = if !q.is_empty() {
                    let lq = q.to_lowercase();
                    filtered_projects = self.projects.iter()
                        .filter(|p| p.name.to_lowercase().contains(&lq) || p.tasks.iter().any(|t| t.title.to_lowercase().contains(&lq)))
                        .cloned().collect();
                    &filtered_projects
                } else {
                    &self.projects
                };
                pages::projects::projects_view(projects, &self.project_input, &self.project_task_inputs, &row_ctx, s)
            }
            WhatPage::Waiting => pages::waiting::waiting_view(tasks, &row_ctx, s),
            WhatPage::Someday => pages::someday::someday_view(tasks, &row_ctx, s),
            WhatPage::Habits => {
                let filtered_habits: Vec<Habit>;
                let habits = if !q.is_empty() {
                    let lq = q.to_lowercase();
                    filtered_habits = self.habits.iter().filter(|h| h.task.title.to_lowercase().contains(&lq)).cloned().collect();
                    &filtered_habits
                } else { &self.habits };
                pages::habits::habits_view(habits, &self.habit_input, s)
            }
            WhatPage::Conflicts => pages::conflicts::conflicts_view(&self.sync_conflicts, s),
            WhatPage::Review => pages::review::review_view(&self.all_tasks_cache, &self.projects, &self.habits, &self.review_checked, s),
            WhatPage::Tickler => {
                let flat_cals = self.all_discovered_calendars();
                pages::temporal::agenda_view(
                    &self.all_tasks_cache, &self.habits, &self.events,
                    self.event_form.as_ref(), &row_ctx, &flat_cals, &self.month_calendar, s,
                )
            }
            WhatPage::Media => pages::list::list_view(
                &self.media_items, &self.media_input, crate::fl!("media-placeholder"),
                crate::fl!("media-empty"), ListKind::Media,
                &self.flipped_list_items, self.pending_delete_list_item, &self.note_inputs, s,
            ),
            WhatPage::Shopping => pages::list::shopping_view(
                &self.shopping_tasks, &self.shopping_input,
                &self.flipped_list_items, self.pending_delete_list_item, &self.note_inputs, s,
            ),
            WhatPage::Contacts => pages::contacts::contacts_view(
                &self.contacts, &self.contact_input, &self.flipped_contacts,
                self.editing_contact, self.pending_delete_contact, s,
            ),
            WhatPage::Accounts => pages::accounts::accounts_view(
                &self.accounts, &self.account_input, self.expanded_account,
                self.pending_delete_account, s,
            ),
            WhatPage::Notes => pages::notes::notes_view(
                &self.notes, &self.note_input, &self.flipped_notes,
                self.editing_note, self.pending_delete_note,
                &self.note_body_buffer, &self.note_edit_buffer,
                &self.note_link_search, &self.backlink_index,
                &self.contacts, &self.accounts, &self.projects,
                &self.all_tasks_cache, &self.media_items, &self.shopping_tasks, s,
            ),
            WhatPage::Archive => pages::archive::archive_view(&self.archive_tasks, &self.archive_search, s),
            WhatPage::Settings => pages::settings::settings_view(
                &self.config, &self.settings_context_input, &self.service_passwords,
                &self.service_test_status, &self.discovered_calendars, &self.sync_status, s,
            ),
        };

        widgets.page_content.append(&page);
    }
}

// --- Impl block for data methods (unchanged from original) ---
impl Lamp {
    fn route_task_by_state(&mut self, task: Task) {
        match task.state {
            TaskState::Todo => self.inbox_tasks.push(task),
            TaskState::Next => self.next_tasks.push(task),
            TaskState::Waiting => self.waiting_tasks.push(task),
            TaskState::Someday => self.someday_tasks.push(task),
            _ => self.inbox_tasks.push(task),
        }
    }

    fn rebuild_cache(&mut self) {
        let mut tasks = Vec::new();
        tasks.extend(self.inbox_tasks.iter().cloned());
        tasks.extend(self.next_tasks.iter().cloned());
        tasks.extend(self.waiting_tasks.iter().cloned());
        tasks.extend(self.someday_tasks.iter().cloned());
        for project in &self.projects { tasks.extend(project.tasks.iter().cloned()); }
        self.all_tasks_cache = tasks;
        self.rebuild_task_index();
    }

    fn rebuild_task_index(&mut self) {
        self.task_index.clear();
        for task in &self.inbox_tasks { self.task_index.insert(task.id, TaskLocation::Inbox); }
        for task in &self.next_tasks { self.task_index.insert(task.id, TaskLocation::Next); }
        for task in &self.waiting_tasks { self.task_index.insert(task.id, TaskLocation::Waiting); }
        for task in &self.someday_tasks { self.task_index.insert(task.id, TaskLocation::Someday); }
        for project in &self.projects {
            for task in &project.tasks { self.task_index.insert(task.id, TaskLocation::Project(project.name.clone())); }
        }
    }

    fn all_active_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        tasks.extend(self.inbox_tasks.iter().cloned());
        tasks.extend(self.next_tasks.iter().cloned());
        tasks.extend(self.waiting_tasks.iter().cloned());
        tasks.extend(self.someday_tasks.iter().cloned());
        for project in &self.projects { tasks.extend(project.tasks.iter().cloned()); }
        tasks.extend(self.habits.iter().map(|h| {
            let mut task = h.task.clone();
            // Stamp completions into logbook_entries so they sync to CalDAV
            task.logbook_entries = h.completions.clone();
            task
        }));
        tasks.extend(self.shopping_tasks.iter().cloned());
        tasks
    }

    fn toggle_done(&mut self, id: uuid::Uuid) {
        if let Some(mut task) = self.remove_task(id) {
            if task.state.is_done() {
                task.state = TaskState::Todo;
                task.completed = None;
                self.route_task_by_state(task);
            } else {
                task.complete();
                if let Some(ref sync_href) = task.sync_href {
                    let ical = crate::sync::vtodo::task_to_vcalendar(&task);
                    self.pending_completions.push((sync_href.clone(), ical));
                    self.save_pending_ops();
                }
                let archive_path = self.config.archive_path();
                if OrgWriter::append_to_file(&archive_path, &task).is_err() {
                    self.route_task_by_state(task);
                }
                self.save_all();
                return;
            }
            self.save_all();
        }
    }

    fn set_task_state(&mut self, id: uuid::Uuid, state: TaskState) {
        if let Some(mut task) = self.remove_task(id) {
            let old_state = task.state.clone();
            task.state = state.clone();
            if state.is_done() && !old_state.is_done() { task.completed = Some(chrono::Local::now().naive_local()); }
            if !state.is_done() && old_state.is_done() { task.completed = None; }
            if state == TaskState::Waiting && old_state != TaskState::Waiting {
                if task.delegated.is_none() { task.delegated = Some(chrono::Local::now().date_naive()); }
            }
            if state != TaskState::Waiting && old_state == TaskState::Waiting { task.delegated = None; task.follow_up = None; }
            if let Some(ref project_name) = task.project {
                let project_name = project_name.clone();
                if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                    project.tasks.push(task);
                } else { task.project = None; self.route_task_by_state(task); }
            } else {
                self.route_task_by_state(task);
            }
            self.save_all();
        }
    }

    fn set_task_priority(&mut self, id: uuid::Uuid, priority: Option<Priority>) {
        let list = match self.task_index.get(&id) {
            Some(TaskLocation::Inbox) => Some(&mut self.inbox_tasks as &mut Vec<Task>),
            Some(TaskLocation::Next) => Some(&mut self.next_tasks),
            Some(TaskLocation::Waiting) => Some(&mut self.waiting_tasks),
            Some(TaskLocation::Someday) => Some(&mut self.someday_tasks),
            Some(TaskLocation::Project(name)) => {
                let name = name.clone();
                self.projects.iter_mut().find(|p| p.name == name).map(|p| &mut p.tasks as &mut Vec<Task>)
            }
            None => None,
        };
        if let Some(list) = list {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) { task.priority = priority; self.save_all(); }
        }
    }

    fn modify_task(&mut self, id: uuid::Uuid, f: impl FnOnce(&mut Task)) {
        let list = match self.task_index.get(&id) {
            Some(TaskLocation::Inbox) => Some(&mut self.inbox_tasks as &mut Vec<Task>),
            Some(TaskLocation::Next) => Some(&mut self.next_tasks),
            Some(TaskLocation::Waiting) => Some(&mut self.waiting_tasks),
            Some(TaskLocation::Someday) => Some(&mut self.someday_tasks),
            Some(TaskLocation::Project(name)) => {
                let name = name.clone();
                self.projects.iter_mut().find(|p| p.name == name).map(|p| &mut p.tasks as &mut Vec<Task>)
            }
            None => None,
        };
        if let Some(list) = list {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) { f(task); self.save_all(); }
        }
    }

    fn remove_task(&mut self, id: uuid::Uuid) -> Option<Task> {
        let list = match self.task_index.get(&id) {
            Some(TaskLocation::Inbox) => Some(&mut self.inbox_tasks as &mut Vec<Task>),
            Some(TaskLocation::Next) => Some(&mut self.next_tasks),
            Some(TaskLocation::Waiting) => Some(&mut self.waiting_tasks),
            Some(TaskLocation::Someday) => Some(&mut self.someday_tasks),
            Some(TaskLocation::Project(name)) => {
                let name = name.clone();
                self.projects.iter_mut().find(|p| p.name == name).map(|p| &mut p.tasks as &mut Vec<Task>)
            }
            None => return None,
        };
        list.and_then(|list| {
            list.iter().position(|t| t.id == id).map(|pos| list.remove(pos))
        })
    }

    fn ensure_day_plan(&mut self) -> &mut DayPlan {
        let today = chrono::Local::now().date_naive();
        if self.day_plan.as_ref().is_none_or(|dp| dp.is_stale(today)) {
            let mut plan = DayPlan::new(today);
            for habit in &self.habits { if habit.is_due(today) { plan.confirmed_task_ids.push(habit.task.id); } }
            self.day_plan = Some(plan);
            self.rejected_suggestions.clear();
        }
        self.day_plan.as_mut().unwrap()
    }

    fn save_day_plan(&self) {
        if let Some(ref plan) = self.day_plan {
            let content = OrgWriter::write_day_plan(plan);
            let _ = std::fs::write(self.config.dayplan_path(), &content);
        }
    }

    fn save_pending_ops(&self) {
        let path = self.config.org_directory.join(".pending_ops.json");
        let data = serde_json::json!({
            "completions": self.pending_completions,
            "deletions": self.pending_deletions,
        });
        let _ = std::fs::write(&path, serde_json::to_string(&data).unwrap_or_default());
    }

    fn load_pending_ops(config: &LampConfig) -> (Vec<(String, String)>, Vec<String>) {
        let path = config.org_directory.join(".pending_ops.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let completions = data["completions"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| {
                            let arr = v.as_array()?;
                            Some((arr[0].as_str()?.to_string(), arr[1].as_str()?.to_string()))
                        }).collect())
                        .unwrap_or_default();
                    let deletions = data["deletions"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    return (completions, deletions);
                }
                (Vec::new(), Vec::new())
            }
            Err(_) => (Vec::new(), Vec::new()),
        }
    }

    fn save_inbox(&mut self) {
        let content = OrgWriter::write_file("Inbox", &self.inbox_tasks);
        let _ = std::fs::write(self.config.inbox_path(), &content);
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
            let _ = std::fs::write(&path, &content);
        }
        self.save_projects();
        self.save_habits();
        self.rebuild_cache();
    }

    fn save_projects(&self) {
        let content = OrgWriter::write_projects_file(&self.projects);
        let _ = std::fs::write(self.config.projects_path(), &content);
    }

    fn save_media(&self) {
        let content = OrgWriter::write_list_items_file("Media Recommendations", &self.media_items);
        let _ = std::fs::write(self.config.media_path(), &content);
    }

    fn save_shopping(&self) {
        let content = OrgWriter::write_file("Shopping", &self.shopping_tasks);
        let _ = std::fs::write(self.config.shopping_path(), &content);
    }

    fn save_contacts(&self) {
        let _ = crate::sync::carddav::save_contacts(&self.config.contacts_path(), &self.contacts);
    }

    fn save_accounts(&self) {
        let content = OrgWriter::write_accounts_file(&self.accounts);
        let _ = std::fs::write(self.config.accounts_path(), &content);
    }

    fn save_note(&self, note: &Note) {
        let path = self.config.notes_dir().join(format!("{}.org", note.id));
        let content = OrgWriter::write_note_file(note);
        let _ = std::fs::write(&path, &content);
    }

    fn save_all_notes(&self) {
        for note in &self.notes { self.save_note(note); }
    }

    fn delete_note_file(&self, id: uuid::Uuid) {
        let path = self.config.notes_dir().join(format!("{}.org", id));
        if path.exists() { let _ = std::fs::remove_file(&path); }
    }

    fn save_habits(&self) {
        let mut out = String::new();
        out.push_str("#+TITLE: Habits\n#+TODO: TODO NEXT WAITING SOMEDAY | DONE CANCELLED\n\n");
        for habit in &self.habits { out.push_str(&OrgWriter::write_habit_task(&habit.task, &habit.completions)); out.push('\n'); }
        let _ = std::fs::write(self.config.habits_path(), &out);
    }

    fn save_events(&self) { event::save_events(&self.config.events_cache_path(), &self.events); }

    fn all_discovered_calendars(&self) -> Vec<CalendarInfo> { self.discovered_calendars.clone() }

    fn save_config(&self) { self.config.save(); }

    fn finish_sync_op(&mut self) {
        self.sync_ops_pending = self.sync_ops_pending.saturating_sub(1);
        if self.sync_ops_pending == 0 && self.sync_status == SyncStatus::Syncing {
            let now = chrono::Local::now().format("%H:%M").to_string();
            self.sync_status = SyncStatus::LastSynced(now);
        }
    }
}

// --- Free functions ---

fn parse_form_datetime(date_str: &str, time_str: &str, all_day: bool) -> Option<chrono::NaiveDateTime> {
    let date = chrono::NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d").ok()?;
    if all_day { Some(date.and_hms_opt(0, 0, 0).unwrap()) }
    else { let time = chrono::NaiveTime::parse_from_str(time_str.trim(), "%H:%M").ok()?; Some(date.and_time(time)) }
}

fn parse_optional_date(s: &str) -> Result<Option<chrono::NaiveDate>, String> {
    let s = s.trim();
    if s.is_empty() { Ok(None) }
    else { chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map(Some).map_err(|e| format!("{}", e)) }
}

fn sentence_case(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() { return String::new(); }
    let mut chars = s.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    first + chars.as_str()
}

fn load_tasks(path: &std::path::Path) -> Vec<Task> {
    std::fs::read_to_string(path).ok().map(|c| convert::parse_tasks(&c)).unwrap_or_default()
}

fn load_habits(path: &std::path::Path) -> Vec<Habit> {
    std::fs::read_to_string(path).ok().map(|c| convert::parse_habits(&c)).unwrap_or_default()
}

fn load_projects(path: &std::path::Path) -> Vec<Project> {
    std::fs::read_to_string(path).ok().map(|c| convert::parse_projects(&c)).unwrap_or_default()
}

fn load_list_items(path: &std::path::Path) -> Vec<ListItem> {
    std::fs::read_to_string(path).ok().map(|c| convert::parse_list_items(&c)).unwrap_or_default()
}

fn load_shopping_tasks(path: &std::path::Path) -> Vec<Task> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let mut tasks = convert::parse_tasks(&content);
            for task in &mut tasks {
                if !task.extra_tags.contains(&"shopping".to_string()) { task.extra_tags.push("shopping".to_string()); }
            }
            if tasks.is_empty() {
                let items = convert::parse_list_items(&content);
                tasks = items.into_iter().map(|item| {
                    let mut task = Task::new(item.title);
                    task.id = item.id; task.notes = item.notes; task.created = item.created;
                    task.extra_tags = vec!["shopping".to_string()];
                    if item.done { task.state = TaskState::Done; task.completed = Some(item.created); }
                    task
                }).collect();
            }
            tasks
        }
        Err(_) => Vec::new(),
    }
}

fn load_day_plan(path: &std::path::Path) -> Option<DayPlan> {
    std::fs::read_to_string(path).ok().and_then(|c| convert::parse_day_plan(&c))
}

fn load_accounts(path: &std::path::Path) -> Vec<Account> {
    std::fs::read_to_string(path).ok().map(|c| convert::parse_accounts(&c)).unwrap_or_default()
}

fn load_notes_dir(notes_dir: &std::path::Path, old_notes_path: &std::path::Path) -> Vec<Note> {
    let dir_has_files = notes_dir.read_dir().ok()
        .map(|mut rd| rd.any(|e| e.is_ok_and(|e| e.path().extension().is_some_and(|ext| ext == "org"))))
        .unwrap_or(false);
    if dir_has_files {
        load_notes_from_dir(notes_dir)
    } else if old_notes_path.exists() {
        let notes = std::fs::read_to_string(old_notes_path).ok().map(|c| convert::parse_notes(&c)).unwrap_or_default();
        for note in &notes {
            let path = notes_dir.join(format!("{}.org", note.id));
            let content = OrgWriter::write_note_file(note);
            let _ = std::fs::write(&path, &content);
        }
        let _ = std::fs::remove_file(old_notes_path);
        notes
    } else { Vec::new() }
}

fn load_notes_from_dir(dir: &std::path::Path) -> Vec<Note> {
    let mut notes = Vec::new();
    let entries = match std::fs::read_dir(dir) { Ok(e) => e, Err(_) => return notes };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "org") {
            if let Ok(content) = std::fs::read_to_string(&path) { notes.append(&mut convert::parse_notes(&content)); }
        }
    }
    notes
}

fn build_backlink_index(notes: &[Note]) -> HashMap<LinkTarget, Vec<uuid::Uuid>> {
    let mut idx: HashMap<LinkTarget, Vec<uuid::Uuid>> = HashMap::new();
    for note in notes { for link in &note.links { idx.entry(link.clone()).or_default().push(note.id); } }
    idx
}
