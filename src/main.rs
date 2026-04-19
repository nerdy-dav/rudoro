use std::io::{self, stdout, Write};
use std::panic;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};

#[derive(Parser, Debug)]
#[command(name = "rudoro", about = "Terminal Pomodoro timer")]
struct Args {
    /// Work phase length in minutes
    #[arg(long = "work-minutes", short = 'w', default_value_t = 25)]
    work_minutes: u32,

    /// Rest phase length in minutes
    #[arg(long = "rest-minutes", short = 'r', default_value_t = 5)]
    rest_minutes: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Work,
    Rest,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Phase::Work => "Work",
            Phase::Rest => "Rest",
        }
    }
}

struct App {
    phase: Phase,
    work: Duration,
    rest: Duration,
    paused: bool,
    /// Wall-clock instant when the current phase ends (only while running).
    ends_at: Option<Instant>,
    /// Remaining time in the current phase while paused.
    remaining: Duration,
}

impl App {
    fn new(work: Duration, rest: Duration) -> Self {
        let now = Instant::now();
        Self {
            phase: Phase::Work,
            work,
            rest,
            paused: false,
            ends_at: Some(now + work),
            remaining: work,
        }
    }

    fn phase_duration(&self) -> Duration {
        match self.phase {
            Phase::Work => self.work,
            Phase::Rest => self.rest,
        }
    }

    fn remaining_at(&self, now: Instant) -> Duration {
        if self.paused {
            self.remaining
        } else if let Some(end) = self.ends_at {
            end.saturating_duration_since(now)
        } else {
            Duration::ZERO
        }
    }

    fn tick(&mut self, now: Instant) {
        if self.paused {
            return;
        }
        let Some(end) = self.ends_at else {
            return;
        };
        if now < end {
            return;
        }
        self.phase = match self.phase {
            Phase::Work => Phase::Rest,
            Phase::Rest => Phase::Work,
        };
        let dur = self.phase_duration();
        self.remaining = dur;
        self.ends_at = Some(now + dur);
    }

    fn toggle_pause(&mut self, now: Instant) {
        if self.paused {
            self.paused = false;
            self.ends_at = Some(now + self.remaining);
        } else {
            self.paused = true;
            if let Some(end) = self.ends_at {
                self.remaining = end.saturating_duration_since(now);
            }
            self.ends_at = None;
        }
    }
}

fn format_countdown(d: Duration) -> String {
    let total = d.as_secs();
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn draw(f: &mut Frame, app: &App) {
    let now = Instant::now();
    let remaining = app.remaining_at(now);
    let phase_dur = app.phase_duration();
    let ratio = if phase_dur.is_zero() {
        0.0
    } else {
        let elapsed = phase_dur.saturating_sub(remaining).as_secs_f64();
        let total = phase_dur.as_secs_f64();
        (elapsed / total).clamp(0.0, 1.0)
    };

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title = format!(
        " rudoro — {} {}",
        app.phase.label(),
        if app.paused { "(paused)" } else { "" }
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).bold());

    let inner = block.inner(main[0]);
    f.render_widget(block, main[0]);

    let countdown = Paragraph::new(format_countdown(remaining))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(if app.phase == Phase::Work {
                    Color::Green
                } else {
                    Color::Yellow
                })
                .add_modifier(Modifier::BOLD),
        );
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    f.render_widget(countdown, vchunks[1]);

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(if app.phase == Phase::Work {
                    Color::Green
                } else {
                    Color::Yellow
                })
                .bg(Color::DarkGray),
        )
        .ratio(ratio);
    f.render_widget(gauge, main[1]);

    let hints = Paragraph::new(" Space pause/resume   q quit ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hints, main[2]);
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let tick = Duration::from_millis(100);
    loop {
        let now = Instant::now();
        app.tick(now);

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
