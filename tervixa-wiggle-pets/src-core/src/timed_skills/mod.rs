//! Deterministic Timed Skills Engine — hydration, reminders, pomodoro,
//! eye-care, notes. Zero AI. Zero tokens. Fully offline.

pub mod parser;

use crate::db::Db;
use chrono::{Datelike, Local, NaiveTime};
use serde::Serialize;

fn now_secs() -> i64 {
    Local::now().timestamp()
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

// ---------------------------------------------------------------------------
// Hydration skill
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct HydrationConfig {
    pub interval_min: i64,
    pub goal_ml: i64,
    pub cup_ml: i64,
    pub active_start: String,
    pub active_end: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HydrationStatus {
    pub day: String,
    pub drunk_ml: i64,
    pub goal_ml: i64,
    pub cups: i64,
    pub streak: i64,
    pub due: bool,
    pub config: HydrationConfig,
}

pub fn hydration_cfg(db: &Db) -> HydrationConfig {
    db.with_conn(|c| {
        c.query_row(
            "SELECT interval_min, goal_ml, cup_ml, active_start, active_end, enabled
             FROM hydration_cfg WHERE id = 1",
            [],
            |r| {
                Ok(HydrationConfig {
                    interval_min: r.get(0)?,
                    goal_ml: r.get(1)?,
                    cup_ml: r.get(2)?,
                    active_start: r.get(3)?,
                    active_end: r.get(4)?,
                    enabled: r.get::<_, i64>(5)? != 0,
                })
            },
        )
        .map_err(rusqlite::Error::from)
    })
    .unwrap_or(HydrationConfig {
        interval_min: 45,
        goal_ml: 2500,
        cup_ml: 250,
        active_start: "08:00".into(),
        active_end: "22:00".into(),
        enabled: true,
    })
}

pub fn hydration_set_cfg(db: &Db, cfg: &HydrationConfig) -> Result<(), String> {
    db.with_conn(|c| {
        c.execute(
            "UPDATE hydration_cfg SET interval_min=?2, goal_ml=?3, cup_ml=?4,
             active_start=?5, active_end=?6, enabled=?7 WHERE id = 1",
            rusqlite::params![
                1,
                cfg.interval_min,
                cfg.goal_ml,
                cfg.cup_ml,
                cfg.active_start,
                cfg.active_end,
                cfg.enabled as i64
            ],
        )?;
        Ok(())
    })
}

pub fn hydration_log(db: &Db, ml: i64) -> Result<(), String> {
    db.with_conn(|c| {
        c.execute(
            "INSERT INTO hydration_log (day, ml, ts) VALUES (?1, ?2, ?3)",
            rusqlite::params![today(), ml, now_secs()],
        )?;
        Ok(())
    })
}

fn in_active_window(cfg: &HydrationConfig) -> bool {
    let t = Local::now().time();
    let start = NaiveTime::parse_from_str(&cfg.active_start, "%H:%M").unwrap_or(NaiveTime::MIN);
    let end = NaiveTime::parse_from_str(&cfg.active_end, "%H:%M").unwrap_or(NaiveTime::MIN);
    start <= t && t <= end
}

pub fn hydration_status(db: &Db) -> HydrationStatus {
    let cfg = hydration_cfg(db);
    let day = today();
    let (drunk, streak) = db
        .with_conn(|c| {
            let drunk: i64 = c
                .query_row(
                    "SELECT COALESCE(SUM(ml), 0) FROM hydration_log WHERE day = ?1",
                    rusqlite::params![day],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            // Streak: consecutive days (ending today/yesterday) meeting the goal.
            let mut streak = 0i64;
            for back in 0..400i64 {
                let d = (Local::now() - chrono::Duration::days(back))
                    .format("%Y-%m-%d")
                    .to_string();
                let sum: i64 = c
                    .query_row(
                        "SELECT COALESCE(SUM(ml), 0) FROM hydration_log WHERE day = ?1",
                        rusqlite::params![d],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if sum >= cfg.goal_ml {
                    if back == 0 || streak + 1 == back + 1 {
                        streak += 1;
                    } else if back as i64 == streak {
                        streak += 1;
                    }
                } else if back == 0 {
                    // today not finished yet — do not break the streak
                } else {
                    break;
                }
            }
            Ok((drunk, streak))
        })
        .unwrap_or((0, 0));
    let due = cfg.enabled && in_active_window(&cfg);
    HydrationStatus {
        day,
        drunk_ml: drunk,
        goal_ml: cfg.goal_ml,
        cups: drunk / cfg.cup_ml.max(1),
        streak,
        due,
        config: cfg,
    }
}

// ---------------------------------------------------------------------------
// Reminders
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Reminder {
    pub id: i64,
    pub text: String,
    pub kind: String, // once | interval | daily | weekly | note
    pub at_epoch: Option<i64>,
    pub interval_min: Option<i64>,
    pub daily_hhmm: Option<String>,
    pub weekdays: Option<String>,
    pub note: Option<String>,
    pub enabled: bool,
}

pub fn add_reminder(db: &Db, r: &Reminder) -> Result<i64, String> {
    db.with_conn(|c| {
        c.execute(
            "INSERT INTO reminders (text, kind, at_epoch, interval_min, daily_hhmm, weekdays, note, enabled, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params![
                r.text,
                r.kind,
                r.at_epoch,
                r.interval_min,
                r.daily_hhmm,
                r.weekdays,
                r.note,
                r.enabled as i64,
                now_secs()
            ],
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn list_reminders(db: &Db) -> Vec<Reminder> {
    db.with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id, text, kind, at_epoch, interval_min, daily_hhmm, weekdays, note, enabled
             FROM reminders WHERE enabled = 1 ORDER BY id",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Reminder {
                    id: r.get(0)?,
                    text: r.get(1)?,
                    kind: r.get(2)?,
                    at_epoch: r.get(3)?,
                    interval_min: r.get(4)?,
                    daily_hhmm: r.get(5)?,
                    weekdays: r.get(6)?,
                    note: r.get(7)?,
                    enabled: r.get::<_, i64>(8)? != 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    })
    .unwrap_or_default()
}

pub fn disable_reminder(db: &Db, id: i64) -> Result<(), String> {
    db.with_conn(|c| {
        c.execute(
            "UPDATE reminders SET enabled = 0 WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    })
}

/// Reminders whose time has come. `once` reminders are disabled after firing.
pub fn due_reminders(db: &Db) -> Vec<Reminder> {
    let all = list_reminders(db);
    let now = now_secs();
    let today_wd = Local::now().weekday().number_from_monday(); // 1..=7
    let hhmm = Local::now().format("%H:%M").to_string();
    let mut fired = Vec::new();
    for r in all {
        let due = match r.kind.as_str() {
            "once" => r.at_epoch.map(|t| t <= now).unwrap_or(false),
            "interval" => {
                let last_key = format!("reminder.last.{}", r.id);
                let last: i64 = db.get_setting(&last_key).and_then(|v| v.parse().ok()).unwrap_or(0);
                let iv = r.interval_min.unwrap_or(60) as i64 * 60;
                r.at_epoch.map(|start| now >= last + iv && now >= start).unwrap_or(false)
            }
            "daily" => r.daily_hhmm.as_deref() == Some(hhmm.as_str()),
            "weekly" => r
                .weekdays
                .as_deref()
                .map(|w| w.split(',').any(|d| d.trim().parse::<u32>().ok() == Some(today_wd as u32)))
                .unwrap_or(false)
                && r.daily_hhmm.as_deref() == Some(hhmm.as_str()),
            "note" => false,
            _ => false,
        };
        if due {
            fired.push(r.clone());
            if r.kind == "once" {
                let _ = disable_reminder(db, r.id);
            } else if r.kind == "daily" || r.kind == "weekly" {
                // mark fired this minute so we don't refire within the same minute
                let key = format!("reminder.fired.{}.{}", r.id, now / 60);
                let _ = db.set_setting(&key, "1");
            } else if r.kind == "interval" {
                let key = format!("reminder.last.{}", r.id);
                let _ = db.set_setting(&key, &now.to_string());
            }
        }
    }
    fired
}

// ---------------------------------------------------------------------------
// Pomodoro / focus + eye care
// ---------------------------------------------------------------------------

pub const POMODORO_WORK_MIN: i64 = 25;
pub const POMODORO_BREAK_MIN: i64 = 5;
pub const EYE_CARE_INTERVAL_MIN: i64 = 20;

pub fn set_pomodoro(db: &Db, active: bool) -> Result<(), String> {
    let val = if active { now_secs().to_string() } else { String::new() };
    db.set_setting("pomodoro.started", &val)
}

pub fn pomodoro_state(db: &Db) -> serde_json::Value {
    let started = db
        .get_setting("pomodoro.started")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    if started == 0 {
        return serde_json::json!({ "active": false });
    }
    let elapsed = now_secs() - started;
    let cycle = POMODORO_WORK_MIN + POMODORO_BREAK_MIN;
    let pos = elapsed % cycle;
    let phase = if pos < POMODORO_WORK_MIN * 60 { "work" } else { "break" };
    let remaining = if phase == "work" {
        POMODORO_WORK_MIN * 60 - pos
    } else {
        cycle * 60 - pos
    };
    let rounds = elapsed / cycle;
    serde_json::json!({
        "active": true,
        "phase": phase,
        "remaining_secs": remaining,
        "rounds": rounds
    })
}
