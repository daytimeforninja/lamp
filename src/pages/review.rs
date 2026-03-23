use std::collections::HashSet;

use chrono::Duration;
use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::core::habit::Habit;
use crate::core::project::Project;
use crate::core::task::{Task, TaskState};
use crate::fl;
use crate::message::Message;
use crate::ui::{self, Sender};

/// A task is "unprocessed" (belongs in inbox) if it's Todo without a project.
/// This matches the inbox page's `is_inbox_task` definition.
fn is_unprocessed(task: &Task) -> bool {
    task.state == TaskState::Todo && task.project.is_none()
}

pub fn review_view(
    all_tasks: &[Task],
    projects: &[Project],
    habits: &[Habit],
    checked: &HashSet<usize>,
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(16);
    let today = chrono::Local::now().naive_local();
    let today_date = chrono::Local::now().date_naive();

    // Precompute data used by multiple steps
    let inbox_count = all_tasks.iter().filter(|t| is_unprocessed(t)).count();

    let next_count = all_tasks
        .iter()
        .filter(|t| t.state == TaskState::Next)
        .count();

    let waiting_tasks: Vec<&Task> = all_tasks
        .iter()
        .filter(|t| t.state == TaskState::Waiting)
        .collect();

    let stuck_projects: Vec<&Project> = projects.iter().filter(|p| p.is_stuck()).collect();

    let someday_tasks: Vec<&Task> = all_tasks
        .iter()
        .filter(|t| t.state == TaskState::Someday)
        .collect();

    let fourteen_days = today_date + Duration::days(14);
    let mut upcoming_tasks: Vec<&Task> = all_tasks
        .iter()
        .filter(|t| {
            if t.state.is_done() {
                return false;
            }
            let sched_in_range = t
                .scheduled
                .is_some_and(|d| d >= today_date && d <= fourteen_days);
            let dead_in_range = t
                .deadline
                .is_some_and(|d| d >= today_date && d <= fourteen_days);
            sched_in_range || dead_in_range
        })
        .collect();
    upcoming_tasks.sort_by_key(|t| {
        let s = t.scheduled.unwrap_or(chrono::NaiveDate::MAX);
        let d = t.deadline.unwrap_or(chrono::NaiveDate::MAX);
        s.min(d)
    });

    let total_steps: usize = 9;

    // -- Phase 1: Get Clear --
    content.append(&ui::title3(&fl!("review-phase-clear")));

    // Step 0: Process inbox
    content.append(&review_step(0, &fl!("review-step-inbox"), checked, sender));
    let inbox_label = ui::caption(&fl!(
        "review-inbox-count",
        count = inbox_count.to_string()
    ));
    content.append(&inbox_label);

    // Step 1: Review captured notes
    content.append(&review_step(1, &fl!("review-step-notes"), checked, sender));

    // -- Phase 2: Get Current --
    content.append(&ui::title3(&fl!("review-phase-current")));

    // Step 2: Review Next Actions
    content.append(&review_step(2, &fl!("review-step-next"), checked, sender));
    let next_label = ui::caption(&format!("{} next actions", next_count));
    content.append(&next_label);

    // Step 3: Review Waiting For
    content.append(&review_step(3, &fl!("review-step-waiting"), checked, sender));
    if !waiting_tasks.is_empty() {
        let waiting_col = ui::vbox(2);
        waiting_col.set_margin_start(28);
        for task in &waiting_tasks {
            let days = (today - task.created).num_days();
            let mut label = fl!(
                "review-waiting-age",
                title = task.title.clone(),
                days = days.to_string()
            );
            if let Some(ref wf) = task.waiting_for {
                label = format!("{} (@{})", label, wf);
            }
            if let Some(follow_up) = task.follow_up {
                let overdue = follow_up < today_date;
                let fu_label = if overdue {
                    format!(" [OVERDUE follow-up: {}]", follow_up.format("%Y-%m-%d"))
                } else {
                    format!(" [follow-up: {}]", follow_up.format("%Y-%m-%d"))
                };
                label.push_str(&fu_label);
            }
            waiting_col.append(&ui::caption(&label));
        }
        content.append(&waiting_col);
    }

    // Step 4: Review projects for next actions
    content.append(&review_step(4, &fl!("review-step-projects"), checked, sender));
    if !stuck_projects.is_empty() {
        let stuck_col = ui::vbox(2);
        stuck_col.set_margin_start(28);
        for project in &stuck_projects {
            stuck_col.append(&ui::caption(&format!("{} — stuck", project.name)));
        }
        content.append(&stuck_col);
    }

    // Step 5: Review Someday/Maybe
    content.append(&review_step(5, &fl!("review-step-someday"), checked, sender));
    if !someday_tasks.is_empty() {
        let someday_col = ui::vbox(2);
        someday_col.set_margin_start(28);
        for task in &someday_tasks {
            someday_col.append(&ui::caption(&task.title));
        }
        content.append(&someday_col);
    }

    // -- Phase 3: Get Creative --
    content.append(&ui::title3(&fl!("review-phase-creative")));

    // Step 6: Review upcoming calendar
    content.append(&review_step(6, &fl!("review-step-calendar"), checked, sender));
    if !upcoming_tasks.is_empty() {
        let upcoming_col = ui::vbox(2);
        upcoming_col.set_margin_start(28);
        for task in &upcoming_tasks {
            let date = match task.scheduled.or(task.deadline) {
                Some(d) => d,
                None => continue,
            };
            let label = format!("{} — {}", date.format("%b %d"), task.title);
            upcoming_col.append(&ui::caption(&label));
        }
        content.append(&upcoming_col);
    }

    // Step 7: Capture new ideas
    content.append(&review_step(7, &fl!("review-step-capture"), checked, sender));

    // Step 8: Review goals and horizons
    content.append(&review_step(8, &fl!("review-step-horizons"), checked, sender));

    // Habit completion this week (informational, not a checklist step)
    if !habits.is_empty() {
        let habit_section = ui::vbox(2);
        habit_section.append(&ui::title4(&fl!("review-habits-week")));
        let week_start = today - Duration::days(7);
        for habit in habits {
            let completions_this_week = habit
                .completions
                .iter()
                .filter(|dt| **dt >= week_start)
                .count();
            let label = format!("{}: {}/7", habit.task.title, completions_this_week);
            habit_section.append(&ui::caption(&label));
        }
        content.append(&habit_section);
    }

    // Completion message
    let checked_count = checked.len();
    if checked_count == total_steps {
        content.append(&ui::title4(&fl!("review-complete")));
    }

    ui::page_wrapper(&content).upcast()
}

fn review_step(idx: usize, label: &str, checked: &HashSet<usize>, sender: &Sender) -> gtk::Box {
    let is_checked = checked.contains(&idx);
    let row = ui::centered_hbox(8);
    let cb = ui::check_button_with_signal(is_checked, Message::ToggleReviewStep(idx), sender);
    cb.set_label(Some(label));
    row.append(&cb);
    row
}
