use chrono::NaiveDateTime;
use std::io::Write;

use crate::core::account::Account;
use crate::core::day_plan::DayPlan;
use crate::core::list_item::ListItem;
use crate::core::note::Note;
use crate::core::project::Project;
use crate::core::task::Task;

/// Writes tasks to org-mode format.
pub struct OrgWriter;

impl OrgWriter {
    /// Write a complete org file with header and tasks.
    pub fn write_file(title: &str, tasks: &[Task]) -> String {
        let mut out = String::new();
        out.push_str(&format!("#+TITLE: {}\n", title));
        out.push_str("#+TODO: TODO NEXT WAITING SOMEDAY | DONE CANCELLED\n\n");

        for task in tasks {
            out.push_str(&Self::write_task(task));
            out.push('\n');
        }

        out
    }

    /// Write a single task as an org heading.
    pub fn write_task(task: &Task) -> String {
        Self::write_task_at_level(task, 1)
    }

    /// Write a task at a specific heading level.
    pub fn write_task_at_level(task: &Task, level: usize) -> String {
        let mut out = String::new();
        let stars: String = "*".repeat(level);
        let indent = "  ";

        // Headline: ** STATE [#P] Title :tags:
        out.push_str(&stars);
        out.push(' ');
        out.push_str(task.state.as_keyword());
        out.push(' ');

        if let Some(ref priority) = task.priority {
            out.push_str(priority.as_org());
            out.push(' ');
        }

        out.push_str(&task.title);

        // Tags
        let all_tags = task.contexts.clone();
        if !all_tags.is_empty() {
            out.push_str(" :");
            out.push_str(&all_tags.join(":"));
            out.push(':');
        }
        out.push('\n');

        // CLOSED timestamp
        if let Some(closed) = task.completed {
            out.push_str(&format!(
                "{indent}CLOSED: [{}]\n",
                closed.format("%Y-%m-%d %a %H:%M")
            ));
        }

        // Planning line (SCHEDULED / DEADLINE)
        let mut planning = Vec::new();
        if let Some(scheduled) = task.scheduled {
            let day_name = scheduled.format("%a");
            let mut sched_str = format!("SCHEDULED: <{} {}", scheduled.format("%Y-%m-%d"), day_name);
            if let Some(ref recurrence) = task.recurrence {
                sched_str.push(' ');
                sched_str.push_str(&recurrence.to_string());
            }
            sched_str.push('>');
            planning.push(sched_str);
        }
        if let Some(deadline) = task.deadline {
            let day_name = deadline.format("%a");
            planning.push(format!("DEADLINE: <{} {}>", deadline.format("%Y-%m-%d"), day_name));
        }
        if !planning.is_empty() {
            out.push_str(indent);
            out.push_str(&planning.join(" "));
            out.push('\n');
        }

        // Properties drawer
        out.push_str(&format!("{indent}:PROPERTIES:\n"));
        out.push_str(&format!("{indent}:ID: {}\n", task.id));
        out.push_str(&format!(
            "{indent}:CREATED: [{}]\n",
            task.created.format("%Y-%m-%d %a %H:%M")
        ));
        if let Some(esc) = task.esc {
            out.push_str(&format!("{indent}:ESC: {}\n", esc));
        }
        if let Some(ref wf) = task.waiting_for {
            out.push_str(&format!("{indent}:WAITING_FOR: {}\n", wf));
        }
        if let Some(delegated) = task.delegated {
            out.push_str(&format!("{indent}:DELEGATED: {}\n", delegated.format("%Y-%m-%d")));
        }
        if let Some(follow_up) = task.follow_up {
            out.push_str(&format!("{indent}:FOLLOW_UP: {}\n", follow_up.format("%Y-%m-%d")));
        }
        if let Some(ref sync_href) = task.sync_href {
            out.push_str(&format!("{indent}:SYNC_HREF: {}\n", sync_href));
        }
        if let Some(sync_hash) = task.sync_hash {
            out.push_str(&format!("{indent}:SYNC_HASH: {}\n", sync_hash));
        }
        if let Some(ref sync_uid) = task.sync_uid {
            out.push_str(&format!("{indent}:SYNC_UID: {}\n", sync_uid));
        }
        out.push_str(&format!("{indent}:END:\n"));

        // Notes
        if !task.notes.is_empty() {
            for line in task.notes.lines() {
                out.push_str(indent);
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }

    /// Write a complete projects file.
    pub fn write_projects_file(projects: &[Project]) -> String {
        let mut out = String::new();
        out.push_str("#+TITLE: Projects\n");
        out.push_str("#+TODO: TODO NEXT WAITING SOMEDAY | DONE CANCELLED\n\n");

        for project in projects {
            out.push_str(&format!("* Project: {}\n", project.name));

            // Properties drawer (always written for ID)
            out.push_str("  :PROPERTIES:\n");
            out.push_str(&format!("  :ID: {}\n", project.id));
            if !project.purpose.is_empty() {
                out.push_str(&format!("  :PURPOSE: {}\n", project.purpose));
            }
            if !project.outcome.is_empty() {
                out.push_str(&format!("  :OUTCOME: {}\n", project.outcome));
            }
            out.push_str("  :END:\n");

            // Brainstorm as body text
            if !project.brainstorm.is_empty() {
                for line in project.brainstorm.lines() {
                    out.push_str("  ");
                    out.push_str(line);
                    out.push('\n');
                }
            }

            for task in &project.tasks {
                out.push_str(&Self::write_task_at_level(task, 2));
                out.push('\n');
            }
        }

        out
    }

    /// Write a complete org file for list items (no #+TODO line).
    pub fn write_list_items_file(title: &str, items: &[ListItem]) -> String {
        let mut out = String::new();
        out.push_str(&format!("#+TITLE: {}\n\n", title));

        for item in items {
            out.push_str(&Self::write_list_item(item));
            out.push('\n');
        }

        out
    }

    /// Write a single list item as an org heading.
    pub fn write_list_item(item: &ListItem) -> String {
        let mut out = String::new();
        let indent = "  ";

        if item.done {
            out.push_str(&format!("* DONE {}\n", item.title));
        } else {
            out.push_str(&format!("* {}\n", item.title));
        }

        // Properties drawer
        out.push_str(&format!("{indent}:PROPERTIES:\n"));
        out.push_str(&format!("{indent}:ID: {}\n", item.id));
        out.push_str(&format!(
            "{indent}:CREATED: [{}]\n",
            item.created.format("%Y-%m-%d %a %H:%M")
        ));
        out.push_str(&format!("{indent}:END:\n"));

        // Notes
        if !item.notes.is_empty() {
            for line in item.notes.lines() {
                out.push_str(indent);
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }

    /// Write a complete org file for accounts.
    pub fn write_accounts_file(accounts: &[Account]) -> String {
        let mut out = String::new();
        out.push_str("#+TITLE: Accounts\n\n");

        for account in accounts {
            out.push_str(&Self::write_account(account));
            out.push('\n');
        }

        out
    }

    /// Write a single account as an org heading.
    pub fn write_account(account: &Account) -> String {
        let mut out = String::new();
        let indent = "  ";

        out.push_str(&format!("* {}\n", account.name));

        // Properties drawer
        out.push_str(&format!("{indent}:PROPERTIES:\n"));
        out.push_str(&format!("{indent}:ID: {}\n", account.id));
        if !account.url.is_empty() {
            out.push_str(&format!("{indent}:URL: {}\n", account.url));
        }
        if let Some(date) = account.last_checked {
            out.push_str(&format!("{indent}:LAST_CHECKED: [{}]\n", date.format("%Y-%m-%d")));
        }
        out.push_str(&format!("{indent}:END:\n"));

        // Notes as body text
        if !account.notes.is_empty() {
            for line in account.notes.lines() {
                out.push_str(indent);
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }

    /// Append a single account to an existing org file (e.g. closed_accounts.org).
    pub fn append_account_to_file(path: &std::path::Path, account: &Account) -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let content = Self::write_account(account);
        file.write_all(content.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    /// Write a day plan to org format.
    pub fn write_day_plan(plan: &DayPlan) -> String {
        let mut out = String::new();
        out.push_str("#+TITLE: Day Plan\n");
        out.push_str(&format!("#+DATE: {}\n", plan.date.format("%Y-%m-%d")));
        out.push_str(&format!("#+SPOON_BUDGET: {}\n", plan.spoon_budget));
        out.push_str(&format!("#+SPENT_SPOONS: {}\n\n", plan.spent_spoons));

        out.push_str("* Active Contexts\n");
        for ctx in &plan.active_contexts {
            out.push_str(&format!("  - {}\n", ctx));
        }
        out.push('\n');

        out.push_str("* Confirmed Tasks\n");
        for id in &plan.confirmed_task_ids {
            out.push_str(&format!("  - {}\n", id));
        }
        out.push('\n');

        out.push_str("* Completed Tasks\n");
        for ct in &plan.completed_tasks {
            let esc_str = ct.esc.map(|e| e.to_string()).unwrap_or_default();
            out.push_str(&format!("  - {} | {} | {}\n", ct.id, ct.title, esc_str));
        }
        out.push('\n');

        out.push_str("* Picked Media\n");
        for id in &plan.picked_media_ids {
            out.push_str(&format!("  - {}\n", id));
        }
        out.push('\n');

        out.push_str("* Picked Shopping\n");
        for id in &plan.picked_shopping_ids {
            out.push_str(&format!("  - {}\n", id));
        }

        out
    }

    /// Append a single task to an existing org file (e.g. archive.org).
    pub fn append_to_file(path: &std::path::Path, task: &Task) -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let content = Self::write_task(task);
        file.write_all(content.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    /// Append a single list item to an existing org file (e.g. consumed.org, bought.org).
    pub fn append_list_item_to_file(path: &std::path::Path, item: &ListItem) -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let content = Self::write_list_item(item);
        file.write_all(content.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    /// Write a logbook entry for a state change.
    pub fn format_logbook_entry(
        new_state: &str,
        old_state: &str,
        timestamp: NaiveDateTime,
    ) -> String {
        format!(
            "  - State \"{}\" from \"{}\" [{}]",
            new_state,
            old_state,
            timestamp.format("%Y-%m-%d %a %H:%M")
        )
    }

    /// Write a complete task with logbook entries (for habits).
    pub fn write_habit_task(task: &Task, completions: &[NaiveDateTime]) -> String {
        let mut out = String::new();

        // Headline
        out.push_str("* ");
        out.push_str(task.state.as_keyword());
        out.push(' ');
        out.push_str(&task.title);

        let mut all_tags = task.contexts.clone();
        if !all_tags.contains(&"habit".to_string()) {
            all_tags.push("habit".to_string());
        }
        out.push_str(" :");
        out.push_str(&all_tags.join(":"));
        out.push_str(":\n");

        // Planning
        if let Some(scheduled) = task.scheduled {
            let day_name = scheduled.format("%a");
            let mut sched_str = format!("  SCHEDULED: <{} {}", scheduled.format("%Y-%m-%d"), day_name);
            if let Some(ref recurrence) = task.recurrence {
                sched_str.push(' ');
                sched_str.push_str(&recurrence.to_string());
            }
            sched_str.push_str(">\n");
            out.push_str(&sched_str);
        }

        // Properties
        out.push_str("  :PROPERTIES:\n");
        out.push_str(&format!("  :ID: {}\n", task.id));
        out.push_str("  :STYLE: habit\n");
        out.push_str(&format!(
            "  :CREATED: [{}]\n",
            task.created.format("%Y-%m-%d %a %H:%M")
        ));
        out.push_str("  :END:\n");

        // Logbook
        if !completions.is_empty() {
            out.push_str("  :LOGBOOK:\n");
            for completion in completions.iter().rev() {
                out.push_str(&Self::format_logbook_entry(
                    "DONE",
                    "TODO",
                    *completion,
                ));
                out.push('\n');
            }
            out.push_str("  :END:\n");
        }

        out
    }

    /// Write a single note as a standalone org file (for per-file storage).
    pub fn write_note_file(note: &Note) -> String {
        let mut out = String::new();
        out.push_str(&format!("#+TITLE: {}\n\n", note.title));
        out.push_str(&Self::write_note(note));
        out
    }

    /// Write a single note as an org heading.
    pub fn write_note(note: &Note) -> String {
        let mut out = String::new();
        let indent = "  ";

        // Headline with tags
        out.push_str(&format!("* {}", note.title));
        if !note.tags.is_empty() {
            out.push_str(" :");
            out.push_str(&note.tags.join(":"));
            out.push(':');
        }
        out.push('\n');

        // Properties drawer
        out.push_str(&format!("{indent}:PROPERTIES:\n"));
        out.push_str(&format!("{indent}:ID: {}\n", note.id));
        out.push_str(&format!(
            "{indent}:CREATED: [{}]\n",
            note.created.format("%Y-%m-%d %a %H:%M")
        ));
        out.push_str(&format!(
            "{indent}:MODIFIED: [{}]\n",
            note.modified.format("%Y-%m-%d %a %H:%M")
        ));
        if let Some(ref source) = note.source {
            out.push_str(&format!("{indent}:SOURCE: {}\n", source));
        }
        if !note.links.is_empty() {
            let links_str: Vec<String> = note.links.iter().map(|l| l.to_org()).collect();
            out.push_str(&format!("{indent}:LINKS: {}\n", links_str.join(" ")));
        }
        if let Some(ref etag) = note.sync_etag {
            out.push_str(&format!("{indent}:SYNC_ETAG: {}\n", etag));
        }
        out.push_str(&format!("{indent}:END:\n"));

        // Body
        if !note.body.is_empty() {
            for line in note.body.lines() {
                out.push_str(indent);
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::task::{Priority, Task, TaskState};
    use chrono::NaiveDate;

    #[test]
    fn write_simple_task() {
        let task = Task {
            title: "Fix the faucet".to_string(),
            state: TaskState::Todo,
            priority: Some(Priority::A),
            contexts: vec!["@home".to_string()],
            scheduled: Some(NaiveDate::from_ymd_opt(2026, 2, 24).unwrap()),
            ..Task::new("unused")
        };
        let output = OrgWriter::write_task(&task);
        assert!(output.contains("* TODO [#A] Fix the faucet :@home:"));
        assert!(output.contains("SCHEDULED: <2026-02-24 Tue>"));
        assert!(output.contains(":ID:"));
        assert!(output.contains(":CREATED:"));
    }

    #[test]
    fn write_file_roundtrip() {
        let tasks = vec![
            Task::new("First task"),
            Task::new("Second task"),
        ];
        let output = OrgWriter::write_file("Inbox", &tasks);
        assert!(output.starts_with("#+TITLE: Inbox"));
        assert!(output.contains("* TODO First task"));
        assert!(output.contains("* TODO Second task"));
    }
}
