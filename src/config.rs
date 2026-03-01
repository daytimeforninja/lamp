use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CONFIG_VERSION: u64 = 2;

fn default_org_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, CosmicConfigEntry)]
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
    /// Sync tokens: (calendar_href, token)
    pub sync_tokens: Vec<(String, String)>,
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
            sync_tokens: Vec::new(),
            browser_command: default_browser_command(),
            debug_logging: false,
        }
    }
}

impl LampConfig {
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
        self.sync_tokens
            .iter()
            .find(|(h, _)| h == href)
            .map(|(_, t)| t.as_str())
    }

    /// Set sync token for a calendar.
    pub fn set_sync_token(&mut self, href: &str, token: &str) {
        if let Some(existing) = self
            .sync_tokens
            .iter_mut()
            .find(|(h, _)| h == href)
        {
            existing.1 = token.to_string();
        } else {
            self.sync_tokens
                .push((href.to_string(), token.to_string()));
        }
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
