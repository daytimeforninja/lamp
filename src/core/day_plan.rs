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

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 3, 2).unwrap()
    }

    #[test]
    fn new_plan_defaults() {
        let plan = DayPlan::new(today());
        assert_eq!(plan.spoon_budget, 50);
        assert_eq!(plan.spent_spoons, 0);
        assert!(plan.confirmed_task_ids.is_empty());
        assert!(plan.completed_tasks.is_empty());
    }

    #[test]
    fn complete_task_adds_spoons() {
        let mut plan = DayPlan::new(today());
        let id = Uuid::new_v4();
        plan.confirmed_task_ids.push(id);
        plan.complete_task(id, "Test".into(), Some(15));
        assert_eq!(plan.spent_spoons, 15);
        assert_eq!(plan.completed_tasks.len(), 1);
        assert!(!plan.confirmed_task_ids.contains(&id));
    }

    #[test]
    fn complete_task_no_esc() {
        let mut plan = DayPlan::new(today());
        let id = Uuid::new_v4();
        plan.complete_task(id, "Test".into(), None);
        assert_eq!(plan.spent_spoons, 0);
        assert_eq!(plan.completed_tasks.len(), 1);
    }

    #[test]
    fn uncomplete_task_restores_spoons() {
        let mut plan = DayPlan::new(today());
        let id = Uuid::new_v4();
        plan.complete_task(id, "Test".into(), Some(20));
        assert_eq!(plan.spent_spoons, 20);
        plan.uncomplete_task(id);
        assert_eq!(plan.spent_spoons, 0);
        assert!(plan.confirmed_task_ids.contains(&id));
        assert!(plan.completed_tasks.is_empty());
    }

    #[test]
    fn uncomplete_nonexistent_task_noop() {
        let mut plan = DayPlan::new(today());
        plan.complete_task(Uuid::new_v4(), "Test".into(), Some(10));
        let before = plan.spent_spoons;
        plan.uncomplete_task(Uuid::new_v4()); // different id
        assert_eq!(plan.spent_spoons, before);
    }

    #[test]
    fn is_stale() {
        let plan = DayPlan::new(today());
        assert!(!plan.is_stale(today()));
        let tomorrow = today() + chrono::Duration::days(1);
        assert!(plan.is_stale(tomorrow));
    }

    #[test]
    fn remaining_budget_saturates() {
        let mut plan = DayPlan::new(today());
        plan.spoon_budget = 10;
        plan.spent_spoons = 15;
        assert_eq!(plan.remaining_budget(), 0);
    }
}
