use std::time::{Duration, Instant};

/// A phase of the Pomodoro cycle.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Work,
    Rest,
}

impl Phase {
    /// Human-readable label for the phase.
    pub fn label(self) -> &'static str {
        match self {
            Phase::Work => "Work",
            Phase::Rest => "Rest",
        }
    }
}

/// State of the Pomodoro timer.
pub struct App {
    /// Current phase of the cycle.
    pub phase: Phase,
    /// Duration of the work phase.
    pub work: Duration,
    /// Duration of the rest phase.
    pub rest: Duration,
    /// Whether the timer is paused.
    pub paused: bool,
    /// Wall-clock instant when the current phase ends (unset while paused).
    pub ends_at: Option<Instant>,
    /// Remaining time in the current phase while paused.
    pub remaining: Duration,
}

impl App {
    /// Create a new timer starting a work phase of `work` duration followed by
    /// rest phases of `rest` duration.
    pub fn new(work: Duration, rest: Duration) -> Self {
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

    /// Duration of the current phase.
    pub fn phase_duration(&self) -> Duration {
        match self.phase {
            Phase::Work => self.work,
            Phase::Rest => self.rest,
        }
    }

    /// Time remaining in the current phase at instant `now`.
    pub fn remaining_at(&self, now: Instant) -> Duration {
        if self.paused {
            self.remaining
        } else if let Some(end) = self.ends_at {
            end.saturating_duration_since(now)
        } else {
            Duration::ZERO
        }
    }

    /// Advance the timer to `now`, transitioning phases if the current one has
    /// elapsed. Returns `true` when a phase transition occurred.
    pub fn tick(&mut self, now: Instant) -> bool {
        if self.paused {
            return false;
        }
        let Some(end) = self.ends_at else {
            return false;
        };
        if now < end {
            return false;
        }
        self.phase = match self.phase {
            Phase::Work => Phase::Rest,
            Phase::Rest => Phase::Work,
        };
        let dur = self.phase_duration();
        self.remaining = dur;
        self.ends_at = Some(now + dur);
        true
    }

    /// Toggle between paused and running. Records remaining time on pause and
    /// resumes from that point.
    pub fn toggle_pause(&mut self, now: Instant) {
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
