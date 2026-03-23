use std::collections::{BTreeMap, HashSet};

use chrono::Local;
use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::fl;
use crate::message::{ContactField, Message};
use crate::sync::carddav::Contact;
use crate::ui::{self, Sender};

const PREFERRED_LABELS: &[&str] = &["\u{2014}", "Email", "Phone", "Signal"];

fn preferred_to_index(method: Option<&str>) -> Option<usize> {
    match method {
        None => Some(0),
        Some("Email") => Some(1),
        Some("Phone") => Some(2),
        Some("Signal") => Some(3),
        _ => Some(0),
    }
}

fn index_to_preferred(idx: usize) -> Option<String> {
    match idx {
        1 => Some("Email".to_string()),
        2 => Some("Phone".to_string()),
        3 => Some("Signal".to_string()),
        _ => None,
    }
}

fn last_contacted_text(contact: &Contact) -> String {
    match contact.last_contacted {
        Some(d) => {
            let days = (Local::now().date_naive() - d).num_days();
            fl!("contacts-last-contacted", days = days)
        }
        None => fl!("contacts-never-contacted"),
    }
}

fn detail_line(label_text: &str, value: &Option<String>) -> Option<gtk::Box> {
    value.as_ref().filter(|v| !v.is_empty()).map(|v| {
        let row = ui::hbox(6);
        row.append(&ui::caption(&format!("{}:", label_text)));
        row.append(&ui::caption(v));
        row
    })
}

fn card_front(contact: &Contact) -> gtk::Box {
    let col = ui::vbox(4);

    col.append(&ui::body(&contact.name));

    let groups_label = if contact.groups.is_empty() {
        fl!("contacts-personal")
    } else {
        contact.groups.join(", ")
    };
    col.append(&ui::caption(&groups_label));
    col.append(&ui::caption(&last_contacted_text(contact)));

    if let Some(ref m) = contact.preferred_method {
        col.append(&ui::caption(&fl!("contacts-preferred", method = m.as_str())));
    }

    col
}

fn card_back(
    contact: &Contact,
    index: usize,
    confirming_delete: bool,
    sender: &Sender,
) -> gtk::Box {
    let col = ui::vbox(6);

    col.append(&ui::body(&contact.name));

    if let Some(line) = detail_line(&fl!("contacts-email"), &contact.email) {
        col.append(&line);
    }
    if let Some(line) = detail_line(&fl!("contacts-phone"), &contact.phone) {
        col.append(&line);
    }
    if let Some(line) = detail_line(&fl!("contacts-website"), &contact.website) {
        col.append(&line);
    }
    if let Some(line) = detail_line(&fl!("contacts-signal"), &contact.signal) {
        col.append(&line);
    }

    col.append(&ui::button_with_signal(&fl!("btn-done"), Some("suggested-action"), Message::FlipContact(index), sender));
    col.append(&ui::button_with_signal(&fl!("contacts-mark-contacted"), None, Message::MarkContacted(index), sender));
    col.append(&ui::button_with_signal(&fl!("btn-edit"), None, Message::EditContact(index), sender));

    if confirming_delete {
        let row = ui::hbox(8);
        row.append(&ui::button_with_signal(&fl!("btn-delete"), Some("destructive-action"), Message::DeleteContact(index), sender));
        row.append(&ui::button_with_signal(&fl!("btn-cancel"), None, Message::CancelDeleteContact, sender));
        col.append(&row);
    } else {
        col.append(&ui::icon_button_with_signal("edit-delete-symbolic", Message::ConfirmDeleteContact(index), sender));
    }

    col
}

fn card_edit(contact: &Contact, index: usize, sender: &Sender) -> gtk::Box {
    let col = ui::vbox(6);

    col.append(&ui::body(&contact.name));

    // Email
    let email_val = contact.email.clone().unwrap_or_default();
    let email_entry = ui::entry(&fl!("contacts-email-placeholder"), &email_val);
    {
        let s = sender.clone();
        email_entry.connect_changed(move |e| {
            s.emit(Message::SetContactField(index, ContactField::Email, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        email_entry.connect_activate(move |_| {
            s.emit(Message::FlipContact(index));
        });
    }
    col.append(&email_entry);

    // Phone
    let phone_val = contact.phone.clone().unwrap_or_default();
    let phone_entry = ui::entry(&fl!("contacts-phone-placeholder"), &phone_val);
    {
        let s = sender.clone();
        phone_entry.connect_changed(move |e| {
            s.emit(Message::SetContactField(index, ContactField::Phone, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        phone_entry.connect_activate(move |_| {
            s.emit(Message::FlipContact(index));
        });
    }
    col.append(&phone_entry);

    // Website
    let website_val = contact.website.clone().unwrap_or_default();
    let website_entry = ui::entry(&fl!("contacts-url-placeholder"), &website_val);
    {
        let s = sender.clone();
        website_entry.connect_changed(move |e| {
            s.emit(Message::SetContactField(index, ContactField::Website, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        website_entry.connect_activate(move |_| {
            s.emit(Message::FlipContact(index));
        });
    }
    col.append(&website_entry);

    // Signal
    let signal_val = contact.signal.clone().unwrap_or_default();
    let signal_entry = ui::entry(&fl!("contacts-signal-placeholder"), &signal_val);
    {
        let s = sender.clone();
        signal_entry.connect_changed(move |e| {
            s.emit(Message::SetContactField(index, ContactField::Signal, e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        signal_entry.connect_activate(move |_| {
            s.emit(Message::FlipContact(index));
        });
    }
    col.append(&signal_entry);

    // Preferred method dropdown
    let pref_labels: Vec<String> = PREFERRED_LABELS.iter().map(|s| s.to_string()).collect();
    let pref_selected = preferred_to_index(contact.preferred_method.as_deref());
    let pref_row = ui::centered_hbox(8);
    pref_row.append(&ui::caption(&fl!("contacts-preferred-label")));
    let pref_dd = ui::dropdown_with_signal(
        &pref_labels,
        pref_selected,
        move |idx| {
            let val = index_to_preferred(idx).unwrap_or_default();
            Message::SetContactField(index, ContactField::PreferredMethod, val)
        },
        sender,
    );
    pref_row.append(&pref_dd);
    col.append(&pref_row);

    // Groups text input (comma-separated)
    let groups_val = contact.groups.join(", ");
    let groups_row = ui::centered_hbox(8);
    groups_row.append(&ui::caption(&fl!("contacts-groups")));
    let groups_entry = ui::entry(&fl!("contacts-groups-placeholder"), &groups_val);
    {
        let s = sender.clone();
        groups_entry.connect_changed(move |e| {
            let groups: Vec<String> = e.text().to_string()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            s.emit(Message::SetContactGroups(index, groups));
        });
    }
    {
        let s = sender.clone();
        groups_entry.connect_activate(move |_| {
            s.emit(Message::FlipContact(index));
        });
    }
    groups_row.append(&groups_entry);
    col.append(&groups_row);

    col
}

fn contact_card(
    contact: &Contact,
    index: usize,
    is_flipped: bool,
    is_editing: bool,
    confirming_delete: bool,
    sender: &Sender,
) -> gtk::Widget {
    let inner: gtk::Box = if is_flipped && is_editing {
        card_edit(contact, index, sender)
    } else if is_flipped {
        card_back(contact, index, confirming_delete, sender)
    } else {
        card_front(contact)
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
        // In edit mode, don't wrap in clickable button -- inputs need focus
        frame.upcast()
    } else {
        let btn = gtk::Button::new();
        btn.set_child(Some(&frame));
        btn.add_css_class("flat");
        let s = sender.clone();
        btn.connect_clicked(move |_| {
            s.emit(Message::FlipContact(index));
        });
        btn.upcast()
    }
}

fn card_grid(
    contacts: &[&Contact],
    indices: &[usize],
    flipped: &HashSet<usize>,
    editing: Option<usize>,
    pending_delete: Option<usize>,
    sender: &Sender,
) -> gtk::FlowBox {
    let flow = gtk::FlowBox::new();
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_homogeneous(false);
    flow.set_row_spacing(12);
    flow.set_column_spacing(12);
    flow.set_max_children_per_line(10);
    flow.set_min_children_per_line(1);

    for (contact, idx) in contacts.iter().zip(indices.iter()) {
        let card = contact_card(
            contact,
            *idx,
            flipped.contains(idx),
            editing == Some(*idx),
            pending_delete == Some(*idx),
            sender,
        );
        flow.insert(&card, -1);
    }

    flow
}

pub fn contacts_view(
    contacts: &[Contact],
    contact_input: &str,
    flipped: &HashSet<usize>,
    editing: Option<usize>,
    pending_delete: Option<usize>,
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(12);

    // Add contact input row
    let input_row = ui::centered_hbox(8);
    let entry = ui::entry(&fl!("contacts-placeholder"), contact_input);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::ContactInputChanged(e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::ContactSubmit);
        });
    }
    input_row.append(&entry);
    input_row.append(&ui::icon_button_with_signal("list-add-symbolic", Message::ContactSubmit, sender));
    content.append(&input_row);

    if contacts.is_empty() {
        let empty_label = ui::body(&fl!("contacts-empty"));
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_margin_top(32);
        empty_label.set_margin_bottom(32);
        empty_label.set_hexpand(true);
        content.append(&empty_label);
    } else {
        // Groups to hide from the UI
        const HIDDEN_GROUPS: &[&str] = &["Personal", "archive", "Autosaved"];

        // Group contacts by their visible groups only
        let mut by_group: BTreeMap<String, Vec<(usize, &Contact)>> = BTreeMap::new();
        for (idx, contact) in contacts.iter().enumerate() {
            for group in &contact.groups {
                if !HIDDEN_GROUPS.iter().any(|h| group.eq_ignore_ascii_case(h)) {
                    by_group.entry(group.clone())
                        .or_default()
                        .push((idx, contact));
                }
            }
        }

        for (group_name, group_contacts) in &by_group {
            content.append(&ui::title4(group_name));
            let group_c: Vec<&Contact> = group_contacts.iter().map(|(_, c)| *c).collect();
            let group_i: Vec<usize> = group_contacts.iter().map(|(i, _)| *i).collect();
            content.append(&card_grid(&group_c, &group_i, flipped, editing, pending_delete, sender));
        }
    }

    ui::page_wrapper(&content).upcast()
}
