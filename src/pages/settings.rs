use relm4::adw;
use relm4::adw::prelude::*;
use relm4::gtk;

use crate::config::{CalendarPurpose, LampConfig};
use crate::fl;
use crate::message::{Message, ServiceKind};
use crate::sync::caldav::CalendarInfo;
use crate::sync::SyncStatus;
use crate::ui::Sender;

pub fn settings_view(
    config: &LampConfig,
    settings_context_input: &str,
    service_passwords: &[String; 4],
    service_test_status: &[Option<Result<String, String>>; 4],
    discovered_calendars: &[CalendarInfo],
    sync_status: &SyncStatus,
    sender: &Sender,
) -> gtk::Widget {
    let page = adw::PreferencesPage::new();

    // --- Contexts ---
    {
        let group = adw::PreferencesGroup::new();
        group.set_title(&fl!("settings-contexts"));

        for (idx, ctx) in config.contexts.iter().enumerate() {
            let row = adw::ActionRow::new();
            row.set_title(ctx);

            let delete_btn = gtk::Button::from_icon_name("edit-delete-symbolic");
            delete_btn.set_valign(gtk::Align::Center);
            delete_btn.add_css_class("flat");
            {
                let s = sender.clone();
                delete_btn.connect_clicked(move |_| {
                    s.emit(Message::SettingsRemoveContext(idx));
                });
            }
            row.add_suffix(&delete_btn);
            group.add(&row);
        }

        // Context add input
        let add_row = adw::EntryRow::new();
        add_row.set_title(&fl!("settings-context-placeholder"));
        add_row.set_text(settings_context_input);
        {
            let s = sender.clone();
            add_row.connect_changed(move |r| {
                s.emit(Message::SettingsContextInput(r.text().to_string()));
            });
        }
        {
            let s = sender.clone();
            add_row.connect_activate(move |_| {
                s.emit(Message::SettingsAddContext);
            });
        }
        let add_btn = gtk::Button::from_icon_name("list-add-symbolic");
        add_btn.set_valign(gtk::Align::Center);
        add_btn.add_css_class("flat");
        {
            let s = sender.clone();
            add_btn.connect_clicked(move |_| {
                s.emit(Message::SettingsAddContext);
            });
        }
        add_row.add_suffix(&add_btn);
        group.add(&add_row);

        page.add(&group);
    }

    // --- Browser command ---
    {
        let group = adw::PreferencesGroup::new();
        group.set_title(&fl!("settings-browser"));

        let row = adw::EntryRow::new();
        row.set_title("xdg-open");
        row.set_text(&config.browser_command);
        {
            let s = sender.clone();
            row.connect_changed(move |r| {
                s.emit(Message::SetBrowserCommand(r.text().to_string()));
            });
        }
        group.add(&row);

        // Debug logging toggle
        let switch_row = adw::SwitchRow::new();
        switch_row.set_title(&fl!("settings-debug-logging"));
        switch_row.set_active(config.debug_logging);
        {
            let s = sender.clone();
            switch_row.connect_active_notify(move |_r| {
                s.emit(Message::ToggleDebugLogging);
            });
        }
        group.add(&switch_row);

        page.add(&group);
    }

    // --- Calendars (CalDAV) ---
    {
        let group = adw::PreferencesGroup::new();
        group.set_title(&fl!("sync-calendars"));

        append_service_rows(
            &group,
            ServiceKind::Calendars,
            &config.calendars.url,
            &config.calendars.username,
            &service_passwords[0],
            &service_test_status[0],
            sender,
        );

        // Calendar purpose dropdowns
        for cal in discovered_calendars {
            let current_purpose = config
                .calendar_assignments
                .iter()
                .find(|a| a.calendar_href == cal.href)
                .map(|a| a.purpose.clone())
                .unwrap_or(CalendarPurpose::Disabled);

            let purpose_labels = [
                fl!("sync-calendar-purpose-disabled"),
                fl!("sync-calendar-purpose-tasks"),
                fl!("sync-calendar-purpose-events"),
            ];
            let selected: u32 = match current_purpose {
                CalendarPurpose::Disabled => 0,
                CalendarPurpose::Tasks => 1,
                CalendarPurpose::Events => 2,
            };

            let row = adw::ActionRow::new();
            row.set_title(&cal.display_name);

            let items: Vec<&str> = purpose_labels.iter().map(|s| s.as_str()).collect();
            let model = gtk::StringList::new(&items);
            let dd = gtk::DropDown::new(Some(model), gtk::Expression::NONE);
            dd.set_selected(selected);
            dd.set_valign(gtk::Align::Center);
            {
                let s = sender.clone();
                let cal_href = cal.href.clone();
                dd.connect_selected_notify(move |dd| {
                    let purpose = match dd.selected() {
                        1 => CalendarPurpose::Tasks,
                        2 => CalendarPurpose::Events,
                        _ => CalendarPurpose::Disabled,
                    };
                    s.emit(Message::SetCalendarPurpose(cal_href.clone(), purpose));
                });
            }
            row.add_suffix(&dd);
            group.add(&row);
        }

        page.add(&group);
    }

    // --- Contacts (CardDAV) ---
    {
        let group = adw::PreferencesGroup::new();
        group.set_title(&fl!("sync-contacts-service"));

        append_service_rows(
            &group,
            ServiceKind::Contacts,
            &config.contacts.url,
            &config.contacts.username,
            &service_passwords[1],
            &service_test_status[1],
            sender,
        );

        page.add(&group);
    }

    // --- Notes (WebDAV) ---
    {
        let group = adw::PreferencesGroup::new();
        group.set_title(&fl!("sync-notes-webdav"));

        append_service_rows(
            &group,
            ServiceKind::Notes,
            &config.notes_sync.url,
            &config.notes_sync.username,
            &service_passwords[2],
            &service_test_status[2],
            sender,
        );

        page.add(&group);
    }

    // --- Email Inbox (IMAP) ---
    {
        let group = adw::PreferencesGroup::new();
        group.set_title(&fl!("sync-imap"));

        // Host
        let host_row = adw::EntryRow::new();
        host_row.set_title(&fl!("sync-imap-host"));
        host_row.set_text(&config.imap.host);
        {
            let s = sender.clone();
            host_row.connect_changed(move |r| {
                s.emit(Message::SetServiceUrl(ServiceKind::Imap, r.text().to_string()));
            });
        }
        group.add(&host_row);

        // Username
        let user_row = adw::EntryRow::new();
        user_row.set_title(&fl!("sync-service-username"));
        user_row.set_text(&config.imap.username);
        {
            let s = sender.clone();
            user_row.connect_changed(move |r| {
                s.emit(Message::SetServiceUsername(ServiceKind::Imap, r.text().to_string()));
            });
        }
        group.add(&user_row);

        // Password
        let pw_row = adw::PasswordEntryRow::new();
        pw_row.set_title(&fl!("sync-service-password"));
        pw_row.set_text(&service_passwords[3]);
        {
            let s = sender.clone();
            pw_row.connect_changed(move |r| {
                s.emit(Message::SetServicePassword(ServiceKind::Imap, r.text().to_string()));
            });
        }
        group.add(&pw_row);

        // Folder
        let folder_row = adw::EntryRow::new();
        folder_row.set_title(&fl!("sync-imap-folder"));
        folder_row.set_text(&config.imap.folder);
        {
            let s = sender.clone();
            folder_row.connect_changed(move |r| {
                s.emit(Message::SetImapFolder(r.text().to_string()));
            });
        }
        group.add(&folder_row);

        // Test connection
        append_test_row(&group, ServiceKind::Imap, &service_test_status[3], sender);

        page.add(&group);
    }

    // --- Sync status ---
    {
        let group = adw::PreferencesGroup::new();
        group.set_title(&fl!("sync-title"));

        let status_text = match sync_status {
            SyncStatus::Idle => fl!("sync-status-never"),
            SyncStatus::Syncing => fl!("sync-status-syncing"),
            SyncStatus::Error(e) => fl!("sync-status-error", error = e.as_str()),
            SyncStatus::LastSynced(t) => fl!("sync-status-idle", time = t.as_str()),
        };

        let row = adw::ActionRow::new();
        row.set_title(&status_text);

        group.add(&row);
        page.add(&group);
    }

    page.upcast()
}

fn append_service_rows(
    group: &adw::PreferencesGroup,
    kind: ServiceKind,
    url: &str,
    username: &str,
    password: &str,
    test_status: &Option<Result<String, String>>,
    sender: &Sender,
) {
    // URL
    let url_row = adw::EntryRow::new();
    url_row.set_title(&fl!("sync-service-url"));
    url_row.set_text(url);
    {
        let s = sender.clone();
        url_row.connect_changed(move |r| {
            s.emit(Message::SetServiceUrl(kind, r.text().to_string()));
        });
    }
    group.add(&url_row);

    // Username
    let user_row = adw::EntryRow::new();
    user_row.set_title(&fl!("sync-service-username"));
    user_row.set_text(username);
    {
        let s = sender.clone();
        user_row.connect_changed(move |r| {
            s.emit(Message::SetServiceUsername(kind, r.text().to_string()));
        });
    }
    group.add(&user_row);

    // Password
    let pw_row = adw::PasswordEntryRow::new();
    pw_row.set_title(&fl!("sync-service-password"));
    pw_row.set_text(password);
    {
        let s = sender.clone();
        pw_row.connect_changed(move |r| {
            s.emit(Message::SetServicePassword(kind, r.text().to_string()));
        });
    }
    group.add(&pw_row);

    // Test connection
    append_test_row(group, kind, test_status, sender);
}

fn append_test_row(
    group: &adw::PreferencesGroup,
    kind: ServiceKind,
    test_status: &Option<Result<String, String>>,
    sender: &Sender,
) {
    let row = adw::ActionRow::new();
    row.set_title(&fl!("sync-test-connection"));

    if let Some(result) = test_status {
        match result {
            Ok(msg) => row.set_subtitle(&format!("OK: {}", msg)),
            Err(e) => row.set_subtitle(&format!("Error: {}", e)),
        }
    }

    let btn = gtk::Button::from_icon_name("network-transmit-receive-symbolic");
    btn.set_valign(gtk::Align::Center);
    btn.add_css_class("flat");
    {
        let s = sender.clone();
        btn.connect_clicked(move |_| {
            s.emit(Message::TestServiceConnection(kind));
        });
    }
    row.add_suffix(&btn);
    row.set_activatable_widget(Some(&btn));
    group.add(&row);
}
