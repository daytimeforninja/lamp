use chrono::NaiveDate;

use crate::config::CalendarPurpose;
use crate::core::link::LinkTarget;
use crate::core::task::{Priority, TaskState};
use crate::sync::caldav::CalendarInfo;
use crate::sync::carddav::{Contact, ContactCategory};
use crate::sync::imap::ImapEmail;
use crate::sync::webdav::NoteSyncResult;
use crate::sync::SyncResult;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceKind {
    Calendars,
    Contacts,
    Notes,
    Imap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactField {
    Email,
    Phone,
    Website,
    Signal,
    PreferredMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountField {
    Name,
    Url,
    Notes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteField {
    Title,
    Body,
    Tags,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Plan,
    Do,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    SelectWhen(WhenPage),

    // Mode
    SetMode(AppMode),

    // Task CRUD
    AddTask(String),
    UpdateTaskTitle(Uuid, String),
    SetTaskState(Uuid, TaskState),
    SetTaskPriority(Uuid, Option<Priority>),
    DeleteTask(Uuid),
    ToggleTaskDone(Uuid),
    MoveToProject(Uuid, String),
    AddContext(Uuid, String),
    RemoveContext(Uuid, String),

    // ESC
    SetTaskEsc(Uuid, Option<u32>),

    // Waiting for / delegation
    SetWaitingFor(Uuid, String),
    WaitingForInputChanged(Uuid, String),
    SetFollowUp(Uuid, Option<NaiveDate>),

    // Dates
    SetScheduled(Uuid, Option<NaiveDate>),
    SetDeadline(Uuid, Option<NaiveDate>),

    // Task notes
    ToggleTaskExpand(Uuid),
    NoteInputChanged(Uuid, String),
    AppendNote(Uuid),

    // Inbox input
    InboxInputChanged(String),
    InboxSubmit,

    // Projects
    CreateProject(String),
    DeleteProject(String),
    ProjectInputChanged(String),
    ProjectSubmit,
    ProjectTaskInputChanged(String, String),
    AddTaskToProject(String),
    SetProjectPurpose(String, String),
    SetProjectOutcome(String, String),
    SetProjectBrainstorm(String, String),
    ReorderProjectTask(String, Uuid, isize),

    // Habits
    CompleteHabit(Uuid),
    DeleteHabit(Uuid),
    HabitInputChanged(String),
    HabitSubmit,

    // List items (Media / Shopping)
    ListInputChanged(ListKind, String),
    ListSubmit(ListKind),
    DeleteListItem(ListKind, uuid::Uuid),
    ToggleListItemDone(ListKind, uuid::Uuid),
    FlipListItem(uuid::Uuid),
    ConfirmDeleteListItem(ListKind, uuid::Uuid),
    CancelDeleteListItem,

    // Contacts CRUD
    ContactInputChanged(String),
    ContactSubmit,
    ConfirmDeleteContact(usize),
    CancelDeleteContact,
    DeleteContact(usize),
    SetContactCategory(usize, ContactCategory),
    SetContactField(usize, ContactField, String),
    MarkContacted(usize),
    FlipContact(usize),
    EditContact(usize),

    // Accounts CRUD
    AccountInputChanged(String),
    AccountSubmit,
    ConfirmDeleteAccount(usize),
    CancelDeleteAccount,
    DeleteAccount(usize),
    SetAccountFieldValue(usize, AccountField, String),
    MarkAccountChecked(usize),
    OpenAccountUrl(usize),
    ToggleAccountExpand(usize),

    // Notes CRUD
    ZettelInputChanged(String),
    ZettelSubmit,
    FlipNote(Uuid),
    EditNote(Uuid),
    SetNoteField(Uuid, NoteField, String),
    ConfirmDeleteNote(Uuid),
    CancelDeleteNote,
    DeleteNote(Uuid),
    AddNoteLink(Uuid, LinkTarget),
    RemoveNoteLink(Uuid, LinkTarget),
    OpenNoteInEditor(Uuid),
    NoteEditorAction(cosmic::widget::text_editor::Action),
    NoteLinkSearchChanged(String),

    // Daily Planning
    SetSpoonBudget(u32),
    TogglePlanContext(String),
    ConfirmTask(Uuid),
    UnconfirmTask(Uuid),
    RejectSuggestion(Uuid),
    PickMediaItem(Uuid),
    UnpickMediaItem(Uuid),
    PickShoppingItem(Uuid),
    UnpickShoppingItem(Uuid),

    // Do mode
    DoMarkDone(Uuid),
    DoMarkListItemDone(Uuid),

    // Persistence
    Save,
    Loaded(Result<(), String>),

    // Settings
    OpenSettings,
    SettingsContextInput(String),
    SettingsAddContext,
    SettingsRemoveContext(usize),
    SetBrowserCommand(String),
    ToggleDebugLogging,

    // Review checklist
    ToggleReviewStep(usize),

    // Search filter
    SearchQueryChanged(String),

    // All Tasks sort
    SetAllTasksSort(SortColumn),

    // Task capture form
    OpenNewTaskForm,
    CloseNewTaskForm,
    CaptureFormTitle(String),
    CaptureFormState(TaskState),
    CaptureFormPriority(Option<Priority>),
    CaptureFormEsc(Option<u32>),
    CaptureFormToggleContext(String),
    CaptureFormProject(Option<String>),
    CaptureFormScheduled(String),
    CaptureFormDeadline(String),
    CaptureFormNotes(String),
    CaptureFormSubmit,

    // Config
    ConfigChanged,

    // Sync — flat per-service config
    SyncNow,
    SyncCompleted(Result<SyncResult, String>),
    SetServiceUrl(ServiceKind, String),
    SetServiceUsername(ServiceKind, String),
    SetServicePassword(ServiceKind, String),
    TestServiceConnection(ServiceKind),
    ServiceConnectionTested(ServiceKind, Result<String, String>, Vec<CalendarInfo>),
    SetCalendarPurpose(String, CalendarPurpose),
    SyncNotesCompleted(Result<NoteSyncResult, String>),
    ContactsFetched(Result<Vec<Contact>, String>),
    ContactDeleted(Result<(), String>),

    // IMAP email integration
    ImapFetched(Result<Vec<ImapEmail>, String>),
    ArchiveEmail(u32),
    EmailArchived(Result<u32, String>),
    SetImapFolder(String),

    // AI batch email suggestions
    SetAnthropicApiKey(String),
    TestAnthropicApiKey,
    AnthropicKeyTested(Result<String, String>),
    SuggestEmailTasks,
    BatchSuggestionsReady(Result<Vec<(u32, crate::sync::anthropic::BatchEmailSuggestion)>, String>),
    ApproveSuggestion(u32),
    DismissSuggestion(u32),

    // Event CRUD
    CreateEvent,
    SubmitEvent,
    EditEvent(Uuid),
    UpdateEvent(Uuid),
    DeleteEvent(Uuid),
    CancelEventForm,

    // Event form fields
    SetEventTitle(String),
    SetEventStart(String),
    SetEventEnd(String),
    SetEventAllDay(bool),
    SetEventLocation(String),
    SetEventDescription(String),
    SetEventCalendar(String),

    // Month calendar
    CalendarPrevMonth,
    CalendarNextMonth,
    CalendarSelectDay(NaiveDate),

    // Conflict resolution
    ImportConflictTask(usize),
    DeleteConflict(usize),
    AcceptRemoteState(usize),
    AcceptLocalState(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    Media,
    Shopping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatPage {
    DailyPlanning,
    Inbox,
    AllTasks,
    NextActions,
    Projects,
    Waiting,
    Someday,
    Habits,
    Conflicts,
    Review,
    Tickler,
    Media,
    Shopping,
    Contacts,
    Accounts,
    Notes,
    Settings,
}

impl WhatPage {
    pub fn title(&self) -> &'static str {
        match self {
            Self::DailyPlanning => "Daily Planning",
            Self::Inbox => "Inbox",
            Self::AllTasks => "All Tasks",
            Self::NextActions => "Next Actions",
            Self::Projects => "Projects",
            Self::Waiting => "Waiting For",
            Self::Someday => "Someday/Maybe",
            Self::Habits => "Habits",
            Self::Conflicts => "Conflicts",
            Self::Review => "Weekly Review",
            Self::Tickler => "Agenda",
            Self::Media => "Media",
            Self::Shopping => "Shopping",
            Self::Contacts => "Contacts",
            Self::Accounts => "Accounts",
            Self::Notes => "Notes",
            Self::Settings => "Settings",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::DailyPlanning => "date-symbolic",
            Self::Inbox => "mail-folder-inbox-symbolic",
            Self::AllTasks => "object-select-symbolic",
            Self::NextActions => "go-next-symbolic",
            Self::Projects => "folder-symbolic",
            Self::Waiting => "appointment-soon-symbolic",
            Self::Someday => "weather-few-clouds-symbolic",
            Self::Habits => "checkbox-checked-symbolic",
            Self::Conflicts => "dialog-warning-symbolic",
            Self::Review => "document-open-recent-symbolic",
            Self::Tickler => "x-office-calendar-symbolic",
            Self::Media => "applications-multimedia-symbolic",
            Self::Shopping => "payment-card-symbolic",
            Self::Contacts => "system-users-symbolic",
            Self::Accounts => "contact-new-symbolic",
            Self::Notes => "accessories-text-editor-symbolic",
            Self::Settings => "emblem-system-symbolic",
        }
    }

    pub const ALL: &'static [WhatPage] = &[
        // Planning & Review
        WhatPage::DailyPlanning,
        WhatPage::Review,
        WhatPage::Tickler,
        // Collect
        WhatPage::Inbox,
        WhatPage::Conflicts,
        WhatPage::AllTasks,
        // GTD
        WhatPage::NextActions,
        WhatPage::Projects,
        WhatPage::Waiting,
        WhatPage::Someday,
        WhatPage::Habits,
        // Lists
        WhatPage::Media,
        WhatPage::Shopping,
        WhatPage::Contacts,
        WhatPage::Accounts,
        WhatPage::Notes,
        WhatPage::Settings,
    ];

    /// Pages that start a new sidebar section (divider drawn above them).
    pub const SECTION_STARTS: &'static [WhatPage] = &[
        WhatPage::Inbox,
        WhatPage::NextActions,
        WhatPage::Media,
        WhatPage::Settings,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhenPage {
    Today,
    Tomorrow,
    ThisWeek,
    Upcoming,
}

impl WhenPage {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Tomorrow => "Tomorrow",
            Self::ThisWeek => "This Week",
            Self::Upcoming => "Upcoming",
        }
    }

    pub const ALL: &'static [WhenPage] = &[
        WhenPage::Today,
        WhenPage::Tomorrow,
        WhenPage::ThisWeek,
        WhenPage::Upcoming,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    State,
    Priority,
    Title,
    Context,
    Esc,
    Scheduled,
    Deadline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    What(WhatPage),
    When(WhenPage),
}
