use chrono::NaiveDate;

use super::habit::Habit;
use super::task::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateRange {
    Today,
    Tomorrow,
    ThisWeek,
    Upcoming,
}

/// Aggregated temporal view data.
pub struct TemporalView {
    pub overdue: Vec<Task>,
    pub scheduled: Vec<Task>,
    pub deadlined: Vec<Task>,
    pub habits_due: Vec<Habit>,
}

impl TemporalView {
    pub fn build(tasks: &[Task], habits: &[Habit], today: NaiveDate, range: DateRange) -> Self {
        match range {
            DateRange::Today => Self::build_today(tasks, habits, today),
            DateRange::Tomorrow => Self::build_tomorrow(tasks, today),
            DateRange::ThisWeek => Self::build_this_week(tasks, today),
            DateRange::Upcoming => Self::build_upcoming(tasks),
        }
    }

    fn build_today(tasks: &[Task], habits: &[Habit], today: NaiveDate) -> Self {
        let mut overdue = Vec::new();
        let mut scheduled = Vec::new();
        let mut deadlined = Vec::new();

        for task in tasks {
            if task.state.is_done() {
                continue;
            }
            if let Some(sched) = task.scheduled {
                if sched < today {
                    overdue.push(task.clone());
                    continue;
                } else if sched == today {
                    scheduled.push(task.clone());
                    continue;
                }
            }
            if let Some(deadline) = task.deadline {
                let days_until = (deadline - today).num_days();
                if days_until < 0 {
                    overdue.push(task.clone());
                } else if days_until <= 7 {
                    deadlined.push(task.clone());
                }
            }
        }

        let habits_due: Vec<Habit> = habits
            .iter()
            .filter(|h| h.is_due(today))
            .cloned()
            .collect();

        Self {
            overdue,
            scheduled,
            deadlined,
            habits_due,
        }
    }

    fn build_tomorrow(tasks: &[Task], today: NaiveDate) -> Self {
        let tomorrow = today.succ_opt().unwrap_or(today);
        let mut scheduled = Vec::new();
        let mut deadlined = Vec::new();

        for task in tasks {
            if task.state.is_done() {
                continue;
            }
            if let Some(sched) = task.scheduled {
                if sched == tomorrow {
                    scheduled.push(task.clone());
                    continue;
                }
            }
            if let Some(deadline) = task.deadline {
                if deadline == tomorrow {
                    deadlined.push(task.clone());
                }
            }
        }

        Self {
            overdue: Vec::new(),
            scheduled,
            deadlined,
            habits_due: Vec::new(),
        }
    }

    fn build_this_week(tasks: &[Task], today: NaiveDate) -> Self {
        let week_end = today + chrono::Duration::days(7);
        let mut overdue = Vec::new();
        let mut scheduled = Vec::new();
        let mut deadlined = Vec::new();

        for task in tasks {
            if task.state.is_done() {
                continue;
            }
            if let Some(sched) = task.scheduled {
                if sched < today {
                    overdue.push(task.clone());
                    continue;
                } else if sched >= today && sched < week_end {
                    scheduled.push(task.clone());
                    continue;
                }
            }
            if let Some(deadline) = task.deadline {
                if deadline < today {
                    overdue.push(task.clone());
                } else if deadline < week_end {
                    deadlined.push(task.clone());
                }
            }
        }

        Self {
            overdue,
            scheduled,
            deadlined,
            habits_due: Vec::new(),
        }
    }

    fn build_upcoming(tasks: &[Task]) -> Self {
        let mut scheduled: Vec<Task> = tasks
            .iter()
            .filter(|t| !t.state.is_done() && (t.scheduled.is_some() || t.deadline.is_some()))
            .cloned()
            .collect();

        scheduled.sort_by_key(|t| {
            let s = t.scheduled.unwrap_or(NaiveDate::MAX);
            let d = t.deadline.unwrap_or(NaiveDate::MAX);
            s.min(d)
        });

        Self {
            overdue: Vec::new(),
            scheduled,
            deadlined: Vec::new(),
            habits_due: Vec::new(),
        }
    }

    pub fn total_count(&self) -> usize {
        self.overdue.len()
            + self.scheduled.len()
            + self.deadlined.len()
            + self.habits_due.len()
    }
}
