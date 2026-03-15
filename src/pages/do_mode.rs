use std::collections::HashMap;

use chrono::NaiveDateTime;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use cosmic::Element;

use crate::core::day_plan::DayPlan;
use crate::core::habit::Habit;
use crate::core::list_item::ListItem;
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;

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

pub fn do_mode_view<'a>(
    day_plan: &Option<DayPlan>,
    all_tasks: &[Task],
    habits: &[Habit],
    media_items: &[ListItem],
    shopping_items: &[ListItem],
    expanded_task: Option<uuid::Uuid>,
    note_inputs: &HashMap<uuid::Uuid, String>,
    active_timer: Option<(uuid::Uuid, NaiveDateTime)>,
) -> Element<'a, Message> {
    let today = chrono::Local::now().date_naive();
    let now = chrono::Local::now().naive_local();

    let Some(plan) = day_plan else {
        return container(
            column()
                .spacing(12)
                .push(text::title3(fl!("do-empty")))
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .center(Length::Fill)
        .into();
    };

    let remaining = plan.remaining_budget();
    let budget = plan.spoon_budget;

    let mut content = column().spacing(24).padding(16).width(Length::Fill);

    // Spoon meter
    let meter_text = fl!(
        "do-spoons-remaining",
        remaining = remaining.to_string(),
        budget = budget.to_string()
    );
    content = content.push(text::title3(meter_text));

    // Tasks section — active confirmed tasks (sorted by ESC ascending, None last)
    let mut confirmed_tasks: Vec<&Task> = plan
        .confirmed_task_ids
        .iter()
        .filter_map(|id| all_tasks.iter().find(|t| t.id == *id && !t.state.is_done()))
        .collect();
    confirmed_tasks.sort_by_key(|t| t.esc.unwrap_or(u32::MAX));

    let has_completed = !plan.completed_tasks.is_empty();

    if !confirmed_tasks.is_empty() || has_completed {
        content = content.push(text::title4(fl!("do-tasks")));
        let mut tasks_col = column().spacing(4);

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

            let mut r = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    checkbox("", false)
                        .on_toggle(move |_| Message::DoMarkDone(id)),
                );

            // Play/stop timer button
            let timer_icon_name = if is_active {
                "media-playback-stop-symbolic"
            } else {
                "media-playback-start-symbolic"
            };
            let timer_btn = button::icon(
                cosmic::widget::icon::from_name(timer_icon_name),
            )
            .on_press(Message::ToggleWorkTimer(id));
            r = r.push(timer_btn);

            // Task title
            let title_widget = if is_active {
                text::body(title_text).width(Length::Fill)
            } else {
                text::body(title_text).width(Length::Fill)
            };
            r = r.push(title_widget);

            // Show total tracked time if any
            if total_secs > 0 {
                r = r.push(cosmic::widget::horizontal_space());
                r = r.push(text::caption(format_duration(total_secs)));
            } else {
                r = r.push(cosmic::widget::horizontal_space());
            }

            r = r.push(
                button::icon(cosmic::widget::icon::from_name("accessories-text-editor-symbolic"))
                    .on_press(Message::ToggleTaskExpand(id)),
            );

            let mut task_col = column().spacing(4);
            task_col = task_col.push(r);

            if expanded_task == Some(id) {
                let mut notes_col = column().spacing(4).padding([4, 0, 4, 36]);

                if !task.notes.is_empty() {
                    notes_col = notes_col.push(
                        container(text::body(task.notes.clone()))
                            .padding([4, 8])
                            .width(Length::Fill),
                    );
                }

                let input_value = note_inputs.get(&id).cloned().unwrap_or_default();
                let note_input = text_input::text_input(fl!("task-note-placeholder"), input_value)
                    .on_input(move |v| Message::NoteInputChanged(id, v))
                    .on_submit(move |_| Message::AppendNote(id))
                    .width(Length::Fill);
                notes_col = notes_col.push(note_input);

                task_col = task_col.push(notes_col);
            }

            tasks_col = tasks_col.push(task_col);
        }

        // Completed tasks (shown as checked, clickable to un-complete)
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

            let r = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    checkbox("", true)
                        .on_toggle(move |_| Message::DoMarkDone(id)),
                )
                .push(text::caption(format!("{}{}{}", ct.title, esc_text, time_text)).width(Length::Fill));
            tasks_col = tasks_col.push(r);
        }

        content = content.push(tasks_col);
    }

    // Habits section — show all due (incomplete today) habits
    let due_habits: Vec<&Habit> = habits.iter().filter(|h| h.is_due(today)).collect();

    if !due_habits.is_empty() {
        content = content.push(text::title4(fl!("do-habits")));
        let mut habits_col = column().spacing(4);
        for habit in &due_habits {
            let id = habit.task.id;
            let r = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    checkbox("", false)
                        .on_toggle(move |_| Message::CompleteHabit(id)),
                )
                .push(text::body(habit.task.title.clone()).width(Length::Fill));
            habits_col = habits_col.push(r);
        }
        content = content.push(habits_col);
    }

    // Media items section
    let picked_media: Vec<&ListItem> = plan
        .picked_media_ids
        .iter()
        .filter_map(|id| media_items.iter().find(|i| i.id == *id))
        .collect();

    if !picked_media.is_empty() {
        content = content.push(text::title4(fl!("do-media")));
        let mut media_col = column().spacing(4);
        for item in &picked_media {
            let id = item.id;
            let r = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    checkbox("", false)
                        .on_toggle(move |_| Message::DoMarkListItemDone(id)),
                )
                .push(text::body(item.title.clone()).width(Length::Fill));
            media_col = media_col.push(r);
        }
        content = content.push(media_col);
    }

    // Shopping items section
    let picked_shopping: Vec<&ListItem> = plan
        .picked_shopping_ids
        .iter()
        .filter_map(|id| shopping_items.iter().find(|i| i.id == *id))
        .collect();

    if !picked_shopping.is_empty() {
        content = content.push(text::title4(fl!("do-shopping")));
        let mut shopping_col = column().spacing(4);
        for item in &picked_shopping {
            let id = item.id;
            let r = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    checkbox("", false)
                        .on_toggle(move |_| Message::DoMarkListItemDone(id)),
                )
                .push(text::body(item.title.clone()).width(Length::Fill));
            shopping_col = shopping_col.push(r);
        }
        content = content.push(shopping_col);
    }

    // Empty state if no items at all
    if confirmed_tasks.is_empty() && due_habits.is_empty() && picked_media.is_empty() && picked_shopping.is_empty() {
        content = content.push(text::body(fl!("do-plan-empty")));
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
