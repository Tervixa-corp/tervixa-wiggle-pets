//! TERVIXA WIGGLE PETS — deterministic core engine.
//!
//! Engine A (this crate) is 100% deterministic, zero-AI, zero-token:
//! timed skills, reminders, agent watching, permission bridging and the
//! pet registry. AI is only ever invoked by `ai_router` when the user
//! opens a chat with the pet.

pub mod ai_router;
pub mod agent_watcher;
pub mod db;
pub mod pet_registry;
pub mod permission_bridge;
pub mod protocol;
pub mod server;
pub mod state;
pub mod timed_skills;

/// Default port range for the local hook/IPC server (23340..23347).
/// Hooks discover the live port by scanning this small range.
pub const DEFAULT_SERVER_PORT: u16 = 23340;
pub const SERVER_PORT_COUNT: u32 = 8;

pub fn port_candidates() -> Vec<u16> {
    (0..SERVER_PORT_COUNT)
        .map(|i| DEFAULT_SERVER_PORT + i as u16)
        .collect()
}

pub const APP_DIR_NAME: &str = ".tervixa-wiggle-pets";

pub fn app_dir() -> std::path::PathBuf {
    dirs_home().join(APP_DIR_NAME)
}

pub fn dirs_home() -> std::path::PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}
