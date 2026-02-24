use chrono::NaiveDate;
use uuid::Uuid;

use crate::core::day_plan::DayPlan;
use crate::core::habit::Habit;
use crate::core::list_item::ListItem;
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
            current_project = Some(Project::new(name));
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

    ListItem {
        id,
        title: heading.title.clone(),
        notes: heading.notes.clone(),
        created,
    }
}

/// Parse an org file and return list items (headings without state).
pub fn parse_list_items(input: &str) -> Vec<ListItem> {
    let headings = OrgParser::parse(input);
    headings.iter().map(heading_to_list_item).collect()
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

    // Parse preamble keywords
    for line in input.lines() {
        if line.starts_with('*') {
            break;
        }
        if let Some(rest) = line.strip_prefix("#+DATE:") {
            date = NaiveDate::parse_from_str(rest.trim(), "%Y-%m-%d").ok();
        } else if let Some(rest) = line.strip_prefix("#+SPOON_BUDGET:") {
            spoon_budget = rest.trim().parse().unwrap_or(50);
        }
    }

    let date = date?;

    let mut active_contexts = Vec::new();
    let mut confirmed_task_ids = Vec::new();
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
        picked_media_ids,
        picked_shopping_ids,
    })
}
