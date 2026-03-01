use std::collections::BTreeMap;

use cosmic::iced::Length;
use cosmic::widget::{column, container, scrollable, text};
use cosmic::Element;

use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

pub fn next_actions_view(
    tasks: &[Task],
    ctx: &TaskRowCtx,
) -> Element<'static, Message> {
    let next_tasks: Vec<&Task> = tasks
        .iter()
        .filter(|t| matches!(t.state, crate::core::task::TaskState::Next))
        .collect();

    if next_tasks.is_empty() {
        return container(text::body(fl!("next-actions-empty")))
            .padding(32)
            .center_x(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
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

    let mut content = column().spacing(16);

    for (context, tasks) in &by_context {
        let mut section = column().spacing(4);
        section = section.push(text::title4(context.clone()));
        section = section.push(task_grid(tasks.iter().copied(), ctx, None));
        content = content.push(section);
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
