use chrono::NaiveDate;

use crate::core::task::Task;

/// Merge a locally-changed task with a remotely-changed version.
///
/// `base_hash` is the hash at last sync. When both sides changed the same
/// field, the remote version wins (server-authoritative tiebreak).
///
/// Returns the merged task (always based on the local task, with remote
/// field values applied where appropriate).
pub fn merge_tasks(local: &Task, remote: &Task, base: &Task) -> Task {
    let mut merged = local.clone();

    // Title
    if local.title != base.title && remote.title != base.title {
        // Both changed — take remote
        if local.title != remote.title {
            merged.title = remote.title.clone();
        }
    } else if remote.title != base.title {
        merged.title = remote.title.clone();
    }

    // State
    if local.state != base.state && remote.state != base.state {
        if local.state != remote.state {
            merged.state = remote.state.clone();
        }
    } else if remote.state != base.state {
        merged.state = remote.state.clone();
    }

    // Priority
    if local.priority != base.priority && remote.priority != base.priority {
        if local.priority != remote.priority {
            merged.priority = remote.priority;
        }
    } else if remote.priority != base.priority {
        merged.priority = remote.priority;
    }

    // Contexts
    if local.contexts != base.contexts && remote.contexts != base.contexts {
        if local.contexts != remote.contexts {
            merged.contexts = remote.contexts.clone();
        }
    } else if remote.contexts != base.contexts {
        merged.contexts = remote.contexts.clone();
    }

    // Scheduled
    merge_option_date(&mut merged.scheduled, local.scheduled, remote.scheduled, base.scheduled);

    // Deadline
    merge_option_date(&mut merged.deadline, local.deadline, remote.deadline, base.deadline);

    // Notes
    if local.notes != base.notes && remote.notes != base.notes {
        if local.notes != remote.notes {
            merged.notes = remote.notes.clone();
        }
    } else if remote.notes != base.notes {
        merged.notes = remote.notes.clone();
    }

    // Project
    merge_option_string(&mut merged.project, &local.project, &remote.project, &base.project);

    // Waiting for
    merge_option_string(&mut merged.waiting_for, &local.waiting_for, &remote.waiting_for, &base.waiting_for);

    // ESC
    if local.esc != base.esc && remote.esc != base.esc {
        if local.esc != remote.esc {
            merged.esc = remote.esc;
        }
    } else if remote.esc != base.esc {
        merged.esc = remote.esc;
    }

    // Delegated
    merge_option_date(&mut merged.delegated, local.delegated, remote.delegated, base.delegated);

    // Follow up
    merge_option_date(&mut merged.follow_up, local.follow_up, remote.follow_up, base.follow_up);

    // Completed
    if local.completed != base.completed && remote.completed != base.completed {
        if local.completed != remote.completed {
            merged.completed = remote.completed;
        }
    } else if remote.completed != base.completed {
        merged.completed = remote.completed;
    }

    // Recurrence
    let local_rec = local.recurrence.as_ref().map(|r| r.to_string());
    let remote_rec = remote.recurrence.as_ref().map(|r| r.to_string());
    let base_rec = base.recurrence.as_ref().map(|r| r.to_string());
    if local_rec != base_rec && remote_rec != base_rec {
        if local_rec != remote_rec {
            merged.recurrence = remote.recurrence.clone();
        }
    } else if remote_rec != base_rec {
        merged.recurrence = remote.recurrence.clone();
    }

    merged
}

fn merge_option_date(
    target: &mut Option<NaiveDate>,
    local: Option<NaiveDate>,
    remote: Option<NaiveDate>,
    base: Option<NaiveDate>,
) {
    if local != base && remote != base {
        if local != remote {
            *target = remote;
        }
    } else if remote != base {
        *target = remote;
    }
}

fn merge_option_string(
    target: &mut Option<String>,
    local: &Option<String>,
    remote: &Option<String>,
    base: &Option<String>,
) {
    if local != base && remote != base {
        if local != remote {
            *target = remote.clone();
        }
    } else if remote != base {
        *target = remote.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::task::{Priority, Task, TaskState};
    use chrono::NaiveDate;

    fn make_base() -> Task {
        let mut t = Task::new("Base task");
        t.priority = Some(Priority::B);
        t.contexts = vec!["@home".to_string()];
        t
    }

    #[test]
    fn no_conflict_different_fields() {
        let base = make_base();
        let mut local = base.clone();
        local.title = "Local title".to_string();

        let mut remote = base.clone();
        remote.priority = Some(Priority::A);

        let merged = merge_tasks(&local, &remote, &base);
        assert_eq!(merged.title, "Local title"); // local change kept
        assert_eq!(merged.priority, Some(Priority::A)); // remote change kept
    }

    #[test]
    fn same_field_both_changed_remote_wins() {
        let base = make_base();
        let mut local = base.clone();
        local.title = "Local title".to_string();

        let mut remote = base.clone();
        remote.title = "Remote title".to_string();

        let merged = merge_tasks(&local, &remote, &base);
        assert_eq!(merged.title, "Remote title");
    }

    #[test]
    fn same_field_same_value_no_conflict() {
        let base = make_base();
        let mut local = base.clone();
        local.title = "Same title".to_string();

        let mut remote = base.clone();
        remote.title = "Same title".to_string();

        let merged = merge_tasks(&local, &remote, &base);
        assert_eq!(merged.title, "Same title");
    }

    #[test]
    fn only_local_changed() {
        let base = make_base();
        let mut local = base.clone();
        local.scheduled = Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());

        let remote = base.clone();
        let merged = merge_tasks(&local, &remote, &base);
        assert_eq!(
            merged.scheduled,
            Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap())
        );
    }

    #[test]
    fn only_remote_changed() {
        let base = make_base();
        let local = base.clone();

        let mut remote = base.clone();
        remote.state = TaskState::Next;

        let merged = merge_tasks(&local, &remote, &base);
        assert_eq!(merged.state, TaskState::Next);
    }
}
