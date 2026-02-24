use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CONFIG_VERSION: u64 = 1;

fn default_org_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("lamp")
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, CosmicConfigEntry)]
pub struct LampConfig {
    pub org_directory: PathBuf,
    pub contexts: Vec<String>,
}

impl Default for LampConfig {
    fn default() -> Self {
        Self {
            org_directory: default_org_dir(),
            contexts: default_contexts(),
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

    pub fn shopping_path(&self) -> PathBuf {
        self.org_directory.join("shopping.org")
    }

    pub fn dayplan_path(&self) -> PathBuf {
        self.org_directory.join("dayplan.org")
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
