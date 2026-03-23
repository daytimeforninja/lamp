use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 2;

fn default_org_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".lamp")))
        .expect("Cannot determine data directory: no XDG_DATA_HOME, HOME, or home directory available")
        .join("lamp")
}

fn default_browser_command() -> String {
    "xdg-open".to_string()
}

fn default_contexts() -> Vec<String> {
    vec![
        "@home".into(),
        "@work".into(),
        "@errands".into(),
        "@computer".into(),
        "@phone".into(),
        "@anywhere".into(),
    ]
}

/// What a discovered calendar is used for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarPurpose {
    Tasks,
    Events,
    Disabled,
}

/// Per-service sync configuration (URL + username; password lives in keyring).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
}

/// IMAP inbox configuration (host + username + folder; password lives in keyring).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub username: String,
    #[serde(default, rename = "imap_folder")]
    pub folder: String,
}

/// Maps a discovered calendar to a purpose (Tasks, Events, or Disabled).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarAssignment {
    pub calendar_href: String,
    pub purpose: CalendarPurpose,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LampConfig {
    pub org_directory: PathBuf,
    pub contexts: Vec<String>,
    #[serde(default)]
    pub calendars: ServiceConfig,
    #[serde(default)]
    pub contacts: ServiceConfig,
    #[serde(default)]
    pub notes_sync: ServiceConfig,
    #[serde(default)]
    pub imap: ImapConfig,
    pub calendar_assignments: Vec<CalendarAssignment>,
    /// Sync tokens: calendar_href → token
    #[serde(default)]
    pub sync_tokens: HashMap<String, String>,
    #[serde(default = "default_browser_command")]
    pub browser_command: String,
    #[serde(default)]
    pub debug_logging: bool,
}

impl Default for LampConfig {
    fn default() -> Self {
        Self {
            org_directory: default_org_dir(),
            contexts: default_contexts(),
            calendars: ServiceConfig::default(),
            contacts: ServiceConfig::default(),
            notes_sync: ServiceConfig::default(),
            imap: ImapConfig::default(),
            calendar_assignments: Vec::new(),
            sync_tokens: HashMap::new(),
            browser_command: default_browser_command(),
            debug_logging: false,
        }
    }
}

impl LampConfig {
    /// Load config from JSON file, migrating from COSMIC config if needed.
    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => {
                // Try migrating from COSMIC config
                let mut config = Self::migrate_from_cosmic().unwrap_or_default();
                config.save();
                config
            }
        }
    }

    /// Migrate config from COSMIC desktop config system (RON format per-field files).
    fn migrate_from_cosmic() -> Option<Self> {
        let cosmic_dir = dirs::config_dir()?.join("cosmic/dev.lamp.app/v2");
        if !cosmic_dir.exists() {
            return None;
        }
        log::info!("Migrating config from COSMIC: {}", cosmic_dir.display());

        let mut config = Self::default();

        // Helper to read a COSMIC config file
        let read_field = |name: &str| -> Option<String> {
            std::fs::read_to_string(cosmic_dir.join(name)).ok()
        };

        // org_directory: stored as quoted RON string
        if let Some(val) = read_field("org_directory") {
            let trimmed = val.trim().trim_matches('"');
            if !trimmed.is_empty() {
                config.org_directory = PathBuf::from(trimmed);
            }
        }

        // browser_command: quoted RON string
        if let Some(val) = read_field("browser_command") {
            let trimmed = val.trim().trim_matches('"');
            if !trimmed.is_empty() {
                config.browser_command = trimmed.to_string();
            }
        }

        // debug_logging: RON bool
        if let Some(val) = read_field("debug_logging") {
            config.debug_logging = val.trim() == "true";
        }

        // contexts: RON list of quoted strings
        if let Some(val) = read_field("contexts") {
            let mut contexts = Vec::new();
            for line in val.lines() {
                let trimmed = line.trim().trim_matches(',').trim().trim_matches('"');
                if trimmed.starts_with('@') {
                    contexts.push(trimmed.to_string());
                }
            }
            if !contexts.is_empty() {
                config.contexts = contexts;
            }
        }

        // calendars: RON struct (url, username)
        if let Some(val) = read_field("calendars") {
            config.calendars = parse_ron_service_config(&val);
        }

        // contacts: RON struct (url, username)
        if let Some(val) = read_field("contacts") {
            config.contacts = parse_ron_service_config(&val);
        }

        // notes_sync: RON struct (url, username)
        if let Some(val) = read_field("notes_sync") {
            config.notes_sync = parse_ron_service_config(&val);
        }

        // imap: RON struct (host, username, imap_folder)
        if let Some(val) = read_field("imap") {
            for line in val.lines() {
                let line = line.trim().trim_end_matches(',');
                if let Some(v) = line.strip_prefix("host:") {
                    config.imap.host = v.trim().trim_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("username:") {
                    config.imap.username = v.trim().trim_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("imap_folder:") {
                    config.imap.folder = v.trim().trim_matches('"').to_string();
                }
            }
        }

        // calendar_assignments: RON list of structs
        if let Some(val) = read_field("calendar_assignments") {
            let mut assignments = Vec::new();
            let mut current_href = String::new();
            for line in val.lines() {
                let line = line.trim().trim_end_matches(',');
                if let Some(v) = line.strip_prefix("calendar_href:") {
                    current_href = v.trim().trim_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("purpose:") {
                    let purpose = match v.trim() {
                        "Tasks" => CalendarPurpose::Tasks,
                        "Events" => CalendarPurpose::Events,
                        _ => CalendarPurpose::Disabled,
                    };
                    if !current_href.is_empty() {
                        assignments.push(CalendarAssignment {
                            calendar_href: current_href.clone(),
                            purpose,
                        });
                    }
                }
            }
            config.calendar_assignments = assignments;
        }

        // sync_tokens: RON map
        if let Some(val) = read_field("sync_tokens") {
            for line in val.lines() {
                let line = line.trim().trim_end_matches(',');
                // Format: "key": "value"
                if let Some((k, v)) = line.split_once(':') {
                    let k = k.trim().trim_matches('"');
                    let v = v.trim().trim_matches('"');
                    if k.starts_with('/') {
                        config.sync_tokens.insert(k.to_string(), v.to_string());
                    }
                }
            }
        }

        Some(config)
    }

    /// Save config to JSON file.
    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log::error!("Failed to save config: {}", e);
                }
            }
            Err(e) => log::error!("Failed to serialize config: {}", e),
        }
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lamp")
            .join("config.json")
    }

    pub fn inbox_path(&self) -> PathBuf {
        self.org_directory.join("inbox.org")
    }

    pub fn next_path(&self) -> PathBuf {
        self.org_directory.join("next.org")
    }

    pub fn projects_path(&self) -> PathBuf {
        self.org_directory.join("projects.org")
    }

    pub fn waiting_path(&self) -> PathBuf {
        self.org_directory.join("waiting.org")
    }

    pub fn someday_path(&self) -> PathBuf {
        self.org_directory.join("someday.org")
    }

    pub fn habits_path(&self) -> PathBuf {
        self.org_directory.join("habits.org")
    }

    pub fn archive_path(&self) -> PathBuf {
        self.org_directory.join("archive.org")
    }

    pub fn media_path(&self) -> PathBuf {
        self.org_directory.join("media.org")
    }

    pub fn consumed_path(&self) -> PathBuf {
        self.org_directory.join("consumed.org")
    }

    pub fn shopping_path(&self) -> PathBuf {
        self.org_directory.join("shopping.org")
    }

    pub fn bought_path(&self) -> PathBuf {
        self.org_directory.join("bought.org")
    }

    pub fn dayplan_path(&self) -> PathBuf {
        self.org_directory.join("dayplan.org")
    }

    pub fn contacts_path(&self) -> PathBuf {
        self.org_directory.join("contacts.org")
    }

    pub fn accounts_path(&self) -> PathBuf {
        self.org_directory.join("accounts.org")
    }

    pub fn closed_accounts_path(&self) -> PathBuf {
        self.org_directory.join("closed_accounts.org")
    }

    pub fn notes_path(&self) -> PathBuf {
        self.org_directory.join("notes.org")
    }

    pub fn notes_dir(&self) -> PathBuf {
        self.org_directory.join("notes")
    }

    pub fn events_cache_path(&self) -> PathBuf {
        self.org_directory.join("events.json")
    }

    /// Calendar hrefs assigned to Tasks purpose.
    pub fn task_calendar_hrefs(&self) -> Vec<String> {
        self.calendar_assignments
            .iter()
            .filter(|a| a.purpose == CalendarPurpose::Tasks)
            .map(|a| a.calendar_href.clone())
            .collect()
    }

    /// Calendar hrefs assigned to Events purpose.
    pub fn event_calendar_hrefs(&self) -> Vec<String> {
        self.calendar_assignments
            .iter()
            .filter(|a| a.purpose == CalendarPurpose::Events)
            .map(|a| a.calendar_href.clone())
            .collect()
    }

    /// Get sync token for a calendar.
    pub fn get_sync_token(&self, href: &str) -> Option<&str> {
        self.sync_tokens.get(href).map(|t| t.as_str())
    }

    /// Set sync token for a calendar.
    pub fn set_sync_token(&mut self, href: &str, token: &str) {
        self.sync_tokens.insert(href.to_string(), token.to_string());
    }

    /// Whether CalDAV is configured with at least one calendar assigned.
    pub fn sync_ready(&self) -> bool {
        !self.calendars.url.is_empty()
            && self
                .calendar_assignments
                .iter()
                .any(|a| a.purpose != CalendarPurpose::Disabled)
    }

    /// Ensure the org directory and files exist.
    pub fn ensure_files(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.org_directory)?;

        let files = [
            ("inbox.org", "Inbox"),
            ("next.org", "Next Actions"),
            ("projects.org", "Projects"),
            ("waiting.org", "Waiting For"),
            ("someday.org", "Someday/Maybe"),
            ("habits.org", "Habits"),
            ("archive.org", "Archive"),
        ];

        for (filename, title) in &files {
            let path = self.org_directory.join(filename);
            if !path.exists() {
                let content = format!(
                    "#+TITLE: {}\n#+TODO: TODO NEXT WAITING SOMEDAY | DONE CANCELLED\n\n",
                    title
                );
                std::fs::write(&path, content)?;
            }
        }

        // Notes directory (per-file storage)
        std::fs::create_dir_all(self.notes_dir())?;

        // List files (no #+TODO line — these aren't tasks)
        let list_files = [
            ("media.org", "Media Recommendations"),
            ("shopping.org", "Shopping"),
        ];

        for (filename, title) in &list_files {
            let path = self.org_directory.join(filename);
            if !path.exists() {
                let content = format!("#+TITLE: {}\n\n", title);
                std::fs::write(&path, content)?;
            }
        }

        Ok(())
    }
}

/// Parse a RON-format ServiceConfig struct: `( url: "...", username: "..." )`
fn parse_ron_service_config(ron: &str) -> ServiceConfig {
    let mut cfg = ServiceConfig::default();
    for line in ron.lines() {
        let line = line.trim().trim_end_matches(',');
        if let Some(v) = line.strip_prefix("url:") {
            cfg.url = v.trim().trim_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("username:") {
            cfg.username = v.trim().trim_matches('"').to_string();
        }
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_org_dir_not_tmp() {
        let config = LampConfig::default();
        let dir = config.org_directory.to_string_lossy();
        assert!(!dir.starts_with("/tmp"), "org_directory should not fall back to /tmp, got: {}", dir);
    }

    #[test]
    fn sync_token_get_set() {
        let mut config = LampConfig::default();
        assert!(config.get_sync_token("/cal/1").is_none());

        config.set_sync_token("/cal/1", "token-a");
        assert_eq!(config.get_sync_token("/cal/1"), Some("token-a"));

        config.set_sync_token("/cal/1", "token-b");
        assert_eq!(config.get_sync_token("/cal/1"), Some("token-b"));

        config.set_sync_token("/cal/2", "token-c");
        assert_eq!(config.sync_tokens.len(), 2);
    }

    #[test]
    fn task_and_event_calendar_hrefs() {
        let mut config = LampConfig::default();
        config.calendar_assignments = vec![
            CalendarAssignment { calendar_href: "/a".into(), purpose: CalendarPurpose::Tasks },
            CalendarAssignment { calendar_href: "/b".into(), purpose: CalendarPurpose::Events },
            CalendarAssignment { calendar_href: "/c".into(), purpose: CalendarPurpose::Disabled },
            CalendarAssignment { calendar_href: "/d".into(), purpose: CalendarPurpose::Tasks },
        ];
        assert_eq!(config.task_calendar_hrefs(), vec!["/a", "/d"]);
        assert_eq!(config.event_calendar_hrefs(), vec!["/b"]);
    }

    #[test]
    fn sync_ready_requires_url_and_assignment() {
        let mut config = LampConfig::default();
        assert!(!config.sync_ready());

        config.calendars.url = "https://cal.example.com".into();
        assert!(!config.sync_ready());

        config.calendar_assignments = vec![
            CalendarAssignment { calendar_href: "/a".into(), purpose: CalendarPurpose::Disabled },
        ];
        assert!(!config.sync_ready());

        config.calendar_assignments.push(
            CalendarAssignment { calendar_href: "/b".into(), purpose: CalendarPurpose::Tasks },
        );
        assert!(config.sync_ready());
    }

    #[test]
    fn path_helpers_join_correctly() {
        let mut config = LampConfig::default();
        config.org_directory = PathBuf::from("/data/lamp");
        assert_eq!(config.inbox_path(), PathBuf::from("/data/lamp/inbox.org"));
        assert_eq!(config.next_path(), PathBuf::from("/data/lamp/next.org"));
        assert_eq!(config.projects_path(), PathBuf::from("/data/lamp/projects.org"));
        assert_eq!(config.notes_dir(), PathBuf::from("/data/lamp/notes"));
    }
}
