use serde::{Deserialize, Serialize};

use super::task::Task;

/// A GTD project — a multi-step outcome with associated tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub tasks: Vec<Task>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tasks: Vec::new(),
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
