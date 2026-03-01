use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, icon, row, scrollable, text, text_input};
use cosmic::Element;

use crate::config::{CalendarPurpose, LampConfig};
use crate::message::{Message, ServiceKind};
use crate::sync::caldav::CalendarInfo;
use crate::sync::SyncStatus;

pub fn settings_view<'a>(
    config: &'a LampConfig,
    settings_context_input: &'a str,
    service_passwords: &[String; 4],
    service_test_status: &[Option<Result<String, String>>; 4],
    discovered_calendars: &[CalendarInfo],
    anthropic_api_key_input: &str,
    anthropic_test_status: &Option<Result<String, String>>,
    sync_status: &SyncStatus,
) -> Element<'a, Message> {
    let mut content = column().spacing(12);

    // --- Contexts ---
    content = content.push(text::title4("Contexts"));

    for (idx, ctx) in config.contexts.iter().enumerate() {
        content = content.push(
            row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::body(ctx.clone()).width(Length::Fill))
                .push(
                    button::icon(icon::from_name("edit-delete-symbolic"))
                        .on_press(Message::SettingsRemoveContext(idx)),
                ),
        );
    }

    let input = text_input::text_input("New context (e.g. gym)", settings_context_input)
        .on_input(Message::SettingsContextInput)
        .on_submit(|_| Message::SettingsAddContext)
        .width(Length::Fill);

    content = content.push(
        row()
            .spacing(8)
            .push(input)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::SettingsAddContext),
            ),
    );

    // --- Debug logging ---
    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(text::body(crate::fl!("settings-debug-logging")).width(Length::Fill))
            .push(
                cosmic::widget::toggler(config.debug_logging)
                    .on_toggle(|_| Message::ToggleDebugLogging),
            ),
    );

    // --- Browser ---
    content = content.push(text::title4(crate::fl!("settings-browser")));
    content = content.push(
        text_input::text_input("xdg-open", &config.browser_command)
            .on_input(Message::SetBrowserCommand)
            .width(Length::Fill),
    );

    // --- Calendars (CalDAV) ---
    content = content.push(text::title4(crate::fl!("sync-calendars")));
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-service-url"),
            &config.calendars.url,
        )
        .on_input(|v| Message::SetServiceUrl(ServiceKind::Calendars, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-service-username"),
            &config.calendars.username,
        )
        .on_input(|v| Message::SetServiceUsername(ServiceKind::Calendars, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::secure_input(
            crate::fl!("sync-service-password"),
            service_passwords[0].clone(),
            None::<Message>,
            true,
        )
        .on_input(|v| Message::SetServicePassword(ServiceKind::Calendars, v))
        .width(Length::Fill),
    );
    {
        let mut test_row = row().spacing(8).align_y(Alignment::Center);
        test_row = test_row.push(
            button::standard(crate::fl!("sync-test-connection"))
                .on_press(Message::TestServiceConnection(ServiceKind::Calendars)),
        );
        if let Some(ref result) = service_test_status[0] {
            match result {
                Ok(msg) => test_row = test_row.push(text::body(format!("✓ {}", msg))),
                Err(e) => test_row = test_row.push(text::body(format!("✗ {}", e))),
            }
        }
        content = content.push(test_row);
    }

    // Show discovered calendars with purpose dropdowns
    for cal in discovered_calendars {
        let current_purpose = config
            .calendar_assignments
            .iter()
            .find(|a| a.calendar_href == cal.href)
            .map(|a| a.purpose.clone())
            .unwrap_or(CalendarPurpose::Disabled);

        let purpose_names = vec![
            crate::fl!("sync-calendar-purpose-disabled"),
            crate::fl!("sync-calendar-purpose-tasks"),
            crate::fl!("sync-calendar-purpose-events"),
        ];
        let selected_purpose = match current_purpose {
            CalendarPurpose::Disabled => Some(0),
            CalendarPurpose::Tasks => Some(1),
            CalendarPurpose::Events => Some(2),
        };
        let cal_href = cal.href.clone();
        content = content.push(
            row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    text::body(cal.display_name.clone())
                        .width(Length::Fill),
                )
                .push(
                    cosmic::widget::dropdown(
                        purpose_names,
                        selected_purpose,
                        move |sel| {
                            let purpose = match sel {
                                1 => CalendarPurpose::Tasks,
                                2 => CalendarPurpose::Events,
                                _ => CalendarPurpose::Disabled,
                            };
                            Message::SetCalendarPurpose(
                                cal_href.clone(),
                                purpose,
                            )
                        },
                    )
                    .width(Length::Fixed(120.0)),
                ),
        );
    }

    // --- Contacts (CardDAV) ---
    content = content.push(text::title4(crate::fl!("sync-contacts-service")));
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-service-url"),
            &config.contacts.url,
        )
        .on_input(|v| Message::SetServiceUrl(ServiceKind::Contacts, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-service-username"),
            &config.contacts.username,
        )
        .on_input(|v| Message::SetServiceUsername(ServiceKind::Contacts, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::secure_input(
            crate::fl!("sync-service-password"),
            service_passwords[1].clone(),
            None::<Message>,
            true,
        )
        .on_input(|v| Message::SetServicePassword(ServiceKind::Contacts, v))
        .width(Length::Fill),
    );
    {
        let mut test_row = row().spacing(8).align_y(Alignment::Center);
        test_row = test_row.push(
            button::standard(crate::fl!("sync-test-connection"))
                .on_press(Message::TestServiceConnection(ServiceKind::Contacts)),
        );
        if let Some(ref result) = service_test_status[1] {
            match result {
                Ok(msg) => test_row = test_row.push(text::body(format!("✓ {}", msg))),
                Err(e) => test_row = test_row.push(text::body(format!("✗ {}", e))),
            }
        }
        content = content.push(test_row);
    }

    // --- Notes (WebDAV) ---
    content = content.push(text::title4(crate::fl!("sync-notes-webdav")));
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-service-url"),
            &config.notes_sync.url,
        )
        .on_input(|v| Message::SetServiceUrl(ServiceKind::Notes, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-service-username"),
            &config.notes_sync.username,
        )
        .on_input(|v| Message::SetServiceUsername(ServiceKind::Notes, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::secure_input(
            crate::fl!("sync-service-password"),
            service_passwords[2].clone(),
            None::<Message>,
            true,
        )
        .on_input(|v| Message::SetServicePassword(ServiceKind::Notes, v))
        .width(Length::Fill),
    );
    {
        let mut test_row = row().spacing(8).align_y(Alignment::Center);
        test_row = test_row.push(
            button::standard(crate::fl!("sync-test-connection"))
                .on_press(Message::TestServiceConnection(ServiceKind::Notes)),
        );
        if let Some(ref result) = service_test_status[2] {
            match result {
                Ok(msg) => test_row = test_row.push(text::body(format!("✓ {}", msg))),
                Err(e) => test_row = test_row.push(text::body(format!("✗ {}", e))),
            }
        }
        content = content.push(test_row);
    }

    // --- Email Inbox (IMAP) ---
    content = content.push(text::title4(crate::fl!("sync-imap")));
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-imap-host"),
            &config.imap.host,
        )
        .on_input(|v| Message::SetServiceUrl(ServiceKind::Imap, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-service-username"),
            &config.imap.username,
        )
        .on_input(|v| Message::SetServiceUsername(ServiceKind::Imap, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::secure_input(
            crate::fl!("sync-service-password"),
            service_passwords[3].clone(),
            None::<Message>,
            true,
        )
        .on_input(|v| Message::SetServicePassword(ServiceKind::Imap, v))
        .width(Length::Fill),
    );
    content = content.push(
        text_input::text_input(
            crate::fl!("sync-imap-folder"),
            if config.imap.folder.is_empty() { "flup" } else { &config.imap.folder },
        )
        .on_input(|v| Message::SetImapFolder(v))
        .width(Length::Fill),
    );
    {
        let mut test_row = row().spacing(8).align_y(Alignment::Center);
        test_row = test_row.push(
            button::standard(crate::fl!("sync-test-connection"))
                .on_press(Message::TestServiceConnection(ServiceKind::Imap)),
        );
        if let Some(ref result) = service_test_status[3] {
            match result {
                Ok(msg) => test_row = test_row.push(text::body(format!("✓ {}", msg))),
                Err(e) => test_row = test_row.push(text::body(format!("✗ {}", e))),
            }
        }
        content = content.push(test_row);
    }

    // --- AI Task Extraction ---
    content = content.push(text::title4(crate::fl!("settings-ai")));
    content = content.push(
        text_input::secure_input(
            crate::fl!("settings-ai-api-key"),
            anthropic_api_key_input.to_string(),
            None::<Message>,
            true,
        )
        .on_input(Message::SetAnthropicApiKey)
        .width(Length::Fill),
    );
    {
        let mut test_row = row().spacing(8).align_y(Alignment::Center);
        test_row = test_row.push(
            button::standard(crate::fl!("sync-test-connection"))
                .on_press(Message::TestAnthropicApiKey),
        );
        if let Some(result) = anthropic_test_status {
            match result {
                Ok(msg) => test_row = test_row.push(text::body(format!("✓ {}", msg))),
                Err(e) => test_row = test_row.push(text::body(format!("✗ {}", e))),
            }
        }
        content = content.push(test_row);
    }

    // Sync status
    let status_text = match sync_status {
        SyncStatus::Idle => crate::fl!("sync-status-never"),
        SyncStatus::Syncing => crate::fl!("sync-status-syncing"),
        SyncStatus::Error(e) => crate::fl!("sync-status-error", error = e.as_str()),
        SyncStatus::LastSynced(t) => crate::fl!("sync-status-idle", time = t.as_str()),
    };
    content = content.push(text::caption(status_text));

    container(scrollable(content.padding(16)))
        .width(Length::Fill)
        .into()
}
