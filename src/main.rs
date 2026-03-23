mod application;
mod components;
mod localize;
mod message;
mod pages;
mod ui;

use lamp::config;
use lamp::core;
use lamp::org;
use lamp::sync;

use application::{Lamp, LaunchMode};
use config::LampConfig;
use relm4::prelude::*;

fn main() {
    let config = LampConfig::load();

    // Set up logging to the systemd user journal (`journalctl --user -t lamp -f`).
    {
        struct FilteredJournal {
            inner: systemd_journal_logger::JournalLog,
        }

        impl log::Log for FilteredJournal {
            fn enabled(&self, metadata: &log::Metadata) -> bool {
                let target = metadata.target();
                if target.starts_with("lamp") || target.starts_with("application") || target.starts_with("pages") || target.starts_with("components") {
                    let max = if lamp::debug_logging() { log::LevelFilter::Debug } else { log::LevelFilter::Info };
                    metadata.level() <= max
                } else {
                    metadata.level() <= log::LevelFilter::Warn
                }
            }
            fn log(&self, record: &log::Record) {
                if self.enabled(record.metadata()) {
                    self.inner.log(record);
                }
            }
            fn flush(&self) {
                self.inner.flush();
            }
        }

        lamp::set_debug_logging(config.debug_logging);

        match systemd_journal_logger::JournalLog::new() {
            Ok(journal) => {
                let journal = journal.with_syslog_identifier("lamp".to_string());
                if log::set_boxed_logger(Box::new(FilteredJournal { inner: journal })).is_ok() {
                    log::set_max_level(log::LevelFilter::Debug);
                }
            }
            Err(e) => {
                eprintln!("lamp: failed to initialize journal logger: {e}");
                log::set_max_level(log::LevelFilter::Off);
            }
        }
    }

    localize::localize();

    // Parse CLI flags
    let launch_mode = {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--capture") {
            LaunchMode::Capture
        } else if args.iter().any(|a| a == "--today") {
            LaunchMode::Today
        } else {
            LaunchMode::Normal
        }
    };

    let app = RelmApp::new("dev.lamp.app");
    app.run::<Lamp>((config, launch_mode));
}
