use chrono::Duration;
use cosmic::iced::Length;
use cosmic::widget::{column, container, scrollable, text};
use cosmic::Element;

use crate::core::habit::Habit;
use crate::core::project::Project;
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

pub fn review_view(
    inbox_tasks: &[Task],
    next_tasks: &[Task],
    waiting_tasks: &[Task],
    projects: &[Project],
    habits: &[Habit],
) -> Element<'static, Message> {
    let mut content = column().spacing(16);
    let today = chrono::Local::now().naive_local();

    // Inbox count
    let inbox_count = inbox_tasks.len();
    content = content.push(text::title4(fl!(
        "review-inbox-count",
        count = inbox_count.to_string()
    )));

    // Stale tasks (created >7 days ago, still active)
    let seven_days_ago = today - Duration::days(7);
    let stale_tasks: Vec<&Task> = next_tasks
        .iter()
        .chain(waiting_tasks.iter())
        .filter(|t| t.state.is_active() && t.created < seven_days_ago)
        .collect();

    if !stale_tasks.is_empty() {
        let mut section = column().spacing(4);
        section = section.push(text::title4(fl!("review-stale-tasks")));
        for task in &stale_tasks {
            section = section.push(text::body(task.title.clone()));
        }
        content = content.push(section);
    }

    // Stuck projects (no next action)
    let stuck_projects: Vec<&Project> = projects.iter().filter(|p| p.is_stuck()).collect();
    if !stuck_projects.is_empty() {
        let mut section = column().spacing(4);
        section = section.push(text::title4(fl!("review-no-next")));
        for project in &stuck_projects {
            section = section.push(text::body(project.name.clone()));
        }
        content = content.push(section);
    }

    // Habit completion rate this week
    if !habits.is_empty() {
        let mut section = column().spacing(4);
        section = section.push(text::title4(fl!("review-habits-week")));
        let week_start = today - Duration::days(7);
        for habit in habits {
            let completions_this_week = habit
                .completions
                .iter()
                .filter(|dt| **dt >= week_start)
                .count();
            let label = format!("{}: {}/7", habit.task.title, completions_this_week);
            section = section.push(text::body(label));
        }
        content = content.push(section);
    }

    // Actionable prompt
    let prompt = if inbox_count > 0 {
        fl!("review-prompt-inbox")
    } else if !stuck_projects.is_empty() {
        fl!("review-prompt-stuck")
    } else {
        fl!("review-prompt-good")
    };
    content = content.push(text::body(prompt));

    container(scrollable(content.padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
