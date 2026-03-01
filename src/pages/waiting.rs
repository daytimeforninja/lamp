use cosmic::iced::Length;
use cosmic::widget::{container, scrollable, text};
use cosmic::Element;

use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

pub fn waiting_view(
    tasks: &[Task],
    ctx: &TaskRowCtx,
) -> Element<'static, Message> {
    let waiting_tasks: Vec<&Task> = tasks
        .iter()
        .filter(|t| matches!(t.state, crate::core::task::TaskState::Waiting))
        .collect();

    if waiting_tasks.is_empty() {
        return container(text::body(fl!("waiting-empty")))
            .padding(32)
            .center_x(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    container(scrollable(container(task_grid(waiting_tasks.into_iter(), ctx, None)).padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
