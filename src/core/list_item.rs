use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ListItem {
    pub id: Uuid,
    pub title: String,
    pub notes: String,
    pub created: NaiveDateTime,
    pub done: bool,
}

impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            notes: String::new(),
            created: chrono::Local::now().naive_local(),
            done: false,
        }
    }
}
