//! SQLite persistence for all deterministic skills (~/.tervixa-wiggle-pets/skills.db).

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

pub fn db_path() -> PathBuf {
    crate::app_dir().join("skills.db")
}

impl Db {
    pub fn open() -> Result<Self, String> {
        let dir = crate::app_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let conn = Connection::open(db_path()).map_err(|e| e.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>) -> Result<T, String> {
        let conn = self.conn.lock().map_err(|_| "db lock poisoned")?;
        f(&conn).map_err(|e| e.to_string())
    }

    fn migrate(&self) -> Result<(), String> {
        self.with_conn(|c| {
            c.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS reminders (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    text        TEXT NOT NULL,
                    kind        TEXT NOT NULL,           -- once | interval | daily | weekly | note
                    at_epoch    INTEGER,                 -- for `once` (secs)
                    interval_min INTEGER,                -- for `interval`
                    daily_hhmm  TEXT,                    -- for `daily`
                    weekdays    TEXT,                    -- for `weekly` ("1,4" = Mon,Thu)
                    note        TEXT,                    -- persistent_note payload
                    enabled     INTEGER NOT NULL DEFAULT 1,
                    created_at  INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS hydration_log (
                    day         TEXT NOT NULL,
                    ml          INTEGER NOT NULL,
                    ts          INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS hydration_cfg (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    interval_min INTEGER NOT NULL DEFAULT 45,
                    goal_ml      INTEGER NOT NULL DEFAULT 2500,
                    cup_ml       INTEGER NOT NULL DEFAULT 250,
                    active_start TEXT NOT NULL DEFAULT '08:00',
                    active_end   TEXT NOT NULL DEFAULT '22:00',
                    enabled      INTEGER NOT NULL DEFAULT 1
                );
                INSERT OR IGNORE INTO hydration_cfg (id) VALUES (1);
                CREATE TABLE IF NOT EXISTS sessions (
                    agent       TEXT NOT NULL,
                    session_id  TEXT NOT NULL,
                    state       TEXT NOT NULL,
                    detail      TEXT,
                    workspace   TEXT,
                    updated_at  INTEGER NOT NULL,
                    PRIMARY KEY (agent, session_id)
                );
                CREATE TABLE IF NOT EXISTS notes (
                    id    INTEGER PRIMARY KEY AUTOINCREMENT,
                    text  TEXT NOT NULL,
                    ts    INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS settings (
                    key   TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                "#,
            )
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.with_conn(|c| {
            let val = c
                .query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    rusqlite::params![key],
                    |r| r.get::<_, String>(0),
                )
                .ok();
            Ok(val)
        })
        .ok()
        .flatten()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        self.with_conn(|c| {
            c.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![key, value],
            )?;
            Ok(())
        })
    }
}
