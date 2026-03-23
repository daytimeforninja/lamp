use std::collections::HashMap;

use chrono::NaiveDateTime;
use relm4::gtk;
use relm4::gtk::prelude::*;
use uuid::Uuid;

use crate::core::day_plan::DayPlan;
use crate::core::habit::Habit;
use crate::core::list_item::ListItem;
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;
use crate::ui::{self, Sender};

fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {:02}m", h, m)
    } else {
        format!("{}m {:02}s", m, s)
    }
}

pub fn do_mode_view(
    day_plan: &Option<DayPlan>,
    all_tasks: &[Task],
    habits: &[Habit],
    media_items: &[ListItem],
    shopping_tasks: &[Task],
    expanded_task: Option<Uuid>,
    note_inputs: &HashMap<Uuid, String>,
    active_timer: Option<(Uuid, NaiveDateTime)>,
    sender: &Sender,
) -> gtk::Widget {
    let today = chrono::Local::now().date_naive();
    let now = chrono::Local::now().naive_local();

    let Some(plan) = day_plan else {
        return ui::status_page("media-playback-start-symbolic", &fl!("do-empty"), "Plan your day first, then switch to Do mode").upcast();
    };

    let remaining = plan.remaining_budget();
    let budget = plan.spoon_budget;

    let content = ui::vbox(24);

    // Spoon meter
    let meter_text = fl!(
        "do-spoons-remaining",
        remaining = remaining.to_string(),
        budget = budget.to_string()
    );
    content.append(&ui::title3(&meter_text));

    // Tasks section — active confirmed tasks (sorted by ESC ascending, None last)
    let mut confirmed_tasks: Vec<&Task> = plan
        .confirmed_task_ids
        .iter()
        .filter_map(|id| all_tasks.iter().find(|t| t.id == *id && !t.state.is_done()))
        .collect();
    confirmed_tasks.sort_by_key(|t| t.esc.unwrap_or(u32::MAX));

    let has_completed = !plan.completed_tasks.is_empty();

    if !confirmed_tasks.is_empty() || has_completed {
        content.append(&ui::title4(&fl!("do-tasks")));
        let tasks_col = ui::vbox(4);

        // Active tasks
        for task in &confirmed_tasks {
            let id = task.id;
            let esc_text = task.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();
            let is_active = active_timer.map(|(tid, _)| tid == id).unwrap_or(false);

            // Total work time (completed sessions + current active if running)
            let mut total_secs = task.total_work_secs();
            if let Some((tid, start)) = active_timer {
                if tid == id {
                    total_secs += (now - start).num_seconds().max(0);
                }
            }

            let title_text = if is_active {
                let elapsed = active_timer
                    .map(|(_, start)| (now - start).num_seconds().max(0))
                    .unwrap_or(0);
                format!(">> {} {} — {}", task.title, esc_text, format_duration(elapsed))
            } else {
                format!("{}{}", task.title, esc_text)
            };

            let r = ui::centered_hbox(8);

            // Checkbox
            let cb = ui::check_button_with_signal(false, Message::DoMarkDone(id), sender);
            r.append(&cb);

            // Play/stop timer button
            let timer_icon_name = if is_active {
                "media-playback-stop-symbolic"
            } else {
                "media-playback-start-symbolic"
            };
            let timer_btn = ui::icon_button_with_signal(timer_icon_name, Message::ToggleWorkTimer(id), sender);
            r.append(&timer_btn);

            // Task title
            let title_widget = ui::body(&title_text);
            title_widget.set_hexpand(true);
            r.append(&title_widget);

            // Show total tracked time if any
            if total_secs > 0 {
                r.append(&ui::caption(&format_duration(total_secs)));
            }

            // Expand/notes button
            let expand_btn = ui::icon_button_with_signal(
                "accessories-text-editor-symbolic",
                Message::ToggleTaskExpand(id),
                sender,
            );
            r.append(&expand_btn);

            let task_col = ui::vbox(4);
            task_col.append(&r);

            if expanded_task == Some(id) {
                let notes_col = ui::vbox(4);
                notes_col.set_margin_start(36);
                notes_col.set_margin_top(4);
                notes_col.set_margin_bottom(4);

                if !task.notes.is_empty() {
                    let notes_label = ui::body(&task.notes);
                    notes_label.set_hexpand(true);
                    notes_label.set_margin_start(8);
                    notes_label.set_margin_end(8);
                    notes_col.append(&notes_label);
                }

                let input_value = note_inputs.get(&id).cloned().unwrap_or_default();
                let note_input = ui::entry_with_signal(
                    &fl!("task-note-placeholder"),
                    &input_value,
                    move |v| Message::NoteInputChanged(id, v),
                    Some(Box::new(move || Message::AppendNote(id))),
                    sender,
                );
                notes_col.append(&note_input);

                task_col.append(&notes_col);
            }

            tasks_col.append(&task_col);
        }

        // Completed tasks (shown as checked)
        for ct in &plan.completed_tasks {
            let id = ct.id;
            let esc_text = ct.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();

            // Show total tracked time for completed tasks too
            let total_secs = all_tasks
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.total_work_secs())
                .unwrap_or(0);
            let time_text = if total_secs > 0 {
                format!("  ({})", format_duration(total_secs))
            } else {
                String::new()
            };

            let r = ui::centered_hbox(8);
            let cb = ui::check_button_with_signal(true, Message::DoMarkDone(id), sender);
            r.append(&cb);

            let lbl = ui::caption(&format!("{}{}{}", ct.title, esc_text, time_text));
            lbl.set_hexpand(true);
            r.append(&lbl);

            tasks_col.append(&r);
        }

        content.append(&tasks_col);
    }

    // Habits section — show due + completed-today habits
    let due_habits: Vec<&Habit> = habits.iter().filter(|h| h.is_due(today)).collect();
    let done_habits: Vec<&Habit> = habits.iter().filter(|h| {
        !h.is_due(today) && h.completions.iter().any(|c| c.date() == today)
    }).collect();

    if !due_habits.is_empty() || !done_habits.is_empty() {
        content.append(&ui::title4(&fl!("do-habits")));
        let habits_col = ui::vbox(4);
        for habit in &due_habits {
            let id = habit.task.id;
            let r = ui::centered_hbox(8);
            let cb = ui::check_button_with_signal(false, Message::CompleteHabit(id), sender);
            r.append(&cb);
            let lbl = ui::body(&habit.task.title);
            lbl.set_hexpand(true);
            r.append(&lbl);
            habits_col.append(&r);
        }
        for habit in &done_habits {
            let r = ui::centered_hbox(8);
            let cb = gtk::CheckButton::new();
            cb.set_active(true);
            cb.set_sensitive(false);
            r.append(&cb);
            let lbl = ui::caption(&habit.task.title);
            lbl.set_hexpand(true);
            r.append(&lbl);
            habits_col.append(&r);
        }
        content.append(&habits_col);
    }

    // Media items section
    let picked_media: Vec<&ListItem> = plan
        .picked_media_ids
        .iter()
        .filter_map(|id| media_items.iter().find(|i| i.id == *id))
        .collect();

    if !picked_media.is_empty() {
        content.append(&ui::title4(&fl!("do-media")));
        let media_col = ui::vbox(4);
        for item in &picked_media {
            let id = item.id;
            let r = ui::centered_hbox(8);
            let cb = ui::check_button_with_signal(false, Message::DoMarkListItemDone(id), sender);
            r.append(&cb);
            let lbl = ui::body(&item.title);
            lbl.set_hexpand(true);
            r.append(&lbl);
            media_col.append(&r);
        }
        content.append(&media_col);
    }

    // Shopping tasks section
    let picked_shopping: Vec<&Task> = plan
        .picked_shopping_ids
        .iter()
        .filter_map(|id| shopping_tasks.iter().find(|t| t.id == *id))
        .filter(|t| !t.state.is_done())
        .collect();

    if !picked_shopping.is_empty() {
        content.append(&ui::title4(&fl!("do-shopping")));
        let shopping_col = ui::vbox(4);
        for task in &picked_shopping {
            let id = task.id;
            let r = ui::centered_hbox(8);
            let cb = ui::check_button_with_signal(false, Message::DoMarkListItemDone(id), sender);
            r.append(&cb);
            let lbl = ui::body(&task.title);
            lbl.set_hexpand(true);
            r.append(&lbl);
            shopping_col.append(&r);
        }
        content.append(&shopping_col);
    }

    // Empty state if no items at all
    if confirmed_tasks.is_empty() && due_habits.is_empty() && picked_media.is_empty() && picked_shopping.is_empty() {
        content.append(&ui::status_page("media-playback-start-symbolic", &fl!("do-plan-empty"), "Add tasks to your day plan to see them here"));
    }

    ui::page_wrapper(&content).upcast()
}
