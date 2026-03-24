use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::application::EventForm;
use crate::components::habit_chart::habit_chart;
use crate::components::month_calendar::{month_calendar_view, MonthCalendarState};
use crate::components::task_row::{task_grid, TaskRowCtx};
use crate::core::event::CalendarEvent;
use crate::core::habit::Habit;
use crate::core::task::Task;
use crate::fl;
use crate::message::Message;
use crate::sync::caldav::CalendarInfo;
use crate::ui::{self, Sender};

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
    calendars: &[CalendarInfo],
    month_calendar_state: &MonthCalendarState,
    sender: &Sender,
) -> gtk::Widget {
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

    // Build busy days set
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

    let content = ui::vbox(16);

    // Month calendar widget
    let cal_widget = month_calendar_view(month_calendar_state, events, tasks, habits, sender);
    content.append(&cal_widget);

    // Add Event button
    content.append(&ui::button_with_signal(&fl!("agenda-add-event"), Some("suggested-action"), Message::CreateEvent, sender));

    // Event form (inline when present)
    if let Some(form) = event_form {
        content.append(&event_form_view(form, calendars, sender));
    }

    if total_items == 0 && event_form.is_none() {
        let empty_content = ui::vbox(16);
        empty_content.set_margin_top(32);
        empty_content.set_hexpand(true);
        empty_content.set_halign(gtk::Align::Center);
        empty_content.append(&ui::button_with_signal(&fl!("agenda-add-event"), Some("suggested-action"), Message::CreateEvent, sender));
        empty_content.append(&ui::body(&fl!("agenda-empty")));
        content.append(&empty_content);
        return ui::page_wrapper(&content).upcast();
    }

    // Overdue section
    if !overdue_tasks.is_empty() {
        let section = ui::vbox(4);
        section.append(&ui::title4(&fl!("agenda-overdue")));
        let overdue_owned: Vec<Task> = overdue_tasks.iter().map(|t| (*t).clone()).collect();
        section.append(&task_grid(overdue_owned.iter(), ctx, None, sender));
        content.append(&section);
    }

    // Day sections
    for (date, day_items) in &days {
        if day_items.is_empty() {
            continue;
        }

        let header = format_day_header(*date, today);
        let section = ui::vbox(4);
        section.append(&ui::title4(&header));

        // Events
        for event in &day_items.events {
            section.append(&event_row(event, sender));
        }

        // Scheduled tasks
        if !day_items.scheduled_tasks.is_empty() {
            let owned: Vec<Task> = day_items.scheduled_tasks.iter().map(|t| (*t).clone()).collect();
            section.append(&task_grid(owned.iter(), ctx, None, sender));
        }

        // Deadline tasks
        if !day_items.deadline_tasks.is_empty() {
            let owned: Vec<Task> = day_items.deadline_tasks.iter().map(|t| (*t).clone()).collect();
            section.append(&task_grid(owned.iter(), ctx, None, sender));
        }

        // Habits
        for habit in &day_items.habits {
            section.append(&habit_chart(habit, sender));
        }

        content.append(&section);
    }

    ui::page_wrapper(&content).upcast()
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

fn event_row(event: &CalendarEvent, sender: &Sender) -> gtk::Box {
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

    let row = ui::centered_hbox(8);

    let time_lbl = ui::body(&time_str);
    time_lbl.set_width_request(56);
    row.append(&time_lbl);

    let icon = gtk::Image::from_icon_name("view-grid-symbolic");
    icon.set_pixel_size(16);
    row.append(&icon);

    let title_lbl = ui::body(&event.title);
    title_lbl.set_hexpand(true);
    row.append(&title_lbl);

    row.append(&ui::caption(&cal_label));

    row.append(&ui::icon_button_with_signal("edit-copy-symbolic", Message::EditEvent(id), sender));
    row.append(&ui::icon_button_with_signal("edit-delete-symbolic", Message::DeleteEvent(id), sender));

    row
}

fn event_form_view(
    form: &EventForm,
    discovered_calendars: &[CalendarInfo],
    sender: &Sender,
) -> gtk::Box {
    let content = ui::vbox(8);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let form_title = if form.editing.is_some() {
        fl!("agenda-event-edit")
    } else {
        fl!("agenda-add-event")
    };
    content.append(&ui::title4(&form_title));

    // Title
    let title_entry = ui::entry(&fl!("agenda-event-title"), &form.title);
    {
        let s = sender.clone();
        title_entry.connect_changed(move |e| {
            s.emit(Message::SetEventTitle(e.text().to_string()));
        });
    }
    content.append(&title_entry);

    // All day toggle
    let all_day_row = ui::centered_hbox(8);
    let all_day_label = ui::body(&fl!("agenda-event-all-day"));
    all_day_label.set_hexpand(true);
    all_day_row.append(&all_day_label);
    let all_day_switch = gtk::Switch::new();
    all_day_switch.set_active(form.all_day);
    {
        let s = sender.clone();
        all_day_switch.connect_state_set(move |_, state| {
            s.emit(Message::SetEventAllDay(state));
            gtk::glib::Propagation::Proceed
        });
    }
    all_day_row.append(&all_day_switch);
    content.append(&all_day_row);

    // Start
    content.append(&ui::caption(&fl!("agenda-event-start")));
    if form.all_day {
        let start_entry = ui::entry(&fl!("event-date-placeholder"), &form.start_date);
        {
            let s = sender.clone();
            start_entry.connect_changed(move |e| {
                s.emit(Message::SetEventStart(e.text().to_string()));
            });
        }
        content.append(&start_entry);
    } else {
        let start_row = ui::hbox(8);
        let start_date_entry = ui::entry(&fl!("event-date-placeholder"), &form.start_date);
        {
            let s = sender.clone();
            start_date_entry.connect_changed(move |e| {
                s.emit(Message::SetEventStart(e.text().to_string()));
            });
        }
        start_row.append(&start_date_entry);
        let start_time_entry = ui::entry(&fl!("event-time-placeholder"), &form.start_time);
        start_time_entry.set_width_request(80);
        start_time_entry.set_hexpand(false);
        {
            let s = sender.clone();
            start_time_entry.connect_changed(move |e| {
                s.emit(Message::SetEventStartTime(e.text().to_string()));
            });
        }
        start_row.append(&start_time_entry);
        content.append(&start_row);
    }
    if let Some(ref err) = form.start_error {
        content.append(&ui::caption(err));
    }

    // End
    content.append(&ui::caption(&fl!("agenda-event-end")));
    if form.all_day {
        let end_entry = ui::entry(&fl!("event-date-placeholder"), &form.end_date);
        {
            let s = sender.clone();
            end_entry.connect_changed(move |e| {
                s.emit(Message::SetEventEnd(e.text().to_string()));
            });
        }
        content.append(&end_entry);
    } else {
        let end_row = ui::hbox(8);
        let end_date_entry = ui::entry(&fl!("event-date-placeholder"), &form.end_date);
        {
            let s = sender.clone();
            end_date_entry.connect_changed(move |e| {
                s.emit(Message::SetEventEnd(e.text().to_string()));
            });
        }
        end_row.append(&end_date_entry);
        let end_time_entry = ui::entry(&fl!("event-time-placeholder"), &form.end_time);
        end_time_entry.set_width_request(80);
        end_time_entry.set_hexpand(false);
        {
            let s = sender.clone();
            end_time_entry.connect_changed(move |e| {
                s.emit(Message::SetEventEndTime(e.text().to_string()));
            });
        }
        end_row.append(&end_time_entry);
        content.append(&end_row);
    }
    if let Some(ref err) = form.end_error {
        content.append(&ui::caption(err));
    }

    // Location
    let loc_entry = ui::entry(&fl!("agenda-event-location"), &form.location);
    {
        let s = sender.clone();
        loc_entry.connect_changed(move |e| {
            s.emit(Message::SetEventLocation(e.text().to_string()));
        });
    }
    content.append(&loc_entry);

    // Description
    let desc_entry = ui::entry(&fl!("agenda-event-description"), &form.description);
    {
        let s = sender.clone();
        desc_entry.connect_changed(move |e| {
            s.emit(Message::SetEventDescription(e.text().to_string()));
        });
    }
    content.append(&desc_entry);

    // Calendar dropdown
    let event_cals: Vec<&CalendarInfo> = discovered_calendars
        .iter()
        .filter(|c| c.supports_vevent)
        .collect();
    if !event_cals.is_empty() {
        let cal_names: Vec<String> = event_cals.iter().map(|c| c.display_name.clone()).collect();
        let selected = event_cals.iter().position(|c| c.href == form.calendar_href);
        let hrefs: Vec<String> = event_cals.iter().map(|c| c.href.clone()).collect();
        let dd = ui::dropdown_with_signal(
            &cal_names,
            selected,
            move |idx| Message::SetEventCalendar(hrefs[idx].clone()),
            sender,
        );
        dd.set_hexpand(true);
        content.append(&dd);
    }

    // Save / Cancel buttons
    let save_msg = if let Some(id) = form.editing {
        Message::UpdateEvent(id)
    } else {
        Message::SubmitEvent
    };

    let btn_row = ui::hbox(8);
    btn_row.append(&ui::button_with_signal(&fl!("agenda-event-save"), Some("suggested-action"), save_msg, sender));
    btn_row.append(&ui::button_with_signal(&fl!("agenda-event-cancel"), None, Message::CancelEventForm, sender));
    content.append(&btn_row);

    content
}
