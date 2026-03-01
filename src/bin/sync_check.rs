use std::collections::{HashMap, HashSet};

#[tokio::main]
async fn main() {
    systemd_journal_logger::JournalLog::new()
        .unwrap()
        .with_syslog_identifier("lamp-sync-check".to_string())
        .install()
        .unwrap();
    log::set_max_level(log::LevelFilter::Info);

    // Load config
    let cosmic_cfg = cosmic::cosmic_config::Config::new("dev.lamp.app", lamp::config::CONFIG_VERSION)
        .expect("Failed to load config");
    let config = <lamp::config::LampConfig as cosmic::cosmic_config::CosmicConfigEntry>::get_entry(&cosmic_cfg)
        .unwrap_or_else(|(_, cfg)| cfg);

    println!("=== CalDAV vs Local Comparison ===\n");

    // Load local tasks from org files
    let read = |path: &std::path::Path| -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    };

    let mut local_tasks: Vec<lamp::core::task::Task> = Vec::new();
    local_tasks.extend(lamp::org::convert::parse_tasks(&read(&config.inbox_path())));
    local_tasks.extend(lamp::org::convert::parse_tasks(&read(&config.next_path())));
    local_tasks.extend(lamp::org::convert::parse_tasks(&read(&config.waiting_path())));
    local_tasks.extend(lamp::org::convert::parse_tasks(&read(&config.someday_path())));

    let projects = lamp::org::convert::parse_projects(&read(&config.projects_path()));
    for p in &projects {
        local_tasks.extend(p.tasks.clone());
    }

    let local_events = lamp::core::event::load_events(&config.events_cache_path());

    println!("Local: {} tasks, {} events\n", local_tasks.len(), local_events.len());

    // Build local lookup by sync_href
    let mut local_by_href: HashMap<String, &lamp::core::task::Task> = HashMap::new();
    for task in &local_tasks {
        if let Some(ref href) = task.sync_href {
            local_by_href.insert(href.clone(), task);
        }
    }

    // Connect to CalDAV using flat config
    let caldav_url = config.calendars.url.trim();
    if caldav_url.is_empty() {
        println!("No CalDAV URL configured.");
        return;
    }

    println!("--- CalDAV: {} ---", caldav_url);

    let creds = lamp::sync::keyring::load_credentials(caldav_url).await;
    let password = match creds {
        Ok(Some((_, pw))) => pw,
        Ok(None) => { println!("  No credentials found"); return; }
        Err(e) => { println!("  Keyring error: {}", e); return; }
    };

    let client = match lamp::sync::caldav::CalDavClient::new(caldav_url, &config.calendars.username, &password) {
        Ok(c) => c,
        Err(e) => { println!("  Client error: {}", e); return; }
    };

    // Check each calendar assignment
    for assignment in &config.calendar_assignments {
        match assignment.purpose {
            lamp::config::CalendarPurpose::Tasks => {
                println!("\n  Tasks calendar: {}", assignment.calendar_href);
                match client.list_vtodos(&assignment.calendar_href).await {
                    Ok(remote_vtodos) => {
                        println!("  Remote: {} VTODOs", remote_vtodos.len());

                        let mut remote_only = Vec::new();
                        let mut local_only = Vec::new();
                        let mut status_mismatch = Vec::new();
                        let mut matched = 0;

                        for rv in &remote_vtodos {
                            let remote_task = lamp::sync::vtodo::vcalendar_to_task(&rv.ical_body);
                            let remote_task = match remote_task {
                                Some(t) => t,
                                None => continue,
                            };

                            if let Some(&local) = local_by_href.get(&rv.href) {
                                matched += 1;
                                let local_state = local.state.as_keyword();
                                let remote_state = remote_task.state.as_keyword();
                                if local_state != remote_state {
                                    status_mismatch.push((
                                        remote_task.title.clone(),
                                        local_state.to_string(),
                                        remote_state.to_string(),
                                    ));
                                }
                            } else {
                                remote_only.push((remote_task.title.clone(), remote_task.state.as_keyword().to_string(), rv.href.clone()));
                            }
                        }

                        let remote_hrefs: HashSet<String> =
                            remote_vtodos.iter().map(|r| r.href.clone()).collect();
                        for task in &local_tasks {
                            if let Some(ref href) = task.sync_href {
                                if href.contains(&assignment.calendar_href) && !remote_hrefs.contains(href) {
                                    local_only.push((task.title.clone(), task.state.as_keyword().to_string()));
                                }
                            }
                        }

                        println!("  Matched: {}", matched);

                        if !status_mismatch.is_empty() {
                            println!("\n  STATUS MISMATCHES:");
                            for (title, local_s, remote_s) in &status_mismatch {
                                println!("    {} — local: {}, remote: {}", title, local_s, remote_s);
                            }
                        }

                        if !remote_only.is_empty() {
                            println!("\n  ON SERVER ONLY ({}):", remote_only.len());
                            for (title, state, href) in &remote_only {
                                println!("    [{}] {} ({})", state, title, href);
                            }
                        }

                        if !local_only.is_empty() {
                            println!("\n  LOCAL ONLY ({}):", local_only.len());
                            for (title, state) in &local_only {
                                println!("    [{}] {}", state, title);
                            }
                        }

                        if status_mismatch.is_empty() && remote_only.is_empty() && local_only.is_empty() {
                            println!("  All in sync!");
                        }
                    }
                    Err(e) => println!("  Error listing VTODOs: {}", e),
                }
            }
            lamp::config::CalendarPurpose::Events => {
                println!("\n  Events calendar: {}", assignment.calendar_href);
                match client.list_vevents(&assignment.calendar_href).await {
                    Ok(remote_events) => {
                        println!("  Remote: {} VEVENTs", remote_events.len());
                        println!("  Local (cached): {} events total", local_events.iter().filter(|e| e.calendar_href == assignment.calendar_href).count());
                    }
                    Err(e) => println!("  Error listing VEVENTs: {}", e),
                }
            }
            lamp::config::CalendarPurpose::Disabled => {}
        }
    }

    println!("\n=== Done ===");
}
