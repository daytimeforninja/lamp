use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub title: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub all_day: bool,
    pub location: String,
    pub description: String,
    pub status: EventStatus,
    pub calendar_href: String,
    pub calendar_name: String,
    // Sync metadata
    pub sync_href: Option<String>,
    pub sync_hash: Option<u64>,
}

impl CalendarEvent {
    pub fn new(title: String, start: NaiveDateTime, end: NaiveDateTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            start,
            end,
            all_day: false,
            location: String::new(),
            description: String::new(),
            status: EventStatus::Confirmed,
            calendar_href: String::new(),
            calendar_name: String::new(),
            sync_href: None,
            sync_hash: None,
        }
    }
}

pub fn load_events(path: &Path) -> Vec<CalendarEvent> {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_events(path: &Path, events: &[CalendarEvent]) {
    match serde_json::to_string_pretty(events) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                log::error!("Failed to save events: {}", e);
            }
        }
        Err(e) => log::error!("Failed to serialize events: {}", e),
    }
}
