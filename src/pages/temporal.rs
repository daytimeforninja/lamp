use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, icon, row, scrollable, text, text_input};
use cosmic::Element;

use crate::application::EventForm;
use crate::components::habit_chart::habit_chart;
use crate::components::month_calendar::{MonthCalendarState, month_calendar};
use crate::components::task_row::{task_grid, TaskRowCtx};
use crate::core::event::CalendarEvent;
use crate::core::habit::Habit;
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;
use crate::sync::caldav::CalendarInfo;

struct DayItems<'a> {
    events: Vec<&'a CalendarEvent>,
    scheduled_tasks: Vec<&'a Task>,
    deadline_tasks: Vec<&'a Task>,
    habits: Vec<&'a Habit>,
}

impl<'a> DayItems<'a> {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            scheduled_tasks: Vec::new(),
            deadline_tasks: Vec::new(),
            habits: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.events.is_empty()
            && self.scheduled_tasks.is_empty()
            && self.deadline_tasks.is_empty()
            && self.habits.is_empty()
    }
}

pub fn agenda_view(
    tasks: &[Task],
    habits: &[Habit],
    events: &[CalendarEvent],
    event_form: Option<&EventForm>,
    ctx: &TaskRowCtx,
    discovered_calendars: &[CalendarInfo],
    month_state: &MonthCalendarState,
) -> Element<'static, Message> {
    let today = chrono::Local::now().date_naive();
    let horizon = today + chrono::Duration::days(30);

    // Build day-grouped items
    let mut days: BTreeMap<NaiveDate, DayItems> = BTreeMap::new();
    let mut overdue_tasks: Vec<&Task> = Vec::new();

    // Collect tasks
    for task in tasks {
        if task.state.is_done() {
            continue;
        }

        let mut placed = false;

        if let Some(sched) = task.scheduled {
            if sched < today {
                overdue_tasks.push(task);
                placed = true;
            } else if sched <= horizon {
                days.entry(sched).or_insert_with(DayItems::new).scheduled_tasks.push(task);
                placed = true;
            }
        }

        if !placed {
            if let Some(deadline) = task.deadline {
                if deadline < today {
                    overdue_tasks.push(task);
                } else if deadline <= horizon {
                    days.entry(deadline).or_insert_with(DayItems::new).deadline_tasks.push(task);
                }
            }
        }
    }

    // Collect events
    for event in events {
        let date = event.start.date();
        if date >= today && date <= horizon {
            days.entry(date).or_insert_with(DayItems::new).events.push(event);
        }
    }

    // Collect habits due today
    let habits_due: Vec<&Habit> = habits.iter().filter(|h| h.is_due(today)).collect();
    if !habits_due.is_empty() {
        let day = days.entry(today).or_insert_with(DayItems::new);
        day.habits = habits_due;
    }

    // Sort events within each day by start time
    for day in days.values_mut() {
        day.events.sort_by_key(|e| e.start);
    }

    let total_items = overdue_tasks.len()
        + days.values().map(|d| d.events.len() + d.scheduled_tasks.len() + d.deadline_tasks.len() + d.habits.len()).sum::<usize>();

    // Build busy days set from the day-grouped items
    let mut busy_days: HashSet<NaiveDate> = HashSet::new();
    for (date, items) in &days {
        if !items.is_empty() {
            busy_days.insert(*date);
        }
    }
    for task in &overdue_tasks {
        if let Some(sched) = task.scheduled {
            busy_days.insert(sched);
        }
        if let Some(dl) = task.deadline {
            busy_days.insert(dl);
        }
    }

    let mut content = column().spacing(16);

    // Month calendar widget
    content = content.push(month_calendar(month_state, &busy_days, today, events, tasks));

    // Add Event button
    content = content.push(
        row()
            .push(
                button::suggested(fl!("agenda-add-event"))
                    .on_press(Message::CreateEvent),
            ),
    );

    // Event form (inline when present)
    if let Some(form) = event_form {
        content = content.push(event_form_view(form.clone(), discovered_calendars.to_vec()));
    }

    if total_items == 0 && event_form.is_none() {
        return container(
            column()
                .spacing(16)
                .push(
                    button::suggested(fl!("agenda-add-event"))
                        .on_press(Message::CreateEvent),
                )
                .push(
                    text::body(fl!("agenda-empty"))
                )
                .padding(32)
                .width(Length::Fill),
        )
        .center_x(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    // Overdue section
    if !overdue_tasks.is_empty() {
        let mut section = column().spacing(4);
        section = section.push(text::title4(fl!("agenda-overdue")));
        let overdue_owned: Vec<Task> = overdue_tasks.iter().map(|t| (*t).clone()).collect();
        section = section.push(task_grid(overdue_owned.iter(), ctx, None));
        content = content.push(section);
    }

    // Day sections
    for (date, day_items) in &days {
        if day_items.is_empty() {
            continue;
        }

        let header = format_day_header(*date, today);
        let mut section = column().spacing(4);
        section = section.push(text::title4(header));

        // Events
        for event in &day_items.events {
            section = section.push(event_row(event));
        }

        // Scheduled tasks
        if !day_items.scheduled_tasks.is_empty() {
            let owned: Vec<Task> = day_items.scheduled_tasks.iter().map(|t| (*t).clone()).collect();
            section = section.push(task_grid(owned.iter(), ctx, None));
        }

        // Deadline tasks
        if !day_items.deadline_tasks.is_empty() {
            let owned: Vec<Task> = day_items.deadline_tasks.iter().map(|t| (*t).clone()).collect();
            section = section.push(task_grid(owned.iter(), ctx, None));
        }

        // Habits
        for habit in &day_items.habits {
            section = section.push(habit_chart(habit));
        }

        content = content.push(section);
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn format_day_header(date: NaiveDate, today: NaiveDate) -> String {
    if date == today {
        let day_name = date.format("%A").to_string();
        format!("{}, {} {}", fl!("agenda-today"), day_name, date.format("%b %e"))
    } else if date == today.succ_opt().unwrap_or(today) {
        let day_name = date.format("%A").to_string();
        format!("{}, {} {}", fl!("agenda-tomorrow"), day_name, date.format("%b %e"))
    } else {
        date.format("%A, %b %e").to_string()
    }
}

fn event_row(event: &CalendarEvent) -> Element<'static, Message> {
    let time_str = if event.all_day {
        "All day".to_string()
    } else {
        event.start.format("%H:%M").to_string()
    };

    let cal_label = if event.calendar_name.is_empty() {
        String::new()
    } else {
        event.calendar_name.clone()
    };

    let id = event.id;

    row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(
            text::body(time_str)
                .width(Length::Fixed(56.0)),
        )
        .push(
            icon::from_name("x-office-calendar-symbolic")
                .size(16)
                .icon(),
        )
        .push(
            text::body(event.title.clone())
                .width(Length::Fill),
        )
        .push(text::caption(cal_label))
        .push(
            button::icon(icon::from_name("document-edit-symbolic"))
                .on_press(Message::EditEvent(id)),
        )
        .push(
            button::icon(icon::from_name("edit-delete-symbolic"))
                .on_press(Message::DeleteEvent(id)),
        )
        .into()
}

fn event_form_view(
    form: EventForm,
    discovered_calendars: Vec<CalendarInfo>,
) -> Element<'static, Message> {
    let mut content = column().spacing(8);

    content = content.push(text::title4(if form.editing.is_some() {
        fl!("agenda-event-edit")
    } else {
        fl!("agenda-add-event")
    }));

    // Clone all strings so they're owned by the closures / widgets
    let title = form.title.clone();
    let start_date = form.start_date.clone();
    let start_time = form.start_time.clone();
    let end_date = form.end_date.clone();
    let end_time = form.end_time.clone();
    let location = form.location.clone();
    let description = form.description.clone();
    let calendar_href = form.calendar_href.clone();

    // Title
    content = content.push(
        text_input::text_input(fl!("agenda-event-title"), title)
            .on_input(Message::SetEventTitle)
            .width(Length::Fill),
    );

    // All day toggle
    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(text::body(fl!("agenda-event-all-day")).width(Length::Fill))
            .push(
                cosmic::widget::toggler(form.all_day)
                    .on_toggle(Message::SetEventAllDay),
            ),
    );

    // Start
    content = content.push(text::caption(fl!("agenda-event-start")));
    if form.all_day {
        content = content.push(
            text_input::text_input("YYYY-MM-DD", start_date.clone())
                .on_input(|v| Message::SetEventStart(v))
                .width(Length::Fill),
        );
    } else {
        content = content.push(
            row()
                .spacing(8)
                .push(
                    text_input::text_input("YYYY-MM-DD", start_date.clone())
                        .on_input(|v| Message::SetEventStart(v))
                        .width(Length::Fill),
                )
                .push(
                    text_input::text_input("HH:MM", start_time.clone())
                        .on_input(|v| Message::SetEventStart(v))
                        .width(Length::Fixed(80.0)),
                ),
        );
    }

    // End
    content = content.push(text::caption(fl!("agenda-event-end")));
    if form.all_day {
        content = content.push(
            text_input::text_input("YYYY-MM-DD", end_date.clone())
                .on_input(|v| Message::SetEventEnd(v))
                .width(Length::Fill),
        );
    } else {
        content = content.push(
            row()
                .spacing(8)
                .push(
                    text_input::text_input("YYYY-MM-DD", end_date.clone())
                        .on_input(|v| Message::SetEventEnd(v))
                        .width(Length::Fill),
                )
                .push(
                    text_input::text_input("HH:MM", end_time.clone())
                        .on_input(|v| Message::SetEventEnd(v))
                        .width(Length::Fixed(80.0)),
                ),
        );
    }

    // Location
    content = content.push(
        text_input::text_input(fl!("agenda-event-location"), location)
            .on_input(Message::SetEventLocation)
            .width(Length::Fill),
    );

    // Description
    content = content.push(
        text_input::text_input(fl!("agenda-event-description"), description)
            .on_input(Message::SetEventDescription)
            .width(Length::Fill),
    );

    // Calendar dropdown
    let event_cals: Vec<&CalendarInfo> = discovered_calendars
        .iter()
        .filter(|c| c.supports_vevent)
        .collect();
    if !event_cals.is_empty() {
        let cal_names: Vec<String> = event_cals.iter().map(|c| c.display_name.clone()).collect();
        let selected = event_cals.iter().position(|c| c.href == calendar_href);
        let hrefs: Vec<String> = event_cals.iter().map(|c| c.href.clone()).collect();
        content = content.push(
            cosmic::widget::dropdown(cal_names, selected, move |idx| {
                Message::SetEventCalendar(hrefs[idx].clone())
            })
            .width(Length::Fill),
        );
    }

    // Save / Cancel buttons
    let save_msg = if let Some(id) = form.editing {
        Message::UpdateEvent(id)
    } else {
        Message::SubmitEvent
    };

    content = content.push(
        row()
            .spacing(8)
            .push(
                button::suggested(fl!("agenda-event-save"))
                    .on_press(save_msg),
            )
            .push(
                button::standard(fl!("agenda-event-cancel"))
                    .on_press(Message::CancelEventForm),
            ),
    );

    container(content)
        .padding(12)
        .width(Length::Fill)
        .into()
}
