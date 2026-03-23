use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::core::habit::Habit;
use crate::fl;
use crate::message::Message;
use crate::ui::{self, Sender};

/// Habit display with 14-day completion grid and complete button.
pub fn habit_chart(habit: &Habit, sender: &Sender) -> gtk::Widget {
    let today = chrono::Local::now().date_naive();
    let is_due = habit.is_due(today);

    // 14-day completion grid
    let completion_dates: Vec<chrono::NaiveDate> =
        habit.completions.iter().map(|dt| dt.date()).collect();

    let grid = ui::hbox(4);
    for days_ago in (0..14).rev() {
        let date = today - chrono::Duration::days(days_ago);
        let completed = completion_dates.contains(&date);
        let symbol = if completed { "\u{25CF}" } else { "\u{25CB}" };
        grid.append(&ui::caption(symbol));
    }

    // Title row with optional "Done today" button
    let title_row = ui::centered_hbox(8);
    let title_label = ui::body(&habit.task.title);
    title_label.set_hexpand(true);
    title_row.append(&title_label);

    if is_due {
        let id = habit.task.id;
        let done_btn = ui::button_with_signal(
            &fl!("habits-done-today"),
            None,
            Message::CompleteHabit(id),
            sender,
        );
        title_row.append(&done_btn);
    }

    // Stats row
    let streak_text = fl!("habits-streak", count = habit.streak.to_string());
    let best_text = fl!("habits-best", count = habit.best_streak.to_string());
    let stats = ui::hbox(16);
    stats.append(&ui::caption(&streak_text));
    stats.append(&ui::caption(&best_text));

    let col = ui::vbox(4);
    col.append(&title_row);
    col.append(&grid);
    col.append(&stats);

    col.upcast()
}
