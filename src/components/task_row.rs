use std::collections::HashMap;

use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use uuid::Uuid;

use relm4::gtk;
use relm4::gtk::prelude::*;

use crate::core::task::{Priority, Task, TaskState};
use crate::fl;
use crate::message::{Message, SortColumn};
use crate::sync::carddav::Contact;
use crate::ui::{self, Sender};

const STATE_LABELS: &[&str] = &["TODO", "NEXT", "WAIT", "SOME"];
const ESC_LABELS: &[&str] = &["-", "5", "10", "15", "20", "25", "30", "40", "50", "75", "100"];
const ESC_VALUES: &[Option<u32>] = &[
    None,
    Some(5),
    Some(10),
    Some(15),
    Some(20),
    Some(25),
    Some(30),
    Some(40),
    Some(50),
    Some(75),
    Some(100),
];

// Column widths for consistent alignment
const COL_CHECK: i32 = 28;
const COL_STATE: i32 = 76;
const COL_PRI: i32 = 32;
const COL_CTX: i32 = 120;
const COL_PROJECT: i32 = 100;
const COL_DATE: i32 = 96;
const COL_ESC: i32 = 48;
const COL_DELETE: i32 = 40;

/// Context passed to task grid.
pub struct TaskRowCtx {
    pub contexts: Vec<String>,
    pub project_names: Vec<String>,
    pub expanded_task: Option<Uuid>,
    pub note_inputs: HashMap<Uuid, String>,
    pub waiting_for_inputs: HashMap<Uuid, String>,
    pub contacts: Vec<Contact>,
}

// --- Date picker presets ---

struct DatePreset {
    label: String,
    date: Option<NaiveDate>,
}

fn date_presets(today: NaiveDate) -> Vec<DatePreset> {
    let tomorrow = today + Duration::days(1);
    let days_to_monday = (Weekday::Mon.num_days_from_sunday() as i64
        - today.weekday().num_days_from_sunday() as i64
        + 7)
        % 7;
    let next_monday = today + Duration::days(if days_to_monday == 0 { 7 } else { days_to_monday });

    vec![
        DatePreset { label: "\u{2014}".into(), date: None },
        DatePreset { label: format!("Today {}", today.format("%d")), date: Some(today) },
        DatePreset { label: format!("Tmrw {}", tomorrow.format("%d")), date: Some(tomorrow) },
        DatePreset { label: format!("Mon {}", next_monday.format("%d")), date: Some(next_monday) },
        DatePreset { label: format!("+1w {}", (today + Duration::days(7)).format("%b %d")), date: Some(today + Duration::days(7)) },
        DatePreset { label: format!("+2w {}", (today + Duration::days(14)).format("%b %d")), date: Some(today + Duration::days(14)) },
        DatePreset { label: format!("+1mo {}", (today + Duration::days(30)).format("%b %d")), date: Some(today + Duration::days(30)) },
    ]
}

fn date_dropdown(
    current: Option<NaiveDate>,
    on_select: impl Fn(Option<NaiveDate>) -> Message + 'static,
    sender: &Sender,
) -> gtk::DropDown {
    let today = Local::now().date_naive();
    let presets = date_presets(today);

    let mut labels: Vec<String> = presets.iter().map(|p| p.label.clone()).collect();
    let mut dates: Vec<Option<NaiveDate>> = presets.iter().map(|p| p.date).collect();

    let selected: Option<usize> = current.and_then(|d| {
        presets.iter().position(|p| p.date == Some(d))
    });

    // If current date doesn't match any preset, prepend it
    let sel_idx = if let Some(date) = current {
        if selected.is_none() {
            labels.insert(0, date.format("%Y-%m-%d").to_string());
            dates.insert(0, Some(date));
            Some(0)
        } else {
            selected
        }
    } else {
        selected
    };

    let items: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
    let model = gtk::StringList::new(&items);
    let dd = gtk::DropDown::new(Some(model), gtk::Expression::NONE);

    if let Some(idx) = sel_idx {
        dd.set_selected(idx as u32);
    } else {
        dd.set_selected(gtk::INVALID_LIST_POSITION);
    }

    {
        let s = sender.clone();
        dd.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            if idx < dates.len() {
                s.emit(on_select(dates[idx]));
            }
        });
    }

    dd
}

// --- State helpers ---

fn state_to_index(state: &TaskState) -> Option<usize> {
    match state {
        TaskState::Todo => Some(0),
        TaskState::Next => Some(1),
        TaskState::Waiting => Some(2),
        TaskState::Someday => Some(3),
        _ => None,
    }
}

fn index_to_state(idx: usize) -> TaskState {
    match idx {
        0 => TaskState::Todo,
        1 => TaskState::Next,
        2 => TaskState::Waiting,
        3 => TaskState::Someday,
        _ => TaskState::Todo,
    }
}

// --- Sort indicator ---

fn sort_indicator(sort: Option<(SortColumn, bool)>, col: SortColumn) -> &'static str {
    match sort {
        Some((c, true)) if c == col => " \u{25B2}",
        Some((c, false)) if c == col => " \u{25BC}",
        _ => "",
    }
}

// --- Header helpers ---

fn header_label_widget(
    label: &str,
    width: i32,
    sortable: bool,
    sort: Option<(SortColumn, bool)>,
    sort_col: SortColumn,
    sender: &Sender,
) -> gtk::Widget {
    if sortable {
        let display = format!("{}{}", label, sort_indicator(sort, sort_col));
        let btn = ui::button_with_signal(
            &display,
            Some("flat"),
            Message::SetAllTasksSort(sort_col),
            sender,
        );
        btn.add_css_class("caption");
        btn.set_size_request(width, -1);
        btn.upcast()
    } else {
        let l = ui::caption(label);
        l.set_size_request(width, -1);
        l.upcast()
    }
}

fn header_label_fill(
    label: &str,
    sortable: bool,
    sort: Option<(SortColumn, bool)>,
    sort_col: SortColumn,
    sender: &Sender,
) -> gtk::Widget {
    if sortable {
        let display = format!("{}{}", label, sort_indicator(sort, sort_col));
        let btn = ui::button_with_signal(
            &display,
            Some("flat"),
            Message::SetAllTasksSort(sort_col),
            sender,
        );
        btn.add_css_class("caption");
        btn.set_hexpand(true);
        btn.upcast()
    } else {
        let l = ui::caption(label);
        l.set_hexpand(true);
        l.upcast()
    }
}

fn header_row(
    has_projects: bool,
    sortable: bool,
    sort: Option<(SortColumn, bool)>,
    sender: &Sender,
) -> gtk::Widget {
    let r = ui::centered_hbox(8);

    // Empty checkbox column
    let check_spacer = gtk::Label::new(Some(""));
    check_spacer.set_size_request(COL_CHECK, -1);
    r.append(&check_spacer);

    r.append(&header_label_widget(&fl!("col-state"), COL_STATE, sortable, sort, SortColumn::State, sender));
    r.append(&header_label_widget(&fl!("col-priority"), COL_PRI, sortable, sort, SortColumn::Priority, sender));
    r.append(&header_label_fill(&fl!("col-title"), sortable, sort, SortColumn::Title, sender));
    r.append(&header_label_widget(&fl!("col-context"), COL_CTX, sortable, sort, SortColumn::Context, sender));

    if has_projects {
        let proj_label = ui::caption(&fl!("col-project"));
        proj_label.set_size_request(COL_PROJECT, -1);
        r.append(&proj_label);
    }

    r.append(&header_label_widget(&fl!("esc-column"), COL_ESC, sortable, sort, SortColumn::Esc, sender));
    r.append(&header_label_widget(&fl!("col-scheduled"), COL_DATE, sortable, sort, SortColumn::Scheduled, sender));
    r.append(&header_label_widget(&fl!("col-deadline"), COL_DATE, sortable, sort, SortColumn::Deadline, sender));

    let del_spacer = gtk::Label::new(Some(""));
    del_spacer.set_size_request(COL_DELETE, -1);
    r.append(&del_spacer);

    r.set_hexpand(true);
    r.upcast()
}

/// Build a column with header + task rows, all columns aligned via fixed widths.
/// Pass `sort = Some(...)` to enable sortable column headers.
/// Headers are clickable whenever `sort` is provided (even `Some(None)` for no active sort).
pub fn task_grid<'a>(
    tasks: impl Iterator<Item = &'a Task>,
    ctx: &TaskRowCtx,
    sort: Option<Option<(SortColumn, bool)>>,
    sender: &Sender,
) -> gtk::Widget {
    let has_projects = !ctx.project_names.is_empty();
    let sortable = sort.is_some();
    let active_sort = sort.flatten();

    let content = ui::vbox(4);
    content.set_hexpand(true);
    content.append(&header_row(has_projects, sortable, active_sort, sender));

    for task in tasks {
        content.append(&task_row(task, ctx, has_projects, sender));
    }

    content.upcast()
}

fn task_row(
    task: &Task,
    ctx: &TaskRowCtx,
    has_projects: bool,
    sender: &Sender,
) -> gtk::Widget {
    let is_done = task.state.is_done();
    let id = task.id;

    let r = ui::centered_hbox(8);

    // 1. Checkbox
    let check = ui::check_button_with_signal(
        is_done,
        Message::ToggleTaskDone(id),
        sender,
    );
    check.set_size_request(COL_CHECK, -1);
    r.append(&check);

    // 2. State dropdown
    if !is_done {
        let labels: Vec<String> = STATE_LABELS.iter().map(|s| s.to_string()).collect();
        let selected = state_to_index(&task.state);
        let dd = ui::dropdown_with_signal(
            &labels,
            selected,
            move |idx| Message::SetTaskState(id, index_to_state(idx)),
            sender,
        );
        dd.set_size_request(COL_STATE, -1);
        r.append(&dd);
    } else {
        let done_label = ui::caption("done");
        done_label.set_size_request(COL_STATE, -1);
        r.append(&done_label);
    }

    // 3. Priority
    let next_priority = match task.priority {
        None => Some(Priority::A),
        Some(Priority::A) => Some(Priority::B),
        Some(Priority::B) => Some(Priority::C),
        Some(Priority::C) => None,
    };
    let (pri_label, pri_class) = match task.priority {
        Some(Priority::A) => ("A", Some("destructive-action")),
        Some(Priority::B) => ("B", None),
        Some(Priority::C) => ("C", Some("flat")),
        None => ("-", Some("flat")),
    };
    let pri_btn = ui::button_with_signal(
        pri_label,
        pri_class,
        Message::SetTaskPriority(id, next_priority),
        sender,
    );
    pri_btn.set_size_request(COL_PRI, -1);
    r.append(&pri_btn);

    // 4. Title (clickable to expand/collapse notes) + waiting_for label
    let title_box = ui::centered_hbox(6);
    title_box.set_hexpand(true);

    let title_btn = ui::button_with_signal(
        &task.title,
        Some("flat"),
        Message::ToggleTaskExpand(id),
        sender,
    );
    title_box.append(&title_btn);

    if let Some(ref wf) = task.waiting_for {
        let wf_label = format!("\u{2190} @{}", wf);
        let wf_caption = ui::caption(&wf_label);
        title_box.append(&wf_caption);
    }

    r.append(&title_box);

    // 5. Context (tags + add dropdown)
    let ctx_box = ui::centered_hbox(4);
    ctx_box.set_size_request(COL_CTX, -1);

    for ctx_tag in &task.contexts {
        let remove_label = format!("{} x", ctx_tag);
        let ctx_owned = ctx_tag.clone();
        let btn = ui::button_with_signal(
            &remove_label,
            Some("flat"),
            Message::RemoveContext(id, ctx_owned),
            sender,
        );
        btn.add_css_class("caption");
        ctx_box.append(&btn);
    }

    let addable: Vec<String> = ctx.contexts
        .iter()
        .filter(|c| !task.contexts.contains(c))
        .cloned()
        .collect();
    if !addable.is_empty() {
        let addable_for_closure = addable.clone();
        let dd = ui::dropdown_with_signal(
            &addable,
            None,
            move |idx| Message::AddContext(id, addable_for_closure[idx].clone()),
            sender,
        );
        ctx_box.append(&dd);
    }

    r.append(&ctx_box);

    // 6. Project dropdown (conditional)
    if has_projects {
        let names = ctx.project_names.clone();
        let names_for_closure = names.clone();
        let selected: Option<usize> = task
            .project
            .as_ref()
            .and_then(|p| names.iter().position(|n| n == p));
        let dd = ui::dropdown_with_signal(
            &names,
            selected,
            move |idx| Message::MoveToProject(id, names_for_closure[idx].clone()),
            sender,
        );
        dd.set_size_request(COL_PROJECT, -1);
        r.append(&dd);
    }

    // 7. ESC dropdown
    let esc_labels: Vec<String> = ESC_LABELS.iter().map(|s| s.to_string()).collect();
    let esc_selected: Option<usize> = if task.esc.is_none() {
        Some(0)
    } else {
        task.esc.and_then(|v| {
            ESC_VALUES.iter().position(|ev| *ev == Some(v))
        }).or(Some(0))
    };
    let esc_dd = ui::dropdown_with_signal(
        &esc_labels,
        esc_selected,
        move |idx| Message::SetTaskEsc(id, ESC_VALUES[idx]),
        sender,
    );
    esc_dd.set_size_request(COL_ESC, -1);
    r.append(&esc_dd);

    // 8. Scheduled date picker
    let scheduled = task.scheduled;
    let sched_dd = date_dropdown(scheduled, move |d| Message::SetScheduled(id, d), sender);
    sched_dd.set_size_request(COL_DATE, -1);
    r.append(&sched_dd);

    // 9. Deadline date picker
    let deadline = task.deadline;
    let dead_dd = date_dropdown(deadline, move |d| Message::SetDeadline(id, d), sender);
    dead_dd.set_size_request(COL_DATE, -1);
    r.append(&dead_dd);

    // 10. Delete button
    let del_btn = ui::icon_button_with_signal(
        "edit-delete-symbolic",
        Message::DeleteTask(id),
        sender,
    );
    del_btn.set_size_request(COL_DELETE, -1);
    r.append(&del_btn);

    r.set_hexpand(true);

    // If this task is expanded, show notes panel below the row
    if ctx.expanded_task == Some(id) {
        let notes_text = task.notes.clone();
        let input_value = ctx.note_inputs.get(&id).cloned().unwrap_or_default();

        let notes_col = ui::vbox(4);
        notes_col.set_margin_start(36);
        notes_col.set_margin_top(4);
        notes_col.set_margin_bottom(4);

        // Waiting-for input and follow-up date (only for Waiting state tasks)
        if task.state == TaskState::Waiting {
            let wf_value = ctx.waiting_for_inputs
                .get(&id)
                .cloned()
                .unwrap_or_else(|| task.waiting_for.clone().unwrap_or_default());

            let wf_row = ui::centered_hbox(8);
            wf_row.append(&ui::caption("Waiting for:"));

            let wf_value_for_submit = wf_value.clone();
            let wf_input = ui::entry_with_signal(
                "Waiting for...",
                &wf_value,
                move |v| Message::WaitingForInputChanged(id, v),
                Some(Box::new(move || Message::SetWaitingFor(id, wf_value_for_submit.clone()))),
                sender,
            );
            wf_row.append(&wf_input);
            notes_col.append(&wf_row);

            // Follow-up date picker
            let follow_up = task.follow_up;
            let fu_row = ui::centered_hbox(8);
            fu_row.append(&ui::caption("Follow up:"));
            let fu_dd = date_dropdown(follow_up, move |d| Message::SetFollowUp(id, d), sender);
            fu_row.append(&fu_dd);
            notes_col.append(&fu_row);

            // Contact suggestions for waiting_for input
            if !ctx.contacts.is_empty() {
                let current_wf = ctx.waiting_for_inputs
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| task.waiting_for.clone().unwrap_or_default());
                if !current_wf.is_empty() {
                    let prefix = current_wf.to_lowercase();
                    let suggestions: Vec<&Contact> = ctx.contacts
                        .iter()
                        .filter(|c| c.name.to_lowercase().starts_with(&prefix))
                        .take(5)
                        .collect();
                    if !suggestions.is_empty() {
                        let suggestion_row = ui::hbox(4);
                        for contact in suggestions {
                            let name = contact.name.clone();
                            let name2 = name.clone();
                            let btn = ui::button_with_signal(
                                &name,
                                Some("flat"),
                                Message::SetWaitingFor(id, name2),
                                sender,
                            );
                            btn.add_css_class("caption");
                            suggestion_row.append(&btn);
                        }
                        notes_col.append(&suggestion_row);
                    }
                }
            }

            // Show delegated date if set
            if let Some(delegated) = task.delegated {
                let del_text = format!("Delegated: {}", delegated.format("%Y-%m-%d"));
                notes_col.append(&ui::caption(&del_text));
            }
        }

        // Editable title (Enter to confirm + collapse)
        let title_input = ui::entry_with_signal(
            &fl!("task-title-placeholder"),
            &task.title,
            move |v| Message::UpdateTaskTitle(id, v),
            Some(Box::new(move || Message::ToggleTaskExpand(id))),
            sender,
        );
        notes_col.append(&title_input);

        if !notes_text.is_empty() {
            let notes_label = ui::body(&notes_text);
            notes_label.set_margin_start(8);
            notes_label.set_margin_end(8);
            notes_label.set_margin_top(4);
            notes_label.set_margin_bottom(4);
            notes_label.set_hexpand(true);
            notes_col.append(&notes_label);
        }

        let note_input = ui::entry_with_signal(
            &fl!("task-note-placeholder"),
            &input_value,
            move |v| Message::NoteInputChanged(id, v),
            Some(Box::new(move || Message::AppendNote(id))),
            sender,
        );
        notes_col.append(&note_input);

        let wrapper = ui::vbox(0);
        wrapper.set_hexpand(true);
        wrapper.append(&r);
        wrapper.append(&notes_col);
        wrapper.upcast()
    } else {
        r.upcast()
    }
}
