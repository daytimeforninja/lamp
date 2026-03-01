use std::collections::{HashMap, HashSet};

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, flex_row, icon, row, scrollable, text, text_input};
use cosmic::{Element, theme};
use uuid::Uuid;

use crate::core::list_item::ListItem;
use crate::message::{ListKind, Message};

const CARD_WIDTH: f32 = 280.0;

fn done_label(kind: ListKind) -> &'static str {
    match kind {
        ListKind::Media => "Consumed",
        ListKind::Shopping => "Bought",
    }
}

fn card_front(item: &ListItem, kind: ListKind) -> Element<'static, Message> {
    let mut col = column().spacing(4);

    if item.done {
        col = col.push(text::caption(format!("{} {}", done_label(kind), item.title)));
    } else {
        col = col.push(text::body(item.title.clone()));
    }

    let date_str = item.created.format("%Y-%m-%d").to_string();
    col = col.push(text::caption(date_str).size(11.0));

    if !item.notes.is_empty() {
        let preview: String = item.notes.lines().take(2).collect::<Vec<_>>().join("\n");
        col = col.push(text::caption(preview).size(11.0));
    }

    col.into()
}

fn card_back(
    item: &ListItem,
    kind: ListKind,
    confirming_delete: bool,
    note_inputs: &HashMap<Uuid, String>,
) -> Element<'static, Message> {
    let id = item.id;
    let mut col = column().spacing(6);

    col = col.push(text::body(item.title.clone()));

    if !item.notes.is_empty() {
        col = col.push(
            container(text::caption(item.notes.clone()))
                .padding([4, 8])
                .width(Length::Fill),
        );
    }

    // Note input
    let input_value = note_inputs.get(&id).cloned().unwrap_or_default();
    let note_input = text_input::text_input("Add a note...", input_value)
        .on_input(move |v| Message::NoteInputChanged(id, v))
        .on_submit(move |_| Message::AppendNote(id))
        .width(Length::Fill);
    col = col.push(note_input);

    // Consumed/Bought toggle
    let toggle_label = if item.done {
        format!("Undo {}", done_label(kind).to_lowercase())
    } else {
        done_label(kind).to_string()
    };
    if item.done {
        col = col.push(
            button::standard(toggle_label)
                .on_press(Message::ToggleListItemDone(kind, id)),
        );
    } else {
        col = col.push(
            button::suggested(toggle_label)
                .on_press(Message::ToggleListItemDone(kind, id)),
        );
    }

    // Close
    col = col.push(
        button::standard("Close")
            .on_press(Message::FlipListItem(id)),
    );

    // Delete with confirmation
    if confirming_delete {
        col = col.push(
            row()
                .spacing(8)
                .push(
                    button::destructive("Delete")
                        .on_press(Message::DeleteListItem(kind, id)),
                )
                .push(
                    button::standard("Cancel")
                        .on_press(Message::CancelDeleteListItem),
                ),
        );
    } else {
        col = col.push(
            button::icon(icon::from_name("edit-delete-symbolic"))
                .on_press(Message::ConfirmDeleteListItem(kind, id)),
        );
    }

    col.into()
}

fn list_card(
    item: &ListItem,
    kind: ListKind,
    is_flipped: bool,
    confirming_delete: bool,
    note_inputs: &HashMap<Uuid, String>,
) -> Element<'static, Message> {
    let id = item.id;
    let inner: Element<'static, Message> = if is_flipped {
        card_back(item, kind, confirming_delete, note_inputs)
    } else {
        card_front(item, kind)
    };

    let card_body = container(inner)
        .padding(12)
        .width(Length::Fixed(CARD_WIDTH))
        .class(theme::Container::Card);

    if is_flipped {
        card_body.into()
    } else {
        button::custom(card_body)
            .padding(0)
            .class(theme::Button::Text)
            .on_press(Message::FlipListItem(id))
            .into()
    }
}

pub fn list_view(
    items: &[ListItem],
    input_value: &str,
    placeholder: String,
    empty_msg: String,
    kind: ListKind,
    flipped: &HashSet<Uuid>,
    pending_delete: Option<(ListKind, Uuid)>,
    note_inputs: &HashMap<Uuid, String>,
) -> Element<'static, Message> {
    let mut content = column().spacing(12);

    // Creation input
    let input = text_input::text_input(placeholder, input_value.to_string())
        .on_input(move |v| Message::ListInputChanged(kind, v))
        .on_submit(move |_| Message::ListSubmit(kind))
        .width(Length::Fill);

    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(input)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::ListSubmit(kind)),
            ),
    );

    if items.is_empty() {
        content = content.push(
            container(text::body(empty_msg))
                .padding(32)
                .center_x(Length::Fill)
                .width(Length::Fill),
        );
    } else {
        // Show active items first, then consumed/bought
        let mut active: Vec<&ListItem> = items.iter().filter(|i| !i.done).collect();
        let mut done: Vec<&ListItem> = items.iter().filter(|i| i.done).collect();
        active.sort_by(|a, b| b.created.cmp(&a.created));
        done.sort_by(|a, b| b.created.cmp(&a.created));

        if !active.is_empty() {
            let cards: Vec<Element<'static, Message>> = active
                .iter()
                .map(|item| {
                    let confirming = pending_delete
                        .is_some_and(|(k, id)| k == kind && id == item.id);
                    list_card(item, kind, flipped.contains(&item.id), confirming, note_inputs)
                })
                .collect();

            content = content.push(
                flex_row(cards)
                    .row_spacing(12)
                    .column_spacing(12),
            );
        }

        if !done.is_empty() {
            let cards: Vec<Element<'static, Message>> = done
                .iter()
                .map(|item| {
                    let confirming = pending_delete
                        .is_some_and(|(k, id)| k == kind && id == item.id);
                    list_card(item, kind, flipped.contains(&item.id), confirming, note_inputs)
                })
                .collect();

            content = content.push(
                flex_row(cards)
                    .row_spacing(12)
                    .column_spacing(12),
            );
        }
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
