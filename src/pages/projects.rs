use std::collections::HashMap;

use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::components::task_row::{task_grid, TaskRowCtx};
use crate::core::project::Project;
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;
use crate::ui::{self, Sender};

pub fn projects_view(
    projects: &[Project],
    project_input: &str,
    project_task_inputs: &HashMap<String, String>,
    ctx: &TaskRowCtx,
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(16);

    // Project creation input
    let input_row = ui::hbox(8);
    input_row.set_valign(gtk::Align::Center);

    let entry = ui::entry(&fl!("projects-new-placeholder"), project_input);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::ProjectInputChanged(e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::ProjectSubmit);
        });
    }
    input_row.append(&entry);
    input_row.append(&ui::icon_button_with_signal(
        "list-add-symbolic",
        Message::ProjectSubmit,
        sender,
    ));
    content.append(&input_row);

    if projects.is_empty() {
        content.append(&ui::status_page("folder-symbolic", &fl!("projects-empty"), "Create a project using the input above"));
    } else {
        for project in projects {
            let (done, total) = project.completion_ratio();
            let section = ui::vbox(4);

            // Header row: name (done/total) + delete button
            let header_row = ui::centered_hbox(8);
            let header_text = format!("{} ({}/{})", project.name, done, total);
            let header_label = ui::title4(&header_text);
            header_label.set_hexpand(true);
            header_row.append(&header_label);

            header_row.append(&ui::icon_button_with_signal(
                "edit-delete-symbolic",
                Message::DeleteProject(project.name.clone()),
                sender,
            ));
            section.append(&header_row);

            // Stuck indicator
            if project.is_stuck() {
                section.append(&ui::caption(&fl!("projects-stuck")));
            }

            // Task grid for active tasks
            let active: Vec<&Task> = project.tasks.iter().filter(|t| t.state.is_active()).collect();
            if !active.is_empty() {
                section.append(&task_grid(active.into_iter(), ctx, None, sender));
            }

            // Per-project task input
            let task_input_value = project_task_inputs
                .get(&project.name)
                .cloned()
                .unwrap_or_default();
            let task_row = ui::hbox(8);
            task_row.set_valign(gtk::Align::Center);

            let task_entry = ui::entry(&fl!("inbox-placeholder"), &task_input_value);
            {
                let s = sender.clone();
                let pname = project.name.clone();
                task_entry.connect_changed(move |e| {
                    s.emit(Message::ProjectTaskInputChanged(
                        pname.clone(),
                        e.text().to_string(),
                    ));
                });
            }
            {
                let s = sender.clone();
                let pname = project.name.clone();
                task_entry.connect_activate(move |_| {
                    s.emit(Message::AddTaskToProject(pname.clone()));
                });
            }
            task_row.append(&task_entry);
            task_row.append(&ui::icon_button_with_signal(
                "list-add-symbolic",
                Message::AddTaskToProject(project.name.clone()),
                sender,
            ));
            section.append(&task_row);

            content.append(&section);
        }
    }

    ui::page_wrapper(&content).upcast()
}
