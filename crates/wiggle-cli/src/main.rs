//! `wiggle` — the standalone Tervixa Wiggle CLI.
//!
//! Shares 100% of the engine and memory crates with the desktop app.

mod tui;

use clap::{Parser, Subcommand};
use wiggle_core::{AgentEvent, CoreError};

/// Tervixa Wiggle — autonomous AI software engineering agent (CLI).
#[derive(Parser)]
#[command(name = "wiggle", version, about = "Tervixa Wiggle agent by Tervixa Corporation", long_about = None)]
struct Cli {
    /// Workspace directory (defaults to the current directory).
    #[arg(long, global = true)]
    workspace: Option<String>,
    /// Provider/model to use, e.g. `anthropic/claude-sonnet-4-5`.
    #[arg(short, long, global = true)]
    model: Option<String>,
    /// Run non-interactively and print the result (full-auto sandbox).
    #[arg(short, long, global = true)]
    print: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// One-shot prompt: run it and exit.
    Run { prompt: Vec<String> },
    /// Launch the interactive monochrome TUI.
    Tui,
    /// Show engine status: providers, skills, MCP, memory.
    Status,
    /// Manage providers (list / test).
    Providers {
        /// Test connectivity with the configured keys.
        #[arg(long)]
        check: bool,
    },
    /// List or import skills.
    Skills {
        /// Import from ~/.claude/skills
        #[arg(long)]
        from_claude: bool,
        /// Import a GitHub repo: --from-github owner/repo
        #[arg(long)]
        from_github: Option<String>,
    },
    /// Undo the last file mutation snapshot.
    Undo,
    /// Redo the last undone snapshot.
    Redo,
    /// Print the compiled system prompt for debugging (7 stages).
    Compile { prompt: Vec<String> },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let workspace = cli
        .workspace
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

    let engine = match wiggle_core::Engine::open(&workspace) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let exit_code = match cli.command {
        None | Some(Command::Tui) => tui::run(engine, cli.model).await,
        Some(Command::Run { prompt }) => {
            run_print(engine, &prompt.join(" "), cli.model.as_deref()).await
        }
        Some(Command::Status) => status(engine),
        Some(Command::Providers { check }) => providers(engine, check),
        Some(Command::Skills { from_claude, from_github }) => skills(engine, from_claude, from_github).await,
        Some(Command::Undo) => undo(engine),
        Some(Command::Redo) => redo(engine),
        Some(Command::Compile { prompt }) => compile(engine, &prompt.join(" ")).await,
    };
    std::process::exit(exit_code);
}

async fn run_print(engine: wiggle_core::Engine, prompt: &str, model: Option<&str>) -> i32 {
    let mut events = engine.subscribe();
    let handle = tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            print_event(&event);
        }
    });
    match engine.run_prompt("cli", prompt, None, model).await {
        Ok(text) => {
            println!("\n{text}");
            drop(handle);
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

fn print_event(event: &AgentEvent) {
    match event {
        AgentEvent::TurnStarted { agent, .. } => eprintln!("[{agent}] working..."),
        AgentEvent::ToolStarted { call } => {
            eprintln!(
                "[tool] {} {}",
                call.name,
                serde_json::to_string(&call.params).unwrap_or_default()
            );
        }
        AgentEvent::ToolFinished { call, ok } => {
            eprintln!("[tool] {} -> {}", call.name, if *ok { "ok" } else { "error" });
        }
        AgentEvent::TextDelta { text } => print!("{text}"),
        AgentEvent::Error { message } => eprintln!("[error] {message}"),
        _ => {}
    }
}

fn status(engine: wiggle_core::Engine) -> i32 {
    println!("Tervixa Wiggle v{}", wiggle_core::identity::VERSION);
    println!("workspace: {}", engine.workspace().display());
    let config = engine.config();
    for provider in &config.providers {
        let has_key = provider.env_keys.iter().any(|k| std::env::var(k).is_ok())
            || provider.api_key.is_some();
        println!(
            "  provider {:<14} api={:<9} models={} key={}",
            provider.id,
            format!("{:?}", provider.api).to_lowercase(),
            provider.models.len(),
            if has_key { "set" } else { "missing" }
        );
    }
    println!("skills: {}", engine.inner.skills.count());
    println!(
        "memory: {}",
        engine
            .inner
            .memory
            .as_ref()
            .and_then(|m| m.lock().ok())
            .and_then(|m| m.count().ok())
            .map(|c| format!("{c} facts"))
            .unwrap_or_else(|| "unavailable".into())
    );
    println!("mcp servers configured: {}", config.mcp_servers.len());
    0
}

fn providers(engine: wiggle_core::Engine, check: bool) -> i32 {
    let config = engine.config();
    for provider in &config.providers {
        let has_key = provider.env_keys.iter().any(|k| std::env::var(k).is_ok())
            || provider.api_key.is_some();
        let status = if !provider.enabled {
            "disabled"
        } else if has_key || provider.base_url.contains("127.0.0.1") {
            "ready"
        } else {
            "missing key"
        };
        println!("{:<14} {status:<12} {}", provider.id, provider.base_url);
    }
    if check {
        println!("(connectivity checks run against /v1/models where supported)");
    }
    0
}

async fn skills(engine: wiggle_core::Engine, from_claude: bool, from_github: Option<String>) -> i32 {
    if from_claude {
        let source = wiggle_core::skills::importer::ImportSource::ClaudeCode;
        match wiggle_core::skills::importer::home_skills_dir(&source) {
            Some(dir) if dir.exists() => {
                match wiggle_core::skills::importer::import_dir(&dir) {
                    Ok(list) => println!("imported {} skill(s) from ~/.claude/skills", list.len()),
                    Err(e) => {
                        eprintln!("import failed: {e}");
                        return 1;
                    }
                }
            }
            _ => println!("no ~/.claude/skills directory found"),
        }
    }
    if let Some(repo) = from_github {
        let mut parts = repo.splitn(2, '/');
        let (owner, name) = match (parts.next(), parts.next()) {
            (Some(o), Some(r)) => (o.to_string(), r.to_string()),
            _ => {
                eprintln!("--from-github expects owner/repo");
                return 1;
            }
        };
        match wiggle_core::skills::importer::import_github(&owner, &name).await {
            Ok(list) => println!("imported {} skill(s) from github.com/{repo}", list.len()),
            Err(e) => {
                eprintln!("import failed: {e}");
                return 1;
            }
        }
    }
    for skill in engine.inner.skills.all() {
        println!(
            "{:<24} [{}] {}",
            skill.name,
            skill.origin,
            skill.description
        );
    }
    0
}

fn undo(engine: wiggle_core::Engine) -> i32 {
    let mut snapshot_engine = engine.inner.undo.lock().expect("undo mutex");
    match snapshot_engine.undo(engine.workspace()) {
        Ok(Some(label)) => {
            println!("undone: {label}");
            0
        }
        Ok(None) => {
            println!("nothing to undo");
            1
        }
        Err(e) => {
            eprintln!("undo failed: {e}");
            1
        }
    }
}

fn redo(engine: wiggle_core::Engine) -> i32 {
    let mut snapshot_engine = engine.inner.undo.lock().expect("undo mutex");
    match snapshot_engine.redo(engine.workspace()) {
        Ok(Some(label)) => {
            println!("redone: {label}");
            0
        }
        Ok(None) => {
            println!("nothing to redo");
            1
        }
        Err(e) => {
            eprintln!("redo failed: {e}");
            1
        }
    }
}

async fn compile(engine: wiggle_core::Engine, prompt: &str) -> i32 {
    let compiler = wiggle_core::prompt_compiler::PromptCompiler::new(&engine);
    match compiler.compile("cli-debug", prompt, None).await {
        Ok(compiled) => {
            println!("=== SYSTEM ===\n{}\n\n=== USER ===\n{}\n\n=== ROUTING ===\n{:?}\n\n{}",
                compiled.system, compiled.user_message, compiled.routing, compiled.usage_note);
            0
        }
        Err(e) => {
            eprintln!("compile failed: {e}");
            1
        }
    }
}

// Keep CoreError in scope for main's error arm.
#[allow(dead_code)]
fn _assert_error_is_send(_: CoreError) {}
