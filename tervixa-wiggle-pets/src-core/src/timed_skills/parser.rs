//! Deterministic chrono/regex intent parser.
//! Parses commands like:
//!   "remind me to check the build in 20 minutes"
//!   "remind me every day at 8:00 PM to push my code"
//!   "remind me every monday and thursday at 6pm to back up the database"
//!   "remind me in 2 hours to stretch"
//!   "keep in mind: API port for WaveMuzix is 8085"
//!   "note: deploy key is rotated weekly"
//! Zero AI — pure regex + arithmetic.

use super::Reminder;
use crate::db::Db;
use chrono::Local;

#[derive(Debug)]
pub enum Intent {
    Reminder(ReminderDraft),
    Note(String),
    Unknown(String),
}

#[derive(Debug)]
pub struct ReminderDraft {
    pub text: String,
    pub kind: String,
    pub at_epoch: Option<i64>,
    pub interval_min: Option<i64>,
    pub daily_hhmm: Option<String>,
    pub weekdays: Option<String>,
}

pub fn parse(input: &str) -> Intent {
    let raw = input.trim();
    let lower = raw.to_lowercase();

    if lower.starts_with("keep in mind") || lower.starts_with("note:") || lower.starts_with("remember:") {
        let body = raw
            .split_once(':')
            .map(|(_, b)| b.trim().to_string())
            .unwrap_or_else(|| raw.to_string());
        return Intent::Note(body);
    }

    if !(lower.starts_with("remind") || lower.starts_with("alert") || lower.starts_with("timer")) {
        return Intent::Unknown(raw.to_string());
    }

    // Split off the trailing task text.
    let body = strip_leading(raw);

    // 1) "in N minutes/hours/seconds"
    if let Some(draft) = parse_in(body.clone()) {
        return Intent::Reminder(draft);
    }
    // 2) "every day at HH[:MM] [am|pm] to TASK"
    if let Some(draft) = parse_every_day(body.clone()) {
        return Intent::Reminder(draft);
    }
    // 3) "every monday and thursday at HH[:MM] [am|pm] to TASK"
    if let Some(draft) = parse_weekly(body.clone()) {
        return Intent::Reminder(draft);
    }
    // 4) "every N minutes/hours to TASK"
    if let Some(draft) = parse_every_interval(body.clone()) {
        return Intent::Reminder(draft);
    }
    // 5) "tomorrow at HH to TASK"
    if let Some(draft) = parse_at(body.clone()) {
        return Intent::Reminder(draft);
    }

    Intent::Unknown(raw.to_string())
}

pub fn parse_and_store(db: &Db, input: &str) -> Result<String, String> {
    match parse(input) {
        Intent::Note(text) => {
            let r = Reminder {
                id: 0,
                text: String::new(),
                kind: "note".into(),
                at_epoch: None,
                interval_min: None,
                daily_hhmm: None,
                weekdays: None,
                note: Some(text.clone()),
                enabled: true,
            };
            let id = super::add_reminder(db, &r)?;
            Ok(format!("Saved note #{id}: {text}"))
        }
        Intent::Reminder(d) => {
            let r = Reminder {
                id: 0,
                text: d.text,
                kind: d.kind,
                at_epoch: d.at_epoch,
                interval_min: d.interval_min,
                daily_hhmm: d.daily_hhmm,
                weekdays: d.weekdays,
                note: None,
                enabled: true,
            };
            let id = super::add_reminder(db, &r)?;
            Ok(format!("Reminder #{id} set: {}", r.text))
        }
        Intent::Unknown(text) => Ok(format!(
            "Could not parse a time from \"{text}\". Try: remind me in 20 minutes to ..., \
             remind me every day at 8pm to ..., or keep in mind: ..."
        )),
    }
}

fn strip_leading(s: &str) -> String {
    let lower = s.to_lowercase();
    let cut = [
        "remind me to ",
        "remind me ",
        "remind ",
        "alert me to ",
        "alert me ",
        "alert ",
        "timer to ",
        "timer ",
    ];
    for c in cut {
        if lower.starts_with(c) {
            return s[c.len()..].trim().to_string();
        }
    }
    s.trim().to_string()
}

fn take_task_after_time(s: &str) -> String {
    // Task text may follow "to ...", "that ...", or be the whole string.
    for marker in [" to ", " that ", " — ", " - "] {
        if let Some(idx) = s.to_lowercase().find(marker) {
            return s[idx + marker.len()..].trim().to_string();
        }
    }
    s.trim().to_string()
}

fn parse_in(body: String) -> Option<ReminderDraft> {
    let lower = body.to_lowercase();
    if !lower.starts_with("in ") {
        return None;
    }
    let mut total = 0i64;
    let mut matched = false;
    for (word, mult) in [("hour", 3600i64), ("hr", 3600), ("minute", 60), ("min", 60), ("second", 1), ("sec", 1)] {
        if let Some(n) = scan_num_unit(&lower[3..], word) {
            total += n * mult;
            matched = true;
        }
    }
    if !matched || total == 0 {
        return None;
    }
    // Task = text after the time phrase (strip the leading "in ...").
    let task = take_task_after_time(&body);
    let task = strip_time_phrase(&task);
    Some(ReminderDraft {
        text: if task.is_empty() { body.clone() } else { task },
        kind: "once".into(),
        at_epoch: Some(Local::now().timestamp() + total),
        interval_min: None,
        daily_hhmm: None,
        weekdays: None,
    })
}

fn strip_time_phrase(s: &str) -> String {
    // "check the build in 20 minutes" -> "check the build"
    let lower = s.to_lowercase();
    if let Some(i) = lower.find(" in ") {
        if lower[i..].contains("min") || lower[i..].contains("hour") || lower[i..].contains("sec") {
            return s[..i].trim().to_string();
        }
    }
    s.to_string()
}

/// Regex-free number+unit scanner: first "N unit" occurrence ("20 minutes", "2h").
fn scan_num_unit(text: &str, unit: &str) -> Option<i64> {
    let mut words = text.split_whitespace().peekable();
    while let Some(w) = words.next() {
        let num: String = w.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !num.is_empty() {
            if w.len() > num.len() && w[num.len()..].starts_with(unit) {
                return num.parse().ok();
            }
            if let Some(next) = words.peek() {
                if next.starts_with(unit) {
                    return num.parse().ok();
                }
            }
        }
    }
    None
}

fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let lower = s.to_lowercase().trim().to_string();
    // 8pm, 8:30 pm, 20:15, 8
    let pm = lower.contains("pm") || lower.contains("p.m");
    let am = lower.contains("am") || lower.contains("a.m");
    let core: String = lower
        .chars()
        .map(|c| if c.is_ascii_digit() || c == ':' { c } else { ' ' })
        .collect();
    let core = core.trim().to_string();
    let (h, m) = if core.contains(':') {
        let mut it = core.split(':');
        let h: u32 = it.next()?.trim().parse().ok()?;
        let m: u32 = it.next().unwrap_or("0").trim().parse().ok()?;
        (h, m)
    } else {
        let h: u32 = core.parse().ok()?;
        (h, 0)
    };
    let mut h24 = h % 24;
    if pm && h < 12 {
        h24 += 12;
    }
    if am && h == 12 {
        h24 = 0;
    }
    Some((h24, m))
}

/// Take the time token plus a following am/pm marker if present.
/// "8:00 PM to push my code" -> ("8:00 PM", " to push my code")
fn take_time_token(s: &str) -> (String, String) {
    let mut toks = s.splitn(2, ' ');
    let first = toks.next().unwrap_or("").trim().to_string();
    let rest = toks.next().unwrap_or("").to_string();
    let lower_rest = rest.to_lowercase();
    for marker in ["pm", "p.m", "am", "a.m"] {
        if lower_rest.trim_start().starts_with(marker)
            || lower_rest.starts_with(&format!("{marker}."))
            || lower_rest.starts_with(marker)
        {
            let l = lower_rest.find(marker).unwrap_or(0);
            let after_marker = &lower_rest[l + marker.len()..];
            let trimmed = after_marker.trim_start();
            let consumes = trimmed.is_empty()
                || after_marker.starts_with(' ')
                || after_marker.starts_with(',')
                || trimmed.starts_with("to ")
                || trimmed.starts_with(',');
            if consumes {
                let rest_after = rest[l + marker.len()..].to_string();
                return (format!("{first} {marker}"), rest_after);
            }
        }
    }
    (first, rest)
}

fn parse_every_day(body: String) -> Option<ReminderDraft> {
    let lower = body.to_lowercase();
    let pats = ["every day at ", "everyday at ", "daily at ", "every night at "];
    for p in pats {
        if let Some(i) = lower.find(p) {
            let after = &body[i + p.len()..];
            let (time_tok, rest) = take_time_token(after);
            let (h, m) = parse_hhmm(&time_tok)?;
            let hhmm = format!("{h:02}:{m:02}");
            let task = take_task_after_time(&rest);
            if task.is_empty() {
                return None;
            }
            return Some(ReminderDraft {
                text: task,
                kind: "daily".into(),
                at_epoch: None,
                interval_min: None,
                daily_hhmm: Some(hhmm),
                weekdays: None,
            });
        }
    }
    None
}

fn parse_weekly(body: String) -> Option<ReminderDraft> {
    let lower = body.to_lowercase();
    if !lower.contains("every ") || !(lower.contains(" and ") || lower.contains(',')) {
        return None;
    }
    let days = [
        ("monday", "1"),
        ("tuesday", "2"),
        ("wednesday", "3"),
        ("thursday", "4"),
        ("friday", "5"),
        ("saturday", "6"),
        ("sunday", "7"),
        ("mon", "1"),
        ("tue", "2"),
        ("tues", "2"),
        ("wed", "3"),
        ("thu", "4"),
        ("thur", "4"),
        ("thurs", "4"),
        ("fri", "5"),
        ("sat", "6"),
        ("sun", "7"),
    ];
    let mut wds: Vec<String> = Vec::new();
    for (name, code) in days {
        if lower.contains(name) && !wds.contains(&code.to_string()) {
            wds.push(code.to_string());
        }
    }
    if wds.is_empty() {
        return None;
    }
    // time after " at "
    let at = lower.find(" at ")? + 4;
    let after = &body[at..];
    let (time_tok, rest) = take_time_token(after);
    let (h, m) = parse_hhmm(&time_tok)?;
    let hhmm = format!("{h:02}:{m:02}");
    let task = take_task_after_time(&rest);
    if task.is_empty() {
        return None;
    }
    Some(ReminderDraft {
        text: task,
        kind: "weekly".into(),
        at_epoch: None,
        interval_min: None,
        daily_hhmm: Some(hhmm),
        weekdays: Some(wds.join(",")),
    })
}

fn parse_every_interval(body: String) -> Option<ReminderDraft> {
    let lower = body.to_lowercase();
    if !lower.starts_with("every ") {
        return None;
    }
    let n_min = scan_num_unit(&lower[6..], "minute").map(|n| n);
    let n_hour = scan_num_unit(&lower[6..], "hour");
    let mins = match (n_min, n_hour) {
        (Some(m), _) => Some(m),
        (None, Some(h)) => Some(h * 60),
        _ => None,
    }?;
    let task = take_task_after_time(&body);
    if task.is_empty() {
        return None;
    }
    Some(ReminderDraft {
        text: task,
        kind: "interval".into(),
        at_epoch: Some(Local::now().timestamp()),
        interval_min: Some(mins),
        daily_hhmm: None,
        weekdays: None,
    })
}

fn parse_at(body: String) -> Option<ReminderDraft> {
    let lower = body.to_lowercase();
    let idx = lower.find(" at ")? + 4;
    let after = &body[idx..];
    let (time_tok, rest) = take_time_token(after);
    let (h, m) = parse_hhmm(&time_tok)?;
    let when = Local::now()
        .date_naive()
        .and_hms_opt(h, m, 0)?
        .and_local_timezone(Local::now().offset().clone())
        .single()?
        .timestamp();
    if when <= Local::now().timestamp() {
        // already passed today -> tomorrow
        let when = when + 86_400;
        let task = take_task_after_time(&rest);
        return Some(ReminderDraft {
            text: task,
            kind: "once".into(),
            at_epoch: Some(when),
            interval_min: None,
            daily_hhmm: None,
            weekdays: None,
        });
    }
    let task = take_task_after_time(&rest);
    Some(ReminderDraft {
        text: task,
        kind: "once".into(),
        at_epoch: Some(when),
        interval_min: None,
        daily_hhmm: None,
        weekdays: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_in_minutes() {
        match parse("remind me in 20 minutes to check the build") {
            Intent::Reminder(d) => {
                assert_eq!(d.kind, "once");
                assert_eq!(d.text, "check the build");
                assert!(d.at_epoch.is_some());
            }
            other => panic!("wrong intent: {other:?}"),
        }
    }

    #[test]
    fn parses_daily() {
        match parse("remind me every day at 8:00 PM to push my code") {
            Intent::Reminder(d) => {
                assert_eq!(d.kind, "daily");
                assert_eq!(d.daily_hhmm.as_deref(), Some("20:00"));
                assert_eq!(d.text, "push my code");
            }
            other => panic!("wrong intent: {other:?}"),
        }
    }

    #[test]
    fn parses_weekly() {
        match parse("remind me every monday and thursday at 6:00 PM to back up the database") {
            Intent::Reminder(d) => {
                assert_eq!(d.kind, "weekly");
                assert_eq!(d.weekdays.as_deref(), Some("1,4"));
                assert_eq!(d.daily_hhmm.as_deref(), Some("18:00"));
                assert_eq!(d.text, "back up the database");
            }
            other => panic!("wrong intent: {other:?}"),
        }
    }

    #[test]
    fn parses_note() {
        match parse("keep in mind: API port for WaveMuzix is 8085") {
            Intent::Note(n) => assert!(n.contains("8085")),
            other => panic!("wrong intent: {other:?}"),
        }
    }

    #[test]
    fn parses_interval() {
        match parse("remind me every 30 minutes to stretch") {
            Intent::Reminder(d) => {
                assert_eq!(d.kind, "interval");
                assert_eq!(d.interval_min, Some(30));
                assert_eq!(d.text, "stretch");
            }
            other => panic!("wrong intent: {other:?}"),
        }
    }
}

