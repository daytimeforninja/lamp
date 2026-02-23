use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Org-mode recurrence patterns.
///
/// - Standard (+1w): next occurrence from the original scheduled date
/// - Relative (.+1d): next occurrence from the completion date
/// - Strict (++1m): next occurrence skipping to the next future date
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recurrence {
    /// +Nd, +Nw, +Nm, +Ny — from original date
    Standard(RecurrenceInterval),
    /// .+Nd, .+Nw, .+Nm, .+Ny — from completion date
    Relative(RecurrenceInterval),
    /// ++Nd, ++Nw, ++Nm, ++Ny — skip to next future occurrence
    Strict(RecurrenceInterval),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecurrenceInterval {
    pub count: u32,
    pub unit: RecurrenceUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceUnit {
    Day,
    Week,
    Month,
    Year,
}

impl RecurrenceInterval {
    pub fn add_to(&self, date: NaiveDate) -> NaiveDate {
        match self.unit {
            RecurrenceUnit::Day => date + chrono::Duration::days(self.count as i64),
            RecurrenceUnit::Week => date + chrono::Duration::weeks(self.count as i64),
            RecurrenceUnit::Month => add_months(date, self.count),
            RecurrenceUnit::Year => add_months(date, self.count * 12),
        }
    }
}

fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let total_months = date.month0() + months;
    let new_year = date.year() + (total_months / 12) as i32;
    let new_month = (total_months % 12) + 1;
    // Clamp day to valid range for the new month
    let max_day = days_in_month(new_year, new_month);
    let new_day = date.day().min(max_day);
    NaiveDate::from_ymd_opt(new_year, new_month, new_day).unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

impl Recurrence {
    /// Compute the next occurrence date.
    ///
    /// - `original`: the original scheduled date
    /// - `completed`: when the task was completed
    /// - `today`: today's date (for Strict mode)
    pub fn next_date(
        &self,
        original: NaiveDate,
        completed: NaiveDate,
        today: NaiveDate,
    ) -> NaiveDate {
        match self {
            Self::Standard(interval) => interval.add_to(original),
            Self::Relative(interval) => interval.add_to(completed),
            Self::Strict(interval) => {
                let mut date = original;
                while date <= today {
                    date = interval.add_to(date);
                }
                date
            }
        }
    }

    /// Parse an org-mode recurrence string like "+1w", ".+1d", "++1m"
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let (kind, rest) = if let Some(rest) = s.strip_prefix(".+") {
            ("relative", rest)
        } else if let Some(rest) = s.strip_prefix("++") {
            ("strict", rest)
        } else if let Some(rest) = s.strip_prefix('+') {
            ("standard", rest)
        } else {
            return None;
        };

        // Parse count and unit (e.g., "1w", "2d", "3m", "1y")
        let (count_str, unit_char) = rest.split_at(rest.len() - 1);
        let count: u32 = count_str.parse().ok()?;
        let unit = match unit_char {
            "d" => RecurrenceUnit::Day,
            "w" => RecurrenceUnit::Week,
            "m" => RecurrenceUnit::Month,
            "y" => RecurrenceUnit::Year,
            _ => return None,
        };

        let interval = RecurrenceInterval { count, unit };
        Some(match kind {
            "relative" => Self::Relative(interval),
            "strict" => Self::Strict(interval),
            _ => Self::Standard(interval),
        })
    }
}

impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (prefix, interval) = match self {
            Self::Standard(i) => ("+", i),
            Self::Relative(i) => (".+", i),
            Self::Strict(i) => ("++", i),
        };
        let unit = match interval.unit {
            RecurrenceUnit::Day => "d",
            RecurrenceUnit::Week => "w",
            RecurrenceUnit::Month => "m",
            RecurrenceUnit::Year => "y",
        };
        write!(f, "{}{}{}", prefix, interval.count, unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard() {
        let r = Recurrence::parse("+1w").unwrap();
        assert!(matches!(r, Recurrence::Standard(_)));
        assert_eq!(r.to_string(), "+1w");
    }

    #[test]
    fn parse_relative() {
        let r = Recurrence::parse(".+1d").unwrap();
        assert!(matches!(r, Recurrence::Relative(_)));
        assert_eq!(r.to_string(), ".+1d");
    }

    #[test]
    fn parse_strict() {
        let r = Recurrence::parse("++1m").unwrap();
        assert!(matches!(r, Recurrence::Strict(_)));
        assert_eq!(r.to_string(), "++1m");
    }

    #[test]
    fn next_date_standard() {
        let original = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let completed = NaiveDate::from_ymd_opt(2026, 2, 5).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 2, 5).unwrap();
        let r = Recurrence::parse("+1w").unwrap();
        assert_eq!(
            r.next_date(original, completed, today),
            NaiveDate::from_ymd_opt(2026, 2, 8).unwrap()
        );
    }

    #[test]
    fn next_date_relative() {
        let original = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let completed = NaiveDate::from_ymd_opt(2026, 2, 5).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 2, 5).unwrap();
        let r = Recurrence::parse(".+1d").unwrap();
        assert_eq!(
            r.next_date(original, completed, today),
            NaiveDate::from_ymd_opt(2026, 2, 6).unwrap()
        );
    }

    #[test]
    fn next_date_strict() {
        let original = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let completed = NaiveDate::from_ymd_opt(2026, 2, 5).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 2, 5).unwrap();
        let r = Recurrence::parse("++1m").unwrap();
        assert_eq!(
            r.next_date(original, completed, today),
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
        );
    }
}
