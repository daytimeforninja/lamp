use cosmic::iced::Length;
use cosmic::widget::{column, container, row, scrollable, text, text_input};
use cosmic::Element;

use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

pub fn archive_view(
    tasks: &[Task],
    search: &str,
) -> Element<'static, Message> {
    let lq = search.to_lowercase();
    let mut filtered: Vec<&Task> = if lq.is_empty() {
        tasks.iter().collect()
    } else {
        tasks
            .iter()
            .filter(|t| t.title.to_lowercase().contains(&lq))
            .collect()
    };

    // Sort by completion date, newest first
    filtered.sort_by(|a, b| b.completed.cmp(&a.completed));

    let search_input = text_input::text_input(
        fl!("search-placeholder"),
        search.to_string(),
    )
    .on_input(Message::ArchiveSearchChanged)
    .width(Length::Fill);

    if filtered.is_empty() {
        let msg = if search.is_empty() {
            fl!("archive-empty")
        } else {
            fl!("archive-no-results")
        };
        return container(
            column()
                .spacing(8)
                .push(container(search_input).padding([0, 16]))
                .push(
                    container(text::body(msg))
                        .padding(32)
                        .center_x(Length::Fill)
                        .width(Length::Fill),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    let count_label = text::caption(format!(
        "{} {}",
        filtered.len(),
        fl!("archive-count-suffix")
    ));

    let rows: Vec<Element<'static, Message>> = filtered
        .iter()
        .map(|task| {
            let title = task.title.clone();
            let state_label = format!("{:?}", task.state);
            let completed_str = task
                .completed
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            let project_str = task.project.clone().unwrap_or_default();
            let contexts_str = task.contexts.join(", ");

            let mut info_parts: Vec<Element<'static, Message>> = Vec::new();

            // State badge
            info_parts.push(
                container(text::caption(state_label))
                    .padding([2, 6])
                    .into(),
            );

            // Completed date
            if !completed_str.is_empty() {
                info_parts.push(text::caption(completed_str).into());
            }

            // Project
            if !project_str.is_empty() {
                info_parts.push(text::caption(format!("[{}]", project_str)).into());
            }

            // Contexts
            if !contexts_str.is_empty() {
                info_parts.push(text::caption(contexts_str).into());
            }

            let info_row = row::with_children(info_parts).spacing(8);

            container(
                column()
                    .spacing(2)
                    .push(text::body(title))
                    .push(info_row),
            )
            .padding([8, 16])
            .width(Length::Fill)
            .into()
        })
        .collect();

    let list = column::with_children(rows).spacing(0);

    container(
        column()
            .spacing(8)
            .push(container(search_input).padding([0, 16]))
            .push(container(count_label).padding([0, 16]))
            .push(scrollable(container(list).padding([0, 0]))),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
