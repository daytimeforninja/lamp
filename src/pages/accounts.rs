use chrono::Local;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, icon, row, scrollable, text, text_input};
use cosmic::{Element, theme};

use crate::core::account::Account;
use crate::fl;
use crate::message::{AccountField, Message};

fn last_checked_text(account: &Account) -> String {
    match account.last_checked {
        Some(d) => {
            let days = (Local::now().date_naive() - d).num_days();
            fl!("accounts-last-checked", days = days)
        }
        None => fl!("accounts-never-checked"),
    }
}

fn account_row(
    account: &Account,
    index: usize,
    expanded: bool,
    confirming_delete: bool,
) -> Element<'static, Message> {
    let name_text = text::body(account.name.clone());

    let url_badge: Element<'static, Message> = if !account.url.is_empty() {
        let label = account.url.clone();
        button::custom(text::caption(label).size(11.0))
            .padding([0, 0])
            .class(theme::Button::Text)
            .on_press(Message::OpenAccountUrl(index))
            .into()
    } else {
        text::caption("").into()
    };

    let last_text = text::caption(last_checked_text(account)).size(11.0);

    let title_btn = button::custom(name_text)
        .padding([0, 0])
        .class(theme::Button::Text)
        .on_press(Message::ToggleAccountExpand(index));

    let mut summary_row = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(title_btn)
        .push(url_badge)
        .push(container(last_text).width(Length::Fill));

    if confirming_delete {
        summary_row = summary_row
            .push(
                button::destructive("Delete")
                    .on_press(Message::DeleteAccount(index)),
            )
            .push(
                button::standard("Cancel")
                    .on_press(Message::CancelDeleteAccount),
            );
    } else {
        summary_row = summary_row.push(
            button::icon(icon::from_name("edit-delete-symbolic"))
                .on_press(Message::ConfirmDeleteAccount(index)),
        );
    }

    let mut col = column().push(summary_row);

    if expanded {
        let mut detail = column().spacing(6).padding([4, 0, 8, 24]);

        // Name
        detail = detail.push(
            row().spacing(8).align_y(Alignment::Center)
                .push(container(text::caption("Name")).width(Length::Fixed(80.0)))
                .push(
                    text_input::text_input("Account name", account.name.clone())
                        .on_input(move |v| Message::SetAccountFieldValue(index, AccountField::Name, v))
                        .on_submit(move |_| Message::ToggleAccountExpand(index))
                        .width(Length::Fill),
                ),
        );

        // URL
        detail = detail.push(
            row().spacing(8).align_y(Alignment::Center)
                .push(container(text::caption("URL")).width(Length::Fixed(80.0)))
                .push(
                    text_input::text_input("https://...", account.url.clone())
                        .on_input(move |v| Message::SetAccountFieldValue(index, AccountField::Url, v))
                        .on_submit(move |_| Message::ToggleAccountExpand(index))
                        .width(Length::Fill),
                )
                .push(
                    button::icon(icon::from_name("web-browser-symbolic"))
                        .on_press(Message::OpenAccountUrl(index)),
                ),
        );

        // Notes
        detail = detail.push(
            row().spacing(8).align_y(Alignment::Center)
                .push(container(text::caption("Notes")).width(Length::Fixed(80.0)))
                .push(
                    text_input::text_input("Notes...", account.notes.clone())
                        .on_input(move |v| Message::SetAccountFieldValue(index, AccountField::Notes, v))
                        .on_submit(move |_| Message::ToggleAccountExpand(index))
                        .width(Length::Fill),
                ),
        );

        // Mark checked button
        detail = detail.push(
            button::standard(fl!("accounts-mark-checked"))
                .on_press(Message::MarkAccountChecked(index)),
        );

        col = col.push(detail);
    }

    col.into()
}

pub fn accounts_view(
    accounts: &[Account],
    account_input: &str,
    expanded_account: Option<usize>,
    pending_delete: Option<usize>,
) -> Element<'static, Message> {
    let mut content = column().spacing(12);

    // Add account input row
    let input = text_input::text_input(fl!("accounts-placeholder"), account_input.to_string())
        .on_input(Message::AccountInputChanged)
        .on_submit(|_| Message::AccountSubmit)
        .width(Length::Fill);

    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(input)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::AccountSubmit),
            ),
    );

    if accounts.is_empty() {
        content = content.push(
            container(text::body(fl!("accounts-empty")))
                .padding(32)
                .center_x(Length::Fill)
                .width(Length::Fill),
        );
    } else {
        for (idx, account) in accounts.iter().enumerate() {
            content = content.push(account_row(
                account,
                idx,
                expanded_account == Some(idx),
                pending_delete == Some(idx),
            ));
        }
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
