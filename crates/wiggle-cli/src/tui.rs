//! Minimal monochrome Ratatui TUI in the OpenCode / Claude Code style:
//! flat colors, 1px hairline borders, zero decoration.

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use wiggle_core::AgentEvent;

pub async fn run(engine: wiggle_core::Engine, model: Option<String>) -> i32 {
    let result = ratatui::crossterm::terminal::enable_raw_mode();
    if let Err(e) = result {
        eprintln!("terminal init failed: {e}");
        return 1;
    }
    let mut stdout = std::io::stdout();
    let _ = ratatui::crossterm::execute!(
        stdout,
        ratatui::crossterm::terminal::EnterAlternateScreen,
        ratatui::crossterm::event::EnableMouseCapture
    );

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let state = TuiState::new(engine.clone(), model);

    let exit_code = run_loop(&mut terminal, state, engine).await.unwrap_or(1);

    let _ = ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::event::DisableMouseCapture,
        ratatui::crossterm::terminal::LeaveAlternateScreen
    );
    let _ = ratatui::crossterm::terminal::disable_raw_mode();
    exit_code
}

struct TuiState {
    engine: wiggle_core::Engine,
    input: String,
    transcript: Vec<(String, String)>, // (role, text)
    events: Vec<String>,
    model: Option<String>,
    busy: bool,
    /// Vertical chat scroll offset (kept for future PgUp/PgDn support).
    #[allow(dead_code)]
    scroll: u16,
}

impl TuiState {
    fn new(engine: wiggle_core::Engine, model: Option<String>) -> TuiState {
        TuiState {
            engine,
            input: String::new(),
            transcript: Vec::new(),
            events: vec![format!(
                "Tervixa Wiggle v{} — type a task, /undo /redo /skills /quit",
                wiggle_core::identity::VERSION
            )],
            model,
            busy: false,
            scroll: 0,
        }
    }
}

async fn run_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    mut state: TuiState,
    engine: wiggle_core::Engine,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut event_rx = engine.subscribe();

    loop {
        terminal.draw(|frame| render(frame, &state))?;

        if !state.busy && event::poll(std::time::Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(ratatui::crossterm::event::KeyModifiers::CONTROL) => {
                            return Ok(0);
                        }
                        KeyCode::Enter => {
                            let prompt = state.input.trim().to_string();
                            state.input.clear();
                            if prompt.is_empty() {
                                continue;
                            }
                            match prompt.as_str() {
                                "/quit" | "/exit" => return Ok(0),
                                "/undo" => {
                                    let mut undo = engine.inner.undo.lock().expect("undo");
                                    match undo.undo(engine.workspace()) {
                                        Ok(Some(label)) => state.events.push(format!("undone: {label}")),
                                        Ok(None) => state.events.push("nothing to undo".into()),
                                        Err(e) => state.events.push(format!("undo failed: {e}")),
                                    }
                                }
                                "/redo" => {
                                    let mut undo = engine.inner.undo.lock().expect("undo");
                                    match undo.redo(engine.workspace()) {
                                        Ok(Some(label)) => state.events.push(format!("redone: {label}")),
                                        Ok(None) => state.events.push("nothing to redo".into()),
                                        Err(e) => state.events.push(format!("redo failed: {e}")),
                                    }
                                }
                                "/skills" => {
                                    for skill in engine.inner.skills.all() {
                                        state.events.push(format!("skill: {} ({})", skill.name, skill.origin));
                                    }
                                }
                                _ => {
                                    state.transcript.push(("user".into(), prompt.clone()));
                                    state.busy = true;
                                    let engine = engine.clone();
                                    let model = state.model.clone();
                                    let prompt = prompt.clone();
                                    tokio::spawn(async move {
                                        let _ = engine.run_prompt("tui", &prompt, None, model.as_deref()).await;
                                    });
                                    // Drain engine events while busy.
                                    while let Ok(event) = event_rx.try_recv() {
                                        match event {
                                            AgentEvent::TextDelta { text } => {
                                                append_stream(&mut state.transcript, text)
                                            }
                                            other => state.events.push(format!("{other:?}")),
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            state.input.pop();
                        }
                        KeyCode::Char(c) => state.input.push(c),
                        _ => {}
                    }
                }
            }
        } else if state.busy {
            while let Ok(event) = event_rx.try_recv() {
                match event {
                    AgentEvent::TextDelta { text } => append_stream(&mut state.transcript, text),
                    AgentEvent::AgentFinished { summary } => {
                        state.transcript.push(("assistant".into(), summary));
                        state.busy = false;
                    }
                    AgentEvent::Error { message } => {
                        state.events.push(format!("error: {message}"));
                        state.busy = false;
                    }
                    AgentEvent::TurnStarted { agent, .. } => {
                        state.events.push(format!("[{agent}] started"));
                    }
                    _ => {}
                }
            }
            // Safety: if no AgentFinished arrived (engine errored inside
            // the spawned task), unstick after the transcript stalls.
            if !state.busy {
                continue;
            }
        }
    }
}

fn append_stream(transcript: &mut Vec<(String, String)>, text: String) {
    match transcript.last_mut() {
        Some((role, buffer)) if role == "stream" => buffer.push_str(&text),
        _ => transcript.push(("stream".into(), text)),
    }
}

fn render(frame: &mut Frame, state: &TuiState) {
    let hairline = Style::new().fg(Color::Rgb(0x22, 0x22, 0x22));
    let primary = Style::new().fg(Color::Rgb(0xED, 0xED, 0xED));
    let secondary = Style::new().fg(Color::Rgb(0x99, 0x99, 0x99));
    let muted = Style::new().fg(Color::Rgb(0x66, 0x66, 0x66));
    let canvas = Color::Rgb(0x0A, 0x0A, 0x0A);
    let panel = Color::Rgb(0x11, 0x11, 0x11);

    let [sessions_area, chat_area, input_area] = Layout::vertical([
        Constraint::Length(6),
        Constraint::Min(4),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    // Sessions / status panel (flat, left-aligned).
    let session_lines = vec![
        Line::from(Span::styled("Tervixa Wiggle", primary.bold())),
        Line::from(Span::styled(format!("workspace {}", state.engine.workspace().display()), secondary)),
        Line::from(Span::styled(
            format!("skills {}", state.engine.inner.skills.count()),
            secondary,
        )),
        Line::from(Span::styled(
            if state.busy { "status: working".to_string() } else { "status: idle".to_string() },
            muted,
        )),
    ];
    frame.render_widget(
        Paragraph::new(session_lines)
            .style(Style::new().bg(panel))
            .block(Block::new().borders(Borders::BOTTOM).border_style(hairline)),
        sessions_area,
    );

    // Transcript.
    let items: Vec<ListItem> = state
        .transcript
        .iter()
        .map(|(role, text)| {
            let (label, style) = match role.as_str() {
                "user" => ("you ", primary),
                "stream" | "assistant" => ("wiggle ", secondary),
                other => (other, muted),
            };
            ListItem::new(Line::from(vec![
                Span::styled(label, muted),
                Span::styled(text.clone(), style),
            ]))
        })
        .collect();
    frame.render_widget(
        List::new(items)
            .style(Style::new().bg(canvas))
            .block(Block::new().borders(Borders::BOTTOM).border_style(hairline)),
        chat_area,
    );

    // Input box.
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("> ", secondary),
            Span::styled(state.input.clone(), primary),
            Span::styled("_", muted),
        ]))
        .style(Style::new().bg(panel))
        .block(Block::new().borders(Borders::ALL).border_style(hairline)),
        input_area,
    );
}
