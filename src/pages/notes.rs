use std::collections::{HashMap, HashSet};

use relm4::gtk;
use relm4::gtk::prelude::*;
use uuid::Uuid;

use crate::application::NoteEditBuffer;
use crate::core::account::Account;
use crate::core::link::LinkTarget;
use crate::core::list_item::ListItem;
use crate::core::note::Note;
use crate::core::project::Project;
use crate::core::task::Task;
use crate::fl;
use crate::message::{Message, NoteField};
use crate::sync::carddav::Contact;
use crate::ui::{self, Sender};

/// Resolve a link target to a display name.
fn link_display_name(
    target: &LinkTarget,
    notes: &[Note],
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_tasks: &[Task],
) -> String {
    match target {
        LinkTarget::Note(id) => notes
            .iter()
            .find(|n| n.id == *id)
            .map(|n| n.title.clone())
            .unwrap_or_else(|| format!("Note {}", &id.to_string()[..8])),
        LinkTarget::Contact(id) => contacts
            .iter()
            .find(|c| c.id == *id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("Contact {}", &id.to_string()[..8])),
        LinkTarget::Account(id) => accounts
            .iter()
            .find(|a| a.id == *id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| format!("Account {}", &id.to_string()[..8])),
        LinkTarget::Project(id) => projects
            .iter()
            .find(|p| p.id == *id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Project {}", &id.to_string()[..8])),
        LinkTarget::Task(id) => tasks
            .iter()
            .find(|t| t.id == *id)
            .map(|t| t.title.clone())
            .unwrap_or_else(|| format!("Task {}", &id.to_string()[..8])),
        LinkTarget::MediaItem(id) => media_items
            .iter()
            .find(|i| i.id == *id)
            .map(|i| i.title.clone())
            .unwrap_or_else(|| format!("Media {}", &id.to_string()[..8])),
        LinkTarget::ShoppingItem(id) => shopping_tasks
            .iter()
            .find(|t| t.id == *id)
            .map(|t| t.title.clone())
            .unwrap_or_else(|| format!("Shopping {}", &id.to_string()[..8])),
    }
}

fn card_front(note: &Note) -> gtk::Box {
    let col = ui::vbox(4);

    col.append(&ui::body(&note.title));

    // Tag badges
    if !note.tags.is_empty() {
        let tags_row = gtk::FlowBox::new();
        tags_row.set_selection_mode(gtk::SelectionMode::None);
        tags_row.set_row_spacing(4);
        tags_row.set_column_spacing(4);
        tags_row.set_max_children_per_line(10);
        for tag in &note.tags {
            let badge = gtk::Frame::new(None);
            let lbl = ui::caption(tag);
            lbl.set_margin_start(6);
            lbl.set_margin_end(6);
            lbl.set_margin_top(2);
            lbl.set_margin_bottom(2);
            badge.set_child(Some(&lbl));
            badge.add_css_class("card");
            tags_row.insert(&badge, -1);
        }
        col.append(&tags_row);
    }

    if let Some(ref source) = note.source {
        let source_frame = gtk::Frame::new(None);
        let lbl = ui::caption(source);
        lbl.set_margin_start(6);
        lbl.set_margin_end(6);
        lbl.set_margin_top(2);
        lbl.set_margin_bottom(2);
        source_frame.set_child(Some(&lbl));
        source_frame.add_css_class("card");
        col.append(&source_frame);
    }

    col.append(&ui::caption(&note.created.format("%Y-%m-%d").to_string()));

    if !note.links.is_empty() {
        col.append(&ui::caption(&fl!("notes-links", count = (note.links.len() as i64))));
    }

    col
}

#[allow(clippy::too_many_arguments)]
fn card_back(
    note: &Note,
    confirming_delete: bool,
    backlinks: &HashMap<LinkTarget, Vec<Uuid>>,
    all_notes: &[Note],
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_tasks: &[Task],
    sender: &Sender,
) -> gtk::Box {
    let col = ui::vbox(6);
    let note_id = note.id;

    col.append(&ui::body(&note.title));

    // Body preview (first ~4 lines)
    if !note.body.is_empty() {
        let preview: String = note
            .body
            .lines()
            .take(4)
            .collect::<Vec<_>>()
            .join("\n");
        col.append(&ui::caption(&preview));
    }

    // Linked entities
    if !note.links.is_empty() {
        col.append(&ui::caption(&fl!("notes-links", count = (note.links.len() as i64))));
        for link in &note.links {
            let name = link_display_name(link, all_notes, contacts, accounts, projects, tasks, media_items, shopping_tasks);
            let link_row = ui::hbox(4);
            link_row.append(&ui::caption(&format!("[{}]", link.kind_label())));
            link_row.append(&ui::caption(&name));
            col.append(&link_row);
        }
    }

    // Backlinks
    let note_target = LinkTarget::Note(note_id);
    if let Some(referencing_ids) = backlinks.get(&note_target) {
        col.append(&ui::caption(&fl!(
            "notes-backlinks",
            count = (referencing_ids.len() as i64)
        )));
        for ref_id in referencing_ids {
            if let Some(ref_note) = all_notes.iter().find(|n| n.id == *ref_id) {
                col.append(&ui::caption(&ref_note.title));
            }
        }
    }

    // Action buttons
    col.append(&ui::button_with_signal(&fl!("btn-edit"), None, Message::EditNote(note_id), sender));
    col.append(&ui::button_with_signal(&fl!("notes-open-editor"), None, Message::OpenNoteInEditor(note_id), sender));

    if confirming_delete {
        let row = ui::hbox(8);
        row.append(&ui::button_with_signal(&fl!("btn-delete"), Some("destructive-action"), Message::DeleteNote(note_id), sender));
        row.append(&ui::button_with_signal(&fl!("btn-cancel"), None, Message::CancelDeleteNote, sender));
        col.append(&row);
    } else {
        col.append(&ui::icon_button_with_signal("edit-delete-symbolic", Message::ConfirmDeleteNote(note_id), sender));
    }

    col
}

#[allow(clippy::too_many_arguments)]
fn card_edit(
    note: &Note,
    note_body_buffer: &Option<(Uuid, String)>,
    edit_buffer: &Option<NoteEditBuffer>,
    link_search: &str,
    all_notes: &[Note],
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_tasks: &[Task],
    sender: &Sender,
) -> gtk::Box {
    let col = ui::vbox(6);
    let note_id = note.id;

    // Read from edit buffer
    let buf_title = edit_buffer
        .as_ref()
        .filter(|b| b.id == note_id)
        .map(|b| b.title.clone())
        .unwrap_or_else(|| note.title.clone());
    let buf_tags = edit_buffer
        .as_ref()
        .filter(|b| b.id == note_id)
        .map(|b| b.tags.clone())
        .unwrap_or_else(|| note.tags.join(", "));
    let buf_source = edit_buffer
        .as_ref()
        .filter(|b| b.id == note_id)
        .map(|b| b.source.clone())
        .unwrap_or_else(|| note.source.clone().unwrap_or_default());

    // Title
    let title_entry = ui::entry(&fl!("notes-title-placeholder"), &buf_title);
    {
        let s = sender.clone();
        title_entry.connect_changed(move |e| {
            s.emit(Message::SetNoteField(note_id, NoteField::Title, e.text().to_string()));
        });
    }
    col.append(&title_entry);

    // Body via TextView
    let body_text = note_body_buffer
        .as_ref()
        .filter(|(id, _)| *id == note_id)
        .map(|(_, text)| text.as_str())
        .unwrap_or(&note.body);

    let text_buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
    text_buffer.set_text(body_text);
    let text_view = gtk::TextView::with_buffer(&text_buffer);
    text_view.set_wrap_mode(gtk::WrapMode::WordChar);
    text_view.set_height_request(120);
    text_view.set_hexpand(true);
    {
        let s = sender.clone();
        text_buffer.connect_changed(move |buf| {
            let start = buf.start_iter();
            let end = buf.end_iter();
            let text = buf.text(&start, &end, false).to_string();
            s.emit(Message::NoteBodyChanged(text));
        });
    }
    let tv_frame = gtk::Frame::new(None);
    tv_frame.set_child(Some(&text_view));
    col.append(&tv_frame);

    // Tags
    let tags_entry = ui::entry(&fl!("notes-tags-placeholder"), &buf_tags);
    {
        let s = sender.clone();
        tags_entry.connect_changed(move |e| {
            s.emit(Message::SetNoteField(note_id, NoteField::Tags, e.text().to_string()));
        });
    }
    col.append(&tags_entry);

    // Source
    let source_entry = ui::entry(&fl!("notes-source"), &buf_source);
    {
        let s = sender.clone();
        source_entry.connect_changed(move |e| {
            s.emit(Message::SetNoteField(note_id, NoteField::Source, e.text().to_string()));
        });
    }
    col.append(&source_entry);

    // Media quick-pick
    if !media_items.is_empty() {
        col.append(&ui::caption(&fl!("notes-pick-media")));
        let media_flow = gtk::FlowBox::new();
        media_flow.set_selection_mode(gtk::SelectionMode::None);
        media_flow.set_row_spacing(4);
        media_flow.set_column_spacing(4);
        media_flow.set_max_children_per_line(10);
        for item in media_items.iter().take(10) {
            let title = item.title.clone();
            let btn = ui::button_with_signal(
                &item.title,
                Some("flat"),
                Message::SetNoteField(note_id, NoteField::Source, title),
                sender,
            );
            media_flow.insert(&btn, -1);
        }
        col.append(&media_flow);
    }

    // Existing links with unlink
    if !note.links.is_empty() {
        col.append(&ui::caption(&fl!("notes-links", count = (note.links.len() as i64))));
        for link in &note.links {
            let name = link_display_name(link, all_notes, contacts, accounts, projects, tasks, media_items, shopping_tasks);
            let link_row = ui::centered_hbox(4);
            link_row.append(&ui::caption(&format!("[{}] {}", link.kind_label(), name)));
            let link_clone = link.clone();
            link_row.append(&ui::icon_button_with_signal(
                "edit-delete-symbolic",
                Message::RemoveNoteLink(note_id, link_clone),
                sender,
            ));
            col.append(&link_row);
        }
    }

    // Link picker search
    let search_entry = ui::entry(&fl!("notes-link-search"), link_search);
    {
        let s = sender.clone();
        search_entry.connect_changed(move |e| {
            s.emit(Message::NoteLinkSearchChanged(e.text().to_string()));
        });
    }
    col.append(&search_entry);

    // Link search results
    if !link_search.is_empty() {
        let results_col = ui::vbox(4);
        let lq = link_search.to_lowercase();
        let mut count = 0;

        for n in all_notes {
            if count >= 8 { break; }
            if n.id != note_id && n.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::Note(n.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results_col.append(&ui::button_with_signal(
                        &format!("[Note] {}", n.title),
                        Some("flat"),
                        Message::AddNoteLink(note_id, t),
                        sender,
                    ));
                    count += 1;
                }
            }
        }
        for task in tasks {
            if count >= 8 { break; }
            if task.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::Task(task.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results_col.append(&ui::button_with_signal(
                        &format!("[Task] {}", task.title),
                        Some("flat"),
                        Message::AddNoteLink(note_id, t),
                        sender,
                    ));
                    count += 1;
                }
            }
        }
        for c in contacts {
            if count >= 8 { break; }
            if c.name.to_lowercase().contains(&lq) {
                let target = LinkTarget::Contact(c.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results_col.append(&ui::button_with_signal(
                        &format!("[Contact] {}", c.name),
                        Some("flat"),
                        Message::AddNoteLink(note_id, t),
                        sender,
                    ));
                    count += 1;
                }
            }
        }
        for a in accounts {
            if count >= 8 { break; }
            if a.name.to_lowercase().contains(&lq) {
                let target = LinkTarget::Account(a.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results_col.append(&ui::button_with_signal(
                        &format!("[Account] {}", a.name),
                        Some("flat"),
                        Message::AddNoteLink(note_id, t),
                        sender,
                    ));
                    count += 1;
                }
            }
        }
        for p in projects {
            if count >= 8 { break; }
            if p.name.to_lowercase().contains(&lq) {
                let target = LinkTarget::Project(p.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results_col.append(&ui::button_with_signal(
                        &format!("[Project] {}", p.name),
                        Some("flat"),
                        Message::AddNoteLink(note_id, t),
                        sender,
                    ));
                    count += 1;
                }
            }
        }
        for item in media_items {
            if count >= 8 { break; }
            if item.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::MediaItem(item.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results_col.append(&ui::button_with_signal(
                        &format!("[Media] {}", item.title),
                        Some("flat"),
                        Message::AddNoteLink(note_id, t),
                        sender,
                    ));
                    count += 1;
                }
            }
        }
        for task in shopping_tasks {
            if count >= 8 { break; }
            if task.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::ShoppingItem(task.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results_col.append(&ui::button_with_signal(
                        &format!("[Shopping] {}", task.title),
                        Some("flat"),
                        Message::AddNoteLink(note_id, t),
                        sender,
                    ));
                    count += 1;
                }
            }
        }

        col.append(&results_col);
    }

    // Done button
    col.append(&ui::button_with_signal(&fl!("btn-done"), Some("suggested-action"), Message::FlipNote(note_id), sender));

    col
}

#[allow(clippy::too_many_arguments)]
fn note_card(
    note: &Note,
    is_flipped: bool,
    is_editing: bool,
    confirming_delete: bool,
    note_body_buffer: &Option<(Uuid, String)>,
    edit_buffer: &Option<NoteEditBuffer>,
    link_search: &str,
    backlinks: &HashMap<LinkTarget, Vec<Uuid>>,
    all_notes: &[Note],
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_tasks: &[Task],
    sender: &Sender,
) -> gtk::Widget {
    let note_id = note.id;

    let inner: gtk::Box = if is_flipped && is_editing {
        card_edit(note, note_body_buffer, edit_buffer, link_search, all_notes, contacts, accounts, projects, tasks, media_items, shopping_tasks, sender)
    } else if is_flipped {
        card_back(note, confirming_delete, backlinks, all_notes, contacts, accounts, projects, tasks, media_items, shopping_tasks, sender)
    } else {
        card_front(note)
    };

    let frame = gtk::Frame::new(None);
    inner.set_margin_start(12);
    inner.set_margin_end(12);
    inner.set_margin_top(12);
    inner.set_margin_bottom(12);
    inner.set_width_request(280);
    frame.set_child(Some(&inner));
    frame.add_css_class("card");

    if is_editing {
        frame.upcast()
    } else {
        let btn = gtk::Button::new();
        btn.set_child(Some(&frame));
        btn.add_css_class("flat");
        let s = sender.clone();
        btn.connect_clicked(move |_| {
            s.emit(Message::FlipNote(note_id));
        });
        btn.upcast()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn notes_view(
    notes: &[Note],
    note_input: &str,
    flipped: &HashSet<Uuid>,
    editing: Option<Uuid>,
    pending_delete: Option<Uuid>,
    note_body_buffer: &Option<(Uuid, String)>,
    note_edit_buffer: &Option<NoteEditBuffer>,
    note_link_search: &str,
    backlink_index: &HashMap<LinkTarget, Vec<Uuid>>,
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media: &[ListItem],
    shopping: &[Task],
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(12);

    // Input row
    let input_row = ui::centered_hbox(8);
    let entry = ui::entry(&fl!("notes-placeholder"), note_input);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::ZettelInputChanged(e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::ZettelSubmit);
        });
    }
    input_row.append(&entry);
    input_row.append(&ui::icon_button_with_signal("list-add-symbolic", Message::ZettelSubmit, sender));
    content.append(&input_row);

    if notes.is_empty() {
        let empty_label = ui::body(&fl!("notes-empty"));
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_margin_top(32);
        empty_label.set_margin_bottom(32);
        empty_label.set_hexpand(true);
        content.append(&empty_label);
    } else {
        let flow = gtk::FlowBox::new();
        flow.set_selection_mode(gtk::SelectionMode::None);
        flow.set_homogeneous(false);
        flow.set_row_spacing(12);
        flow.set_column_spacing(12);
        flow.set_max_children_per_line(10);
        flow.set_min_children_per_line(1);

        for note in notes {
            let card = note_card(
                note,
                flipped.contains(&note.id),
                editing == Some(note.id),
                pending_delete == Some(note.id),
                note_body_buffer,
                note_edit_buffer,
                note_link_search,
                backlink_index,
                notes,
                contacts,
                accounts,
                projects,
                tasks,
                media,
                shopping,
                sender,
            );
            flow.insert(&card, -1);
        }

        content.append(&flow);
    }

    ui::page_wrapper(&content).upcast()
}
