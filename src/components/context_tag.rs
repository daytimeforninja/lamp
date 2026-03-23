use relm4::gtk;
use relm4::gtk::prelude::*;

/// Render a context tag chip (e.g., @home, @work, @errands).
#[allow(dead_code)]
pub fn context_tag(tag: &str) -> gtk::Button {
    let btn = gtk::Button::with_label(tag);
    btn.add_css_class("flat");
    btn
}
