use std::io::{self, stdout, Write};
use std::panic;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod ui;

use app::App;
use ui::{draw, restore_terminal};

/// CLI arguments parsed from the command line.
#[derive(Parser, Debug)]
#[command(name = "rudoro", about = "Terminal Pomodoro timer")]
struct Args {
    /// Work phase length in minutes.
    #[arg(long = "work-minutes", short = 'w', default_value_t = 25)]
    work_minutes: u32,

    /// Rest phase length in minutes.
    #[arg(long = "rest-minutes", short = 'r', default_value_t = 5)]
    rest_minutes: u32,
}

/// Main event loop: advances the app at ~10 Hz, redraws the UI, and handles
/// keyboard input (`q`/`Esc` to quit, `Space` to pause/resume).
fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let tick = Duration::from_millis(100);
    loop {
        let now = Instant::now();
        if app.tick(now) {
            write!(stdout(), "\x07\x07\x07")?;
            stdout().flush()?;
        }

        terminal.draw(|f| draw(f, app))?;

        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char(' ') => app.toggle_pause(Instant::now()),
                    _ => {}
                }
            }
        }
    }
}

/// Parse arguments, initialise the terminal and app, install a panic hook
/// that restores the terminal, then hand control to the event loop.
fn main() -> io::Result<()> {
    let args = Args::parse();
    let work = Duration::from_secs(u64::from(args.work_minutes) * 60);
    let rest = Duration::from_secs(u64::from(args.rest_minutes) * 60);

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original_hook(info);
    }));

    stdout().flush()?;
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = App::new(work, rest);
    let result = run(&mut terminal, &mut app);

    restore_terminal()?;

    result
}
