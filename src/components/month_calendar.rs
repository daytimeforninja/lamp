use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, Weekday};
use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::core::event::CalendarEvent;
use crate::core::habit::Habit;
use crate::core::task::Task;
use crate::message::Message;
use crate::ui::{self, Sender};

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
pub fn month_calendar_view(
    state: &MonthCalendarState,
    events: &[CalendarEvent],
    tasks: &[Task],
    _habits: &[Habit],
    sender: &Sender,
) -> gtk::Widget {
    let today = chrono::Local::now().date_naive();
    let first = state.displayed_month;
    let year = first.year();
    let month = first.month();

    // Collect busy days (days with events or tasks)
    let mut busy_days = HashSet::new();
    for e in events {
        busy_days.insert(e.start.date());
    }
    for t in tasks {
        if !t.state.is_done() {
            if let Some(d) = t.scheduled {
                busy_days.insert(d);
            }
            if let Some(d) = t.deadline {
                busy_days.insert(d);
            }
        }
    }

    // Header: < Month Year >
    let month_label = first.format("%B %Y").to_string();

    let header = ui::centered_hbox(8);
    let prev_btn = ui::icon_button_with_signal(
        "go-previous-symbolic",
        Message::CalendarPrevMonth,
        sender,
    );
    header.append(&prev_btn);

    let month_text = ui::body(&month_label);
    month_text.set_hexpand(true);
    month_text.set_halign(gtk::Align::Center);
    header.append(&month_text);

    let next_btn = ui::icon_button_with_signal(
        "go-next-symbolic",
        Message::CalendarNextMonth,
        sender,
    );
    header.append(&next_btn);

    // Day labels: Mo Tu We Th Fr Sa Su
    let day_labels_row = ui::hbox(0);
    for lbl in &["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] {
        let l = ui::caption(lbl);
        l.set_hexpand(true);
        l.set_halign(gtk::Align::Center);
        day_labels_row.append(&l);
    }

    let grid = ui::vbox(2);
    grid.append(&header);
    grid.append(&day_labels_row);

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
        let week_row = ui::hbox(0);
        let mut any_in_month = false;

        for day_of_week in 0..7 {
            let date = grid_start + chrono::Duration::days(week * 7 + day_of_week);
            let in_month = date.month() == month && date.year() == year;

            if in_month {
                any_in_month = true;
            }

            if !in_month {
                let spacer = gtk::Label::new(Some(" "));
                spacer.set_hexpand(true);
                week_row.append(&spacer);
            } else {
                let day_num = date.day().to_string();
                let is_today = date == today;
                let is_busy = busy_days.contains(&date);
                let is_selected = state.selected_day == Some(date);

                let label_text = if is_busy {
                    format!("{}\n\u{00B7}", day_num)
                } else {
                    format!("{}\n ", day_num)
                };

                let btn = gtk::Button::with_label(&label_text);
                btn.set_hexpand(true);
                btn.add_css_class("flat");

                if is_today {
                    btn.add_css_class("accent");
                }
                if is_selected {
                    btn.add_css_class("suggested-action");
                }

                {
                    let s = sender.clone();
                    btn.connect_clicked(move |_| {
                        s.emit(Message::CalendarSelectDay(date));
                    });
                }

                week_row.append(&btn);
            }
        }

        if any_in_month {
            grid.append(&week_row);
        }
    }

    let content = ui::vbox(8);
    grid.set_margin_start(8);
    grid.set_margin_end(8);
    grid.set_margin_top(8);
    grid.set_margin_bottom(8);
    content.append(&grid);

    // Detail panel for selected day
    if let Some(selected) = state.selected_day {
        for widget in day_detail(selected, today, events, tasks) {
            content.append(&widget);
        }
    }

    content.upcast()
}

/// Render a compact detail panel for the selected day's events and tasks.
fn day_detail(
    date: NaiveDate,
    today: NaiveDate,
    events: &[CalendarEvent],
    tasks: &[Task],
) -> Vec<gtk::Widget> {
    let mut items: Vec<gtk::Widget> = Vec::new();

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

    items.push(ui::title4(&header).upcast());

    for event in &day_events {
        let time_str = if event.all_day {
            "All day".to_string()
        } else {
            format!("{} \u{2013} {}", event.start.format("%H:%M"), event.end.format("%H:%M"))
        };

        let r = ui::centered_hbox(8);
        let time_label = ui::caption(&time_str);
        time_label.set_size_request(100, -1);
        r.append(&time_label);

        let title_label = ui::body(&event.title);
        title_label.set_hexpand(true);
        r.append(&title_label);

        if !event.location.is_empty() {
            r.append(&ui::caption(&event.location));
        }

        items.push(r.upcast());
    }

    for task in &day_tasks {
        let prefix = if task.deadline == Some(date) {
            "Due"
        } else {
            "Scheduled"
        };

        let r = ui::centered_hbox(8);
        let prefix_label = ui::caption(prefix);
        prefix_label.set_size_request(100, -1);
        r.append(&prefix_label);

        let title_label = ui::body(&task.title);
        title_label.set_hexpand(true);
        r.append(&title_label);

        items.push(r.upcast());
    }

    items
}
