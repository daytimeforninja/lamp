use chrono::NaiveDate;

use crate::core::task::{Priority, TaskState};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    SelectWhen(WhenPage),

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

    // Dates
    SetScheduled(Uuid, Option<NaiveDate>),
    SetDeadline(Uuid, Option<NaiveDate>),

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

    // Habits
    CompleteHabit(Uuid),
    DeleteHabit(Uuid),
    HabitInputChanged(String),
    HabitSubmit,

    // Persistence
    Save,
    Loaded(Result<(), String>),

    // Settings
    ToggleSettings,
    SettingsContextInput(String),
    SettingsAddContext,
    SettingsRemoveContext(usize),

    // Config
    ConfigChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatPage {
    Inbox,
    AllTasks,
    NextActions,
    Projects,
    Waiting,
    Someday,
    Habits,
    Review,
}

impl WhatPage {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Inbox => "Inbox",
            Self::AllTasks => "All Tasks",
            Self::NextActions => "Next Actions",
            Self::Projects => "Projects",
            Self::Waiting => "Waiting For",
            Self::Someday => "Someday/Maybe",
            Self::Habits => "Habits",
            Self::Review => "Weekly Review",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Inbox => "mail-inbox-symbolic",
            Self::AllTasks => "view-list-bullet-symbolic",
            Self::NextActions => "go-next-symbolic",
            Self::Projects => "folder-symbolic",
            Self::Waiting => "appointment-soon-symbolic",
            Self::Someday => "weather-few-clouds-symbolic",
            Self::Habits => "view-list-symbolic",
            Self::Review => "document-open-recent-symbolic",
        }
    }

    pub const ALL: &'static [WhatPage] = &[
        WhatPage::Inbox,
        WhatPage::AllTasks,
        WhatPage::NextActions,
        WhatPage::Projects,
        WhatPage::Waiting,
        WhatPage::Someday,
        WhatPage::Habits,
        WhatPage::Review,
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
pub enum ActiveView {
    What(WhatPage),
    When(WhenPage),
}
