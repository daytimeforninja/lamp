use std::collections::BTreeMap;

use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::components::task_row::{task_grid, TaskRowCtx};
use crate::core::task::Task;
use crate::fl;
use crate::ui::{self, Sender};

pub fn next_actions_view(
    tasks: &[Task],
    ctx: &TaskRowCtx,
    sender: &Sender,
) -> gtk::Widget {
    let next_tasks: Vec<&Task> = tasks
        .iter()
        .filter(|t| matches!(t.state, crate::core::task::TaskState::Next))
        .collect();

    if next_tasks.is_empty() {
        return ui::status_page("pan-end-symbolic", &fl!("next-actions-empty"), "Set tasks to NEXT to see them here").upcast();
    }

    // Group by context
    let mut by_context: BTreeMap<String, Vec<&Task>> = BTreeMap::new();
    for task in &next_tasks {
        if task.contexts.is_empty() {
            by_context
                .entry("No Context".to_string())
                .or_default()
                .push(task);
        } else {
            for ctx_tag in &task.contexts {
                by_context.entry(ctx_tag.clone()).or_default().push(task);
            }
        }
    }

    let content = ui::vbox(16);

    for (context, tasks) in &by_context {
        let section = ui::vbox(4);
        section.append(&ui::title4(context));
        section.append(&task_grid(tasks.iter().copied(), ctx, None, sender));
        content.append(&section);
    }

    ui::page_wrapper(&content).upcast()
}
