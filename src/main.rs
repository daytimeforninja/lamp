#![allow(dead_code)]

use cosmic::app::Settings;
use cosmic::iced::Limits;

mod application;
mod components;
mod config;
mod core;
mod localize;
mod message;
mod org;
mod pages;
mod sync;

use application::{Flags, Lamp};
use config::LampConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();

    let config = LampConfig::default();

    let mut settings = Settings::default();
    settings = settings.size_limits(Limits::NONE.min_width(400.0).min_height(300.0));

    let flags = Flags { config };
    cosmic::app::run::<Lamp>(settings, flags)?;

    Ok(())
}
