use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub notes: String,
    pub last_checked: Option<NaiveDate>,
}

impl Account {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            url: String::new(),
            notes: String::new(),
            last_checked: None,
        }
    }
}
