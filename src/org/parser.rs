use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::core::recurrence::Recurrence;
use crate::core::task::{Priority, Task, TaskState};

static HEADLINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<stars>\*+)\s+(?:(?P<state>TODO|NEXT|WAITING|SOMEDAY|DONE|CANCELLED)\s+)?(?:\[#(?P<priority>[ABC])\]\s+)?(?P<title>.+?)(?:\s+:(?P<tags>[^:]+(?::[^:]+)*):)?$").unwrap()
});

static SCHEDULED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"SCHEDULED:\s*<(?P<date>\d{4}-\d{2}-\d{2})\s+\w+(?:\s+(?P<recurrence>[.+]+\d+[dwmy]))?>")
        .unwrap()
});

static DEADLINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"DEADLINE:\s*<(?P<date>\d{4}-\d{2}-\d{2})\s+\w+(?:\s+(?P<recurrence>[.+]+\d+[dwmy]))?>")
        .unwrap()
});

static PROPERTY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*:(?P<key>[A-Z_]+):\s+(?P<value>.+)$").unwrap()
});

static LOGBOOK_ENTRY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"- State "DONE"\s+from\s+"[^"]*"\s+\[(?P<datetime>\d{4}-\d{2}-\d{2}\s+\w+\s+\d{2}:\d{2})\]"#,
    )
    .unwrap()
});

pub struct OrgParser;

/// A parsed org heading with all its metadata.
#[derive(Debug, Clone)]
pub struct ParsedHeading {
    pub level: usize,
    pub state: Option<TaskState>,
    pub priority: Option<Priority>,
    pub title: String,
    pub tags: Vec<String>,
    pub scheduled: Option<NaiveDate>,
    pub deadline: Option<NaiveDate>,
    pub recurrence: Option<Recurrence>,
    pub properties: Vec<(String, String)>,
    pub logbook_entries: Vec<NaiveDateTime>,
    pub notes: String,
}

impl OrgParser {
    /// Parse an org file string into a list of headings.
    pub fn parse(input: &str) -> Vec<ParsedHeading> {
        let lines: Vec<&str> = input.lines().collect();
        let mut headings = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            if let Some(captures) = HEADLINE_RE.captures(lines[i]) {
                let level = captures
                    .name("stars")
                    .map(|m| m.as_str().len())
                    .unwrap_or(1);
                let state = captures
                    .name("state")
                    .and_then(|m| TaskState::from_keyword(m.as_str()));
                let priority = captures
                    .name("priority")
                    .and_then(|m| Priority::from_org(m.as_str()));
                let title = captures["title"].to_string();
                let tags: Vec<String> = captures
                    .name("tags")
                    .map(|m| m.as_str().split(':').map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                i += 1;

                // Parse planning line (SCHEDULED/DEADLINE)
                let mut scheduled = None;
                let mut deadline = None;
                let mut recurrence = None;

                if i < lines.len() {
                    let line = lines[i];
                    if line.trim_start().starts_with("SCHEDULED:")
                        || line.trim_start().starts_with("DEADLINE:")
                    {
                        if let Some(caps) = SCHEDULED_RE.captures(line) {
                            scheduled = NaiveDate::parse_from_str(&caps["date"], "%Y-%m-%d").ok();
                            if let Some(rec_match) = caps.name("recurrence") {
                                recurrence = Recurrence::parse(rec_match.as_str());
                            }
                        }
                        if let Some(caps) = DEADLINE_RE.captures(line) {
                            deadline = NaiveDate::parse_from_str(&caps["date"], "%Y-%m-%d").ok();
                            if recurrence.is_none() {
                                if let Some(rec_match) = caps.name("recurrence") {
                                    recurrence = Recurrence::parse(rec_match.as_str());
                                }
                            }
                        }
                        i += 1;
                    }
                }

                // Parse properties drawer
                let mut properties = Vec::new();
                if i < lines.len() && lines[i].trim() == ":PROPERTIES:" {
                    i += 1;
                    while i < lines.len() && lines[i].trim() != ":END:" {
                        if let Some(caps) = PROPERTY_RE.captures(lines[i]) {
                            properties.push((caps["key"].to_string(), caps["value"].to_string()));
                        }
                        i += 1;
                    }
                    if i < lines.len() {
                        i += 1; // skip :END:
                    }
                }

                // Parse LOGBOOK
                let mut logbook_entries = Vec::new();
                if i < lines.len() && lines[i].trim() == ":LOGBOOK:" {
                    i += 1;
                    while i < lines.len() && lines[i].trim() != ":END:" {
                        if let Some(caps) = LOGBOOK_ENTRY_RE.captures(lines[i]) {
                            if let Ok(dt) = NaiveDateTime::parse_from_str(
                                &caps["datetime"],
                                "%Y-%m-%d %a %H:%M",
                            ) {
                                logbook_entries.push(dt);
                            }
                        }
                        i += 1;
                    }
                    if i < lines.len() {
                        i += 1; // skip :END:
                    }
                }

                // Collect notes (everything until next heading or EOF)
                let mut notes = String::new();
                while i < lines.len() && !lines[i].starts_with('*') {
                    if !notes.is_empty() {
                        notes.push('\n');
                    }
                    notes.push_str(lines[i]);
                    i += 1;
                }
                let notes = notes.trim().to_string();

                headings.push(ParsedHeading {
                    level,
                    state,
                    priority,
                    title,
                    tags,
                    scheduled,
                    deadline,
                    recurrence,
                    properties,
                    logbook_entries,
                    notes,
                });
            } else {
                i += 1;
            }
        }

        headings
    }

    /// Extract a property value by key.
    pub fn get_property<'a>(props: &'a [(String, String)], key: &str) -> Option<&'a str> {
        props
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// Convert a ParsedHeading into a Task.
pub fn heading_to_task(heading: &ParsedHeading) -> Task {
    let id = OrgParser::get_property(&heading.properties, "ID")
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    let created = OrgParser::get_property(&heading.properties, "CREATED")
        .and_then(|s| {
            // Format: [2026-02-23 Mon 14:00]
            let s = s.trim_matches(|c| c == '[' || c == ']');
            NaiveDateTime::parse_from_str(s, "%Y-%m-%d %a %H:%M").ok()
        })
        .unwrap_or_else(|| chrono::Local::now().naive_local());

    let contexts: Vec<String> = heading
        .tags
        .iter()
        .filter(|t| t.starts_with('@'))
        .cloned()
        .collect();

    let waiting_for = heading
        .tags
        .iter()
        .any(|t| t == "waiting")
        .then(|| heading.notes.lines().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty());

    let esc = OrgParser::get_property(&heading.properties, "ESC")
        .and_then(|s| s.trim().parse::<u32>().ok());

    Task {
        id,
        title: heading.title.clone(),
        state: heading.state.clone().unwrap_or(TaskState::Todo),
        priority: heading.priority,
        contexts,
        scheduled: heading.scheduled,
        deadline: heading.deadline,
        recurrence: heading.recurrence.clone(),
        notes: heading.notes.clone(),
        created,
        completed: None,
        project: None,
        waiting_for,
        esc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_todo() {
        let input = "* TODO Fix the leaky faucet :@home:\n";
        let headings = OrgParser::parse(input);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].title, "Fix the leaky faucet");
        assert_eq!(headings[0].state, Some(TaskState::Todo));
        assert_eq!(headings[0].tags, vec!["@home"]);
    }

    #[test]
    fn parse_with_priority_and_schedule() {
        let input = "\
* TODO [#A] Fix the leaky faucet :@home:
  SCHEDULED: <2026-02-24 Tue>
  :PROPERTIES:
  :ID: 550e8400-e29b-41d4-a716-446655440000
  :CREATED: [2026-02-23 Mon 14:00]
  :END:
";
        let headings = OrgParser::parse(input);
        assert_eq!(headings.len(), 1);
        let h = &headings[0];
        assert_eq!(h.priority, Some(Priority::A));
        assert_eq!(
            h.scheduled,
            Some(NaiveDate::from_ymd_opt(2026, 2, 24).unwrap())
        );
        assert_eq!(
            OrgParser::get_property(&h.properties, "ID"),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn parse_habit_with_logbook() {
        let input = "\
* TODO Meditate :@home:habit:
  SCHEDULED: <2026-02-23 Mon .+1d>
  :PROPERTIES:
  :ID: 550e8400-e29b-41d4-a716-446655440001
  :STYLE: habit
  :END:
  :LOGBOOK:
  - State \"DONE\" from \"TODO\" [2026-02-22 Sun 07:30]
  - State \"DONE\" from \"TODO\" [2026-02-21 Sat 07:15]
  :END:
";
        let headings = OrgParser::parse(input);
        assert_eq!(headings.len(), 1);
        let h = &headings[0];
        assert!(h.tags.contains(&"habit".to_string()));
        assert_eq!(h.logbook_entries.len(), 2);
        assert!(h.recurrence.is_some());
        assert!(matches!(h.recurrence, Some(Recurrence::Relative(_))));
    }

    #[test]
    fn parse_multiple_headings() {
        let input = "\
#+TITLE: Inbox
#+TODO: TODO NEXT WAITING SOMEDAY | DONE CANCELLED

* TODO First task
* NEXT Second task :@work:
* DONE Third task
";
        let headings = OrgParser::parse(input);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].state, Some(TaskState::Todo));
        assert_eq!(headings[1].state, Some(TaskState::Next));
        assert_eq!(headings[2].state, Some(TaskState::Done));
    }
}
