use std::collections::HashMap;

use cosmic::app::{Core, Task as CosmicTask, context_drawer};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, icon, nav_bar, row, scrollable, text, text_input};
use cosmic::{Application, Element, executor};

use crate::config::LampConfig;
use crate::core::habit::Habit;
use crate::core::project::Project;
use crate::core::task::Task;
use crate::core::temporal::DateRange;
use crate::message::{ActiveView, Message, WhatPage, WhenPage};
use crate::org::convert;
use crate::org::writer::OrgWriter;
use crate::pages;

pub struct Lamp {
    core: Core,
    nav_model: nav_bar::Model,
    config: LampConfig,
    active_view: ActiveView,

    // Data
    inbox_tasks: Vec<Task>,
    next_tasks: Vec<Task>,
    waiting_tasks: Vec<Task>,
    someday_tasks: Vec<Task>,
    projects: Vec<Project>,
    habits: Vec<Habit>,

    // Cached for today view (rebuilt on data changes)
    all_tasks_cache: Vec<Task>,

    // UI state
    inbox_input: String,
    project_input: String,
    project_task_inputs: HashMap<String, String>,
    habit_input: String,
    settings_context_input: String,
}

pub struct Flags {
    pub config: LampConfig,
}

impl Application for Lamp {
    type Executor = executor::Default;
    type Flags = Flags;
    type Message = Message;

    const APP_ID: &'static str = "dev.lamp.app";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, CosmicTask<Self::Message>) {
        let config = flags.config;

        // Ensure org files exist
        if let Err(e) = config.ensure_files() {
            log::error!("Failed to create org directory: {}", e);
        }

        // Build sidebar navigation model (What pages only)
        let mut nav_model = nav_bar::Model::default();
        for page in WhatPage::ALL {
            nav_model
                .insert()
                .text(page.title())
                .icon(icon::from_name(page.icon_name()).size(16).icon())
                .data(*page);
        }

        // Load tasks from org files
        let inbox_tasks = load_tasks(&config.inbox_path());
        let next_tasks = load_tasks(&config.next_path());
        let waiting_tasks = load_tasks(&config.waiting_path());
        let someday_tasks = load_tasks(&config.someday_path());
        let projects = load_projects(&config.projects_path());
        let habits = load_habits(&config.habits_path());

        let mut app = Self {
            core,
            nav_model,
            config,
            active_view: ActiveView::When(WhenPage::Today),
            inbox_tasks,
            next_tasks,
            waiting_tasks,
            someday_tasks,
            projects,
            habits,
            all_tasks_cache: Vec::new(),
            inbox_input: String::new(),
            project_input: String::new(),
            project_task_inputs: HashMap::new(),
            habit_input: String::new(),
            settings_context_input: String::new(),
        };
        app.rebuild_cache();

        (app, CosmicTask::none())
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> CosmicTask<Message> {
        if let Some(page) = self.nav_model.data::<WhatPage>(id).cloned() {
            self.active_view = ActiveView::What(page);
            self.nav_model.activate(id);
        }
        CosmicTask::none()
    }

    fn header_center(&self) -> Vec<Element<'_, Message>> {
        Vec::new()
    }

    fn update(&mut self, message: Message) -> CosmicTask<Message> {
        match message {
            Message::SelectWhen(page) => {
                self.active_view = ActiveView::When(page);
                // Deselect sidebar
                self.nav_model.activate(nav_bar::Id::default());
            }

            Message::InboxInputChanged(value) => {
                self.inbox_input = value;
            }

            Message::InboxSubmit => {
                let title = self.inbox_input.trim().to_string();
                if !title.is_empty() {
                    let task = Task::new(title);
                    self.inbox_tasks.push(task);
                    self.inbox_input.clear();
                    self.save_inbox();
                }
            }

            Message::AddTask(title) => {
                let task = Task::new(title);
                self.inbox_tasks.push(task);
                self.save_inbox();
            }

            Message::ToggleTaskDone(id) => {
                self.toggle_done(id);
            }

            Message::SetTaskState(id, state) => {
                self.set_task_state(id, state);
            }

            Message::SetTaskPriority(id, priority) => {
                self.set_task_priority(id, priority);
            }

            Message::DeleteTask(id) => {
                self.inbox_tasks.retain(|t| t.id != id);
                self.next_tasks.retain(|t| t.id != id);
                self.waiting_tasks.retain(|t| t.id != id);
                self.someday_tasks.retain(|t| t.id != id);
                for project in &mut self.projects {
                    project.tasks.retain(|t| t.id != id);
                }
                self.save_all();
            }

            Message::MoveToProject(id, ref project_name) => {
                if let Some(mut task) = self.remove_task(id) {
                    task.project = Some(project_name.clone());
                    if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                        project.tasks.push(task);
                    }
                    self.save_all();
                }
            }

            Message::AddContext(id, ref ctx) => {
                self.modify_task(id, |task| {
                    if !task.contexts.contains(ctx) {
                        task.contexts.push(ctx.clone());
                    }
                });
            }

            Message::RemoveContext(id, ref ctx) => {
                self.modify_task(id, |task| {
                    task.contexts.retain(|c| c != ctx);
                });
            }

            Message::SetScheduled(id, date) => {
                self.modify_task(id, |task| {
                    task.scheduled = date;
                });
            }

            Message::SetDeadline(id, date) => {
                self.modify_task(id, |task| {
                    task.deadline = date;
                });
            }

            Message::ToggleSettings => {
                self.core.window.show_context = !self.core.window.show_context;
            }

            Message::SettingsContextInput(value) => {
                self.settings_context_input = value;
            }

            Message::SettingsAddContext => {
                let ctx = self.settings_context_input.trim().to_string();
                if !ctx.is_empty() && !self.config.contexts.contains(&ctx) {
                    let ctx = if ctx.starts_with('@') { ctx } else { format!("@{}", ctx) };
                    if !self.config.contexts.contains(&ctx) {
                        self.config.contexts.push(ctx);
                    }
                    self.settings_context_input.clear();
                }
            }

            Message::SettingsRemoveContext(idx) => {
                if idx < self.config.contexts.len() {
                    self.config.contexts.remove(idx);
                }
            }

            Message::ProjectInputChanged(value) => {
                self.project_input = value;
            }

            Message::ProjectSubmit => {
                let name = self.project_input.trim().to_string();
                if !name.is_empty() {
                    self.projects.push(crate::core::project::Project::new(name));
                    self.project_input.clear();
                    self.save_projects();
                }
            }

            Message::CreateProject(name) => {
                if !name.is_empty() {
                    self.projects.push(crate::core::project::Project::new(name));
                    self.save_projects();
                }
            }

            Message::DeleteProject(name) => {
                self.projects.retain(|p| p.name != name);
                self.save_projects();
                self.rebuild_cache();
            }

            Message::ProjectTaskInputChanged(ref project_name, ref value) => {
                self.project_task_inputs
                    .insert(project_name.clone(), value.clone());
            }

            Message::AddTaskToProject(ref project_name) => {
                let input = self
                    .project_task_inputs
                    .get(project_name)
                    .cloned()
                    .unwrap_or_default();
                let title = input.trim().to_string();
                if !title.is_empty() {
                    let mut task = Task::new(title);
                    task.project = Some(project_name.clone());
                    if let Some(project) = self.projects.iter_mut().find(|p| p.name == *project_name) {
                        project.tasks.push(task);
                    }
                    self.project_task_inputs.insert(project_name.clone(), String::new());
                    self.save_all();
                }
            }

            Message::CompleteHabit(id) => {
                let today = chrono::Local::now().date_naive();
                if let Some(habit) = self.habits.iter_mut().find(|h| h.task.id == id) {
                    if habit.is_due(today) {
                        habit
                            .completions
                            .push(chrono::Local::now().naive_local());
                        habit.recalculate_streak(today);
                    }
                }
                self.save_habits();
            }

            Message::DeleteHabit(id) => {
                self.habits.retain(|h| h.task.id != id);
                self.save_habits();
            }

            Message::HabitInputChanged(value) => {
                self.habit_input = value;
            }

            Message::HabitSubmit => {
                let title = self.habit_input.trim().to_string();
                if !title.is_empty() {
                    use crate::core::recurrence::{Recurrence, RecurrenceInterval, RecurrenceUnit};

                    let today = chrono::Local::now().date_naive();
                    let mut task = Task::new(title);
                    task.scheduled = Some(today);
                    task.recurrence = Some(Recurrence::Relative(RecurrenceInterval {
                        count: 1,
                        unit: RecurrenceUnit::Day,
                    }));
                    task.contexts.push("habit".to_string());
                    let habit = crate::core::habit::Habit::new(task);
                    self.habits.push(habit);
                    self.habit_input.clear();
                    self.save_habits();
                }
            }

            Message::Save => {
                self.save_all();
            }

            _ => {}
        }

        CosmicTask::none()
    }

    fn header_end(&self) -> Vec<Element<'_, Message>> {
        vec![
            button::icon(icon::from_name("emblem-system-symbolic").size(16))
                .on_press(Message::ToggleSettings)
                .into(),
        ]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Message>> {
        if !self.core.window.show_context {
            return None;
        }

        let mut content = column().spacing(12);

        content = content.push(text::title4("Contexts"));

        // List existing contexts with remove buttons
        for (idx, ctx) in self.config.contexts.iter().enumerate() {
            content = content.push(
                row()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(text::body(ctx.clone()).width(Length::Fill))
                    .push(
                        button::icon(icon::from_name("edit-delete-symbolic").size(16))
                            .on_press(Message::SettingsRemoveContext(idx)),
                    ),
            );
        }

        // Add new context input
        let input = text_input::text_input("New context (e.g. gym)", &self.settings_context_input)
            .on_input(Message::SettingsContextInput)
            .on_submit(|_| Message::SettingsAddContext)
            .width(Length::Fill);

        content = content.push(
            row()
                .spacing(8)
                .push(input)
                .push(
                    button::icon(icon::from_name("list-add-symbolic").size(16))
                        .on_press(Message::SettingsAddContext),
                ),
        );

        Some(context_drawer::context_drawer(
            container(scrollable(content.padding(16)))
                .width(Length::Fill),
            Message::ToggleSettings,
        ).title("Settings"))
    }

    fn view(&self) -> Element<'_, Message> {
        let project_names: Vec<String> = self.projects.iter().map(|p| p.name.clone()).collect();
        let row_ctx = crate::components::task_row::TaskRowCtx {
            contexts: &self.config.contexts,
            project_names: &project_names,
        };

        // When navigation bar at the top of the content area
        let active_when = match self.active_view {
            ActiveView::When(w) => Some(w),
            _ => None,
        };
        let when_buttons = WhenPage::ALL.iter().map(|page| {
            let btn = if active_when == Some(*page) {
                button::suggested(page.title())
            } else {
                button::standard(page.title())
            };
            btn.on_press(Message::SelectWhen(*page)).into()
        });
        let when_bar: Element<'_, Message> = row()
            .spacing(8)
            .padding([8, 16])
            .extend(when_buttons)
            .into();

        let content: Element<'_, Message> = match self.active_view {
            ActiveView::When(when) => {
                let range = match when {
                    WhenPage::Today => DateRange::Today,
                    WhenPage::Tomorrow => DateRange::Tomorrow,
                    WhenPage::ThisWeek => DateRange::ThisWeek,
                    WhenPage::Upcoming => DateRange::Upcoming,
                };
                pages::temporal::temporal_view(
                    &self.all_tasks_cache,
                    &self.habits,
                    range,
                    &row_ctx,
                )
            }
            ActiveView::What(what) => match what {
                WhatPage::Inbox => {
                    pages::inbox::inbox_view(
                        &self.inbox_tasks,
                        &self.inbox_input,
                        &row_ctx,
                    )
                }
                WhatPage::AllTasks => {
                    pages::all_tasks::all_tasks_view(
                        &self.all_tasks_cache,
                        &row_ctx,
                    )
                }
                WhatPage::NextActions => {
                    pages::next_actions::next_actions_view(
                        &self.next_tasks,
                        &row_ctx,
                    )
                }
                WhatPage::Projects => {
                    pages::projects::projects_view(
                        &self.projects,
                        &self.project_input,
                        &self.project_task_inputs,
                        &row_ctx,
                    )
                }
                WhatPage::Waiting => {
                    pages::waiting::waiting_view(
                        &self.waiting_tasks,
                        &row_ctx,
                    )
                }
                WhatPage::Someday => {
                    pages::someday::someday_view(
                        &self.someday_tasks,
                        &row_ctx,
                    )
                }
                WhatPage::Habits => {
                    pages::habits::habits_view(&self.habits, &self.habit_input)
                }
                WhatPage::Review => {
                    pages::review::review_view(
                        &self.inbox_tasks,
                        &self.next_tasks,
                        &self.waiting_tasks,
                        &self.projects,
                        &self.habits,
                    )
                }
            },
        };

        column()
            .push(when_bar)
            .push(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Lamp {
    fn rebuild_cache(&mut self) {
        self.all_tasks_cache = self.all_active_tasks();
    }

    fn all_active_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        tasks.extend(self.inbox_tasks.iter().cloned());
        tasks.extend(self.next_tasks.iter().cloned());
        tasks.extend(self.waiting_tasks.iter().cloned());
        tasks.extend(self.someday_tasks.iter().cloned());
        for project in &self.projects {
            tasks.extend(project.tasks.iter().cloned());
        }
        tasks
    }

    fn toggle_done(&mut self, id: uuid::Uuid) {
        fn toggle_in_list(list: &mut [Task], id: uuid::Uuid) -> bool {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                if task.state.is_done() {
                    task.state = crate::core::task::TaskState::Todo;
                    task.completed = None;
                } else {
                    task.complete();
                }
                return true;
            }
            false
        }

        let mut found = toggle_in_list(&mut self.inbox_tasks, id)
            || toggle_in_list(&mut self.next_tasks, id)
            || toggle_in_list(&mut self.waiting_tasks, id)
            || toggle_in_list(&mut self.someday_tasks, id);

        if !found {
            for project in &mut self.projects {
                if toggle_in_list(&mut project.tasks, id) {
                    found = true;
                    break;
                }
            }
        }

        if found {
            self.save_all();
        }
    }

    fn set_task_state(&mut self, id: uuid::Uuid, state: crate::core::task::TaskState) {
        use crate::core::task::TaskState;

        if let Some(mut task) = self.remove_task(id) {
            // If task belongs to a project, put it back there with the new state
            if let Some(ref project_name) = task.project {
                let project_name = project_name.clone();
                task.state = state;
                if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                    project.tasks.push(task);
                }
            } else {
                task.state = state.clone();
                match state {
                    TaskState::Todo => self.inbox_tasks.push(task),
                    TaskState::Next => self.next_tasks.push(task),
                    TaskState::Waiting => self.waiting_tasks.push(task),
                    TaskState::Someday => self.someday_tasks.push(task),
                    _ => self.inbox_tasks.push(task),
                }
            }
            self.save_all();
        }
    }

    fn set_task_priority(&mut self, id: uuid::Uuid, priority: Option<crate::core::task::Priority>) {
        fn set_in_list(list: &mut [Task], id: uuid::Uuid, priority: Option<crate::core::task::Priority>) -> bool {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                task.priority = priority;
                return true;
            }
            false
        }

        let mut found = set_in_list(&mut self.inbox_tasks, id, priority)
            || set_in_list(&mut self.next_tasks, id, priority)
            || set_in_list(&mut self.waiting_tasks, id, priority)
            || set_in_list(&mut self.someday_tasks, id, priority);

        if !found {
            for project in &mut self.projects {
                if set_in_list(&mut project.tasks, id, priority) {
                    found = true;
                    break;
                }
            }
        }

        if found {
            self.save_all();
        }
    }

    /// Find a task across all lists, apply a mutation, and save.
    fn modify_task(&mut self, id: uuid::Uuid, f: impl FnOnce(&mut Task)) {
        fn find_and_modify(list: &mut [Task], id: uuid::Uuid, f: &mut Option<impl FnOnce(&mut Task)>) -> bool {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                if let Some(func) = f.take() {
                    func(task);
                }
                return true;
            }
            false
        }

        let mut f = Some(f);
        let mut found = find_and_modify(&mut self.inbox_tasks, id, &mut f)
            || find_and_modify(&mut self.next_tasks, id, &mut f)
            || find_and_modify(&mut self.waiting_tasks, id, &mut f)
            || find_and_modify(&mut self.someday_tasks, id, &mut f);

        if !found {
            for project in &mut self.projects {
                if find_and_modify(&mut project.tasks, id, &mut f) {
                    found = true;
                    break;
                }
            }
        }

        if found {
            self.save_all();
        }
    }

    fn remove_task(&mut self, id: uuid::Uuid) -> Option<Task> {
        for list in [
            &mut self.inbox_tasks,
            &mut self.next_tasks,
            &mut self.waiting_tasks,
            &mut self.someday_tasks,
        ] {
            if let Some(pos) = list.iter().position(|t| t.id == id) {
                return Some(list.remove(pos));
            }
        }
        for project in &mut self.projects {
            if let Some(pos) = project.tasks.iter().position(|t| t.id == id) {
                return Some(project.tasks.remove(pos));
            }
        }
        None
    }

    fn save_inbox(&mut self) {
        let content = OrgWriter::write_file("Inbox", &self.inbox_tasks);
        if let Err(e) = std::fs::write(self.config.inbox_path(), &content) {
            log::error!("Failed to save inbox: {}", e);
        }
        self.rebuild_cache();
    }

    fn save_all(&mut self) {
        let saves: Vec<(&str, &[Task], std::path::PathBuf)> = vec![
            ("Inbox", &self.inbox_tasks, self.config.inbox_path()),
            ("Next Actions", &self.next_tasks, self.config.next_path()),
            ("Waiting For", &self.waiting_tasks, self.config.waiting_path()),
            ("Someday/Maybe", &self.someday_tasks, self.config.someday_path()),
        ];

        for (title, tasks, path) in saves {
            let content = OrgWriter::write_file(title, tasks);
            if let Err(e) = std::fs::write(&path, &content) {
                log::error!("Failed to save {}: {}", title, e);
            }
        }
        self.save_projects();
        self.save_habits();
        self.rebuild_cache();
    }

    fn save_projects(&self) {
        let content = OrgWriter::write_projects_file(&self.projects);
        if let Err(e) = std::fs::write(self.config.projects_path(), &content) {
            log::error!("Failed to save projects: {}", e);
        }
    }

    fn save_habits(&self) {
        let mut out = String::new();
        out.push_str("#+TITLE: Habits\n");
        out.push_str("#+TODO: TODO NEXT WAITING SOMEDAY | DONE CANCELLED\n\n");
        for habit in &self.habits {
            out.push_str(&OrgWriter::write_habit_task(&habit.task, &habit.completions));
            out.push('\n');
        }
        if let Err(e) = std::fs::write(self.config.habits_path(), &out) {
            log::error!("Failed to save habits: {}", e);
        }
    }
}

fn load_tasks(path: &std::path::Path) -> Vec<Task> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_tasks(&content),
        Err(_) => Vec::new(),
    }
}

fn load_habits(path: &std::path::Path) -> Vec<Habit> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_habits(&content),
        Err(_) => Vec::new(),
    }
}

fn load_projects(path: &std::path::Path) -> Vec<crate::core::project::Project> {
    match std::fs::read_to_string(path) {
        Ok(content) => convert::parse_projects(&content),
        Err(_) => Vec::new(),
    }
}
