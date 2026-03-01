use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, icon, row, scrollable, text, text_input};
use cosmic::Element;

use crate::components::habit_chart::habit_chart;
use crate::core::habit::Habit;
use crate::fl;
use crate::message::Message;

pub fn habits_view<'a>(habits: &[Habit], habit_input: &str) -> Element<'a, Message> {
    let mut content = column().spacing(12);

    // Creation input
    let input = text_input::text_input(fl!("habits-new-placeholder"), habit_input.to_string())
        .on_input(Message::HabitInputChanged)
        .on_submit(|_| Message::HabitSubmit)
        .width(Length::Fill);

    content = content.push(
        row()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(input)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .on_press(Message::HabitSubmit),
            ),
    );

    if habits.is_empty() {
        content = content.push(
            container(text::body(fl!("habits-empty")))
                .padding(32)
                .center_x(Length::Fill)
                .width(Length::Fill),
        );
    } else {
        for habit in habits {
            let delete_btn = button::icon(icon::from_name("edit-delete-symbolic"))
                .on_press(Message::DeleteHabit(habit.task.id));

            let habit_row = row()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(habit_chart(habit)).width(Length::Fill))
                .push(delete_btn);

            content = content.push(habit_row);
        }
    }

    container(scrollable(content.padding(16).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
