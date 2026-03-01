#![allow(dead_code)]

pub mod config;
pub mod core;
pub mod org;
pub mod sync;

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether debug logging is active, shared between the logger filter and the settings toggle.
static DEBUG_LOGGING: AtomicBool = AtomicBool::new(false);

pub fn set_debug_logging(enabled: bool) {
    DEBUG_LOGGING.store(enabled, Ordering::Relaxed);
}

pub fn debug_logging() -> bool {
    DEBUG_LOGGING.load(Ordering::Relaxed)
}
