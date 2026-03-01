use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, Weekday};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, row, text};
use cosmic::Element;

use crate::core::event::CalendarEvent;
use crate::core::task::Task;
use crate::message::Message;

#[derive(Debug, Clone)]
pub struct MonthCalendarState {
    /// First day of the displayed month.
    pub displayed_month: NaiveDate,
    /// Currently selected day (shows detail panel).
    pub selected_day: Option<NaiveDate>,
}

impl Default for MonthCalendarState {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive();
        Self {
            displayed_month: NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap(),
            selected_day: Some(today),
        }
    }
}

impl MonthCalendarState {
    pub fn prev_month(&mut self) {
        self.displayed_month = self
            .displayed_month
            .checked_sub_months(chrono::Months::new(1))
            .unwrap_or(self.displayed_month);
        self.selected_day = None;
    }

    pub fn next_month(&mut self) {
        self.displayed_month = self
            .displayed_month
            .checked_add_months(chrono::Months::new(1))
            .unwrap_or(self.displayed_month);
        self.selected_day = None;
    }

    pub fn select_day(&mut self, date: NaiveDate) {
        if self.selected_day == Some(date) {
            self.selected_day = None;
        } else {
            self.selected_day = Some(date);
        }
    }
}

/// Render a month calendar grid widget with an optional detail panel for the selected day.
pub fn month_calendar<'a>(
    state: &MonthCalendarState,
    busy_days: &HashSet<NaiveDate>,
    today: NaiveDate,
    events: &[CalendarEvent],
    tasks: &[Task],
) -> Element<'a, Message> {
    let first = state.displayed_month;
    let year = first.year();
    let month = first.month();

    // Header: < Month Year >
    let month_label = first.format("%B %Y").to_string();

    let header = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(
            button::icon(cosmic::widget::icon::from_name("go-previous-symbolic"))
                .on_press(Message::CalendarPrevMonth),
        )
        .push(
            text::body(month_label)
                .width(Length::Fill)
                .center(),
        )
        .push(
            button::icon(cosmic::widget::icon::from_name("go-next-symbolic"))
                .on_press(Message::CalendarNextMonth),
        );

    // Day labels: Mo Tu We Th Fr Sa Su
    let day_labels = row()
        .spacing(0)
        .push(day_label("Mo"))
        .push(day_label("Tu"))
        .push(day_label("We"))
        .push(day_label("Th"))
        .push(day_label("Fr"))
        .push(day_label("Sa"))
        .push(day_label("Su"));

    let mut grid = column().spacing(2).push(header).push(day_labels);

    // Find the Monday on or before the first of the month
    let weekday_offset = match first.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };
    let grid_start = first - chrono::Duration::days(weekday_offset as i64);

    // Render 6 rows of 7 days
    for week in 0..6 {
        let mut week_row = row().spacing(0);
        let mut any_in_month = false;

        for day_of_week in 0..7 {
            let date = grid_start + chrono::Duration::days(week * 7 + day_of_week);
            let in_month = date.month() == month && date.year() == year;

            if in_month {
                any_in_month = true;
            }

            let cell: Element<'a, Message> = if !in_month {
                container(text::body(" "))
                    .width(Length::FillPortion(1))
                    .center_x(Length::FillPortion(1))
                    .into()
            } else {
                let day_num = date.day().to_string();
                let is_today = date == today;
                let is_busy = busy_days.contains(&date);
                let is_selected = state.selected_day == Some(date);

                let label = if is_busy {
                    format!("{}\n·", day_num)
                } else {
                    format!("{}\n ", day_num)
                };

                let txt = if is_today {
                    text::body(label).font(cosmic::iced::Font {
                        weight: cosmic::iced::font::Weight::Bold,
                        ..Default::default()
                    })
                } else {
                    text::body(label)
                };

                let cell_content = container(txt.center())
                    .center_x(Length::Fill);

                let btn = if is_selected {
                    button::custom(cell_content)
                        .class(cosmic::theme::Button::Suggested)
                        .on_press(Message::CalendarSelectDay(date))
                        .width(Length::FillPortion(1))
                } else {
                    button::custom(cell_content)
                        .class(cosmic::theme::Button::Text)
                        .on_press(Message::CalendarSelectDay(date))
                        .width(Length::FillPortion(1))
                };

                btn.into()
            };

            week_row = week_row.push(cell);
        }

        if any_in_month {
            grid = grid.push(week_row);
        }
    }

    let mut content = column().spacing(8).push(
        container(grid)
            .width(Length::Fill)
            .padding(8),
    );

    // Detail panel for selected day
    if let Some(selected) = state.selected_day {
        for item in day_detail(selected, today, events, tasks) {
            content = content.push(item);
        }
    }

    content.into()
}

/// Render a compact detail panel for the selected day's events and tasks.
fn day_detail<'a>(
    date: NaiveDate,
    today: NaiveDate,
    events: &[CalendarEvent],
    tasks: &[Task],
) -> Vec<Element<'a, Message>> {
    let mut items: Vec<Element<'a, Message>> = Vec::new();

    let header = if date == today {
        format!("Today, {}", date.format("%A %b %e"))
    } else if date == today.succ_opt().unwrap_or(today) {
        format!("Tomorrow, {}", date.format("%A %b %e"))
    } else {
        date.format("%A, %b %e").to_string()
    };

    // Collect events for this date
    let day_events: Vec<&CalendarEvent> = events
        .iter()
        .filter(|e| e.start.date() == date)
        .collect();

    // Collect tasks scheduled/due this date
    let day_tasks: Vec<&Task> = tasks
        .iter()
        .filter(|t| {
            !t.state.is_done()
                && (t.scheduled == Some(date) || t.deadline == Some(date))
        })
        .collect();

    if day_events.is_empty() && day_tasks.is_empty() {
        return items;
    }

    items.push(text::title4(header).into());

    for event in &day_events {
        let time_str = if event.all_day {
            "All day".to_string()
        } else {
            format!("{} – {}", event.start.format("%H:%M"), event.end.format("%H:%M"))
        };

        let mut r = row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(text::caption(time_str).width(Length::Fixed(100.0)))
            .push(text::body(event.title.clone()).width(Length::Fill));

        if !event.location.is_empty() {
            r = r.push(text::caption(event.location.clone()));
        }

        items.push(r.into());
    }

    for task in &day_tasks {
        let prefix = if task.deadline == Some(date) {
            "Due"
        } else {
            "Scheduled"
        };
        items.push(
            row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(text::caption(prefix).width(Length::Fixed(100.0)))
                .push(text::body(task.title.clone()).width(Length::Fill))
                .into(),
        );
    }

    items
}

fn day_label(label: &str) -> Element<'_, Message> {
    container(text::caption(label).center())
        .width(Length::FillPortion(1))
        .center_x(Length::FillPortion(1))
        .into()
}
