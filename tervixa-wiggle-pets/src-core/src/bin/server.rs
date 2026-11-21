//! wiggle-server — the deterministic pet daemon.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use wiggle_core::{
    agent_watcher::Watcher, db::Db, permission_bridge::PermissionBridge, server, state::StateStore,
    DEFAULT_SERVER_PORT,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--uninstall") {
        let results = wiggle_core::agent_watcher::installers::uninstall_all();
        for (agent, r) in results {
            println!("{agent}: {}", if r.is_ok() { "removed" } else { "failed" });
        }
        return;
    }
    if args.iter().any(|a| a == "--install-hooks") {
        let results = wiggle_core::agent_watcher::installers::install_all();
        for (agent, r) in results {
            println!("{agent}: {}", if r.is_ok() { "installed" } else { "failed" });
        }
        return;
    }

    let port: u16 = args
        .iter()
        .position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_SERVER_PORT);

    let db = match Db::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("fatal: cannot open skills.db: {e}");
            std::process::exit(1);
        }
    };
    let store = Arc::new(StateStore::new());
    let app = Arc::new(server::App {
        db,
        store: store.clone(),
        bridge: PermissionBridge::new(store.clone()),
        stop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        chat_history: std::sync::Mutex::new(Vec::new()),
    });

    // 1-click hook install on first run (idempotent, marker-scoped).
    if wiggle_core::agent_watcher::installers::install_all()
        .iter()
        .all(|(_, r)| r.is_ok())
    {
        eprintln!("[tervixa] agent hooks installed (claude-code, codex, cursor, opencode)");
    }

    let watcher = Watcher::new(store.clone());
    watcher.spawn();
    server::spawn_scheduler(app.clone());

    eprintln!(
        "[tervixa] wiggle-server v{} listening on 127.0.0.1:{port}",
        env!("CARGO_PKG_VERSION")
    );
    if let Err(e) = server::serve(app.clone(), port) {
        eprintln!("fatal: {e}");
        std::process::exit(1);
    }
    app.stop.store(true, Ordering::Relaxed);
}
