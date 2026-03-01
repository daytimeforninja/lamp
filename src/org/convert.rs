use chrono::NaiveDate;
use uuid::Uuid;

use crate::core::account::Account;
use crate::core::day_plan::{CompletedTask, DayPlan};
use crate::core::habit::Habit;
use crate::core::link::LinkTarget;
use crate::core::list_item::ListItem;
use crate::core::note::Note;
use crate::core::project::Project;
use crate::core::task::Task;

use super::parser::{OrgParser, ParsedHeading, heading_to_task};

/// Convert parsed headings to tasks, optionally extracting habits.
pub fn headings_to_tasks(headings: &[ParsedHeading]) -> Vec<Task> {
    headings.iter().map(heading_to_task).collect()
}

/// Identify daily habit headings and convert them.
/// Only habits with daily recurrence (or no recurrence, assumed daily) are included.
pub fn extract_habits(headings: &[ParsedHeading]) -> Vec<Habit> {
    use crate::core::recurrence::RecurrenceUnit;

    let is_daily = |h: &ParsedHeading| -> bool {
        match &h.recurrence {
            None => true, // No recurrence = assume daily
            Some(r) => {
                let interval = match r {
                    crate::core::recurrence::Recurrence::Standard(i) => i,
                    crate::core::recurrence::Recurrence::Relative(i) => i,
                    crate::core::recurrence::Recurrence::Strict(i) => i,
                };
                interval.unit == RecurrenceUnit::Day && interval.count == 1
            }
        }
    };

    headings
        .iter()
        .filter(|h| {
            (h.tags.contains(&"habit".to_string())
                || OrgParser::get_property(&h.properties, "STYLE")
                    .is_some_and(|v| v == "habit"))
                && is_daily(h)
        })
        .map(|h| {
            let task = heading_to_task(h);
            let mut habit = Habit::new(task);
            habit.completions = h.logbook_entries.clone();
            habit.recalculate_streak(chrono::Local::now().date_naive());
            habit
        })
        .collect()
}

/// Extract projects from parsed headings using heading levels.
/// Level-1 headings without a state are project containers.
/// Level-2 headings under a project container are project tasks.
pub fn extract_projects(headings: &[ParsedHeading]) -> Vec<Project> {
    let mut projects = Vec::new();
    let mut current_project: Option<Project> = None;

    for heading in headings {
        if heading.level == 1 && heading.state.is_none() {
            // Save previous project if any
            if let Some(proj) = current_project.take() {
                projects.push(proj);
            }
            // Strip "Project: " prefix if present
            let name = heading
                .title
                .strip_prefix("Project: ")
                .unwrap_or(&heading.title)
                .to_string();
            let mut proj = Project::new(name);
            if let Some(id_str) = OrgParser::get_property(&heading.properties, "ID") {
                if let Ok(id) = uuid::Uuid::parse_str(id_str) {
                    proj.id = id;
                }
            }
            proj.purpose = OrgParser::get_property(&heading.properties, "PURPOSE")
                .unwrap_or("")
                .to_string();
            proj.outcome = OrgParser::get_property(&heading.properties, "OUTCOME")
                .unwrap_or("")
                .to_string();
            proj.brainstorm = heading.notes.clone();
            current_project = Some(proj);
        } else if heading.level == 2 && current_project.is_some() {
            // Task under current project
            let mut task = heading_to_task(heading);
            if let Some(ref proj) = current_project {
                task.project = Some(proj.name.clone());
            }
            current_project.as_mut().unwrap().tasks.push(task);
        } else if heading.level == 1 && heading.state.is_some() {
            // Standalone task with state at level 1 — close current project
            if let Some(proj) = current_project.take() {
                projects.push(proj);
            }
        }
    }

    // Don't forget the last project
    if let Some(proj) = current_project {
        projects.push(proj);
    }

    projects
}

/// Convert a ParsedHeading into a ListItem (no state, priority, dates).
fn heading_to_list_item(heading: &ParsedHeading) -> ListItem {
    let id = OrgParser::get_property(&heading.properties, "ID")
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .unwrap_or_else(uuid::Uuid::new_v4);

    let created = OrgParser::get_property(&heading.properties, "CREATED")
        .and_then(|s| {
            let s = s.trim_matches(|c| c == '[' || c == ']');
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %a %H:%M").ok()
        })
        .unwrap_or_else(|| chrono::Local::now().naive_local());

    let done = heading.state == Some(crate::core::task::TaskState::Done);

    ListItem {
        id,
        title: heading.title.clone(),
        notes: heading.notes.clone(),
        created,
        done,
    }
}

/// Parse an org file and return list items (headings without state).
pub fn parse_list_items(input: &str) -> Vec<ListItem> {
    let headings = OrgParser::parse(input);
    headings.iter().map(heading_to_list_item).collect()
}

/// Convert a ParsedHeading into an Account.
fn heading_to_account(heading: &ParsedHeading) -> Account {
    let id = OrgParser::get_property(&heading.properties, "ID")
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .unwrap_or_else(uuid::Uuid::new_v4);

    let url = OrgParser::get_property(&heading.properties, "URL")
        .unwrap_or("")
        .to_string();

    let last_checked = OrgParser::get_property(&heading.properties, "LAST_CHECKED")
        .and_then(|s| {
            let s = s.trim_matches(|c| c == '[' || c == ']');
            NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
        });

    Account {
        id,
        name: heading.title.clone(),
        url,
        notes: heading.notes.clone(),
        last_checked,
    }
}

/// Parse an org file and return accounts.
pub fn parse_accounts(input: &str) -> Vec<Account> {
    let headings = OrgParser::parse(input);
    headings.iter().map(heading_to_account).collect()
}

/// Parse an org file and return tasks.
pub fn parse_tasks(input: &str) -> Vec<Task> {
    let headings = OrgParser::parse(input);
    headings_to_tasks(&headings)
}

/// Parse an org file and return habits.
pub fn parse_habits(input: &str) -> Vec<Habit> {
    let headings = OrgParser::parse(input);
    extract_habits(&headings)
}

/// Parse an org file and return projects.
pub fn parse_projects(input: &str) -> Vec<Project> {
    let headings = OrgParser::parse(input);
    extract_projects(&headings)
}

/// Parse a dayplan.org file into a DayPlan.
pub fn parse_day_plan(input: &str) -> Option<DayPlan> {
    let mut date: Option<NaiveDate> = None;
    let mut spoon_budget: u32 = 50;
    let mut spent_spoons: u32 = 0;

    // Parse preamble keywords
    for line in input.lines() {
        if line.starts_with('*') {
            break;
        }
        if let Some(rest) = line.strip_prefix("#+DATE:") {
            date = NaiveDate::parse_from_str(rest.trim(), "%Y-%m-%d").ok();
        } else if let Some(rest) = line.strip_prefix("#+SPOON_BUDGET:") {
            spoon_budget = rest.trim().parse().unwrap_or(50);
        } else if let Some(rest) = line.strip_prefix("#+SPENT_SPOONS:") {
            spent_spoons = rest.trim().parse().unwrap_or(0);
        }
    }

    let date = date?;

    let mut active_contexts = Vec::new();
    let mut confirmed_task_ids = Vec::new();
    let mut completed_tasks: Vec<CompletedTask> = Vec::new();
    let mut picked_media_ids = Vec::new();
    let mut picked_shopping_ids = Vec::new();

    // Parse sections
    let mut current_section = "";
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed == "* Active Contexts" {
            current_section = "contexts";
        } else if trimmed == "* Confirmed Tasks" {
            current_section = "tasks";
        } else if trimmed == "* Completed Tasks" {
            current_section = "completed";
        } else if trimmed == "* Picked Media" {
            current_section = "media";
        } else if trimmed == "* Picked Shopping" {
            current_section = "shopping";
        } else if trimmed.starts_with("* ") {
            current_section = "";
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            match current_section {
                "contexts" => active_contexts.push(item.to_string()),
                "tasks" => {
                    if let Ok(id) = Uuid::parse_str(item) {
                        confirmed_task_ids.push(id);
                    }
                }
                "completed" => {
                    // Format: "uuid | title | esc"
                    let parts: Vec<&str> = item.splitn(3, " | ").collect();
                    if let Some(id_str) = parts.first() {
                        if let Ok(id) = Uuid::parse_str(id_str.trim()) {
                            let title = parts.get(1).unwrap_or(&"").trim().to_string();
                            let esc = parts.get(2).and_then(|s| s.trim().parse().ok());
                            completed_tasks.push(CompletedTask { id, title, esc });
                        }
                    }
                }
                "media" => {
                    if let Ok(id) = Uuid::parse_str(item) {
                        picked_media_ids.push(id);
                    }
                }
                "shopping" => {
                    if let Ok(id) = Uuid::parse_str(item) {
                        picked_shopping_ids.push(id);
                    }
                }
                _ => {}
            }
        }
    }

    Some(DayPlan {
        date,
        spoon_budget,
        active_contexts,
        confirmed_task_ids,
        completed_tasks,
        spent_spoons,
        picked_media_ids,
        picked_shopping_ids,
    })
}

/// Convert a ParsedHeading into a Note.
fn heading_to_note(heading: &ParsedHeading) -> Note {
    let id = OrgParser::get_property(&heading.properties, "ID")
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    let created = OrgParser::get_property(&heading.properties, "CREATED")
        .and_then(|s| {
            let s = s.trim_matches(|c| c == '[' || c == ']');
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %a %H:%M")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M"))
                .ok()
        })
        .unwrap_or_else(|| chrono::Local::now().naive_local());

    let modified = OrgParser::get_property(&heading.properties, "MODIFIED")
        .and_then(|s| {
            let s = s.trim_matches(|c| c == '[' || c == ']');
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %a %H:%M")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M"))
                .ok()
        })
        .unwrap_or(created);

    let source = OrgParser::get_property(&heading.properties, "SOURCE")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let links = OrgParser::get_property(&heading.properties, "LINKS")
        .map(|s| {
            s.split_whitespace()
                .filter_map(LinkTarget::from_org)
                .collect()
        })
        .unwrap_or_default();

    let sync_etag = OrgParser::get_property(&heading.properties, "SYNC_ETAG")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Note {
        id,
        title: heading.title.clone(),
        body: heading.notes.clone(),
        tags: heading.tags.clone(),
        links,
        source,
        created,
        modified,
        sync_etag,
    }
}

/// Parse an org file and return notes.
pub fn parse_notes(input: &str) -> Vec<Note> {
    let headings = OrgParser::parse(input);
    headings.iter().map(heading_to_note).collect()
}
