use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::recurrence::Recurrence;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Todo,
    Next,
    Waiting,
    Someday,
    Done,
    Cancelled,
}

impl TaskState {
    pub fn as_keyword(&self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Next => "NEXT",
            Self::Waiting => "WAITING",
            Self::Someday => "SOMEDAY",
            Self::Done => "DONE",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn from_keyword(s: &str) -> Option<Self> {
        match s {
            "TODO" => Some(Self::Todo),
            "NEXT" => Some(Self::Next),
            "WAITING" => Some(Self::Waiting),
            "SOMEDAY" => Some(Self::Someday),
            "DONE" => Some(Self::Done),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        !self.is_done()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    A,
    B,
    C,
}

impl Priority {
    pub fn as_org(&self) -> &'static str {
        match self {
            Self::A => "[#A]",
            Self::B => "[#B]",
            Self::C => "[#C]",
        }
    }

    pub fn from_org(s: &str) -> Option<Self> {
        match s {
            "A" | "#A" | "[#A]" => Some(Self::A),
            "B" | "#B" | "[#B]" => Some(Self::B),
            "C" | "#C" | "[#C]" => Some(Self::C),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub state: TaskState,
    pub priority: Option<Priority>,
    pub contexts: Vec<String>,
    pub scheduled: Option<NaiveDate>,
    pub deadline: Option<NaiveDate>,
    pub recurrence: Option<Recurrence>,
    pub notes: String,
    pub created: NaiveDateTime,
    pub completed: Option<NaiveDateTime>,
    pub project: Option<String>,
    pub waiting_for: Option<String>,
    pub esc: Option<u32>,
    pub delegated: Option<NaiveDate>,
    pub follow_up: Option<NaiveDate>,
    /// Non-context tags (tags that don't start with @) for org round-trip.
    pub extra_tags: Vec<String>,
    /// Raw time string from SCHEDULED timestamp (e.g. "10:00" or "10:00-11:00").
    pub scheduled_time: Option<String>,
    /// Raw time string from DEADLINE timestamp.
    pub deadline_time: Option<String>,
    /// Logbook state-change entries (timestamps of DONE transitions).
    pub logbook_entries: Vec<NaiveDateTime>,
    /// Date this task was added to the day plan (synced via X-LAMP-DAYPLAN).
    pub dayplan_date: Option<NaiveDate>,
    pub sync_href: Option<String>,
    pub sync_hash: Option<u64>,
    /// Original CalDAV UID string (preserved for case-sensitive roundtrip)
    pub sync_uid: Option<String>,
    /// CalDAV etag for conditional PUT (avoids overwriting concurrent changes).
    pub sync_etag: Option<String>,
}

impl Task {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            state: TaskState::Todo,
            priority: None,
            contexts: Vec::new(),
            scheduled: None,
            deadline: None,
            recurrence: None,
            notes: String::new(),
            created: chrono::Local::now().naive_local(),
            completed: None,
            project: None,
            waiting_for: None,
            esc: None,
            delegated: None,
            follow_up: None,
            extra_tags: Vec::new(),
            scheduled_time: None,
            deadline_time: None,
            logbook_entries: Vec::new(),
            dayplan_date: None,
            sync_href: None,
            sync_hash: None,
            sync_uid: None,
            sync_etag: None,
        }
    }

    pub fn complete(&mut self) {
        self.state = TaskState::Done;
        self.completed = Some(chrono::Local::now().naive_local());
    }

    pub fn cancel(&mut self) {
        self.state = TaskState::Cancelled;
        self.completed = Some(chrono::Local::now().naive_local());
    }

    /// Returns true if this task should appear in the "today" view.
    pub fn is_today(&self, today: NaiveDate) -> bool {
        if self.state.is_done() {
            return false;
        }
        // Scheduled for today or overdue
        if let Some(scheduled) = self.scheduled {
            if scheduled <= today {
                return true;
            }
        }
        // Deadline within the next 7 days
        if let Some(deadline) = self.deadline {
            let days_until = (deadline - today).num_days();
            if days_until <= 7 && days_until >= 0 {
                return true;
            }
        }
        false
    }

    pub fn has_context(&self, ctx: &str) -> bool {
        self.contexts.iter().any(|c| c == ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 2).unwrap()
    }

    #[test]
    fn is_today_scheduled_today() {
        let mut task = Task::new("Test");
        task.scheduled = Some(today());
        assert!(task.is_today(today()));
    }

    #[test]
    fn is_today_scheduled_past() {
        let mut task = Task::new("Test");
        task.scheduled = Some(today() - chrono::Duration::days(3));
        assert!(task.is_today(today()));
    }

    #[test]
    fn is_today_scheduled_future() {
        let mut task = Task::new("Test");
        task.scheduled = Some(today() + chrono::Duration::days(3));
        assert!(!task.is_today(today()));
    }

    #[test]
    fn is_today_deadline_within_7_days() {
        let mut task = Task::new("Test");
        task.deadline = Some(today() + chrono::Duration::days(5));
        assert!(task.is_today(today()));
    }

    #[test]
    fn is_today_deadline_beyond_7_days() {
        let mut task = Task::new("Test");
        task.deadline = Some(today() + chrono::Duration::days(10));
        assert!(!task.is_today(today()));
    }

    #[test]
    fn is_today_done_task_excluded() {
        let mut task = Task::new("Test");
        task.scheduled = Some(today());
        task.state = TaskState::Done;
        assert!(!task.is_today(today()));
    }

    #[test]
    fn is_today_no_dates() {
        let task = Task::new("Test");
        assert!(!task.is_today(today()));
    }

    #[test]
    fn complete_sets_done_and_timestamp() {
        let mut task = Task::new("Test");
        task.complete();
        assert_eq!(task.state, TaskState::Done);
        assert!(task.completed.is_some());
    }

    #[test]
    fn cancel_sets_cancelled_and_timestamp() {
        let mut task = Task::new("Test");
        task.cancel();
        assert_eq!(task.state, TaskState::Cancelled);
        assert!(task.completed.is_some());
    }

    #[test]
    fn state_keyword_roundtrip() {
        let states = [
            TaskState::Todo,
            TaskState::Next,
            TaskState::Waiting,
            TaskState::Someday,
            TaskState::Done,
            TaskState::Cancelled,
        ];
        for state in &states {
            let kw = state.as_keyword();
            let parsed = TaskState::from_keyword(kw).unwrap();
            assert_eq!(*state, parsed);
        }
        assert!(TaskState::from_keyword("INVALID").is_none());
    }

    #[test]
    fn priority_from_org_variants() {
        assert_eq!(Priority::from_org("A"), Some(Priority::A));
        assert_eq!(Priority::from_org("#B"), Some(Priority::B));
        assert_eq!(Priority::from_org("[#C]"), Some(Priority::C));
        assert_eq!(Priority::from_org("D"), None);
    }

    #[test]
    fn has_context() {
        let mut task = Task::new("Test");
        task.contexts = vec!["@home".to_string(), "@work".to_string()];
        assert!(task.has_context("@home"));
        assert!(!task.has_context("@errands"));
    }
}
