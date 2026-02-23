use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use super::task::Task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub task: Task,
    pub completions: Vec<NaiveDateTime>,
    pub streak: u32,
    pub best_streak: u32,
}

impl Habit {
    pub fn new(task: Task) -> Self {
        Self {
            task,
            completions: Vec::new(),
            streak: 0,
            best_streak: 0,
        }
    }

    /// Recalculate streak from completion history.
    pub fn recalculate_streak(&mut self, today: NaiveDate) {
        if self.completions.is_empty() {
            self.streak = 0;
            return;
        }

        let mut dates: Vec<NaiveDate> = self.completions.iter().map(|dt| dt.date()).collect();
        dates.sort();
        dates.dedup();

        // Count consecutive days ending at today (or yesterday)
        let mut streak = 0u32;
        let mut check_date = today;

        // Allow today to be incomplete — check yesterday first if today not done
        if !dates.contains(&today) {
            check_date = today.pred_opt().unwrap_or(today);
        }

        for date in dates.iter().rev() {
            if *date == check_date {
                streak += 1;
                check_date = check_date.pred_opt().unwrap_or(check_date);
            } else if *date < check_date {
                break;
            }
        }

        self.streak = streak;

        // Compute best streak
        let mut best = 0u32;
        let mut current = 1u32;
        for window in dates.windows(2) {
            let diff = (window[1] - window[0]).num_days();
            if diff == 1 {
                current += 1;
            } else {
                best = best.max(current);
                current = 1;
            }
        }
        self.best_streak = best.max(current).max(self.streak);
    }

    pub fn is_due(&self, today: NaiveDate) -> bool {
        // A habit is due if it hasn't been completed today
        !self.completions.iter().any(|dt| dt.date() == today)
    }
}
