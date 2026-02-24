use cosmic::iced::{Alignment, Length};
use cosmic::widget::{checkbox, column, container, row, scrollable, text};
use cosmic::Element;

use crate::core::day_plan::DayPlan;
use crate::core::habit::Habit;
use crate::core::list_item::ListItem;
use crate::core::task::Task;
use crate::message::Message;

pub fn do_mode_view<'a>(
    day_plan: &Option<DayPlan>,
    all_tasks: &[Task],
    habits: &[Habit],
    media_items: &[ListItem],
    shopping_items: &[ListItem],
) -> Element<'a, Message> {
    let today = chrono::Local::now().date_naive();

    let Some(plan) = day_plan else {
        return container(
            column()
                .spacing(12)
                .push(text::title3("No plan for today"))
                .push(text::body("Switch to Plan mode and use Daily Planning."))
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .center(Length::Fill)
        .into();
    };

    let remaining = plan.remaining_budget(all_tasks);
    let budget = plan.spoon_budget;

    let mut content = column().spacing(24).padding(16).width(Length::Fill);

    // Spoon meter
    let pct = if budget > 0 {
        (remaining as f32 / budget as f32 * 100.0) as u32
    } else {
        0
    };
    let color_label = if pct > 50 {
        "green"
    } else if pct > 25 {
        "yellow"
    } else {
        "red"
    };
    let meter_text = format!("{}/{} spoons remaining ({})", remaining, budget, color_label);
    content = content.push(text::title3(meter_text));

    // Tasks section (sorted by ESC ascending, None last)
    let mut confirmed_tasks: Vec<&Task> = plan
        .confirmed_task_ids
        .iter()
        .filter_map(|id| all_tasks.iter().find(|t| t.id == *id))
        .collect();
    confirmed_tasks.sort_by_key(|t| t.esc.unwrap_or(u32::MAX));

    if !confirmed_tasks.is_empty() {
        content = content.push(text::title4("Tasks"));
        let mut tasks_col = column().spacing(4);
        for task in &confirmed_tasks {
            let id = task.id;
            let is_done = task.state.is_done();
            let esc_text = task.esc.map(|e| format!(" [{}]", e)).unwrap_or_default();

            let r = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    checkbox("", is_done)
                        .on_toggle(move |_| Message::DoMarkDone(id)),
                )
                .push(text::body(format!("{}{}", task.title, esc_text)).width(Length::Fill));
            tasks_col = tasks_col.push(r);
        }
        content = content.push(tasks_col);
    }

    // Habits section — show all due (incomplete today) habits
    let due_habits: Vec<&Habit> = habits.iter().filter(|h| h.is_due(today)).collect();

    if !due_habits.is_empty() {
        content = content.push(text::title4("Habits"));
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
        content = content.push(text::title4("Media"));
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
        content = content.push(text::title4("Shopping"));
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
        content = content.push(
            text::body("Your plan is empty. Switch to Plan mode to add tasks and items.")
        );
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
