use chrono::Duration;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, row, text};
use cosmic::Element;

use crate::core::habit::Habit;
use crate::fl;
use crate::message::Message;

/// Habit display with 14-day completion grid and complete button.
pub fn habit_chart(habit: &Habit) -> Element<'static, Message> {
    let today = chrono::Local::now().date_naive();
    let title = habit.task.title.clone();
    let is_due = habit.is_due(today);

    // 14-day completion grid
    let completion_dates: Vec<chrono::NaiveDate> =
        habit.completions.iter().map(|dt| dt.date()).collect();

    let mut grid = row().spacing(4);
    for days_ago in (0..14).rev() {
        let date = today - Duration::days(days_ago);
        let completed = completion_dates.contains(&date);
        let symbol = if completed { "\u{25CF}" } else { "\u{25CB}" };
        grid = grid.push(text::caption(symbol));
    }

    // Title row with optional "Done today" button
    let mut title_row = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(text::body(title).width(Length::Fill));

    if is_due {
        title_row = title_row.push(
            button::standard(fl!("habits-done-today"))
                .on_press(Message::CompleteHabit(habit.task.id)),
        );
    }

    // Stats row
    let streak_text = fl!("habits-streak", count = habit.streak.to_string());
    let best_text = fl!("habits-best", count = habit.best_streak.to_string());
    let stats = row()
        .spacing(16)
        .push(text::caption(streak_text))
        .push(text::caption(best_text));

    column()
        .spacing(4)
        .push(title_row)
        .push(grid)
        .push(stats)
        .into()
}
