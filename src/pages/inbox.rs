use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::components::task_row::{task_grid, TaskRowCtx};
use crate::core::task::{Task, TaskState};
use crate::fl;
use crate::message::Message;
use crate::sync::imap::ImapEmail;
use crate::ui::{self, Sender};

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

pub fn inbox_view(
    tasks: &[Task],
    imap_emails: &[ImapEmail],
    input_value: &str,
    ctx: &TaskRowCtx,
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(8);

    // Text entry for new tasks
    let placeholder = fl!("inbox-placeholder");
    let entry = ui::entry(&placeholder, input_value);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::InboxInputChanged(e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::InboxSubmit);
        });
    }
    content.append(&entry);

    let inbox: Vec<&Task> = tasks.iter().filter(|t| is_inbox_task(t)).collect();

    if inbox.is_empty() && imap_emails.is_empty() {
        content.append(&ui::status_page("mail-folder-inbox-symbolic", &fl!("inbox-empty"), "Add tasks using the input above"));
    } else {
        if !inbox.is_empty() {
            let grid = task_grid(inbox.into_iter(), ctx, None, sender);
            content.append(&grid);
        }

        if !imap_emails.is_empty() {
            content.append(&ui::title4(&fl!("inbox-emails")));

            for email in imap_emails {
                let uid = email.uid;
                let email_col = ui::vbox(4);

                let info_row = ui::centered_hbox(8);

                let info_col = ui::vbox(2);
                info_col.set_hexpand(true);
                info_col.append(&ui::body(&email.subject));

                let mut secondary = format!("From: {}", email.from);
                if let Some(date) = email.date {
                    secondary.push_str(&format!(" — {}", date.format("%Y-%m-%d %H:%M")));
                }
                info_col.append(&ui::caption(&secondary));

                info_row.append(&info_col);

                // Create Task button
                let create_btn = ui::button_with_signal(
                    &fl!("email-create-task"),
                    Some("suggested-action"),
                    Message::CreateTaskFromEmail(uid),
                    sender,
                );
                info_row.append(&create_btn);

                // Archive button
                let archive_btn = ui::button_with_signal(
                    &fl!("email-archive"),
                    None,
                    Message::ArchiveEmail(uid),
                    sender,
                );
                info_row.append(&archive_btn);

                email_col.append(&info_row);
                content.append(&email_col);
            }
        }
    }

    ui::page_wrapper(&content).upcast()
}
