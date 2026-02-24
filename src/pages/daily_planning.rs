use std::collections::HashSet;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, row, scrollable, text};
use cosmic::Element;

use crate::core::day_plan::DayPlan;
use crate::core::list_item::ListItem;
use crate::core::task::{Priority, Task, TaskState};
use crate::message::Message;

const BUDGET_PRESETS: &[u32] = &[5, 10, 20, 30, 50, 75, 100];

pub fn daily_planning_view<'a>(
    day_plan: &Option<DayPlan>,
    all_tasks: &[Task],
    media_items: &[ListItem],
    shopping_items: &[ListItem],
    contexts: &[String],
    rejected: &HashSet<uuid::Uuid>,
) -> Element<'a, Message> {
    let budget = day_plan.as_ref().map(|dp| dp.spoon_budget).unwrap_or(50);
    let active_contexts: Vec<String> = day_plan
        .as_ref()
        .map(|dp| dp.active_contexts.clone())
        .unwrap_or_default();
    let confirmed_ids: Vec<uuid::Uuid> = day_plan
        .as_ref()
        .map(|dp| dp.confirmed_task_ids.clone())
        .unwrap_or_default();
    let picked_media: Vec<uuid::Uuid> = day_plan
        .as_ref()
        .map(|dp| dp.picked_media_ids.clone())
        .unwrap_or_default();
    let picked_shopping: Vec<uuid::Uuid> = day_plan
        .as_ref()
        .map(|dp| dp.picked_shopping_ids.clone())
        .unwrap_or_default();

    let committed = day_plan
        .as_ref()
        .map(|dp| dp.committed_esc(all_tasks))
        .unwrap_or(0);
    let remaining = budget.saturating_sub(committed);

    let mut content = column().spacing(24).padding(16).width(Length::Fill);

    // Section 1: Spoon Budget
    content = content.push(text::title4("Spoon Budget"));
    let mut budget_row = row().spacing(8);
    for &preset in BUDGET_PRESETS {
        let btn = if budget == preset {
            button::suggested(preset.to_string())
        } else {
            button::standard(preset.to_string())
        };
        budget_row = budget_row.push(btn.on_press(Message::SetSpoonBudget(preset)));
    }
    content = content.push(budget_row);

    // Section 2: Active Contexts
    content = content.push(text::title4("Active Contexts"));
    let mut ctx_row = row().spacing(8);
    for ctx in contexts {
        let is_active = active_contexts.contains(ctx);
        let btn = if is_active {
            button::suggested(ctx.clone())
        } else {
            button::standard(ctx.clone())
        };
        let ctx_owned = ctx.clone();
        ctx_row = ctx_row.push(btn.on_press(Message::TogglePlanContext(ctx_owned)));
    }
    content = content.push(ctx_row);

    // Section 3: Suggestions
    let header_text = format!("Suggested tasks ({}/{} spoons committed)", committed, budget);
    content = content.push(text::title4(header_text));

    let today = chrono::Local::now().date_naive();
    let suggestions = build_suggestions(all_tasks, &confirmed_ids, rejected, &active_contexts, remaining, today);

    if suggestions.is_empty() {
        content = content.push(text::body("No more suggestions fit your remaining budget."));
    } else {
        let mut suggestion_col = column().spacing(4);
        for task in &suggestions {
            let id = task.id;
            let esc_text = task.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();
            let title_text = format!("{}{}", task.title, esc_text);

            let r = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::body(title_text).width(Length::Fill))
                .push(
                    button::suggested("Confirm")
                        .on_press(Message::ConfirmTask(id)),
                )
                .push(
                    button::standard("Skip")
                        .on_press(Message::RejectSuggestion(id)),
                );
            suggestion_col = suggestion_col.push(r);
        }
        content = content.push(suggestion_col);
    }

    // Section 4: Today's Plan
    content = content.push(text::title4("Today's Plan"));

    // Confirmed tasks
    if confirmed_ids.is_empty() {
        content = content.push(text::body("No tasks confirmed yet."));
    } else {
        let mut tasks_col = column().spacing(4);
        for task_id in &confirmed_ids {
            if let Some(task) = all_tasks.iter().find(|t| t.id == *task_id) {
                let id = task.id;
                let esc_text = task.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();
                let done_marker = if task.state.is_done() { "[done] " } else { "" };
                let title_text = format!("{}{}{}", done_marker, task.title, esc_text);

                let r = row()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(text::body(title_text).width(Length::Fill))
                    .push(
                        button::standard("Remove")
                            .on_press(Message::UnconfirmTask(id)),
                    );
                tasks_col = tasks_col.push(r);
            }
        }
        content = content.push(tasks_col);
    }

    // Picked media items
    content = content.push(text::caption("Media"));
    let mut media_col = column().spacing(4);
    for item in media_items {
        let id = item.id;
        let is_picked = picked_media.contains(&id);
        let btn = if is_picked {
            row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::body(item.title.clone()).width(Length::Fill))
                .push(button::standard("Remove").on_press(Message::UnpickMediaItem(id)))
        } else {
            row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::body(item.title.clone()).width(Length::Fill))
                .push(button::standard("Add").on_press(Message::PickMediaItem(id)))
        };
        media_col = media_col.push(btn);
    }
    content = content.push(media_col);

    // Picked shopping items
    content = content.push(text::caption("Shopping"));
    let mut shopping_col = column().spacing(4);
    for item in shopping_items {
        let id = item.id;
        let is_picked = picked_shopping.contains(&id);
        let btn = if is_picked {
            row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::body(item.title.clone()).width(Length::Fill))
                .push(button::standard("Remove").on_press(Message::UnpickShoppingItem(id)))
        } else {
            row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::body(item.title.clone()).width(Length::Fill))
                .push(button::standard("Add").on_press(Message::PickShoppingItem(id)))
        };
        shopping_col = shopping_col.push(btn);
    }
    content = content.push(shopping_col);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn build_suggestions(
    all_tasks: &[Task],
    confirmed_ids: &[uuid::Uuid],
    rejected: &HashSet<uuid::Uuid>,
    active_contexts: &[String],
    remaining_budget: u32,
    today: chrono::NaiveDate,
) -> Vec<Task> {
    let mut candidates: Vec<Task> = all_tasks
        .iter()
        .filter(|t| {
            // Must be eligible: NEXT, or scheduled/deadline <= today
            let eligible = t.state == TaskState::Next
                || t.scheduled.is_some_and(|d| d <= today)
                || t.deadline.is_some_and(|d| d <= today);
            if !eligible || t.state.is_done() {
                return false;
            }
            // Not already confirmed or rejected
            if confirmed_ids.contains(&t.id) || rejected.contains(&t.id) {
                return false;
            }
            // Context filter: no contexts on task OR at least one matches
            if !t.contexts.is_empty() && !active_contexts.is_empty() {
                if !t.contexts.iter().any(|c| active_contexts.contains(c)) {
                    return false;
                }
            }
            // ESC fits remaining budget (None always eligible)
            match t.esc {
                Some(esc) => esc <= remaining_budget,
                None => true,
            }
        })
        .cloned()
        .collect();

    // Sort: overdue/scheduled first, then by priority (A>B>C>none), then ESC ascending
    candidates.sort_by(|a, b| {
        let a_urgent = a.scheduled.is_some_and(|d| d <= today) || a.deadline.is_some_and(|d| d <= today);
        let b_urgent = b.scheduled.is_some_and(|d| d <= today) || b.deadline.is_some_and(|d| d <= today);
        b_urgent.cmp(&a_urgent)
            .then_with(|| priority_rank(a.priority).cmp(&priority_rank(b.priority)))
            .then_with(|| a.esc.unwrap_or(0).cmp(&b.esc.unwrap_or(0)))
    });

    candidates
}

fn priority_rank(p: Option<Priority>) -> u8 {
    match p {
        Some(Priority::A) => 0,
        Some(Priority::B) => 1,
        Some(Priority::C) => 2,
        None => 3,
    }
}
