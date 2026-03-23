//! GTK widget helper functions — thin wrappers to reduce boilerplate in pages.

use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::message::Message;

pub type Sender = relm4::Sender<Message>;

// --- Layout helpers ---

pub fn vbox(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Vertical, spacing)
}

pub fn hbox(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Horizontal, spacing)
}

/// Wrap content in a scrolled window that expands to fill.
pub fn scrolled(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    let sw = gtk::ScrolledWindow::new();
    sw.set_child(Some(child));
    sw.set_vexpand(true);
    sw.set_hexpand(true);
    sw.set_propagate_natural_height(true);
    sw
}

/// Standard page wrapper: scrolled vbox with padding.
pub fn page_wrapper(content: &gtk::Box) -> gtk::ScrolledWindow {
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    scrolled(content)
}

// --- Text helpers ---

pub fn label(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l
}

pub fn title3(text: &str) -> gtk::Label {
    let l = label(text);
    l.add_css_class("title-3");
    l
}

pub fn title4(text: &str) -> gtk::Label {
    let l = label(text);
    l.add_css_class("title-4");
    l
}

pub fn body(text: &str) -> gtk::Label {
    let l = label(text);
    l.add_css_class("body");
    l
}

pub fn caption(text: &str) -> gtk::Label {
    let l = label(text);
    l.add_css_class("caption");
    l.add_css_class("dim-label");
    l
}

// --- Button helpers ---

pub fn suggested_button(label_text: &str) -> gtk::Button {
    let btn = gtk::Button::with_label(label_text);
    btn.add_css_class("suggested-action");
    btn
}

pub fn standard_button(label_text: &str) -> gtk::Button {
    gtk::Button::with_label(label_text)
}

pub fn destructive_button(label_text: &str) -> gtk::Button {
    let btn = gtk::Button::with_label(label_text);
    btn.add_css_class("destructive-action");
    btn
}

pub fn icon_button(icon_name: &str) -> gtk::Button {
    gtk::Button::from_icon_name(icon_name)
}

pub fn flat_button(label_text: &str) -> gtk::Button {
    let btn = gtk::Button::with_label(label_text);
    btn.add_css_class("flat");
    btn
}

// --- Input helpers ---

pub fn entry(placeholder: &str, value: &str) -> gtk::Entry {
    let e = gtk::Entry::new();
    e.set_placeholder_text(Some(placeholder));
    e.set_text(value);
    e.set_hexpand(true);
    e
}

pub fn password_entry(placeholder: &str, value: &str) -> gtk::PasswordEntry {
    let e = gtk::PasswordEntry::new();
    e.set_placeholder_text(Some(placeholder));
    e.set_text(value);
    e.set_hexpand(true);
    e.set_show_peek_icon(true);
    e
}

pub fn search_entry(placeholder: &str, value: &str) -> gtk::SearchEntry {
    let e = gtk::SearchEntry::new();
    e.set_placeholder_text(Some(placeholder));
    e.set_text(value);
    e.set_hexpand(true);
    e
}

// --- Widget construction with signals ---

pub fn entry_with_signal(
    placeholder: &str,
    value: &str,
    on_changed: impl Fn(String) -> Message + 'static,
    on_activate: Option<Box<dyn Fn() -> Message + 'static>>,
    sender: &Sender,
) -> gtk::Entry {
    let e = entry(placeholder, value);
    {
        let s = sender.clone();
        e.connect_changed(move |entry| {
            s.emit(on_changed(entry.text().to_string()));
        });
    }
    if let Some(on_activate) = on_activate {
        let s = sender.clone();
        e.connect_activate(move |_| {
            s.emit(on_activate());
        });
    }
    e
}

pub fn button_with_signal(
    label_text: &str,
    css_class: Option<&str>,
    msg: Message,
    sender: &Sender,
) -> gtk::Button {
    let btn = gtk::Button::with_label(label_text);
    if let Some(class) = css_class {
        btn.add_css_class(class);
    }
    let s = sender.clone();
    btn.connect_clicked(move |_| {
        s.emit(msg.clone());
    });
    btn
}

pub fn icon_button_with_signal(
    icon_name: &str,
    msg: Message,
    sender: &Sender,
) -> gtk::Button {
    let btn = gtk::Button::from_icon_name(icon_name);
    btn.add_css_class("flat");
    let s = sender.clone();
    btn.connect_clicked(move |_| {
        s.emit(msg.clone());
    });
    btn
}

pub fn check_button_with_signal(
    is_checked: bool,
    msg: Message,
    sender: &Sender,
) -> gtk::CheckButton {
    let cb = gtk::CheckButton::new();
    cb.set_active(is_checked);
    let s = sender.clone();
    cb.connect_toggled(move |_| {
        s.emit(msg.clone());
    });
    cb
}

/// Build a dropdown (ComboBoxText) from labels, calling `on_select(index)` on change.
pub fn dropdown_with_signal(
    labels: &[String],
    selected: Option<usize>,
    on_select: impl Fn(usize) -> Message + 'static,
    sender: &Sender,
) -> gtk::DropDown {
    let items: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
    let model = gtk::StringList::new(&items);
    let dd = gtk::DropDown::new(Some(model), gtk::Expression::NONE);
    if let Some(sel) = selected {
        dd.set_selected(sel as u32);
    } else {
        dd.set_selected(gtk::INVALID_LIST_POSITION);
    }
    let s = sender.clone();
    dd.connect_selected_notify(move |dd| {
        let idx = dd.selected() as usize;
        s.emit(on_select(idx));
    });
    dd
}

// --- Container helpers ---

/// Remove all children from a container.
pub fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

/// A row with centered vertical alignment.
pub fn centered_hbox(spacing: i32) -> gtk::Box {
    let b = hbox(spacing);
    b.set_valign(gtk::Align::Center);
    b
}

/// Empty state placeholder per HIG — uses AdwStatusPage.
pub fn status_page(icon_name: &str, title: &str, description: &str) -> relm4::adw::StatusPage {
    let page = relm4::adw::StatusPage::new();
    page.set_icon_name(Some(icon_name));
    page.set_title(title);
    page.set_description(Some(description));
    page.set_vexpand(true);
    page
}
