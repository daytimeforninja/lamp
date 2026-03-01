use chrono::NaiveDate;
use uuid::Uuid;

/// A completed task entry: (id, title, esc).
#[derive(Debug, Clone)]
pub struct CompletedTask {
    pub id: Uuid,
    pub title: String,
    pub esc: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DayPlan {
    pub date: NaiveDate,
    pub spoon_budget: u32,
    pub active_contexts: Vec<String>,
    pub confirmed_task_ids: Vec<Uuid>,
    pub completed_tasks: Vec<CompletedTask>,
    pub spent_spoons: u32,
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
            completed_tasks: Vec::new(),
            spent_spoons: 0,
            picked_media_ids: Vec::new(),
            picked_shopping_ids: Vec::new(),
        }
    }

    pub fn is_stale(&self, today: NaiveDate) -> bool {
        self.date != today
    }

    pub fn remaining_budget(&self) -> u32 {
        self.spoon_budget.saturating_sub(self.spent_spoons)
    }

    /// Record a task as completed and add its ESC to spent spoons.
    pub fn complete_task(&mut self, task_id: Uuid, title: String, esc: Option<u32>) {
        self.confirmed_task_ids.retain(|id| *id != task_id);
        self.completed_tasks.push(CompletedTask { id: task_id, title, esc });
        self.spent_spoons += esc.unwrap_or(0);
    }

    /// Un-complete a task: move back to confirmed, subtract spoons.
    pub fn uncomplete_task(&mut self, task_id: Uuid) {
        if let Some(pos) = self.completed_tasks.iter().position(|ct| ct.id == task_id) {
            let ct = self.completed_tasks.remove(pos);
            self.spent_spoons = self.spent_spoons.saturating_sub(ct.esc.unwrap_or(0));
            self.confirmed_task_ids.push(task_id);
        }
    }
}
