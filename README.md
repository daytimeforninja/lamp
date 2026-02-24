# Lamp

A GTD (Getting Things Done) task manager for the COSMIC desktop, backed by plain org-mode files.

Lamp gives you a full GTD workflow — inbox capture, next actions, projects, waiting-for, someday/maybe, habits, and weekly review — stored as human-readable `.org` files you can edit with any text editor.

## Features

### GTD Workflow
- **Inbox** — capture tasks quickly, process them later
- **Next Actions** — tasks you've committed to doing next
- **Projects** — multi-task outcomes with stuck detection and completion tracking
- **Waiting For** — delegated tasks you're tracking
- **Someday/Maybe** — ideas for later
- **Weekly Review** — dashboard showing stale projects, stuck items, and inbox count

### Temporal Views
- **Today / Tomorrow / This Week / Upcoming** — date-filtered views based on scheduled dates and deadlines

### Habits
- Daily habit tracking with streak counting
- Logbook-based completion history (compatible with Org-mode habit format)

### Lists
- **Media** — track recommendations (books, movies, shows, etc.)
- **Shopping** — shopping list with notes

### Plan/Do Mode
Lamp has two operating modes, toggled from the header bar:

- **Plan Mode** — the full GTD toolkit with sidebar navigation, temporal views, and a **Daily Planning** page where you:
  - Set your **spoon budget** (energy capacity on a 5-100 scale)
  - Choose **active contexts** (e.g. @home, @computer) to filter suggestions
  - Review **suggested tasks** ranked by urgency, priority, and energy cost
  - **Confirm tasks** and **pick list items** for your day
- **Do Mode** — a focused execution view showing only:
  - A **spoon meter** tracking remaining energy capacity
  - Today's **confirmed tasks** with checkboxes
  - **Due habits** with completion checkboxes
  - **Picked media and shopping items** with checkboxes

### ESC (Estimated Spoon Cost)
Every task can have an ESC value (5-100) representing its estimated energy cost. The Daily Planning suggestion engine uses ESC to recommend tasks that fit your remaining spoon budget. Tasks without an ESC value are always eligible for suggestion.

### Task Properties
- **State**: TODO, NEXT, WAITING, SOMEDAY, DONE, CANCELLED
- **Priority**: A, B, C (or none)
- **Contexts**: tag-based filtering (e.g. @home, @work, @errands)
- **Scheduled date**: when you plan to work on it
- **Deadline**: when it's due
- **Recurrence**: standard (`+1w`), relative (`.+1d`), or strict (`++1m`)
- **ESC**: estimated spoon cost (5-100)
- **Notes**: timestamped note entries
- **Project assignment**: associate tasks with projects

## Org-Mode Format

All data is stored as plain `.org` files in `~/.local/share/lamp/` (configurable):

| File | Contents |
|------|----------|
| `inbox.org` | Unprocessed captured tasks |
| `next.org` | Next actions |
| `waiting.org` | Waiting-for tasks |
| `someday.org` | Someday/maybe tasks |
| `projects.org` | Projects with sub-tasks |
| `habits.org` | Daily habits with logbook |
| `media.org` | Media recommendations |
| `shopping.org` | Shopping list |
| `dayplan.org` | Today's plan (spoon budget, confirmed tasks, picked items) |
| `archive.org` | Archived completed tasks |

Files use standard Org-mode syntax with `#+TODO` keywords, `:PROPERTIES:` drawers, `SCHEDULED`/`DEADLINE` timestamps, and `:LOGBOOK:` entries. You can edit them directly with Emacs, Vim, or any text editor.

Example task:
```org
* NEXT [#A] Fix the leaky faucet :@home:
  SCHEDULED: <2026-02-24 Tue>
  :PROPERTIES:
  :ID: 550e8400-e29b-41d4-a716-446655440000
  :CREATED: [2026-02-23 Mon 14:00]
  :ESC: 20
  :END:
  [2026-02-23 Mon 15:30] Called plumber, no answer
```

## Building

### With Nix (recommended)

```bash
nix build              # Build the package
nix develop            # Enter dev shell with all dependencies
```

### With Cargo

Requires Rust 1.85+ and system libraries for COSMIC/Wayland:

```bash
# Install system dependencies (Fedora example)
sudo dnf install libxkbcommon-devel wayland-devel vulkan-loader-devel \
  libinput-devel systemd-devel mesa-libEGL-devel expat-devel \
  fontconfig-devel freetype-devel

cargo build --release
```

### Just Commands

```
just build     # cargo build
just release   # cargo build --release
just run       # cargo run
just check     # cargo check
just clean     # cargo clean
just install   # Install binary and desktop files
```

## Installation

### NixOS

Add the flake to your inputs and include the package:

```nix
{
  inputs.lamp.url = "github:daytimeforninja/lamp";
}
```

### Manual

```bash
just release
just install DESTDIR=/usr/local
```

This installs the binary, `.desktop` file, and AppStream metadata.

## Configuration

Lamp stores its configuration via COSMIC's config system (`dev.lamp.app`). Settings available in the app's context drawer:

- **Contexts** — add/remove context tags (default: @home, @work, @errands, @computer, @phone, @anywhere)

The org file directory defaults to `~/.local/share/lamp/` and is created automatically on first run.

## Architecture

Lamp is built with [libcosmic](https://github.com/pop-os/libcosmic) (the COSMIC desktop toolkit) using the Elm architecture. The data layer reads and writes standard Org-mode files via a built-in parser and writer — no external dependencies like Emacs are needed.

```
src/
  application.rs          # App state, message handling, view dispatch
  message.rs              # Message enum, AppMode, page enums
  config.rs               # Configuration and file paths
  core/                   # Data model
    task.rs               # Task, TaskState, Priority
    project.rs            # Project with sub-tasks
    habit.rs              # Habit with streak tracking
    list_item.rs          # Generic list item (media, shopping)
    day_plan.rs           # DayPlan (spoon budget, confirmed items)
    recurrence.rs         # Recurrence patterns
    temporal.rs           # Date-range filtering
  org/                    # Org-mode I/O
    parser.rs             # Org file parser
    writer.rs             # Org file serializer
    convert.rs            # Parsed headings to domain objects
  components/
    task_row.rs           # Reusable task grid with inline editing
  pages/                  # View functions for each page
    inbox.rs, next_actions.rs, all_tasks.rs, projects.rs,
    waiting.rs, someday.rs, habits.rs, review.rs,
    list.rs, temporal.rs, daily_planning.rs, do_mode.rs
  i18n/en/lamp.ftl        # English localization strings
```

## License

MIT
