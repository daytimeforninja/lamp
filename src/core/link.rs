use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkTarget {
    Note(Uuid),
    Task(Uuid),
    Project(Uuid),
    Contact(Uuid),
    Account(Uuid),
    MediaItem(Uuid),
    ShoppingItem(Uuid),
}

impl LinkTarget {
    /// Serialize to org format: "type:uuid"
    pub fn to_org(&self) -> String {
        let (kind, id) = match self {
            Self::Note(id) => ("note", id),
            Self::Task(id) => ("task", id),
            Self::Project(id) => ("project", id),
            Self::Contact(id) => ("contact", id),
            Self::Account(id) => ("account", id),
            Self::MediaItem(id) => ("media", id),
            Self::ShoppingItem(id) => ("shopping", id),
        };
        format!("{}:{}", kind, id)
    }

    /// Parse from org format: "type:uuid"
    pub fn from_org(s: &str) -> Option<Self> {
        let (kind, id_str) = s.split_once(':')?;
        let id = Uuid::parse_str(id_str).ok()?;
        match kind {
            "note" => Some(Self::Note(id)),
            "task" => Some(Self::Task(id)),
            "project" => Some(Self::Project(id)),
            "contact" => Some(Self::Contact(id)),
            "account" => Some(Self::Account(id)),
            "media" => Some(Self::MediaItem(id)),
            "shopping" => Some(Self::ShoppingItem(id)),
            _ => None,
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Note(id)
            | Self::Task(id)
            | Self::Project(id)
            | Self::Contact(id)
            | Self::Account(id)
            | Self::MediaItem(id)
            | Self::ShoppingItem(id) => *id,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Note(_) => "Note",
            Self::Task(_) => "Task",
            Self::Project(_) => "Project",
            Self::Contact(_) => "Contact",
            Self::Account(_) => "Account",
            Self::MediaItem(_) => "Media",
            Self::ShoppingItem(_) => "Shopping",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let id = Uuid::new_v4();
        let targets = vec![
            LinkTarget::Note(id),
            LinkTarget::Task(id),
            LinkTarget::Project(id),
            LinkTarget::Contact(id),
            LinkTarget::Account(id),
            LinkTarget::MediaItem(id),
            LinkTarget::ShoppingItem(id),
        ];
        for target in targets {
            let org = target.to_org();
            let parsed = LinkTarget::from_org(&org).unwrap();
            assert_eq!(target, parsed);
        }
    }
}
