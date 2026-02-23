use cosmic::iced::Length;
use cosmic::widget::{column, container, scrollable, text};
use cosmic::Element;

use crate::components::habit_chart::habit_chart;
use crate::components::task_row::{TaskRowCtx, task_grid};
use crate::core::habit::Habit;
use crate::core::task::Task;
use crate::core::temporal::{DateRange, TemporalView};
use crate::fl;
use crate::message::Message;

pub fn temporal_view(
    tasks: &[Task],
    habits: &[Habit],
    range: DateRange,
    ctx: &TaskRowCtx,
) -> Element<'static, Message> {
    let today = chrono::Local::now().date_naive();
    let view = TemporalView::build(tasks, habits, today, range);

    if view.total_count() == 0 {
        let empty_msg = match range {
            DateRange::Today => fl!("temporal-empty-today"),
            DateRange::Tomorrow => fl!("temporal-empty-tomorrow"),
            DateRange::ThisWeek => fl!("temporal-empty-week"),
            DateRange::Upcoming => fl!("temporal-empty-upcoming"),
        };
        return container(text::body(empty_msg))
            .padding(32)
            .center_x(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let mut content = column().spacing(16);

    if !view.overdue.is_empty() {
        let mut section = column().spacing(4);
        section = section.push(text::title4(fl!("temporal-overdue")));
        section = section.push(task_grid(view.overdue.iter(), ctx));
        content = content.push(section);
    }

    if !view.scheduled.is_empty() {
        let title = match range {
            DateRange::Today => fl!("temporal-scheduled-today"),
            DateRange::Tomorrow => fl!("temporal-scheduled-tomorrow"),
            DateRange::ThisWeek => fl!("temporal-scheduled-week"),
            DateRange::Upcoming => fl!("temporal-scheduled-upcoming"),
        };
        let mut section = column().spacing(4);
        section = section.push(text::title4(title));
        section = section.push(task_grid(view.scheduled.iter(), ctx));
        content = content.push(section);
    }

    if !view.deadlined.is_empty() {
        let title = match range {
            DateRange::Today => fl!("temporal-deadlines-today"),
            DateRange::Tomorrow => fl!("temporal-deadlines-tomorrow"),
            DateRange::ThisWeek => fl!("temporal-deadlines-week"),
            DateRange::Upcoming => fl!("temporal-deadlines-today"),
        };
        let mut section = column().spacing(4);
        section = section.push(text::title4(title));
        section = section.push(task_grid(view.deadlined.iter(), ctx));
        content = content.push(section);
    }

    if !view.habits_due.is_empty() {
        let mut section = column().spacing(4);
        section = section.push(text::title4(fl!("temporal-habits")));
        for habit in &view.habits_due {
            section = section.push(habit_chart(habit));
        }
        content = content.push(section);
    }

    container(scrollable(content.padding(16)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
