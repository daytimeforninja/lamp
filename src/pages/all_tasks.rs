use cosmic::iced::Length;
use cosmic::widget::{container, scrollable, text};
use cosmic::Element;

use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

pub fn all_tasks_view(
    tasks: &[Task],
    ctx: &TaskRowCtx,
) -> Element<'static, Message> {
    let active: Vec<&Task> = tasks.iter().filter(|t| t.state.is_active()).collect();

    if active.is_empty() {
        return container(text::body(fl!("all-tasks-empty")))
            .padding(32)
            .center_x(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    container(scrollable(container(task_grid(active.into_iter(), ctx)).padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
