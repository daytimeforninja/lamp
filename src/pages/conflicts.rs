use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::fl;
use crate::message::Message;
use crate::sync::SyncConflict;
use crate::ui::{self, Sender};

pub fn conflicts_view(
    conflicts: &[SyncConflict],
    sender: &Sender,
) -> gtk::Widget {
    if conflicts.is_empty() {
        return ui::status_page("object-select-symbolic", "No Conflicts", "All synced tasks are consistent").upcast();
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

    let sections = ui::vbox(24);

    // Status Mismatches section
    if !mismatches.is_empty() {
        let section = ui::vbox(8);
        section.append(&ui::title4(&fl!("conflicts-status-mismatches")));

        for (idx, conflict) in &mismatches {
            if let SyncConflict::StateMismatch {
                title,
                local_state,
                remote_state,
                ..
            } = conflict
            {
                let conflict_row = ui::centered_hbox(16);

                let label = ui::body(&format!(
                    "{} -- Local: {} / Remote: {}",
                    title, local_state, remote_state
                ));
                label.set_hexpand(true);
                conflict_row.append(&label);

                let buttons = ui::hbox(8);
                buttons.append(&ui::button_with_signal(
                    &fl!("conflicts-accept-remote"),
                    None,
                    Message::AcceptRemoteState(*idx),
                    sender,
                ));
                buttons.append(&ui::button_with_signal(
                    &fl!("conflicts-accept-local"),
                    None,
                    Message::AcceptLocalState(*idx),
                    sender,
                ));
                conflict_row.append(&buttons);
                section.append(&conflict_row);
            }
        }
        sections.append(&section);
    }

    // Remote Only section
    if !remote_only.is_empty() {
        let section = ui::vbox(8);
        section.append(&ui::title4(&fl!("conflicts-remote-only")));

        for (idx, conflict) in &remote_only {
            if let SyncConflict::RemoteOnly { task, .. } = conflict {
                let conflict_row = ui::centered_hbox(16);

                let label = ui::body(&format!(
                    "{} ({})",
                    task.title,
                    task.state.as_keyword()
                ));
                label.set_hexpand(true);
                conflict_row.append(&label);

                let buttons = ui::hbox(8);
                buttons.append(&ui::button_with_signal(
                    &fl!("conflicts-import"),
                    None,
                    Message::ImportConflictTask(*idx),
                    sender,
                ));
                buttons.append(&ui::button_with_signal(
                    &fl!("conflicts-delete"),
                    None,
                    Message::DeleteConflict(*idx),
                    sender,
                ));
                conflict_row.append(&buttons);
                section.append(&conflict_row);
            }
        }
        sections.append(&section);
    }

    // Local Only section
    if !local_only.is_empty() {
        let section = ui::vbox(8);
        section.append(&ui::title4(&fl!("conflicts-local-only")));

        for (idx, conflict) in &local_only {
            if let SyncConflict::LocalOnly {
                title,
                local_state,
                ..
            } = conflict
            {
                let conflict_row = ui::centered_hbox(16);

                let label = ui::body(&format!("{} ({})", title, local_state));
                label.set_hexpand(true);
                conflict_row.append(&label);

                let buttons = ui::hbox(8);
                buttons.append(&ui::button_with_signal(
                    &fl!("conflicts-delete"),
                    None,
                    Message::DeleteConflict(*idx),
                    sender,
                ));
                conflict_row.append(&buttons);
                section.append(&conflict_row);
            }
        }
        sections.append(&section);
    }

    ui::page_wrapper(&sections).upcast()
}
