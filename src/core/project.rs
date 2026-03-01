use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::task::Task;

/// A GTD project — a multi-step outcome with associated tasks.
/// Supports GTD's Natural Planning Model with purpose, outcome, and brainstorm fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub tasks: Vec<Task>,
    pub purpose: String,
    pub outcome: String,
    pub brainstorm: String,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            tasks: Vec::new(),
            purpose: String::new(),
            outcome: String::new(),
            brainstorm: String::new(),
        }
    }

    /// The next action for this project (first task with NEXT state, or first TODO).
    pub fn next_action(&self) -> Option<&Task> {
        self.tasks
            .iter()
            .find(|t| matches!(t.state, super::task::TaskState::Next))
            .or_else(|| {
                self.tasks
                    .iter()
                    .find(|t| matches!(t.state, super::task::TaskState::Todo))
            })
    }

    pub fn is_stuck(&self) -> bool {
        // A project is stuck if it has no NEXT action
        self.tasks
            .iter()
            .any(|t| t.state.is_active())
            && self.next_action().is_none()
    }

    pub fn completion_ratio(&self) -> (usize, usize) {
        let total = self.tasks.len();
        let done = self.tasks.iter().filter(|t| t.state.is_done()).count();
        (done, total)
    }
}
