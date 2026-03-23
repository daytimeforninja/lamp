use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::components::task_row::{task_grid, TaskRowCtx};
use crate::core::task::Task;
use crate::fl;
use crate::ui::{self, Sender};

pub fn someday_view(
    tasks: &[Task],
    ctx: &TaskRowCtx,
    sender: &Sender,
) -> gtk::Widget {
    let someday_tasks: Vec<&Task> = tasks
        .iter()
        .filter(|t| matches!(t.state, crate::core::task::TaskState::Someday))
        .collect();

    if someday_tasks.is_empty() {
        let empty_label = ui::body(&fl!("someday-empty"));
        empty_label.set_halign(gtk::Align::Center);
        empty_label.set_margin_top(32);
        empty_label.set_margin_bottom(32);
        empty_label.set_hexpand(true);
        empty_label.set_vexpand(true);
        return ui::page_wrapper(&{
            let b = ui::vbox(0);
            b.append(&empty_label);
            b
        })
        .upcast();
    }

    let content = ui::vbox(8);
    content.append(&task_grid(someday_tasks.into_iter(), ctx, None, sender));

    ui::page_wrapper(&content).upcast()
}
