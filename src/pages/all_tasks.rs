use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::components::task_row::{task_grid, TaskRowCtx};
use crate::core::task::Task;
use crate::fl;
use crate::message::SortColumn;
use crate::ui::{self, Sender};

pub fn all_tasks_view(
    tasks: &[Task],
    ctx: &TaskRowCtx,
    sort: Option<(SortColumn, bool)>,
    sender: &Sender,
) -> gtk::Widget {
    let mut active: Vec<&Task> = tasks.iter().filter(|t| t.state.is_active()).collect();

    if active.is_empty() {
        let empty_label = ui::body(&fl!("all-tasks-empty"));
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

    if let Some((col, ascending)) = sort {
        active.sort_by(|a, b| {
            let ord = match col {
                SortColumn::State => {
                    fn rank(t: &Task) -> u8 {
                        use crate::core::task::TaskState::*;
                        match t.state {
                            Next => 0,
                            Todo => 1,
                            Waiting => 2,
                            Someday => 3,
                            _ => 4,
                        }
                    }
                    rank(a).cmp(&rank(b))
                }
                SortColumn::Priority => {
                    // A < B < C < None (A is highest priority, sort first)
                    let pa = a.priority.map(|p| p as u8).unwrap_or(255);
                    let pb = b.priority.map(|p| p as u8).unwrap_or(255);
                    pa.cmp(&pb)
                }
                SortColumn::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                SortColumn::Context => {
                    let ca = a.contexts.first().cloned().unwrap_or_default();
                    let cb = b.contexts.first().cloned().unwrap_or_default();
                    ca.cmp(&cb)
                }
                SortColumn::Esc => a.esc.unwrap_or(0).cmp(&b.esc.unwrap_or(0)),
                SortColumn::Scheduled => a.scheduled.cmp(&b.scheduled),
                SortColumn::Deadline => a.deadline.cmp(&b.deadline),
            };
            if ascending { ord } else { ord.reverse() }
        });
    }

    let content = ui::vbox(8);
    content.append(&task_grid(active.into_iter(), ctx, Some(sort), sender));

    ui::page_wrapper(&content).upcast()
}
