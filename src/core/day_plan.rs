use chrono::NaiveDate;
use uuid::Uuid;

use super::task::Task;

#[derive(Debug, Clone)]
pub struct DayPlan {
    pub date: NaiveDate,
    pub spoon_budget: u32,
    pub active_contexts: Vec<String>,
    pub confirmed_task_ids: Vec<Uuid>,
    pub picked_media_ids: Vec<Uuid>,
    pub picked_shopping_ids: Vec<Uuid>,
}

impl DayPlan {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            spoon_budget: 50,
            active_contexts: Vec::new(),
            confirmed_task_ids: Vec::new(),
            picked_media_ids: Vec::new(),
            picked_shopping_ids: Vec::new(),
        }
    }

    pub fn is_stale(&self, today: NaiveDate) -> bool {
        self.date != today
    }

    /// Sum ESC of completed confirmed tasks — spent spoons.
    pub fn spent_esc(&self, tasks: &[Task]) -> u32 {
        self.confirmed_task_ids
            .iter()
            .filter_map(|id| tasks.iter().find(|t| t.id == *id))
            .filter(|t| t.state.is_done())
            .filter_map(|t| t.esc)
            .sum()
    }

    pub fn remaining_budget(&self, tasks: &[Task]) -> u32 {
        self.spoon_budget.saturating_sub(self.spent_esc(tasks))
    }
}
