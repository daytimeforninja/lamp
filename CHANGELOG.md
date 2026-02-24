# Changelog

## [0.2.0] - 2026-02-24

### Added

#### Plan/Do Mode System
- **App modes**: Lamp now has two modes toggled from the header bar — **Plan** (full GTD toolkit) and **Do** (focused execution view).
- **Plan mode**: Shows the full sidebar, temporal When bar, and all existing pages. This is the default mode.
- **Do mode**: Hides the sidebar and When bar. Shows only today's confirmed tasks, due habits, and picked list items in a clean, minimal interface.

#### Daily Planning Page
- New **Daily Planning** page (first item in the sidebar, sunrise icon).
- **Spoon budget**: Row of preset buttons (5, 10, 20, 30, 50, 75, 100) to set your energy capacity for the day.
- **Active contexts**: Toggle buttons for each configured context (e.g. @home, @computer) to filter task suggestions.
- **Suggestion engine**: Automatically recommends tasks based on:
  - Eligibility: NEXT state, or scheduled/deadline on or before today
  - Context filtering: task has no contexts, or at least one matches active contexts
  - Budget fit: task's ESC fits remaining spoon budget (no-ESC tasks always eligible)
  - Sorting: overdue/scheduled first, then priority (A > B > C > none), then ESC ascending
  - Excludes already-confirmed and session-rejected tasks
- **Confirm/Skip**: Accept suggestions into today's plan or skip them for the session.
- **Today's Plan section**: Shows confirmed tasks with remove buttons, plus add/remove toggles for media and shopping items from their master lists.
- **Persistence**: Day plan saved to `dayplan.org` with date, spoon budget, active contexts, and all confirmed/picked item IDs. Automatically cleared on a new day.

#### Do Mode View
- **Spoon meter**: Shows remaining/total spoons with color-coded status (green >50%, yellow 25-50%, red <25%).
- **Tasks section**: Confirmed tasks with checkboxes that mark them Done (uses existing `toggle_done` logic including recurrence handling).
- **Habits section**: All due (not yet completed today) habits with checkboxes. Completing a habit logs the timestamp and updates the streak.
- **Media and Shopping sections**: Picked list items with checkboxes that remove them from today's plan (items remain in their master lists).
- **Empty state**: Clear message directing users to Plan mode when no plan exists.

#### ESC (Estimated Spoon Cost)
- New `esc` field on every task — an optional energy estimate on a scale of 5 to 100.
- **ESC column** in task rows: Dropdown with preset values (-, 5, 10, 15, 20, 25, 30, 40, 50, 75, 100) appears in every task grid.
- **Org persistence**: ESC stored as `:ESC: N` in the `:PROPERTIES:` drawer, parsed on load and written on save.
- **Budget integration**: Daily Planning uses ESC to calculate committed spoons and filter suggestions that fit the remaining budget.

#### Data Model
- `DayPlan` struct (`src/core/day_plan.rs`): date, spoon budget, active contexts, confirmed task IDs, picked media IDs, picked shopping IDs.
  - `committed_esc(tasks)`: sum of ESC for all confirmed tasks (done or not — spent spoons don't come back).
  - `remaining_budget(tasks)`: budget minus committed ESC.
  - `is_stale(today)`: true if the plan's date doesn't match today.
- `AppMode` enum: `Plan` and `Do`.
- `DailyPlanning` variant added to `WhatPage` (first position in sidebar).

#### Messages
- `SetMode(AppMode)` — toggle between Plan and Do.
- `SetTaskEsc(Uuid, Option<u32>)` — set/clear a task's ESC value.
- `SetSpoonBudget(u32)` — set the day's spoon budget.
- `TogglePlanContext(String)` — toggle a context for daily planning filtering.
- `ConfirmTask(Uuid)` / `UnconfirmTask(Uuid)` — add/remove a task from today's plan.
- `RejectSuggestion(Uuid)` — skip a suggestion for this session.
- `PickMediaItem(Uuid)` / `UnpickMediaItem(Uuid)` — add/remove media item from today's plan.
- `PickShoppingItem(Uuid)` / `UnpickShoppingItem(Uuid)` — add/remove shopping item from today's plan.
- `DoMarkDone(Uuid)` — toggle task completion from Do mode.
- `DoMarkListItemDone(Uuid)` — remove a list item from today's picked lists.

#### Configuration
- `dayplan_path()` method on `LampConfig` — returns path to `dayplan.org`.

#### Localization
- Added i18n strings for mode toggle, daily planning (budget, contexts, suggestions, plan sections), Do mode (spoon meter, sections, empty states), and ESC column.

### Fixed
- Removed `.size(16)` calls on `icon::from_name()` in habits and projects pages (incompatible with current libcosmic API).

## [0.1.0] - 2026-02-23

### Added
- Initial release: full GTD workflow with inbox, next actions, projects, waiting-for, someday/maybe.
- Temporal views: today, tomorrow, this week, upcoming.
- Daily habit tracking with streaks and logbook.
- Media recommendations and shopping lists.
- Weekly review dashboard.
- Org-mode file backend with full read/write support.
- COSMIC desktop integration with sidebar navigation and context drawer settings.
- i18n support via Fluent.
- Nix flake for building and development.
