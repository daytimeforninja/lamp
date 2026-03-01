#![allow(dead_code)]

use cosmic::app::Settings;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::Limits;

mod application;
mod components;
mod localize;
mod message;
mod pages;

use lamp::config;
use lamp::core;
use lamp::org;
use lamp::sync;

use application::{Flags, Lamp, LaunchMode};
use config::{LampConfig, CONFIG_VERSION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cosmic_cfg = cosmic::cosmic_config::Config::new("dev.lamp.app", CONFIG_VERSION)
        .expect("Failed to create cosmic config");
    let config = LampConfig::get_entry(&cosmic_cfg).unwrap_or_else(|(_, cfg)| cfg);

    // Set up logging to the systemd user journal (`journalctl --user -t lamp -f`).
    // Wrapper filters: lamp crate at info/debug (per config), everything else at warn.
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

        let journal = systemd_journal_logger::JournalLog::new()
            .unwrap()
            .with_syslog_identifier("lamp".to_string());

        lamp::set_debug_logging(config.debug_logging);

        log::set_boxed_logger(Box::new(FilteredJournal { inner: journal })).unwrap();
        // Global max must be Debug so lamp debug logs can pass through when toggled
        log::set_max_level(log::LevelFilter::Debug);
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

    let mut settings = Settings::default();
    settings = settings.size_limits(Limits::NONE.min_width(400.0).min_height(300.0));

    let flags = Flags { config, cosmic_config: cosmic_cfg, launch_mode };
    cosmic::app::run::<Lamp>(settings, flags)?;

    Ok(())
}
