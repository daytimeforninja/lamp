use cosmic::widget::{button, text};
use cosmic::{Element, theme};

use crate::message::Message;

/// Render a context tag chip (e.g., @home, @work, @errands).
pub fn context_tag(ctx: &str) -> Element<'static, Message> {
    button::custom(text::caption(ctx.to_string()).size(11.0))
        .padding([2, 8])
        .class(theme::Button::Text)
        .into()
}
