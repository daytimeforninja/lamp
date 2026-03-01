use std::collections::HashMap;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, row, scrollable, text, text_input};
use cosmic::Element;

use crate::application::EmailSuggestionState;
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
    task.state == TaskState::Todo && task.project.is_none()
}

pub fn inbox_view<'a>(
    tasks: &[Task],
    imap_emails: &[ImapEmail],
    input_value: &str,
    ctx: &TaskRowCtx,
    email_suggestions: &HashMap<u32, EmailSuggestionState>,
    ai_batch_processing: bool,
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
            // Header row with title and Suggest Tasks button
            let mut header_row = row().spacing(8).align_y(Alignment::Center);
            header_row = header_row.push(text::title4(fl!("inbox-emails")).width(Length::Fill));

            if ai_batch_processing {
                header_row = header_row.push(text::caption(fl!("email-analyzing")));
            } else {
                header_row = header_row.push(
                    button::standard(fl!("email-suggest-tasks"))
                        .on_press(Message::SuggestEmailTasks),
                );
            }

            content = content.push(header_row);

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

                // Archive button always present
                info_row = info_row.push(
                    button::standard(fl!("email-archive"))
                        .on_press(Message::ArchiveEmail(uid)),
                );

                email_col = email_col.push(info_row);

                // Suggestion card
                match email_suggestions.get(&uid) {
                    Some(EmailSuggestionState::Suggested(s)) => {
                        let title = s.title.clone().unwrap_or_default();
                        let mut suggestion_row = row().spacing(8).align_y(Alignment::Center);
                        let mut detail_col = column().spacing(2);
                        detail_col =
                            detail_col.push(text::caption(fl!("email-suggested-task")));
                        detail_col = detail_col.push(text::body(title));

                        // Show metadata summary
                        let mut meta_parts: Vec<String> = Vec::new();
                        if let Some(ref p) = s.priority {
                            meta_parts.push(format!("[#{}]", p));
                        }
                        if let Some(ref ctxs) = s.contexts {
                            if !ctxs.is_empty() {
                                meta_parts.push(ctxs.join(", "));
                            }
                        }
                        if let Some(ref proj) = s.project {
                            meta_parts.push(format!("→ {}", proj));
                        }
                        if let Some(ref d) = s.deadline {
                            meta_parts.push(format!("due {}", d));
                        }
                        if !meta_parts.is_empty() {
                            detail_col =
                                detail_col.push(text::caption(meta_parts.join("  ")));
                        }

                        suggestion_row =
                            suggestion_row.push(detail_col.width(Length::Fill));
                        suggestion_row = suggestion_row.push(
                            button::suggested(fl!("email-approve"))
                                .on_press(Message::ApproveSuggestion(uid)),
                        );
                        suggestion_row = suggestion_row.push(
                            button::standard(fl!("email-dismiss"))
                                .on_press(Message::DismissSuggestion(uid)),
                        );

                        email_col = email_col.push(suggestion_row);
                    }
                    Some(EmailSuggestionState::NoAction) => {
                        email_col = email_col
                            .push(text::caption(fl!("email-no-action")));
                    }
                    Some(EmailSuggestionState::Dismissed) | None => {
                        // No suggestion card shown
                    }
                }

                content = content.push(email_col);
            }
        }
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
