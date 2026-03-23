use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::components::habit_chart::habit_chart;
use crate::core::habit::Habit;
use crate::fl;
use crate::message::Message;
use crate::ui::{self, Sender};

pub fn habits_view(
    habits: &[Habit],
    habit_input: &str,
    sender: &Sender,
) -> gtk::Widget {
    let content = ui::vbox(12);

    // Creation input
    let input_row = ui::hbox(8);
    input_row.set_valign(gtk::Align::Center);

    let entry = ui::entry(&fl!("habits-new-placeholder"), habit_input);
    {
        let s = sender.clone();
        entry.connect_changed(move |e| {
            s.emit(Message::HabitInputChanged(e.text().to_string()));
        });
    }
    {
        let s = sender.clone();
        entry.connect_activate(move |_| {
            s.emit(Message::HabitSubmit);
        });
    }
    input_row.append(&entry);
    input_row.append(&ui::icon_button_with_signal(
        "list-add-symbolic",
        Message::HabitSubmit,
        sender,
    ));
    content.append(&input_row);

    if habits.is_empty() {
        content.append(&ui::status_page("checkbox-checked-symbolic", &fl!("habits-empty"), "Add a daily habit to get started"));
    } else {
        for habit in habits {
            let row = ui::centered_hbox(8);

            let chart = habit_chart(habit, sender);
            chart.set_hexpand(true);
            row.append(&chart);

            row.append(&ui::icon_button_with_signal(
                "edit-delete-symbolic",
                Message::DeleteHabit(habit.task.id),
                sender,
            ));

            content.append(&row);
        }
    }

    ui::page_wrapper(&content).upcast()
}
