use cosmic::iced::Length;
use cosmic::widget::{column, container, scrollable, text, text_input};
use cosmic::Element;

use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

pub fn inbox_view<'a>(
    tasks: &[Task],
    input_value: &str,
    ctx: &TaskRowCtx,
) -> Element<'a, Message> {
    let placeholder = fl!("inbox-placeholder");
    let input = text_input::text_input(placeholder, input_value.to_string())
        .on_input(Message::InboxInputChanged)
        .on_submit(|_| Message::InboxSubmit)
        .width(Length::Fill);

    let mut content = column().spacing(8).push(input);

    if tasks.is_empty() {
        content = content.push(
            container(text::body(fl!("inbox-empty")))
                .padding(32)
                .center_x(Length::Fill),
        );
    } else {
        content = content.push(scrollable(task_grid(tasks.iter(), ctx)));
    }

    container(content)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
