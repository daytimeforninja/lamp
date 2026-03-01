use std::collections::HashSet;

use chrono::Duration;
use cosmic::iced::Length;
use cosmic::widget::{checkbox, column, container, scrollable, text};
use cosmic::Element;

use crate::core::habit::Habit;
use crate::core::project::Project;
use crate::core::task::{Task, TaskState};
use crate::fl;
use crate::message::Message;

/// A task is "unprocessed" (belongs in inbox) if it's still Todo
/// and has none of priority, contexts, project, or ESC set.
fn is_unprocessed(task: &Task) -> bool {
    task.state == TaskState::Todo
        && task.priority.is_none()
        && task.contexts.is_empty()
        && task.project.is_none()
        && task.esc.is_none()
}

pub fn review_view(
    all_tasks: &[Task],
    projects: &[Project],
    habits: &[Habit],
    checked: &HashSet<usize>,
) -> Element<'static, Message> {
    let mut content = column().spacing(16);
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

    // ── Phase 1: Get Clear ──
    content = content.push(text::title3(fl!("review-phase-clear")));

    // Step 0: Process inbox
    content = content.push(review_step(0, &fl!("review-step-inbox"), checked));
    content = content.push(
        text::body(fl!(
            "review-inbox-count",
            count = inbox_count.to_string()
        ))
        .size(13.0),
    );

    // Step 1: Review captured notes
    content = content.push(review_step(1, &fl!("review-step-notes"), checked));

    // ── Phase 2: Get Current ──
    content = content.push(text::title3(fl!("review-phase-current")));

    // Step 2: Review Next Actions
    content = content.push(review_step(2, &fl!("review-step-next"), checked));
    content = content.push(
        text::body(format!("{} next actions", next_count)).size(13.0),
    );

    // Step 3: Review Waiting For
    content = content.push(review_step(3, &fl!("review-step-waiting"), checked));
    if !waiting_tasks.is_empty() {
        let mut waiting_col = column().spacing(2).padding([0, 0, 0, 28]);
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
            waiting_col = waiting_col.push(text::body(label).size(13.0));
        }
        content = content.push(waiting_col);
    }

    // Step 4: Review projects for next actions
    content = content.push(review_step(4, &fl!("review-step-projects"), checked));
    if !stuck_projects.is_empty() {
        let mut stuck_col = column().spacing(2).padding([0, 0, 0, 28]);
        for project in &stuck_projects {
            stuck_col = stuck_col.push(
                text::body(format!("{} — stuck", project.name)).size(13.0),
            );
        }
        content = content.push(stuck_col);
    }

    // Step 5: Review Someday/Maybe
    content = content.push(review_step(5, &fl!("review-step-someday"), checked));
    if !someday_tasks.is_empty() {
        let mut someday_col = column().spacing(2).padding([0, 0, 0, 28]);
        for task in &someday_tasks {
            someday_col = someday_col.push(text::body(task.title.clone()).size(13.0));
        }
        content = content.push(someday_col);
    }

    // ── Phase 3: Get Creative ──
    content = content.push(text::title3(fl!("review-phase-creative")));

    // Step 6: Review upcoming calendar
    content = content.push(review_step(6, &fl!("review-step-calendar"), checked));
    if !upcoming_tasks.is_empty() {
        let mut upcoming_col = column().spacing(2).padding([0, 0, 0, 28]);
        for task in &upcoming_tasks {
            let date = task.scheduled.or(task.deadline).unwrap();
            let label = format!("{} — {}", date.format("%b %d"), task.title);
            upcoming_col = upcoming_col.push(text::body(label).size(13.0));
        }
        content = content.push(upcoming_col);
    }

    // Step 7: Capture new ideas
    content = content.push(review_step(7, &fl!("review-step-capture"), checked));

    // Step 8: Review goals and horizons
    content = content.push(review_step(8, &fl!("review-step-horizons"), checked));

    // Habit completion this week (informational, not a checklist step)
    if !habits.is_empty() {
        let mut habit_section = column().spacing(2);
        habit_section = habit_section.push(text::title4(fl!("review-habits-week")));
        let week_start = today - Duration::days(7);
        for habit in habits {
            let completions_this_week = habit
                .completions
                .iter()
                .filter(|dt| **dt >= week_start)
                .count();
            let label = format!("{}: {}/7", habit.task.title, completions_this_week);
            habit_section = habit_section.push(text::body(label).size(13.0));
        }
        content = content.push(habit_section);
    }

    // Completion message
    let checked_count = checked.len();
    if checked_count == total_steps {
        content = content.push(text::title4(fl!("review-complete")));
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn review_step(idx: usize, label: &str, checked: &HashSet<usize>) -> Element<'static, Message> {
    let is_checked = checked.contains(&idx);
    let step_idx = idx;
    checkbox(label.to_string(), is_checked)
        .on_toggle(move |_| Message::ToggleReviewStep(step_idx))
        .into()
}
