use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, row, scrollable, text, text_input};
use cosmic::Element;

use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::task::{Task, TaskState};
use crate::fl;
use crate::message::Message;
use crate::sync::imap::ImapEmail;

/// A task belongs in the inbox if it's Todo and not assigned to a project.
/// Setting metadata (priority, context, ESC) is part of processing but
/// doesn't move the task out of inbox — only a state change or project
/// assignment does that.
fn is_inbox_task(task: &Task) -> bool {
    task.state == TaskState::Todo
        && task.project.is_none()
        && !task.extra_tags.contains(&"habit".to_string())
        && task.recurrence.is_none()
}

pub fn inbox_view<'a>(
    tasks: &[Task],
    imap_emails: &[ImapEmail],
    input_value: &str,
    ctx: &TaskRowCtx,
) -> Element<'a, Message> {
    let placeholder = fl!("inbox-placeholder");
    let input = text_input::text_input(placeholder, input_value.to_string())
        .on_input(Message::InboxInputChanged)
        .on_submit(|_| Message::InboxSubmit)
        .width(Length::Fill);

    let inbox: Vec<&Task> = tasks.iter().filter(|t| is_inbox_task(t)).collect();

    let mut content = column().spacing(8).push(input);

    if inbox.is_empty() && imap_emails.is_empty() {
        content = content.push(
            container(text::body(fl!("inbox-empty")))
                .padding(32)
                .center_x(Length::Fill),
        );
    } else {
        if !inbox.is_empty() {
            content = content.push(task_grid(inbox.into_iter(), ctx, None));
        }

        if !imap_emails.is_empty() {
            content = content.push(text::title4(fl!("inbox-emails")));

            for email in imap_emails {
                let uid = email.uid;
                let mut email_col = column().spacing(4);

                // Subject and from
                let mut info_row = row().spacing(8).align_y(Alignment::Center);
                let mut info_col = column().spacing(2);
                info_col = info_col.push(text::body(email.subject.clone()));
                let mut secondary = format!("From: {}", email.from);
                if let Some(date) = email.date {
                    secondary.push_str(&format!(" — {}", date.format("%Y-%m-%d %H:%M")));
                }
                info_col = info_col.push(text::caption(secondary));
                info_row = info_row.push(info_col.width(Length::Fill));

                // Create Task button
                info_row = info_row.push(
                    button::suggested(fl!("email-create-task"))
                        .on_press(Message::CreateTaskFromEmail(uid)),
                );

                // Archive button
                info_row = info_row.push(
                    button::standard(fl!("email-archive"))
                        .on_press(Message::ArchiveEmail(uid)),
                );

                email_col = email_col.push(info_row);
                content = content.push(email_col);
            }
        }
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
