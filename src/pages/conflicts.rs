use cosmic::iced::Length;
use cosmic::widget::{button, column, container, row, scrollable, text};
use cosmic::Element;

use crate::fl;
use crate::message::Message;
use crate::sync::SyncConflict;

pub fn conflicts_view(conflicts: &[SyncConflict]) -> Element<'static, Message> {
    if conflicts.is_empty() {
        return container(text::body(fl!("conflicts-empty")))
            .padding(32)
            .center_x(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    // Partition conflicts by type
    let mut mismatches: Vec<(usize, &SyncConflict)> = Vec::new();
    let mut remote_only: Vec<(usize, &SyncConflict)> = Vec::new();
    let mut local_only: Vec<(usize, &SyncConflict)> = Vec::new();

    for (i, conflict) in conflicts.iter().enumerate() {
        match conflict {
            SyncConflict::StateMismatch { .. } => mismatches.push((i, conflict)),
            SyncConflict::RemoteOnly { .. } => remote_only.push((i, conflict)),
            SyncConflict::LocalOnly { .. } => local_only.push((i, conflict)),
        }
    }

    let mut sections = column().spacing(24);

    // Status Mismatches section
    if !mismatches.is_empty() {
        let mut section = column().spacing(8);
        section = section.push(text::title4(fl!("conflicts-status-mismatches")));
        for (idx, conflict) in &mismatches {
            if let SyncConflict::StateMismatch {
                title,
                local_state,
                remote_state,
                ..
            } = conflict
            {
                let label = text::body(format!(
                    "{} — Local: {} / Remote: {}",
                    title, local_state, remote_state
                ));
                let buttons = row()
                    .spacing(8)
                    .push(
                        button::standard(fl!("conflicts-accept-remote"))
                            .on_press(Message::AcceptRemoteState(*idx)),
                    )
                    .push(
                        button::standard(fl!("conflicts-accept-local"))
                            .on_press(Message::AcceptLocalState(*idx)),
                    );
                let conflict_row = row()
                    .spacing(16)
                    .align_y(cosmic::iced::Alignment::Center)
                    .push(container(label).width(Length::Fill))
                    .push(buttons);
                section = section.push(conflict_row);
            }
        }
        sections = sections.push(section);
    }

    // Remote Only section
    if !remote_only.is_empty() {
        let mut section = column().spacing(8);
        section = section.push(text::title4(fl!("conflicts-remote-only")));
        for (idx, conflict) in &remote_only {
            if let SyncConflict::RemoteOnly { task, .. } = conflict {
                let label = text::body(format!(
                    "{} ({})",
                    task.title,
                    task.state.as_keyword()
                ));
                let buttons = row()
                    .spacing(8)
                    .push(
                        button::standard(fl!("conflicts-import"))
                            .on_press(Message::ImportConflictTask(*idx)),
                    )
                    .push(
                        button::standard(fl!("conflicts-delete"))
                            .on_press(Message::DeleteConflict(*idx)),
                    );
                let conflict_row = row()
                    .spacing(16)
                    .align_y(cosmic::iced::Alignment::Center)
                    .push(container(label).width(Length::Fill))
                    .push(buttons);
                section = section.push(conflict_row);
            }
        }
        sections = sections.push(section);
    }

    // Local Only section
    if !local_only.is_empty() {
        let mut section = column().spacing(8);
        section = section.push(text::title4(fl!("conflicts-local-only")));
        for (idx, conflict) in &local_only {
            if let SyncConflict::LocalOnly {
                title,
                local_state,
                ..
            } = conflict
            {
                let label = text::body(format!("{} ({})", title, local_state));
                let buttons = row().spacing(8).push(
                    button::standard(fl!("conflicts-delete"))
                        .on_press(Message::DeleteConflict(*idx)),
                );
                let conflict_row = row()
                    .spacing(16)
                    .align_y(cosmic::iced::Alignment::Center)
                    .push(container(label).width(Length::Fill))
                    .push(buttons);
                section = section.push(conflict_row);
            }
        }
        sections = sections.push(section);
    }

    container(scrollable(container(sections).padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
