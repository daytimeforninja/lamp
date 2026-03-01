use cosmic::iced::Length;
use cosmic::widget::{container, scrollable, text};
use cosmic::Element;

use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::task::Task;
use crate::fl;
use crate::message::{Message, SortColumn};

pub fn all_tasks_view(
    tasks: &[Task],
    ctx: &TaskRowCtx,
    sort: Option<(SortColumn, bool)>,
) -> Element<'static, Message> {
    let mut active: Vec<&Task> = tasks.iter().filter(|t| t.state.is_active()).collect();

    if active.is_empty() {
        return container(text::body(fl!("all-tasks-empty")))
            .padding(32)
            .center_x(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    if let Some((col, ascending)) = sort {
        active.sort_by(|a, b| {
            let ord = match col {
                SortColumn::State => {
                    fn rank(t: &Task) -> u8 {
                        use crate::core::task::TaskState::*;
                        match t.state { Next => 0, Todo => 1, Waiting => 2, Someday => 3, _ => 4 }
                    }
                    rank(a).cmp(&rank(b))
                }
                SortColumn::Priority => {
                    // A < B < C < None (A is highest priority, sort first)
                    let pa = a.priority.map(|p| p as u8).unwrap_or(255);
                    let pb = b.priority.map(|p| p as u8).unwrap_or(255);
                    pa.cmp(&pb)
                }
                SortColumn::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                SortColumn::Context => {
                    let ca = a.contexts.first().cloned().unwrap_or_default();
                    let cb = b.contexts.first().cloned().unwrap_or_default();
                    ca.cmp(&cb)
                }
                SortColumn::Esc => a.esc.unwrap_or(0).cmp(&b.esc.unwrap_or(0)),
                SortColumn::Scheduled => a.scheduled.cmp(&b.scheduled),
                SortColumn::Deadline => a.deadline.cmp(&b.deadline),
            };
            if ascending { ord } else { ord.reverse() }
        });
    }

    container(scrollable(container(task_grid(active.into_iter(), ctx, Some(sort))).padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
