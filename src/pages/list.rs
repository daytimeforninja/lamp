use std::collections::HashMap;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, icon, row, scrollable, text, text_input};
use cosmic::{Element, theme};
use uuid::Uuid;

use crate::core::list_item::ListItem;
use crate::message::{ListKind, Message};

pub fn list_view(
    items: &[ListItem],
    input_value: &str,
    placeholder: String,
    empty_msg: String,
    kind: ListKind,
    expanded_task: Option<Uuid>,
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
        for item in items {
            let id = item.id;
            let is_expanded = expanded_task == Some(id);

            // Title (clickable) + delete button
            let title_btn = button::custom(text::body(item.title.clone()))
                .padding([0, 0])
                .class(theme::Button::Text)
                .on_press(Message::ToggleTaskExpand(id));

            let delete_btn = button::icon(icon::from_name("edit-delete-symbolic"))
                .on_press(Message::DeleteListItem(kind, id));

            let item_row = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(title_btn).width(Length::Fill))
                .push(delete_btn);

            let mut item_col = column().push(item_row);

            if is_expanded {
                let mut notes_col = column().spacing(4).padding([4, 0, 4, 24]);

                if !item.notes.is_empty() {
                    notes_col = notes_col.push(
                        container(text::body(item.notes.clone()))
                            .padding([4, 8])
                            .width(Length::Fill),
                    );
                }

                let input_value = note_inputs.get(&id).cloned().unwrap_or_default();
                let note_input = text_input::text_input("Add a note...", input_value)
                    .on_input(move |v| Message::NoteInputChanged(id, v))
                    .on_submit(move |_| Message::AppendNote(id))
                    .width(Length::Fill);

                notes_col = notes_col.push(note_input);
                item_col = item_col.push(notes_col);
            }

            content = content.push(item_col);
        }
    }

    container(scrollable(content.padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
