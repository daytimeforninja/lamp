use chrono::{NaiveDate, NaiveDateTime};
use uuid::Uuid;

/// UUID v5 namespace for converting non-UUID CalDAV UIDs into stable Uuids.
pub const CALDAV_UUID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
    0xc8,
]);

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y%m%d").to_string()
}

pub fn format_datetime(dt: NaiveDateTime) -> String {
    dt.format("%Y%m%dT%H%M%S").to_string()
}

pub fn parse_ical_date(s: &str) -> Option<NaiveDate> {
    // Handles both "20260224" and date-time formats "20260224T120000"
    let date_str = if s.len() >= 8 { &s[..8] } else { s };
    NaiveDate::parse_from_str(date_str, "%Y%m%d").ok()
}

pub fn parse_ical_datetime(s: &str) -> Option<NaiveDateTime> {
    // "20260224T143000" or "20260224T143000Z"
    let s = s.trim_end_matches('Z');
    NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()
}

pub fn escape_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

pub fn unescape_text(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\;", ";")
        .replace("\\,", ",")
        .replace("\\\\", "\\")
}

/// Parse a line like "KEY;PARAM=VAL:value" -> ("KEY", "value")
pub fn parse_ical_line(line: &str) -> Option<(&str, &str)> {
    let colon_pos = line.find(':')?;
    let key_part = &line[..colon_pos];
    let value = &line[colon_pos + 1..];
    // Strip parameters (e.g., "DTSTART;VALUE=DATE" -> "DTSTART")
    let key = key_part.split(';').next().unwrap_or(key_part);
    Some((key, value))
}

/// Unfold RFC 5545 continuation lines (lines starting with space/tab are appended to previous).
pub fn unfold_lines(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for line in input.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation: append without the leading whitespace
            result.push_str(&line[1..]);
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
    }
    result
}

/// Fold long lines at 75 octets (RFC 5545 requirement).
pub fn fold_line(s: &str) -> String {
    if s.len() <= 75 {
        return s.to_string();
    }
    let mut result = String::new();
    let mut pos = 0;
    while pos < s.len() {
        let mut end = (pos + 75).min(s.len());
        // Don't split in the middle of a multi-byte UTF-8 character
        while end < s.len() && !s.is_char_boundary(end) {
            end -= 1;
        }
        if pos == 0 {
            result.push_str(&s[..end]);
        } else {
            result.push_str("\r\n ");
            result.push_str(&s[pos..end]);
        }
        pos = end;
    }
    result
}
