use std::collections::HashSet;

use chrono::Local;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, dropdown, flex_row, icon, row, scrollable, text, text_input};
use cosmic::{Element, theme};

use crate::fl;
use crate::message::{ContactField, Message};
use crate::sync::carddav::{Contact, ContactCategory};

const CARD_WIDTH: f32 = 280.0;

const PREFERRED_LABELS: &[&str] = &["—", "Email", "Phone", "Signal"];

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

fn detail_line(label: &str, value: &Option<String>) -> Option<Element<'static, Message>> {
    value.as_ref().filter(|v| !v.is_empty()).map(|v| {
        row()
            .spacing(6)
            .push(text::caption(format!("{}:", label)))
            .push(text::caption(v.clone()))
            .into()
    })
}

fn card_front(contact: &Contact) -> Element<'static, Message> {
    let mut col = column().spacing(4);

    col = col.push(text::body(contact.name.clone()));

    let cat_label = match contact.category {
        ContactCategory::Personal => fl!("contacts-personal"),
        ContactCategory::Service => fl!("contacts-service"),
    };
    col = col.push(text::caption(cat_label).size(11.0));

    col = col.push(text::caption(last_contacted_text(contact)).size(11.0));

    if let Some(ref m) = contact.preferred_method {
        col = col.push(text::caption(fl!("contacts-preferred", method = m.as_str())).size(11.0));
    }

    col.into()
}

fn card_back(
    contact: &Contact,
    index: usize,
    confirming_delete: bool,
) -> Element<'static, Message> {
    let mut col = column().spacing(6);

    col = col.push(text::body(contact.name.clone()));

    if let Some(line) = detail_line("Email", &contact.email) {
        col = col.push(line);
    }
    if let Some(line) = detail_line("Phone", &contact.phone) {
        col = col.push(line);
    }
    if let Some(line) = detail_line("Website", &contact.website) {
        col = col.push(line);
    }
    if let Some(line) = detail_line("Signal", &contact.signal) {
        col = col.push(line);
    }

    col = col.push(
        button::suggested("Done")
            .on_press(Message::FlipContact(index)),
    );

    col = col.push(
        button::standard(fl!("contacts-mark-contacted"))
            .on_press(Message::MarkContacted(index)),
    );

    col = col.push(
        button::standard("Edit")
            .on_press(Message::EditContact(index)),
    );

    if confirming_delete {
        col = col.push(
            row()
                .spacing(8)
                .push(
                    button::destructive("Delete")
                        .on_press(Message::DeleteContact(index)),
                )
                .push(
                    button::standard("Cancel")
                        .on_press(Message::CancelDeleteContact),
                ),
        );
    } else {
        col = col.push(
            button::icon(icon::from_name("edit-delete-symbolic"))
                .on_press(Message::ConfirmDeleteContact(index)),
        );
    }

    col.into()
}

fn card_edit(contact: &Contact, index: usize) -> Element<'static, Message> {
    let mut col = column().spacing(6);

    col = col.push(text::body(contact.name.clone()));

    let email_val = contact.email.clone().unwrap_or_default();
    col = col.push(
        text_input::text_input("email@example.com", email_val)
            .on_input(move |v| Message::SetContactField(index, ContactField::Email, v))
            .on_submit(move |_| Message::FlipContact(index))
            .width(Length::Fill),
    );

    let phone_val = contact.phone.clone().unwrap_or_default();
    col = col.push(
        text_input::text_input("+1-555-0000", phone_val)
            .on_input(move |v| Message::SetContactField(index, ContactField::Phone, v))
            .on_submit(move |_| Message::FlipContact(index))
            .width(Length::Fill),
    );

    let website_val = contact.website.clone().unwrap_or_default();
    col = col.push(
        text_input::text_input("https://...", website_val)
            .on_input(move |v| Message::SetContactField(index, ContactField::Website, v))
            .on_submit(move |_| Message::FlipContact(index))
            .width(Length::Fill),
    );

    let signal_val = contact.signal.clone().unwrap_or_default();
    col = col.push(
        text_input::text_input("username", signal_val)
            .on_input(move |v| Message::SetContactField(index, ContactField::Signal, v))
            .on_submit(move |_| Message::FlipContact(index))
            .width(Length::Fill),
    );

    // Preferred method dropdown
    let pref_labels: Vec<String> = PREFERRED_LABELS.iter().map(|s| s.to_string()).collect();
    let pref_selected = preferred_to_index(contact.preferred_method.as_deref());
    col = col.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(text::caption("Preferred"))
            .push(
                dropdown(pref_labels, pref_selected, move |idx| {
                    let val = index_to_preferred(idx).unwrap_or_default();
                    Message::SetContactField(index, ContactField::PreferredMethod, val)
                })
                .width(Length::Shrink),
            ),
    );

    // Category dropdown
    let cat_labels: Vec<String> = vec!["Personal".to_string(), "Service".to_string()];
    let cat_selected = match contact.category {
        ContactCategory::Personal => Some(0usize),
        ContactCategory::Service => Some(1),
    };
    col = col.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(text::caption("Category"))
            .push(
                dropdown(cat_labels, cat_selected, move |idx| {
                    let cat = if idx == 1 {
                        ContactCategory::Service
                    } else {
                        ContactCategory::Personal
                    };
                    Message::SetContactCategory(index, cat)
                })
                .width(Length::Shrink),
            ),
    );

    col.into()
}

fn contact_card(
    contact: &Contact,
    index: usize,
    is_flipped: bool,
    is_editing: bool,
    confirming_delete: bool,
) -> Element<'static, Message> {
    let inner: Element<'static, Message> = if is_flipped && is_editing {
        card_edit(contact, index)
    } else if is_flipped {
        card_back(contact, index, confirming_delete)
    } else {
        card_front(contact)
    };

    let card_body = container(inner)
        .padding(12)
        .width(Length::Fixed(CARD_WIDTH))
        .class(theme::Container::Card);

    if is_editing {
        // In edit mode, don't wrap in a clickable button — inputs need focus
        card_body.into()
    } else {
        button::custom(card_body)
            .padding(0)
            .class(theme::Button::Text)
            .on_press(Message::FlipContact(index))
            .into()
    }
}

fn card_grid(
    contacts: &[(usize, &Contact)],
    flipped: &HashSet<usize>,
    editing: Option<usize>,
    pending_delete: Option<usize>,
) -> Element<'static, Message> {
    let cards: Vec<Element<'static, Message>> = contacts
        .iter()
        .map(|(idx, c)| {
            contact_card(
                c,
                *idx,
                flipped.contains(idx),
                editing == Some(*idx),
                pending_delete == Some(*idx),
            )
        })
        .collect();

    flex_row(cards)
        .row_spacing(12)
        .column_spacing(12)
        .into()
}

pub fn contacts_view(
    contacts: &[Contact],
    contact_input: &str,
    flipped: &HashSet<usize>,
    editing: Option<usize>,
    pending_delete: Option<usize>,
) -> Element<'static, Message> {
    let mut content = column().spacing(12);

    // Add contact input row
    let input = text_input::text_input(fl!("contacts-placeholder"), contact_input.to_string())
        .on_input(Message::ContactInputChanged)
        .on_submit(|_| Message::ContactSubmit)
        .width(Length::Fill);

    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(input)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::ContactSubmit),
            ),
    );

    if contacts.is_empty() {
        content = content.push(
            container(text::body(fl!("contacts-empty")))
                .padding(32)
                .center_x(Length::Fill)
                .width(Length::Fill),
        );
    } else {
        // Personal section
        let personal: Vec<(usize, &Contact)> = contacts
            .iter()
            .enumerate()
            .filter(|(_, c)| c.category == ContactCategory::Personal)
            .collect();

        if !personal.is_empty() {
            content = content.push(text::title4(fl!("contacts-personal")));
            content = content.push(card_grid(&personal, flipped, editing, pending_delete));
        }

        // Service section
        let service: Vec<(usize, &Contact)> = contacts
            .iter()
            .enumerate()
            .filter(|(_, c)| c.category == ContactCategory::Service)
            .collect();

        if !service.is_empty() {
            content = content.push(text::title4(fl!("contacts-service")));
            content = content.push(card_grid(&service, flipped, editing, pending_delete));
        }
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
