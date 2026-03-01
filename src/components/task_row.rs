use std::collections::HashMap;

use chrono::{Duration, Local, NaiveDate, Datelike, Weekday};
use uuid::Uuid;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, checkbox, column, container, dropdown, icon, row, text, text_input};
use cosmic::{Element, theme};

use crate::core::task::{Priority, Task, TaskState};
use crate::message::{Message, SortColumn};

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
const COL_CHECK: f32 = 28.0;
const COL_STATE: f32 = 76.0;
const COL_PRI: f32 = 32.0;
const COL_CTX: f32 = 120.0;
const COL_PROJECT: f32 = 100.0;
const COL_DATE: f32 = 96.0;
const COL_ESC: f32 = 48.0;
const COL_DELETE: f32 = 40.0;

use crate::sync::carddav::Contact;

/// Context passed to task grid.
pub struct TaskRowCtx<'a> {
    pub contexts: &'a [String],
    pub project_names: &'a [String],
    pub expanded_task: Option<Uuid>,
    pub note_inputs: &'a HashMap<Uuid, String>,
    pub waiting_for_inputs: &'a HashMap<Uuid, String>,
    pub contacts: &'a [Contact],
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
        DatePreset { label: "—".into(), date: None },
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
    on_select: impl Fn(Option<NaiveDate>) -> Message + Send + Sync + 'static,
) -> Element<'static, Message> {
    let today = Local::now().date_naive();
    let presets = date_presets(today);

    let labels: Vec<String> = presets.iter().map(|p| p.label.clone()).collect();
    let selected: Option<usize> = current.and_then(|d| {
        presets.iter().position(|p| p.date == Some(d))
    });

    // If current date doesn't match any preset, prepend it
    if let Some(date) = current {
        if selected.is_none() {
            let mut custom_labels = vec![date.format("%Y-%m-%d").to_string()];
            custom_labels.extend(labels);
            let mut custom_dates: Vec<Option<NaiveDate>> = vec![Some(date)];
            custom_dates.extend(presets.iter().map(|p| p.date));

            return dropdown(custom_labels, Some(0usize), move |idx| {
                on_select(custom_dates[idx])
            })
            .width(Length::Shrink)
            .into();
        }
    }

    let dates: Vec<Option<NaiveDate>> = presets.iter().map(|p| p.date).collect();
    dropdown(labels, selected, move |idx| {
        on_select(dates[idx])
    })
    .width(Length::Shrink)
    .into()
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

// --- Fixed-width column helpers ---

fn col(width: f32, content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    container(content).width(Length::Fixed(width)).into()
}

fn col_fill(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    container(content).width(Length::Fill).into()
}

// --- Table-based task list ---

fn sort_indicator(sort: Option<(SortColumn, bool)>, col: SortColumn) -> &'static str {
    match sort {
        Some((c, true)) if c == col => " ▲",
        Some((c, false)) if c == col => " ▼",
        _ => "",
    }
}

fn header_label(
    label: &str,
    width: f32,
    sortable: bool,
    sort: Option<(SortColumn, bool)>,
    sort_col: SortColumn,
) -> Element<'static, Message> {
    if sortable {
        let display = format!("{}{}", label, sort_indicator(sort, sort_col));
        col(width,
            button::custom(text::caption(display).size(12.0))
                .padding([0, 0])
                .class(theme::Button::Text)
                .on_press(Message::SetAllTasksSort(sort_col)),
        )
    } else {
        col(width, text::caption(label.to_string()))
    }
}

fn header_label_fill(
    label: &str,
    sortable: bool,
    sort: Option<(SortColumn, bool)>,
    sort_col: SortColumn,
) -> Element<'static, Message> {
    if sortable {
        let display = format!("{}{}", label, sort_indicator(sort, sort_col));
        col_fill(
            button::custom(text::caption(display).size(12.0))
                .padding([0, 0])
                .class(theme::Button::Text)
                .on_press(Message::SetAllTasksSort(sort_col)),
        )
    } else {
        col_fill(text::caption(label.to_string()))
    }
}

fn header_row(has_projects: bool, sortable: bool, sort: Option<(SortColumn, bool)>) -> Element<'static, Message> {
    let mut r = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(col(COL_CHECK, text::caption("")))
        .push(header_label("State", COL_STATE, sortable, sort, SortColumn::State))
        .push(header_label("Pri", COL_PRI, sortable, sort, SortColumn::Priority))
        .push(header_label_fill("Title", sortable, sort, SortColumn::Title))
        .push(header_label("Context", COL_CTX, sortable, sort, SortColumn::Context));

    if has_projects {
        r = r.push(col(COL_PROJECT, text::caption("Project".to_string())));
    }

    r = r
        .push(header_label("ESC", COL_ESC, sortable, sort, SortColumn::Esc))
        .push(header_label("Sched", COL_DATE, sortable, sort, SortColumn::Scheduled))
        .push(header_label("Due", COL_DATE, sortable, sort, SortColumn::Deadline))
        .push(col(COL_DELETE, text::caption("")));

    r.width(Length::Fill).into()
}

/// Build a column with header + task rows, all columns aligned via fixed widths.
/// Pass `sort = Some(...)` to enable sortable column headers.
/// Headers are clickable whenever `sort` is provided (even `Some(None)` for no active sort).
pub fn task_grid<'a>(
    tasks: impl Iterator<Item = &'a Task>,
    ctx: &TaskRowCtx,
    sort: Option<Option<(SortColumn, bool)>>,
) -> Element<'static, Message> {
    let has_projects = !ctx.project_names.is_empty();
    let sortable = sort.is_some();
    let active_sort = sort.flatten();

    let mut content = column()
        .spacing(4)
        .width(Length::Fill)
        .push(header_row(has_projects, sortable, active_sort));

    for task in tasks {
        content = content.push(task_row(task, ctx, has_projects));
    }

    content.into()
}

fn task_row(
    task: &Task,
    ctx: &TaskRowCtx,
    has_projects: bool,
) -> Element<'static, Message> {
    let is_done = task.state.is_done();
    let id = task.id;

    // 1. Checkbox
    let check: Element<'static, Message> = col(COL_CHECK,
        checkbox("", is_done)
            .on_toggle(move |_| Message::ToggleTaskDone(id)),
    );

    // 2. State dropdown
    let state: Element<'static, Message> = if !is_done {
        let labels: Vec<String> = STATE_LABELS.iter().map(|s| s.to_string()).collect();
        let selected = state_to_index(&task.state);
        col(COL_STATE,
            dropdown(labels, selected, move |idx| {
                Message::SetTaskState(id, index_to_state(idx))
            })
            .width(Length::Shrink),
        )
    } else {
        col(COL_STATE, text::caption("done"))
    };

    // 3. Priority
    let next_priority = match task.priority {
        None => Some(Priority::A),
        Some(Priority::A) => Some(Priority::B),
        Some(Priority::B) => Some(Priority::C),
        Some(Priority::C) => None,
    };
    let (pri_label, pri_style) = match task.priority {
        Some(Priority::A) => ("A", theme::Button::Destructive),
        Some(Priority::B) => ("B", theme::Button::Standard),
        Some(Priority::C) => ("C", theme::Button::Text),
        None => ("-", theme::Button::Text),
    };
    let priority: Element<'static, Message> = col(COL_PRI,
        button::custom(text::body(pri_label).size(12.0))
            .padding([2, 6])
            .class(pri_style)
            .on_press(Message::SetTaskPriority(id, next_priority)),
    );

    // 4. Title (clickable to expand/collapse notes) + waiting_for label
    let title: Element<'static, Message> = {
        let title_btn: Element<'static, Message> = button::custom(text::body(task.title.clone()))
            .padding([0, 0])
            .class(theme::Button::Text)
            .on_press(Message::ToggleTaskExpand(id))
            .into();
        if let Some(ref wf) = task.waiting_for {
            let label = format!("\u{2190} @{}", wf);
            col_fill(
                row()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(title_btn)
                    .push(text::caption(label).size(11.0)),
            )
        } else {
            col_fill(title_btn)
        }
    };

    // 5. Context (tags + add dropdown)
    let mut ctx_items: Vec<Element<'static, Message>> = Vec::new();
    for ctx_tag in &task.contexts {
        let ctx_owned = ctx_tag.clone();
        ctx_items.push(
            button::custom(text::caption(format!("{} x", ctx_tag)).size(11.0))
                .padding([2, 8])
                .class(theme::Button::Text)
                .on_press(Message::RemoveContext(id, ctx_owned))
                .into(),
        );
    }
    let addable: Vec<String> = ctx.contexts
        .iter()
        .filter(|c| !task.contexts.contains(c))
        .cloned()
        .collect();
    if !addable.is_empty() {
        let addable_for_closure = addable.clone();
        ctx_items.push(
            dropdown(addable, None::<usize>, move |idx| {
                Message::AddContext(id, addable_for_closure[idx].clone())
            })
            .width(Length::Shrink)
            .into(),
        );
    }
    let context: Element<'static, Message> = col(COL_CTX,
        row::with_children(ctx_items).spacing(4).align_y(Alignment::Center),
    );

    // 6. Build the row
    let mut r = row()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(check)
        .push(state)
        .push(priority)
        .push(title)
        .push(context);

    // 7. Project dropdown (conditional)
    if has_projects {
        let names = ctx.project_names.to_vec();
        let names_for_closure = names.clone();
        let selected: Option<usize> = task
            .project
            .as_ref()
            .and_then(|p| names.iter().position(|n| n == p));
        r = r.push(col(COL_PROJECT,
            dropdown(names, selected, move |idx| {
                Message::MoveToProject(id, names_for_closure[idx].clone())
            })
            .width(Length::Shrink),
        ));
    }

    // 8. ESC dropdown
    let esc_labels: Vec<String> = ESC_LABELS.iter().map(|s| s.to_string()).collect();
    let esc_selected: Option<usize> = task.esc.and_then(|v| {
        ESC_VALUES.iter().position(|ev| *ev == Some(v))
    }).or(Some(0)); // Default to "-" when None
    let esc_selected = if task.esc.is_none() { Some(0) } else { esc_selected };
    r = r.push(col(COL_ESC,
        dropdown(esc_labels, esc_selected, move |idx| {
            Message::SetTaskEsc(id, ESC_VALUES[idx])
        })
        .width(Length::Shrink),
    ));

    // 9. Scheduled date picker
    let scheduled = task.scheduled;
    r = r.push(col(COL_DATE, date_dropdown(scheduled, move |d| Message::SetScheduled(id, d))));

    // 10. Deadline date picker
    let deadline = task.deadline;
    r = r.push(col(COL_DATE, date_dropdown(deadline, move |d| Message::SetDeadline(id, d))));

    // 11. Delete button
    r = r.push(col(COL_DELETE,
        button::icon(icon::from_name("edit-delete-symbolic"))
            .on_press(Message::DeleteTask(id)),
    ));

    let data_row: Element<'static, Message> = r.width(Length::Fill).into();

    // If this task is expanded, show notes panel below the row
    if ctx.expanded_task == Some(id) {
        let notes_text = task.notes.clone();
        let input_value = ctx.note_inputs.get(&id).cloned().unwrap_or_default();

        let mut notes_col = column().spacing(4).padding([4, 0, 4, 36]);

        // Waiting-for input and follow-up date (only for Waiting state tasks)
        if task.state == TaskState::Waiting {
            let wf_value = ctx.waiting_for_inputs
                .get(&id)
                .cloned()
                .unwrap_or_else(|| task.waiting_for.clone().unwrap_or_default());
            let wf_input = text_input::text_input("Waiting for...", wf_value)
                .on_input(move |v| Message::WaitingForInputChanged(id, v))
                .on_submit(move |_| Message::SetWaitingFor(id, String::new()))
                .width(Length::Fill);
            notes_col = notes_col.push(
                row()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(text::caption("Waiting for:"))
                    .push(wf_input),
            );

            // Follow-up date picker
            let follow_up = task.follow_up;
            notes_col = notes_col.push(
                row()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(text::caption("Follow up:"))
                    .push(date_dropdown(follow_up, move |d| Message::SetFollowUp(id, d))),
            );

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
                        let mut suggestion_row = row().spacing(4);
                        for contact in suggestions {
                            let name = contact.name.clone();
                            suggestion_row = suggestion_row.push(
                                button::custom(text::caption(name.clone()).size(11.0))
                                    .padding([2, 8])
                                    .class(theme::Button::Text)
                                    .on_press(Message::SetWaitingFor(id, name)),
                            );
                        }
                        notes_col = notes_col.push(suggestion_row);
                    }
                }
            }

            // Show delegated date if set
            if let Some(delegated) = task.delegated {
                notes_col = notes_col.push(
                    text::caption(format!("Delegated: {}", delegated.format("%Y-%m-%d"))).size(11.0),
                );
            }
        }

        // Editable title (Enter to confirm + collapse)
        let title_input = text_input::text_input("Task title...", task.title.clone())
            .on_input(move |v| Message::UpdateTaskTitle(id, v))
            .on_submit(move |_| Message::ToggleTaskExpand(id))
            .width(Length::Fill);
        notes_col = notes_col.push(title_input);

        if !notes_text.is_empty() {
            notes_col = notes_col.push(
                container(text::body(notes_text))
                    .padding([4, 8])
                    .width(Length::Fill),
            );
        }

        let note_input = text_input::text_input("Add a note...", input_value)
            .on_input(move |v| Message::NoteInputChanged(id, v))
            .on_submit(move |_| Message::AppendNote(id))
            .width(Length::Fill);

        notes_col = notes_col.push(note_input);

        column()
            .push(data_row)
            .push(notes_col)
            .width(Length::Fill)
            .into()
    } else {
        data_row
    }
}
