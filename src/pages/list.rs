use std::collections::{HashMap, HashSet};

use relm4::gtk;
use relm4::gtk::prelude::*;
use uuid::Uuid;

use crate::core::list_item::ListItem;
use crate::core::task::Task;
use crate::fl;
use crate::message::{ListKind, Message};
use crate::ui::{self, Sender};

fn done_label(kind: ListKind) -> String {
    match kind {
        ListKind::Media => fl!("list-consumed"),
        ListKind::Shopping => fl!("list-bought"),
    }
}

fn card_front(item: &ListItem, kind: ListKind) -> gtk::Box {
    let col = ui::vbox(4);

    if item.done {
        let lbl = ui::caption(&format!("{} {}", &done_label(kind), item.title));
        col.append(&lbl);
    } else {
        col.append(&ui::body(&item.title));
    }

    let date_str = item.created.format("%Y-%m-%d").to_string();
    col.append(&ui::caption(&date_str));

    if !item.notes.is_empty() {
        let preview: String = item.notes.lines().take(2).collect::<Vec<_>>().join("\n");
        col.append(&ui::caption(&preview));
    }

    col
}

fn card_back(
    item: &ListItem,
    kind: ListKind,
    confirming_delete: bool,
    note_inputs: &HashMap<Uuid, String>,
    sender: &Sender,
) -> gtk::Box {
    let id = item.id;
    let col = ui::vbox(6);

    col.append(&ui::body(&item.title));

    if !item.notes.is_empty() {
        let notes_lbl = ui::caption(&item.notes);
        notes_lbl.set_margin_start(8);
        notes_lbl.set_margin_end(8);
        notes_lbl.set_margin_top(4);
        notes_lbl.set_hexpand(true);
        col.append(&notes_lbl);
    }

    // Note input
    let input_value = note_inputs.get(&id).cloned().unwrap_or_default();
    let note_entry = ui::entry(&fl!("task-note-placeholder"), &input_value);
    {
        let s = sender.clone();
        note_entry.connect_changed(move |e| {
            s.emit(Message::NoteInputChanged(id, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        note_entry.connect_activate(move |_| {
            s.emit(Message::AppendNote(id));
        });
    }
    col.append(&note_entry);

    // Consumed/Bought toggle
    let toggle_label = if item.done {
        format!("Undo {}", &done_label(kind).to_lowercase())
    } else {
        done_label(kind)
    };
    let css = if item.done { None } else { Some("suggested-action") };
    let toggle_btn = ui::button_with_signal(&toggle_label, css, Message::ToggleListItemDone(kind, id), sender);
    col.append(&toggle_btn);

    // Close
    let close_btn = ui::button_with_signal(&fl!("btn-close"), None, Message::FlipListItem(id), sender);
    col.append(&close_btn);

    // Delete with confirmation
    if confirming_delete {
        let row = ui::hbox(8);
        row.append(&ui::button_with_signal(&fl!("btn-delete"), Some("destructive-action"), Message::DeleteListItem(kind, id), sender));
        row.append(&ui::button_with_signal(&fl!("btn-cancel"), None, Message::CancelDeleteListItem, sender));
        col.append(&row);
    } else {
        col.append(&ui::icon_button_with_signal("edit-delete-symbolic", Message::ConfirmDeleteListItem(kind, id), sender));
    }

    col
}

fn list_card(
    item: &ListItem,
    kind: ListKind,
    is_flipped: bool,
    confirming_delete: bool,
    note_inputs: &HashMap<Uuid, String>,
    sender: &Sender,
) -> gtk::Widget {
    let id = item.id;

    let inner: gtk::Box = if is_flipped {
        card_back(item, kind, confirming_delete, note_inputs, sender)
    } else {
        card_front(item, kind)
    };

    let frame = gtk::Frame::new(None);
    inner.set_margin_start(12);
    inner.set_margin_end(12);
    inner.set_margin_top(12);
    inner.set_margin_bottom(12);
    inner.set_width_request(280);
    frame.set_child(Some(&inner));
    frame.add_css_class("card");

    if is_flipped {
        frame.upcast()
    } else {
        let btn = gtk::Button::new();
        btn.set_child(Some(&frame));
        btn.add_css_class("flat");
        let s = sender.clone();
        btn.connect_clicked(move |_| {
            s.emit(Message::FlipListItem(id));
        });
        btn.upcast()
    }
}

fn card_flow(cards: Vec<gtk::Widget>) -> gtk::FlowBox {
    let flow = gtk::FlowBox::new();
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_homogeneous(false);
    flow.set_row_spacing(12);
    flow.set_column_spacing(12);
    flow.set_max_children_per_line(10);
    flow.set_min_children_per_line(1);
    for card in cards {
        flow.insert(&card, -1);
    }
    flow
}

pub fn list_view(
    items: &[ListItem],
    input_value: &str,
    placeholder: String,
    empty_text: String,
    kind: ListKind,
    flipped: &HashSet<Uuid>,
    pending_delete: Option<(ListKind, Uuid)>,
    note_inputs: &HashMap<Uuid, String>,
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(12);

    // Creation input row
    let input_row = ui::centered_hbox(8);
    let entry = ui::entry(&placeholder, input_value);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::ListInputChanged(kind, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::ListSubmit(kind));
        });
    }
    input_row.append(&entry);
    input_row.append(&ui::icon_button_with_signal("list-add-symbolic", Message::ListSubmit(kind), sender));
    content.append(&input_row);

    if items.is_empty() {
        let empty_label = ui::body(&empty_text);
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_margin_top(32);
        empty_label.set_margin_bottom(32);
        empty_label.set_hexpand(true);
        content.append(&empty_label);
    } else {
        // Show active items first, then done
        let mut active: Vec<&ListItem> = items.iter().filter(|i| !i.done).collect();
        let mut done: Vec<&ListItem> = items.iter().filter(|i| i.done).collect();
        active.sort_by(|a, b| b.created.cmp(&a.created));
        done.sort_by(|a, b| b.created.cmp(&a.created));

        if !active.is_empty() {
            let cards: Vec<gtk::Widget> = active
                .iter()
                .map(|item| {
                    let confirming = pending_delete
                        .is_some_and(|(k, id)| k == kind && id == item.id);
                    list_card(item, kind, flipped.contains(&item.id), confirming, note_inputs, sender)
                })
                .collect();
            content.append(&card_flow(cards));
        }

        if !done.is_empty() {
            let cards: Vec<gtk::Widget> = done
                .iter()
                .map(|item| {
                    let confirming = pending_delete
                        .is_some_and(|(k, id)| k == kind && id == item.id);
                    list_card(item, kind, flipped.contains(&item.id), confirming, note_inputs, sender)
                })
                .collect();
            content.append(&card_flow(cards));
        }
    }

    ui::page_wrapper(&content).upcast()
}

// --- Shopping view (uses Task instead of ListItem) ---

fn shopping_card_front(task: &Task) -> gtk::Box {
    let col = ui::vbox(4);

    if task.state.is_done() {
        col.append(&ui::caption(&format!("{} {}", fl!("list-bought"), task.title)));
    } else {
        col.append(&ui::body(&task.title));
    }

    let date_str = task.created.format("%Y-%m-%d").to_string();
    col.append(&ui::caption(&date_str));

    if !task.notes.is_empty() {
        let preview: String = task.notes.lines().take(2).collect::<Vec<_>>().join("\n");
        col.append(&ui::caption(&preview));
    }

    col
}

fn shopping_card_back(
    task: &Task,
    confirming_delete: bool,
    note_inputs: &HashMap<Uuid, String>,
    sender: &Sender,
) -> gtk::Box {
    let id = task.id;
    let kind = ListKind::Shopping;
    let col = ui::vbox(6);

    col.append(&ui::body(&task.title));

    if !task.notes.is_empty() {
        let notes_lbl = ui::caption(&task.notes);
        notes_lbl.set_margin_start(8);
        notes_lbl.set_margin_end(8);
        notes_lbl.set_margin_top(4);
        notes_lbl.set_hexpand(true);
        col.append(&notes_lbl);
    }

    // Note input
    let input_value = note_inputs.get(&id).cloned().unwrap_or_default();
    let note_entry = ui::entry(&fl!("task-note-placeholder"), &input_value);
    {
        let s = sender.clone();
        note_entry.connect_changed(move |e| {
            s.emit(Message::NoteInputChanged(id, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        note_entry.connect_activate(move |_| {
            s.emit(Message::AppendNote(id));
        });
    }
    col.append(&note_entry);

    // Bought toggle
    let toggle_label = if task.state.is_done() {
        format!("Undo {}", fl!("list-bought").to_lowercase())
    } else {
        fl!("list-bought")
    };
    let css = if task.state.is_done() { None } else { Some("suggested-action") };
    let toggle_btn = ui::button_with_signal(&toggle_label, css, Message::ToggleListItemDone(kind, id), sender);
    col.append(&toggle_btn);

    // Close
    col.append(&ui::button_with_signal(&fl!("btn-close"), None, Message::FlipListItem(id), sender));

    // Delete with confirmation
    if confirming_delete {
        let row = ui::hbox(8);
        row.append(&ui::button_with_signal(&fl!("btn-delete"), Some("destructive-action"), Message::DeleteListItem(kind, id), sender));
        row.append(&ui::button_with_signal(&fl!("btn-cancel"), None, Message::CancelDeleteListItem, sender));
        col.append(&row);
    } else {
        col.append(&ui::icon_button_with_signal("edit-delete-symbolic", Message::ConfirmDeleteListItem(kind, id), sender));
    }

    col
}

pub fn shopping_view(
    tasks: &[Task],
    input_value: &str,
    flipped: &HashSet<Uuid>,
    pending_delete: Option<(ListKind, Uuid)>,
    note_inputs: &HashMap<Uuid, String>,
    sender: &Sender,
) -> gtk::Widget {
    let kind = ListKind::Shopping;
    let content = ui::vbox(12);

    // Creation input row
    let input_row = ui::centered_hbox(8);
    let entry = ui::entry(&fl!("shopping-placeholder"), input_value);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::ListInputChanged(kind, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::ListSubmit(kind));
        });
    }
    input_row.append(&entry);
    input_row.append(&ui::icon_button_with_signal("list-add-symbolic", Message::ListSubmit(kind), sender));
    content.append(&input_row);

    if tasks.is_empty() {
        let empty_label = ui::body(&fl!("shopping-empty"));
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_margin_top(32);
        empty_label.set_margin_bottom(32);
        empty_label.set_hexpand(true);
        content.append(&empty_label);
    } else {
        let mut active: Vec<&Task> = tasks.iter().filter(|t| !t.state.is_done()).collect();
        let mut done: Vec<&Task> = tasks.iter().filter(|t| t.state.is_done()).collect();
        active.sort_by(|a, b| b.created.cmp(&a.created));
        done.sort_by(|a, b| b.created.cmp(&a.created));

        for group in [active, done] {
            if !group.is_empty() {
                let cards: Vec<gtk::Widget> = group
                    .iter()
                    .map(|task| {
                        let id = task.id;
                        let is_flipped = flipped.contains(&id);
                        let confirming = pending_delete
                            .is_some_and(|(k, did)| k == kind && did == id);

                        let inner: gtk::Box = if is_flipped {
                            shopping_card_back(task, confirming, note_inputs, sender)
                        } else {
                            shopping_card_front(task)
                        };

                        let frame = gtk::Frame::new(None);
                        inner.set_margin_start(12);
                        inner.set_margin_end(12);
                        inner.set_margin_top(12);
                        inner.set_margin_bottom(12);
                        inner.set_width_request(280);
                        frame.set_child(Some(&inner));
                        frame.add_css_class("card");

                        if is_flipped {
                            frame.upcast()
                        } else {
                            let btn = gtk::Button::new();
                            btn.set_child(Some(&frame));
                            btn.add_css_class("flat");
                            let s = sender.clone();
                            btn.connect_clicked(move |_| {
                                s.emit(Message::FlipListItem(id));
                            });
                            btn.upcast()
                        }
                    })
                    .collect();

                content.append(&card_flow(cards));
            }
        }
    }

    ui::page_wrapper(&content).upcast()
}
