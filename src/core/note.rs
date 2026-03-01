use chrono::NaiveDateTime;
use uuid::Uuid;

use super::link::LinkTarget;

#[derive(Debug, Clone)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub links: Vec<LinkTarget>,
    pub source: Option<String>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub sync_etag: Option<String>,
}

impl Note {
    pub fn new(title: impl Into<String>) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            body: String::new(),
            tags: Vec::new(),
            links: Vec::new(),
            source: None,
            created: now,
            modified: now,
            sync_etag: None,
        }
    }
}
