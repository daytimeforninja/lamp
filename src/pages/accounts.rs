use chrono::Local;
use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::core::account::Account;
use crate::fl;
use crate::message::{AccountField, Message};
use crate::ui::{self, Sender};

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
    sender: &Sender,
) -> gtk::Box {
    let col = ui::vbox(0);

    // Summary row
    let summary_row = ui::centered_hbox(8);

    // Clickable name
    let name_btn = ui::flat_button(&account.name);
    {
        let s = sender.clone();
        name_btn.connect_clicked(move |_| {
            s.emit(Message::ToggleAccountExpand(index));
        });
    }
    summary_row.append(&name_btn);

    // URL badge
    if !account.url.is_empty() {
        let url_btn = ui::flat_button(&account.url);
        {
            let s = sender.clone();
            url_btn.connect_clicked(move |_| {
                s.emit(Message::OpenAccountUrl(index));
            });
        }
        summary_row.append(&url_btn);
    }

    // Last checked
    let last_lbl = ui::caption(&last_checked_text(account));
    last_lbl.set_hexpand(true);
    summary_row.append(&last_lbl);

    // Delete
    if confirming_delete {
        summary_row.append(&ui::button_with_signal(&fl!("btn-delete"), Some("destructive-action"), Message::DeleteAccount(index), sender));
        summary_row.append(&ui::button_with_signal(&fl!("btn-cancel"), None, Message::CancelDeleteAccount, sender));
    } else {
        summary_row.append(&ui::icon_button_with_signal("edit-delete-symbolic", Message::ConfirmDeleteAccount(index), sender));
    }

    col.append(&summary_row);

    if expanded {
        let detail = ui::vbox(6);
        detail.set_margin_start(24);
        detail.set_margin_top(4);
        detail.set_margin_bottom(8);

        // Name
        let name_row = ui::centered_hbox(8);
        let name_label = ui::caption(&fl!("accounts-name"));
        name_label.set_width_request(80);
        name_row.append(&name_label);
        let name_entry = ui::entry(&fl!("accounts-name-placeholder"), &account.name);
        {
            let s = sender.clone();
            name_entry.connect_changed(move |e| {
                s.emit(Message::SetAccountFieldValue(index, AccountField::Name, e.text().to_string()));
            });
        }
        {
            let s = sender.clone();
            name_entry.connect_activate(move |_| {
                s.emit(Message::ToggleAccountExpand(index));
            });
        }
        name_row.append(&name_entry);
        detail.append(&name_row);

        // URL
        let url_row = ui::centered_hbox(8);
        let url_label = ui::caption(&fl!("accounts-url"));
        url_label.set_width_request(80);
        url_row.append(&url_label);
        let url_entry = ui::entry(&fl!("contacts-url-placeholder"), &account.url);
        {
            let s = sender.clone();
            url_entry.connect_changed(move |e| {
                s.emit(Message::SetAccountFieldValue(index, AccountField::Url, e.text().to_string()));
            });
        }
        {
            let s = sender.clone();
            url_entry.connect_activate(move |_| {
                s.emit(Message::ToggleAccountExpand(index));
            });
        }
        url_row.append(&url_entry);
        url_row.append(&ui::icon_button_with_signal("adw-external-link-symbolic", Message::OpenAccountUrl(index), sender));
        detail.append(&url_row);

        // Notes
        let notes_row = ui::centered_hbox(8);
        let notes_label = ui::caption(&fl!("accounts-notes"));
        notes_label.set_width_request(80);
        notes_row.append(&notes_label);
        let notes_entry = ui::entry(&fl!("accounts-notes-placeholder"), &account.notes);
        {
            let s = sender.clone();
            notes_entry.connect_changed(move |e| {
                s.emit(Message::SetAccountFieldValue(index, AccountField::Notes, e.text().to_string()));
            });
        }
        {
            let s = sender.clone();
            notes_entry.connect_activate(move |_| {
                s.emit(Message::ToggleAccountExpand(index));
            });
        }
        notes_row.append(&notes_entry);
        detail.append(&notes_row);

        // Mark checked button
        detail.append(&ui::button_with_signal(&fl!("accounts-mark-checked"), None, Message::MarkAccountChecked(index), sender));

        col.append(&detail);
    }

    col
}

pub fn accounts_view(
    accounts: &[Account],
    account_input: &str,
    expanded: Option<usize>,
    pending_delete: Option<usize>,
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(12);

    // Add account input row
    let input_row = ui::centered_hbox(8);
    let entry = ui::entry(&fl!("accounts-placeholder"), account_input);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::AccountInputChanged(e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::AccountSubmit);
        });
    }
    input_row.append(&entry);
    input_row.append(&ui::icon_button_with_signal("list-add-symbolic", Message::AccountSubmit, sender));
    content.append(&input_row);

    if accounts.is_empty() {
        let empty_label = ui::body(&fl!("accounts-empty"));
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_margin_top(32);
        empty_label.set_margin_bottom(32);
        empty_label.set_hexpand(true);
        content.append(&empty_label);
    } else {
        for (idx, account) in accounts.iter().enumerate() {
            content.append(&account_row(
                account,
                idx,
                expanded == Some(idx),
                pending_delete == Some(idx),
                sender,
            ));
        }
    }

    ui::page_wrapper(&content).upcast()
}
