use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Weekday};
use uuid::Uuid;

use super::ical::*;
use crate::core::event::{CalendarEvent, EventStatus};

/// Generate a full VCALENDAR string containing a VEVENT for the given event.
pub fn event_to_vcalendar(event: &CalendarEvent) -> String {
    let mut lines = Vec::new();
    lines.push("BEGIN:VCALENDAR".to_string());
    lines.push("VERSION:2.0".to_string());
    lines.push("PRODID:-//Lamp GTD//EN".to_string());
    lines.push("BEGIN:VEVENT".to_string());

    lines.push(format!("UID:{}", event.id));

    lines.push(format!(
        "SUMMARY:{}",
        fold_line(&escape_text(&event.title))
    ));

    if event.all_day {
        lines.push(format!(
            "DTSTART;VALUE=DATE:{}",
            format_date(event.start.date())
        ));
        lines.push(format!(
            "DTEND;VALUE=DATE:{}",
            format_date(event.end.date())
        ));
    } else {
        lines.push(format!("DTSTART:{}", format_datetime(event.start)));
        lines.push(format!("DTEND:{}", format_datetime(event.end)));
    }

    if !event.location.is_empty() {
        lines.push(format!(
            "LOCATION:{}",
            fold_line(&escape_text(&event.location))
        ));
    }

    if !event.description.is_empty() {
        lines.push(format!(
            "DESCRIPTION:{}",
            fold_line(&escape_text(&event.description))
        ));
    }

    let status = match event.status {
        EventStatus::Confirmed => "CONFIRMED",
        EventStatus::Tentative => "TENTATIVE",
        EventStatus::Cancelled => "CANCELLED",
    };
    lines.push(format!("STATUS:{}", status));

    lines.push(format!(
        "LAST-MODIFIED:{}",
        format_datetime(chrono::Local::now().naive_local())
    ));

    lines.push("END:VEVENT".to_string());
    lines.push("END:VCALENDAR".to_string());

    lines.join("\r\n") + "\r\n"
}

/// Parse a VCALENDAR string and extract CalendarEvent(s) from the VEVENT component.
/// Recurring events (RRULE) are expanded into individual instances.
pub fn vcalendar_to_events(ical: &str) -> Vec<CalendarEvent> {
    let unfolded = unfold_lines(ical);

    let mut in_vevent = false;
    let mut uid: Option<Uuid> = None;
    let mut uid_raw = String::new();
    let mut summary = String::new();
    let mut dtstart: Option<NaiveDateTime> = None;
    let mut dtend: Option<NaiveDateTime> = None;
    let mut all_day = false;
    let mut location = String::new();
    let mut description = String::new();
    let mut status = EventStatus::Confirmed;
    let mut rrule: Option<String> = None;
    let mut exdates: Vec<NaiveDate> = Vec::new();

    for line in unfolded.lines() {
        let line = line.trim_end();
        if line == "BEGIN:VEVENT" {
            in_vevent = true;
            continue;
        }
        if line == "END:VEVENT" {
            break;
        }
        if !in_vevent {
            continue;
        }

        // Check for VALUE=DATE parameter before parsing
        let is_date_only = line.contains("VALUE=DATE") && !line.contains("VALUE=DATE-TIME");

        if let Some((key, value)) = parse_ical_line(line) {
            match key {
                "UID" => {
                    uid_raw = value.to_string();
                    uid = Some(
                        Uuid::parse_str(value).unwrap_or_else(|_| {
                            Uuid::new_v5(&CALDAV_UUID_NAMESPACE, value.as_bytes())
                        }),
                    );
                }
                "SUMMARY" => summary = unescape_text(value),
                "DTSTART" => {
                    if is_date_only {
                        all_day = true;
                        if let Some(d) = parse_ical_date(value) {
                            dtstart = Some(d.and_hms_opt(0, 0, 0).unwrap());
                        }
                    } else {
                        dtstart = parse_ical_datetime(value);
                    }
                }
                "DTEND" => {
                    if is_date_only {
                        if let Some(d) = parse_ical_date(value) {
                            dtend = Some(d.and_hms_opt(0, 0, 0).unwrap());
                        }
                    } else {
                        dtend = parse_ical_datetime(value);
                    }
                }
                "LOCATION" => location = unescape_text(value),
                "DESCRIPTION" => description = unescape_text(value),
                "STATUS" => {
                    status = match value.to_uppercase().as_str() {
                        "TENTATIVE" => EventStatus::Tentative,
                        "CANCELLED" => EventStatus::Cancelled,
                        _ => EventStatus::Confirmed,
                    };
                }
                "RRULE" => {
                    rrule = Some(value.to_string());
                }
                "EXDATE" => {
                    // EXDATE can be a comma-separated list of dates
                    for part in value.split(',') {
                        let part = part.trim();
                        if let Some(d) = parse_ical_date(part) {
                            exdates.push(d);
                        } else if let Some(dt) = parse_ical_datetime(part) {
                            exdates.push(dt.date());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Skip events with empty SUMMARY
    if summary.is_empty() {
        return Vec::new();
    }

    let base_id = uid.unwrap_or_else(Uuid::new_v4);
    let start = match dtstart {
        Some(s) => s,
        None => return Vec::new(),
    };
    let end = dtend.unwrap_or(start);
    let duration = end - start;

    let base = CalendarEvent {
        id: base_id,
        title: summary,
        start,
        end,
        all_day,
        location,
        description,
        status,
        calendar_href: String::new(),
        calendar_name: String::new(),
        sync_href: None,
        sync_hash: None,
    };

    match rrule {
        None => vec![base],
        Some(rule) => expand_rrule(&base, &uid_raw, &rule, &exdates, duration),
    }
}

/// Backwards-compatible wrapper that returns only the first event (or base instance).
pub fn vcalendar_to_event(ical: &str) -> Option<CalendarEvent> {
    vcalendar_to_events(ical).into_iter().next()
}

/// Expand an RRULE into concrete event instances.
/// Supports FREQ=DAILY, WEEKLY, MONTHLY, YEARLY with COUNT and UNTIL.
/// WEEKLY supports BYDAY. Generates up to 1 year ahead from today.
fn expand_rrule(
    base: &CalendarEvent,
    uid_raw: &str,
    rule: &str,
    exdates: &[NaiveDate],
    duration: Duration,
) -> Vec<CalendarEvent> {
    let params = parse_rrule_params(rule);

    let freq = match params.get("FREQ").map(|s| s.as_str()) {
        Some("DAILY") => RruleFreq::Daily,
        Some("WEEKLY") => RruleFreq::Weekly,
        Some("MONTHLY") => RruleFreq::Monthly,
        Some("YEARLY") => RruleFreq::Yearly,
        _ => return vec![base.clone()],
    };

    let interval: u32 = params
        .get("INTERVAL")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    let count: Option<u32> = params.get("COUNT").and_then(|v| v.parse().ok());

    let until: Option<NaiveDate> = params.get("UNTIL").and_then(|v| {
        parse_ical_date(v).or_else(|| parse_ical_datetime(v).map(|dt| dt.date()))
    });

    let byday: Vec<Weekday> = params
        .get("BYDAY")
        .map(|v| {
            v.split(',')
                .filter_map(|d| match d.trim() {
                    "MO" => Some(Weekday::Mon),
                    "TU" => Some(Weekday::Tue),
                    "WE" => Some(Weekday::Wed),
                    "TH" => Some(Weekday::Thu),
                    "FR" => Some(Weekday::Fri),
                    "SA" => Some(Weekday::Sat),
                    "SU" => Some(Weekday::Sun),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    let today = chrono::Local::now().date_naive();
    // Generate instances from the base date up to 1 year from today
    let horizon = today + Duration::days(365);
    let max_count = count.unwrap_or(1000).min(1000) as usize;

    let mut events = Vec::new();
    let base_date = base.start.date();
    let base_time = base.start.time();

    match freq {
        RruleFreq::Daily => {
            let mut current = base_date;
            let step = Duration::days(interval as i64);
            let mut n = 0;
            while current <= horizon && n < max_count {
                if let Some(u) = until {
                    if current > u {
                        break;
                    }
                }
                if !exdates.contains(&current) {
                    let start = current.and_time(base_time);
                    events.push(make_instance(base, uid_raw, start, duration, n));
                }
                current += step;
                n += 1;
            }
        }
        RruleFreq::Weekly => {
            let days: Vec<Weekday> = if byday.is_empty() {
                vec![base_date.weekday()]
            } else {
                byday
            };
            // Start from the week of the base date
            let step_weeks = Duration::weeks(interval as i64);
            let mut week_start = base_date - Duration::days(base_date.weekday().num_days_from_monday() as i64);
            let mut n = 0;
            while week_start <= horizon && n < max_count {
                for &wd in &days {
                    let current = week_start + Duration::days(wd.num_days_from_monday() as i64);
                    if current < base_date {
                        continue;
                    }
                    if current > horizon {
                        break;
                    }
                    if let Some(u) = until {
                        if current > u {
                            break;
                        }
                    }
                    if !exdates.contains(&current) {
                        let start = current.and_time(base_time);
                        events.push(make_instance(base, uid_raw, start, duration, n));
                    }
                    n += 1;
                    if n >= max_count {
                        break;
                    }
                }
                week_start += step_weeks;
            }
        }
        RruleFreq::Monthly => {
            let mut current = base_date;
            let mut n = 0;
            while current <= horizon && n < max_count {
                if let Some(u) = until {
                    if current > u {
                        break;
                    }
                }
                if !exdates.contains(&current) {
                    let start = current.and_time(base_time);
                    events.push(make_instance(base, uid_raw, start, duration, n));
                }
                // Advance by interval months
                let mut month = current.month() + interval;
                let mut year = current.year();
                while month > 12 {
                    month -= 12;
                    year += 1;
                }
                current = NaiveDate::from_ymd_opt(year, month, current.day().min(28))
                    .or_else(|| NaiveDate::from_ymd_opt(year, month, 28))
                    .unwrap_or(current);
                n += 1;
            }
        }
        RruleFreq::Yearly => {
            let mut current = base_date;
            let mut n = 0;
            while current <= horizon && n < max_count {
                if let Some(u) = until {
                    if current > u {
                        break;
                    }
                }
                if !exdates.contains(&current) {
                    let start = current.and_time(base_time);
                    events.push(make_instance(base, uid_raw, start, duration, n));
                }
                let year = current.year() + interval as i32;
                current = NaiveDate::from_ymd_opt(year, current.month(), current.day().min(28))
                    .or_else(|| NaiveDate::from_ymd_opt(year, current.month(), 28))
                    .unwrap_or(current);
                n += 1;
            }
        }
    }

    events
}

fn make_instance(
    base: &CalendarEvent,
    uid_raw: &str,
    start: NaiveDateTime,
    duration: Duration,
    index: usize,
) -> CalendarEvent {
    let instance_id = if index == 0 {
        base.id
    } else {
        // Stable ID per recurrence instance: hash UID + date
        let key = format!("{}:{}", uid_raw, start.date());
        Uuid::new_v5(&CALDAV_UUID_NAMESPACE, key.as_bytes())
    };
    CalendarEvent {
        id: instance_id,
        title: base.title.clone(),
        start,
        end: start + duration,
        all_day: base.all_day,
        location: base.location.clone(),
        description: base.description.clone(),
        status: base.status.clone(),
        calendar_href: base.calendar_href.clone(),
        calendar_name: base.calendar_name.clone(),
        // Only the base instance owns the sync_href
        sync_href: if index == 0 {
            base.sync_href.clone()
        } else {
            None
        },
        sync_hash: base.sync_hash,
    }
}

enum RruleFreq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

fn parse_rrule_params(rule: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for part in rule.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            map.insert(k.to_uppercase(), v.to_string());
        }
    }
    map
}

/// Compute a content hash for an event (for change detection).
pub fn event_content_hash(event: &CalendarEvent) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    event.title.hash(&mut hasher);
    event.start.to_string().hash(&mut hasher);
    event.end.to_string().hash(&mut hasher);
    event.all_day.hash(&mut hasher);
    event.location.hash(&mut hasher);
    event.description.hash(&mut hasher);
    format!("{:?}", event.status).hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn roundtrip_timed_event() {
        let start = NaiveDate::from_ymd_opt(2026, 2, 25)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 2, 25)
            .unwrap()
            .and_hms_opt(15, 0, 0)
            .unwrap();
        let mut event = CalendarEvent::new("Meeting with Alice".to_string(), start, end);
        event.location = "Conference Room B".to_string();

        let id = event.id;
        let ical = event_to_vcalendar(&event);
        let parsed = vcalendar_to_event(&ical).unwrap();
        assert_eq!(parsed.id, id);
        assert_eq!(parsed.title, "Meeting with Alice");
        assert_eq!(parsed.start, start);
        assert_eq!(parsed.end, end);
        assert!(!parsed.all_day);
        assert_eq!(parsed.location, "Conference Room B");
    }

    #[test]
    fn roundtrip_all_day_event() {
        let start = NaiveDate::from_ymd_opt(2026, 3, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 3, 2)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let mut event = CalendarEvent::new("Conference".to_string(), start, end);
        event.all_day = true;

        let ical = event_to_vcalendar(&event);
        let parsed = vcalendar_to_event(&ical).unwrap();
        assert!(parsed.all_day);
        assert_eq!(parsed.title, "Conference");
    }

    #[test]
    fn content_hash_changes() {
        let start = NaiveDate::from_ymd_opt(2026, 2, 25)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let event1 = CalendarEvent::new("Test".to_string(), start, start);
        let mut event2 = event1.clone();
        event2.title = "Changed".to_string();
        assert_ne!(event_content_hash(&event1), event_content_hash(&event2));
    }

    #[test]
    fn expand_weekly_rrule() {
        let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:weekly-test-123\r\nSUMMARY:Acting Class\r\nDTSTART:20260204T180000\r\nDTEND:20260204T200000\r\nRRULE:FREQ=WEEKLY;BYDAY=WE\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = vcalendar_to_events(ical);
        // Should have many instances (weekly for ~1 year)
        assert!(events.len() > 40, "Expected many weekly instances, got {}", events.len());

        // First instance should be the base date
        assert_eq!(events[0].start.date(), NaiveDate::from_ymd_opt(2026, 2, 4).unwrap());
        assert_eq!(events[0].title, "Acting Class");

        // All instances should be Wednesdays
        for e in &events {
            assert_eq!(e.start.weekday(), Weekday::Wed, "Instance on {:?} is not Wednesday", e.start.date());
        }

        // Duration should be preserved (2 hours)
        for e in &events {
            assert_eq!(e.end - e.start, Duration::hours(2));
        }

        // Each instance should have a unique ID
        let ids: std::collections::HashSet<_> = events.iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), events.len(), "Instance IDs should be unique");
    }

    #[test]
    fn rrule_with_count() {
        let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:count-test\r\nSUMMARY:Limited\r\nDTSTART:20260301T100000\r\nDTEND:20260301T110000\r\nRRULE:FREQ=DAILY;COUNT=5\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = vcalendar_to_events(ical);
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn no_rrule_returns_single() {
        let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:single-test\r\nSUMMARY:One-off\r\nDTSTART:20260301T100000\r\nDTEND:20260301T110000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = vcalendar_to_events(ical);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn skip_empty_summary() {
        let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:abc-123\r\nSUMMARY:\r\nDTSTART:20260225T100000\r\nDTEND:20260225T110000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        assert!(vcalendar_to_event(ical).is_none());
    }
}
