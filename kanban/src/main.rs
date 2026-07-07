mod app;
mod db;
mod herdr;
mod theme;
mod ui;

use std::io::{self, stdout};
use std::process::Command;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use db::Database;

const SCHEMA_SQL: &str = include_str!("../../scripts/schema.sql");

fn init_db() -> Database {
    let db_path = std::env::var("HERDR_PLUGIN_STATE_DIR")
        .map(|d| format!("{}/kanban.db", d))
        .unwrap_or_else(|_| "tickets.db".into());

    let existe = std::path::Path::new(&db_path).exists();

    let db = match Database::open(&db_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Impossible d'ouvrir {} : {}", db_path, e);
            std::process::exit(1);
        }
    };

    // Migration légère pour les anciennes bases
    let _ = db.execute_batch(
        "ALTER TABLE tickets ADD COLUMN session_started_le TEXT DEFAULT NULL;",
    );
    let _ = db.execute_batch(
        "ALTER TABLE tickets ADD COLUMN initial_prompt_sent_le TEXT DEFAULT NULL;",
    );

    if !existe {
        if let Err(e) = db.execute_batch(SCHEMA_SQL) {
            eprintln!("Erreur schéma : {}", e);
            std::process::exit(1);
        }
    }

    db
}

fn main() -> io::Result<()> {
    // Désactiver XON/XOFF flow control pour que Ctrl+S et Ctrl+Q atteignent l'app
    let _ = std::process::Command::new("stty").arg("-ixon").output();

    let db = init_db();
    let mut app = App::new(db);

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Erreur : {}", e);
    }

    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                if key.kind == event::KeyEventKind::Repeat
                    && matches!(key.code, event::KeyCode::Char('o'))
                {
                    continue;
                }
                app.handle_key(key);
                if let Some(agent) = app.take_external_agent() {
                    attach_herdr_fullscreen(terminal, &agent)?;
                }
            }
            Event::Mouse(mouse) => {
                app.handle_mouse(mouse);
            }
            Event::Resize(_, _) => {}
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn attach_herdr_fullscreen(
    _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    _agent: &str,
) -> io::Result<()> {
    let kaku_bin = "/Applications/Kaku.app/Contents/MacOS/kaku";

    let herdr_window_exists = Command::new(kaku_bin)
        .args(["cli", "list"])
        .output()
        .map(|o| {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .any(|line| line.contains("PKdev-herdr"))
        })
        .unwrap_or(false);

    if herdr_window_exists {
        let _ = Command::new("osascript")
            .args(["-e", "tell application \"Kaku\" to activate"])
            .output();
        return Ok(());
    }

    // Kaku n'expose pas de pane flottant via sa CLI. `cli spawn --new-window`
    // crée une fenêtre, mais elle peut rester en arrière-plan ; `open -na`
    // demande explicitement à macOS d'ouvrir/présenter une fenêtre applicative.
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let result = Command::new("open")
        .args([
            "-na",
            "/Applications/Kaku.app",
            "--args",
            "start",
            "--cwd",
            cwd.to_string_lossy().as_ref(),
            "--workspace",
            "PKdev-herdr",
            "--",
            "herdr",
        ])
        .output();

    if let Ok(o) = &result {
        if !o.status.success() {
            eprintln!("open kaku stderr: {}", String::from_utf8_lossy(&o.stderr));
        }
    }

    Ok(())
}
