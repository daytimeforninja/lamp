# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
# Preferred: Nix dev shell (provides all system deps)
nix develop
cargo build
cargo run

# Release build
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test test_name

# Check without building
cargo check

# View runtime logs
journalctl --user -t lamp -f
```

Requires Rust 1.85+ (edition 2024). System dependencies for COSMIC/Wayland: libxkbcommon, wayland, vulkan-loader, libinput, systemd, mesa, expat, fontconfig, freetype, openssl.

## Architecture

Lamp is a GTD task manager for the COSMIC desktop (libcosmic/iced) backed by org-mode files. It uses the **Elm architecture**: a single `Lamp` struct holds all state, a `Message` enum describes every possible user action, and `update()` processes messages while `view()` renders the UI.

### Core data flow

```
User Input → Message → application.rs::update() → save to org files → view() → UI
                                                 ↘ async Command (sync, IMAP, AI)
```

### Key modules

- **`application.rs`** — The monolithic app state machine (~3600 lines). Contains the `Lamp` struct with all state, the `update()` match, `view()` dispatch, and persistence logic (`load()`/`save()`). All task collections live here as `Vec<Task>`, `Vec<Project>`, etc.
- **`message.rs`** — The `Message` enum (~100 variants), navigation enums (`WhatPage`, `WhenPage`, `ActiveView`), and `AppMode` (Plan/Do).
- **`core/`** — Domain model types: `Task` (with `TaskState`, `Priority`), `Project`, `Habit`, `DayPlan`, `ListItem`, `Note`, `CalendarEvent`, `Account`, `Recurrence`. All serializable, UI-independent.
- **`org/`** — Org-mode I/O. `parser.rs` reads org files into `ParsedHeading`s (regex-based), `convert.rs` transforms them to domain types, `writer.rs` serializes domain types back to org format. No Emacs dependency.
- **`pages/`** — View functions for each page (inbox, next_actions, projects, daily_planning, do_mode, etc.). Each file exposes a function taking `&Lamp` and returning `Element<Message>`.
- **`components/`** — Reusable widgets: `task_row.rs` (inline-editable task grid), `month_calendar.rs`, `context_tag.rs`, `habit_chart.rs`.
- **`sync/`** — Remote sync engine: `caldav.rs`/`vtodo.rs` (CalDAV tasks ↔ VTODO), `carddav.rs` (contacts), `webdav.rs` (notes), `imap.rs` (email capture), `anthropic.rs` (AI email→task extraction), `merge.rs` (conflict resolution), `keyring.rs` (credential storage via oo7).
- **`config.rs`** — `LampConfig` with file paths. Org files stored in `~/.local/share/lamp/` by default.

### Patterns to follow

- **Adding a new user action**: Add a variant to `Message` in `message.rs`, handle it in the `update()` match in `application.rs`.
- **Adding a new page**: Create a view function in `pages/`, add a variant to `WhatPage`, add to `WhatPage::ALL` and `SECTION_STARTS` if needed, wire up the sidebar nav and view dispatch in `application.rs`.
- **Adding a new domain type**: Define in `core/`, add org parsing in `org/parser.rs` + `org/convert.rs`, add serialization in `org/writer.rs`, add a `Vec<T>` field on `Lamp`, load/save in `application.rs`.
- **Task states**: `TODO` → `NEXT` → `DONE`/`CANCELLED`, with `WAITING` and `SOMEDAY` as alternatives. State transitions happen via `Message::SetTaskState`.
- **Two-mode UI**: Plan mode (full GTD sidebar) and Do mode (focused execution view). Toggled via `Message::SetMode(AppMode)`.
- **Spoon system (ESC)**: Tasks have an optional energy cost (5-100). `DayPlan` tracks a spoon budget and deducts as tasks are completed.

### i18n

Uses fluent via `i18n-embed`. Strings in `i18n/en/lamp.ftl`, accessed with the `fl!("key")` macro.

## Storage format

All data persists as org-mode files (`inbox.org`, `next.org`, `waiting.org`, `someday.org`, `projects.org`, `habits.org`, `media.org`, `shopping.org`, `dayplan.org`, `archive.org`). The org parser/writer must stay compatible with standard Org-mode syntax (`:PROPERTIES:` drawers, `SCHEDULED`/`DEADLINE` timestamps, `:LOGBOOK:` entries, `#+TODO` keyword lines, tags in `:tag:` format).
