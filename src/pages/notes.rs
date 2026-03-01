use std::collections::{HashMap, HashSet};

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{
    button, column, container, flex_row, icon, row, scrollable, text, text_editor, text_input,
};
use cosmic::{Element, theme};
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

const CARD_WIDTH: f32 = 280.0;

/// Resolve a link target to a display name.
fn link_display_name(
    target: &LinkTarget,
    notes: &[Note],
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_items: &[ListItem],
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
        LinkTarget::ShoppingItem(id) => shopping_items
            .iter()
            .find(|i| i.id == *id)
            .map(|i| i.title.clone())
            .unwrap_or_else(|| format!("Shopping {}", &id.to_string()[..8])),
    }
}

fn card_front(note: &Note) -> Element<'static, Message> {
    let mut col = column().spacing(4);

    col = col.push(text::body(note.title.clone()));

    // Tag badges
    if !note.tags.is_empty() {
        let tag_badges: Vec<Element<'static, Message>> = note
            .tags
            .iter()
            .map(|t| {
                container(text::caption(t.clone()).size(11.0))
                    .padding([2, 6])
                    .class(theme::Container::Card)
                    .into()
            })
            .collect();
        col = col.push(flex_row(tag_badges).row_spacing(4).column_spacing(4));
    }

    if let Some(ref source) = note.source {
        col = col.push(
            container(text::caption(source.clone()).size(11.0))
                .padding([2, 6])
                .class(theme::Container::Card),
        );
    }

    col = col.push(
        text::caption(note.created.format("%Y-%m-%d").to_string()).size(11.0),
    );

    if !note.links.is_empty() {
        col = col.push(
            text::caption(fl!("notes-links", count = (note.links.len() as i64))).size(11.0),
        );
    }

    col.into()
}

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
    shopping_items: &[ListItem],
) -> Element<'static, Message> {
    let mut col = column().spacing(6);
    let note_id = note.id;

    col = col.push(text::body(note.title.clone()));

    // Body preview (first ~4 lines)
    if !note.body.is_empty() {
        let preview: String = note
            .body
            .lines()
            .take(4)
            .collect::<Vec<_>>()
            .join("\n");
        col = col.push(text::caption(preview).size(12.0));
    }

    // Linked entities
    if !note.links.is_empty() {
        col = col.push(text::caption(fl!("notes-links", count = (note.links.len() as i64))));
        for link in &note.links {
            let name = link_display_name(link, all_notes, contacts, accounts, projects, tasks, media_items, shopping_items);
            col = col.push(
                row()
                    .spacing(4)
                    .push(text::caption(format!("[{}]", link.kind_label())).size(11.0))
                    .push(text::caption(name).size(11.0)),
            );
        }
    }

    // Backlinks
    let note_target = LinkTarget::Note(note_id);
    if let Some(referencing_ids) = backlinks.get(&note_target) {
        col = col.push(text::caption(fl!(
            "notes-backlinks",
            count = (referencing_ids.len() as i64)
        )));
        for ref_id in referencing_ids {
            if let Some(ref_note) = all_notes.iter().find(|n| n.id == *ref_id) {
                col = col.push(text::caption(ref_note.title.clone()).size(11.0));
            }
        }
    }

    // Action buttons
    col = col.push(
        button::standard("Edit").on_press(Message::EditNote(note_id)),
    );

    col = col.push(
        button::standard(fl!("notes-open-editor"))
            .on_press(Message::OpenNoteInEditor(note_id)),
    );

    if confirming_delete {
        col = col.push(
            row()
                .spacing(8)
                .push(
                    button::destructive("Delete")
                        .on_press(Message::DeleteNote(note_id)),
                )
                .push(
                    button::standard("Cancel")
                        .on_press(Message::CancelDeleteNote),
                ),
        );
    } else {
        col = col.push(
            button::icon(icon::from_name("edit-delete-symbolic"))
                .on_press(Message::ConfirmDeleteNote(note_id)),
        );
    }

    col.into()
}

fn card_edit<'a>(
    note: &Note,
    editor_content: &'a Option<(Uuid, text_editor::Content)>,
    edit_buffer: &Option<NoteEditBuffer>,
    link_search: &str,
    all_notes: &[Note],
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_items: &[ListItem],
) -> Element<'a, Message> {
    let mut col = column().spacing(6);
    let note_id = note.id;

    // Read from edit buffer (avoids mutating the note while typing)
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
    col = col.push(
        text_input::text_input("Title", buf_title)
            .on_input(move |v| Message::SetNoteField(note_id, NoteField::Title, v))
            .width(Length::Fill),
    );

    // Body via text_editor
    if let Some((eid, content)) = editor_content {
        if *eid == note_id {
            col = col.push(
                container(
                    text_editor(content)
                        .on_action(Message::NoteEditorAction)
                        .height(Length::Fixed(120.0)),
                )
                .width(Length::Fill),
            );
        }
    }

    // Tags
    col = col.push(
        text_input::text_input("Tags (comma-separated)", buf_tags)
            .on_input(move |v| Message::SetNoteField(note_id, NoteField::Tags, v))
            .width(Length::Fill),
    );

    // Source — text input with media quick-pick buttons
    col = col.push(
        text_input::text_input(
            fl!("notes-source"),
            buf_source,
        )
        .on_input(move |v| Message::SetNoteField(note_id, NoteField::Source, v))
        .width(Length::Fill),
    );

    // Media quick-pick: show media items as clickable chips to set as source
    if !media_items.is_empty() {
        let media_chips: Vec<Element<'a, Message>> = media_items
            .iter()
            .take(10)
            .map(|item| {
                let title = item.title.clone();
                button::text(item.title.clone())
                    .on_press(Message::SetNoteField(note_id, NoteField::Source, title))
                    .into()
            })
            .collect();
        col = col.push(text::caption("Pick from media:").size(11.0));
        col = col.push(flex_row(media_chips).row_spacing(4).column_spacing(4));
    }

    // Existing links with unlink
    if !note.links.is_empty() {
        col = col.push(text::caption(fl!("notes-links", count = (note.links.len() as i64))));
        for link in &note.links {
            let name = link_display_name(link, all_notes, contacts, accounts, projects, tasks, media_items, shopping_items);
            let link_clone = link.clone();
            col = col.push(
                row()
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .push(text::caption(format!("[{}] {}", link.kind_label(), name)).size(11.0))
                    .push(
                        button::icon(icon::from_name("edit-delete-symbolic"))
                            .on_press(Message::RemoveNoteLink(note_id, link_clone)),
                    ),
            );
        }
    }

    // Link picker — results in a stable sub-column to avoid focus steal
    col = col.push(
        text_input::text_input(fl!("notes-link-search"), link_search.to_string())
            .on_input(Message::NoteLinkSearchChanged)
            .width(Length::Fill),
    );

    let mut results_col = column().spacing(4);

    if !link_search.is_empty() {
        let lq = link_search.to_lowercase();
        let mut results: Vec<Element<'a, Message>> = Vec::new();

        for n in all_notes {
            if n.id != note_id && n.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::Note(n.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results.push(
                        button::text(format!("[Note] {}", n.title))
                            .on_press(Message::AddNoteLink(note_id, t))
                            .into(),
                    );
                }
            }
        }
        for task in tasks {
            if task.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::Task(task.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results.push(
                        button::text(format!("[Task] {}", task.title))
                            .on_press(Message::AddNoteLink(note_id, t))
                            .into(),
                    );
                }
            }
        }
        for c in contacts {
            if c.name.to_lowercase().contains(&lq) {
                let target = LinkTarget::Contact(c.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results.push(
                        button::text(format!("[Contact] {}", c.name))
                            .on_press(Message::AddNoteLink(note_id, t))
                            .into(),
                    );
                }
            }
        }
        for a in accounts {
            if a.name.to_lowercase().contains(&lq) {
                let target = LinkTarget::Account(a.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results.push(
                        button::text(format!("[Account] {}", a.name))
                            .on_press(Message::AddNoteLink(note_id, t))
                            .into(),
                    );
                }
            }
        }
        for p in projects {
            if p.name.to_lowercase().contains(&lq) {
                let target = LinkTarget::Project(p.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results.push(
                        button::text(format!("[Project] {}", p.name))
                            .on_press(Message::AddNoteLink(note_id, t))
                            .into(),
                    );
                }
            }
        }
        for item in media_items {
            if item.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::MediaItem(item.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results.push(
                        button::text(format!("[Media] {}", item.title))
                            .on_press(Message::AddNoteLink(note_id, t))
                            .into(),
                    );
                }
            }
        }
        for item in shopping_items {
            if item.title.to_lowercase().contains(&lq) {
                let target = LinkTarget::ShoppingItem(item.id);
                if !note.links.contains(&target) {
                    let t = target.clone();
                    results.push(
                        button::text(format!("[Shopping] {}", item.title))
                            .on_press(Message::AddNoteLink(note_id, t))
                            .into(),
                    );
                }
            }
        }

        for r in results.into_iter().take(8) {
            results_col = results_col.push(r);
        }
    }

    col = col.push(results_col);

    // Done button to confirm and exit edit mode
    col = col.push(
        button::suggested("Done")
            .on_press(Message::FlipNote(note_id)),
    );

    col.into()
}

fn note_card<'a>(
    note: &Note,
    is_flipped: bool,
    is_editing: bool,
    confirming_delete: bool,
    editor_content: &'a Option<(Uuid, text_editor::Content)>,
    edit_buffer: &Option<NoteEditBuffer>,
    link_search: &str,
    backlinks: &HashMap<LinkTarget, Vec<Uuid>>,
    all_notes: &[Note],
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_items: &[ListItem],
) -> Element<'a, Message> {
    let note_id = note.id;

    let inner: Element<'a, Message> = if is_flipped && is_editing {
        card_edit(note, editor_content, edit_buffer, link_search, all_notes, contacts, accounts, projects, tasks, media_items, shopping_items)
    } else if is_flipped {
        card_back(note, confirming_delete, backlinks, all_notes, contacts, accounts, projects, tasks, media_items, shopping_items)
    } else {
        card_front(note)
    };

    let card_body = container(inner)
        .padding(12)
        .width(Length::Fixed(CARD_WIDTH))
        .class(theme::Container::Card);

    if is_editing {
        card_body.into()
    } else {
        button::custom(card_body)
            .padding(0)
            .class(theme::Button::Text)
            .on_press(Message::FlipNote(note_id))
            .into()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn notes_view<'a>(
    notes: &[Note],
    all_notes: &[Note],
    note_input: &str,
    flipped: &HashSet<Uuid>,
    editing: Option<Uuid>,
    pending_delete: Option<Uuid>,
    editor_content: &'a Option<(Uuid, text_editor::Content)>,
    edit_buffer: &Option<NoteEditBuffer>,
    link_search: &str,
    backlinks: &HashMap<LinkTarget, Vec<Uuid>>,
    contacts: &[Contact],
    accounts: &[Account],
    projects: &[Project],
    tasks: &[Task],
    media_items: &[ListItem],
    shopping_items: &[ListItem],
) -> Element<'a, Message> {
    let mut content = column().spacing(12);

    // Input row
    let input = text_input::text_input(fl!("notes-placeholder"), note_input.to_string())
        .on_input(Message::ZettelInputChanged)
        .on_submit(|_| Message::ZettelSubmit)
        .width(Length::Fill);

    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(input)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::ZettelSubmit),
            ),
    );

    if notes.is_empty() {
        content = content.push(
            container(text::body(fl!("notes-empty")))
                .padding(32)
                .center_x(Length::Fill)
                .width(Length::Fill),
        );
    } else {
        let cards: Vec<Element<'a, Message>> = notes
            .iter()
            .map(|note| {
                note_card(
                    note,
                    flipped.contains(&note.id),
                    editing == Some(note.id),
                    pending_delete == Some(note.id),
                    editor_content,
                    edit_buffer,
                    link_search,
                    backlinks,
                    all_notes,
                    contacts,
                    accounts,
                    projects,
                    tasks,
                    media_items,
                    shopping_items,
                )
            })
            .collect();

        content = content.push(flex_row(cards).row_spacing(12).column_spacing(12));
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
