use std::collections::HashMap;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, icon, row, scrollable, text, text_input};
use cosmic::Element;

use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::project::Project;
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

pub fn projects_view<'a>(
    projects: &[Project],
    project_input: &str,
    project_task_inputs: &HashMap<String, String>,
    ctx: &TaskRowCtx,
) -> Element<'a, Message> {
    let mut content = column().spacing(16);

    // Project creation input
    let input = text_input::text_input(fl!("projects-new-placeholder"), project_input.to_string())
        .on_input(Message::ProjectInputChanged)
        .on_submit(|_| Message::ProjectSubmit)
        .width(Length::Fill);

    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(input)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::ProjectSubmit),
            ),
    );

    if projects.is_empty() {
        content = content.push(
            container(text::body(fl!("projects-empty")))
                .padding(32)
                .center_x(Length::Fill)
                .width(Length::Fill),
        );
    } else {
        for project in projects {
            let (done, total) = project.completion_ratio();
            let mut section = column().spacing(4);

            let header_text = format!("{} ({}/{})", project.name, done, total);
            let header_row = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::title4(header_text).width(Length::Fill))
                .push(
                    button::icon(icon::from_name("edit-delete-symbolic"))
                        .on_press(Message::DeleteProject(project.name.clone())),
                );

            section = section.push(header_row);

            if project.is_stuck() {
                section = section.push(text::caption(fl!("projects-stuck")));
            }

            let active: Vec<&Task> = project.tasks.iter().filter(|t| t.state.is_active()).collect();
            if !active.is_empty() {
                section = section.push(task_grid(active.into_iter(), ctx));
            }

            // Per-project task input
            let task_input_value = project_task_inputs
                .get(&project.name)
                .cloned()
                .unwrap_or_default();
            let pname = project.name.clone();
            let pname2 = project.name.clone();
            let task_input = text_input::text_input(
                fl!("inbox-placeholder"),
                task_input_value,
            )
            .on_input(move |v| Message::ProjectTaskInputChanged(pname.clone(), v))
            .on_submit(move |_| Message::AddTaskToProject(pname2.clone()))
            .width(Length::Fill);

            let pname3 = project.name.clone();
            section = section.push(
                row()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(task_input)
                    .push(
                        button::icon(icon::from_name("list-add-symbolic"))
                            .on_press(Message::AddTaskToProject(pname3)),
                    ),
            );

            content = content.push(section);
        }
    }

    container(scrollable(content.padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
