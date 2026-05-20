use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::{App, Phase};

pub fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
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

pub fn draw(f: &mut Frame, app: &App) {
    let now = std::time::Instant::now();
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
