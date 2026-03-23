use std::collections::HashSet;

use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::core::day_plan::DayPlan;
use crate::core::task::{Priority, Task, TaskState};
use crate::fl;
use crate::message::Message;
use crate::ui::{self, Sender};

const BUDGET_PRESETS: &[u32] = &[5, 10, 20, 30, 50, 75, 100];

pub fn daily_planning_view(
    day_plan: &Option<DayPlan>,
    all_tasks: &[Task],
    media_items: &[crate::core::list_item::ListItem],
    shopping_tasks: &[Task],
    contexts: &[String],
    rejected: &HashSet<uuid::Uuid>,
    sender: &Sender,
) -> gtk::Widget {
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

    let spent = day_plan
        .as_ref()
        .map(|dp| dp.spent_spoons)
        .unwrap_or(0);
    let remaining = budget.saturating_sub(spent);

    let content = ui::vbox(24);

    // Section 1: Spoon Budget
    content.append(&ui::title4("Spoon Budget"));
    let budget_row = ui::hbox(8);
    for &preset in BUDGET_PRESETS {
        let btn = if budget == preset {
            ui::suggested_button(&preset.to_string())
        } else {
            ui::standard_button(&preset.to_string())
        };
        {
            let s = sender.clone();
            btn.connect_clicked(move |_| {
                s.emit(Message::SetSpoonBudget(preset));
            });
        }
        budget_row.append(&btn);
    }
    content.append(&budget_row);

    // Section 2: Active Contexts
    content.append(&ui::title4("Active Contexts"));
    let ctx_row = ui::hbox(8);
    for ctx in contexts {
        let is_active = active_contexts.contains(ctx);
        let btn = if is_active {
            ui::suggested_button(ctx)
        } else {
            ui::standard_button(ctx)
        };
        {
            let s = sender.clone();
            let ctx_owned = ctx.clone();
            btn.connect_clicked(move |_| {
                s.emit(Message::TogglePlanContext(ctx_owned.clone()));
            });
        }
        ctx_row.append(&btn);
    }
    content.append(&ctx_row);

    // Section 3: Due / Scheduled Today
    let today = chrono::Local::now().date_naive();
    let due_today: Vec<&Task> = all_tasks
        .iter()
        .filter(|t| {
            !t.state.is_done()
                && (t.scheduled.is_some_and(|d| d <= today)
                    || t.deadline.is_some_and(|d| d <= today))
        })
        .collect();

    if !due_today.is_empty() {
        content.append(&ui::title4(&fl!("planning-due-today")));
        let due_col = ui::vbox(4);
        for task in &due_today {
            let id = task.id;
            let is_confirmed = confirmed_ids.contains(&id);
            let mut label_parts: Vec<String> = Vec::new();
            if task.deadline.is_some_and(|d| d <= today) {
                label_parts.push("deadline".to_string());
            }
            if task.scheduled.is_some_and(|d| d <= today) {
                label_parts.push("scheduled".to_string());
            }
            let date_badge = format!(" ({})", label_parts.join(", "));
            let esc_text = task.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();
            let title_text = format!("{}{}{}", task.title, esc_text, date_badge);

            let r = ui::centered_hbox(8);
            let lbl = ui::body(&title_text);
            lbl.set_hexpand(true);
            r.append(&lbl);

            if is_confirmed {
                let btn = ui::standard_button("Remove");
                {
                    let s = sender.clone();
                    btn.connect_clicked(move |_| {
                        s.emit(Message::UnconfirmTask(id));
                    });
                }
                r.append(&btn);
            } else {
                let btn = ui::suggested_button("Add");
                {
                    let s = sender.clone();
                    btn.connect_clicked(move |_| {
                        s.emit(Message::ConfirmTask(id));
                    });
                }
                r.append(&btn);
            }
            due_col.append(&r);
        }
        content.append(&due_col);
    }

    // Section 4: Suggestions
    let header_text = format!("Suggested tasks ({}/{} spoons spent)", spent, budget);
    content.append(&ui::title4(&header_text));

    let suggestions = build_suggestions(all_tasks, &confirmed_ids, rejected, &active_contexts, remaining, today);

    if suggestions.is_empty() {
        content.append(&ui::body("No more suggestions fit your remaining budget."));
    } else {
        let suggestion_col = ui::vbox(4);
        for task in &suggestions {
            let id = task.id;
            let esc_text = task.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();
            let title_text = format!("{}{}", task.title, esc_text);

            let r = ui::centered_hbox(8);
            let lbl = ui::body(&title_text);
            lbl.set_hexpand(true);
            r.append(&lbl);

            let btn = ui::suggested_button("Add");
            {
                let s = sender.clone();
                btn.connect_clicked(move |_| {
                    s.emit(Message::ConfirmTask(id));
                });
            }
            r.append(&btn);
            suggestion_col.append(&r);
        }
        content.append(&suggestion_col);
    }

    // Section 5: Today's Plan
    content.append(&ui::title4("Today's Plan"));

    if confirmed_ids.is_empty() {
        content.append(&ui::body("No tasks confirmed yet."));
    } else {
        let mut confirmed_tasks: Vec<&Task> = confirmed_ids
            .iter()
            .filter_map(|id| all_tasks.iter().find(|t| t.id == *id))
            .collect();
        confirmed_tasks.sort_by_key(|t| t.esc.unwrap_or(u32::MAX));

        let tasks_col = ui::vbox(4);
        for task in &confirmed_tasks {
            let id = task.id;
            let esc_text = task.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();
            let done_marker = if task.state.is_done() { "[done] " } else { "" };
            let title_text = format!("{}{}{}", done_marker, task.title, esc_text);

            let r = ui::centered_hbox(8);
            let lbl = ui::body(&title_text);
            lbl.set_hexpand(true);
            r.append(&lbl);

            let btn = ui::standard_button("Remove");
            {
                let s = sender.clone();
                btn.connect_clicked(move |_| {
                    s.emit(Message::UnconfirmTask(id));
                });
            }
            r.append(&btn);
            tasks_col.append(&r);
        }
        content.append(&tasks_col);
    }

    // Picked media items
    content.append(&ui::caption("Media"));
    let media_col = ui::vbox(4);
    for item in media_items {
        let id = item.id;
        let is_picked = picked_media.contains(&id);

        let r = ui::centered_hbox(8);
        let lbl = ui::body(&item.title);
        lbl.set_hexpand(true);
        r.append(&lbl);

        if is_picked {
            let btn = ui::standard_button("Remove");
            {
                let s = sender.clone();
                btn.connect_clicked(move |_| {
                    s.emit(Message::UnpickMediaItem(id));
                });
            }
            r.append(&btn);
        } else {
            let btn = ui::standard_button("Add");
            {
                let s = sender.clone();
                btn.connect_clicked(move |_| {
                    s.emit(Message::PickMediaItem(id));
                });
            }
            r.append(&btn);
        }
        media_col.append(&r);
    }
    content.append(&media_col);

    // Picked shopping tasks
    let active_shopping: Vec<&Task> = shopping_tasks.iter().filter(|t| !t.state.is_done()).collect();
    if !active_shopping.is_empty() {
        content.append(&ui::caption("Shopping"));
        let shopping_col = ui::vbox(4);
        for task in &active_shopping {
            let id = task.id;
            let is_picked = picked_shopping.contains(&id);

            let r = ui::centered_hbox(8);
            let lbl = ui::body(&task.title);
            lbl.set_hexpand(true);
            r.append(&lbl);

            if is_picked {
                let btn = ui::standard_button("Remove");
                {
                    let s = sender.clone();
                    btn.connect_clicked(move |_| {
                        s.emit(Message::UnpickShoppingItem(id));
                    });
                }
                r.append(&btn);
            } else {
                let btn = ui::standard_button("Add");
                {
                    let s = sender.clone();
                    btn.connect_clicked(move |_| {
                        s.emit(Message::PickShoppingItem(id));
                    });
                }
                r.append(&btn);
            }
            shopping_col.append(&r);
        }
        content.append(&shopping_col);
    }

    ui::page_wrapper(&content).upcast()
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
            // Must be eligible: NEXT, scheduled/deadline <= today, or habit
            let is_habit = t.extra_tags.iter().any(|tag| tag == "habit")
                || t.recurrence.is_some();
            let eligible = t.state == TaskState::Next
                || t.scheduled.is_some_and(|d| d <= today)
                || t.deadline.is_some_and(|d| d <= today)
                || is_habit;
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
