use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::core::task::Task;
use crate::fl;
use crate::message::Message;
use crate::ui::{self, Sender};

pub fn archive_view(
    tasks: &[Task],
    search: &str,
    sender: &Sender,
) -> gtk::Widget {
    let lq = search.to_lowercase();
    let mut filtered: Vec<&Task> = if lq.is_empty() {
        tasks.iter().collect()
    } else {
        tasks
            .iter()
            .filter(|t| t.title.to_lowercase().contains(&lq))
            .collect()
    };

    // Sort by completion date, newest first
    filtered.sort_by(|a, b| b.completed.cmp(&a.completed));

    let content = ui::vbox(8);

    // Search input
    let search_entry = ui::entry(&fl!("search-placeholder"), search);
    {
        let s = sender.clone();
        search_entry.connect_changed(move |e| {
            s.emit(Message::ArchiveSearchChanged(e.text().to_string()));
        });
    }
    search_entry.set_margin_start(16);
    search_entry.set_margin_end(16);
    content.append(&search_entry);

    if filtered.is_empty() {
        let (title, desc) = if search.is_empty() {
            ("No Archived Tasks", "Completed tasks will appear here")
        } else {
            ("No Results", "Try a different search term")
        };
        content.append(&ui::status_page("document-open-recent-symbolic", title, desc));

        return ui::scrolled(&content).upcast();
    }

    // Count label
    let count_text = format!("{} {}", filtered.len(), fl!("archive-count-suffix"));
    let count_label = ui::caption(&count_text);
    count_label.set_margin_start(16);
    count_label.set_margin_end(16);
    content.append(&count_label);

    // Task list
    let list = ui::vbox(0);

    for task in &filtered {
        let row_box = ui::vbox(2);
        row_box.set_margin_start(16);
        row_box.set_margin_end(16);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);

        // Title
        row_box.append(&ui::body(&task.title));

        // Info row: state, date, project, contexts
        let info_row = ui::hbox(8);

        let state_label = ui::caption(&format!("{:?}", task.state));
        info_row.append(&state_label);

        if let Some(completed) = task.completed {
            info_row.append(&ui::caption(&completed.format("%Y-%m-%d").to_string()));
        }

        if let Some(ref project) = task.project {
            info_row.append(&ui::caption(&format!("[{}]", project)));
        }

        if !task.contexts.is_empty() {
            info_row.append(&ui::caption(&task.contexts.join(", ")));
        }

        row_box.append(&info_row);
        list.append(&row_box);
    }

    content.append(&ui::scrolled(&list));

    content.upcast()
}
