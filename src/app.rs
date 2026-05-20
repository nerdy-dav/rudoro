use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Work,
    Rest,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Work => "Work",
            Phase::Rest => "Rest",
        }
    }
}

pub struct App {
    pub phase: Phase,
    pub work: Duration,
    pub rest: Duration,
    pub paused: bool,
    pub ends_at: Option<Instant>,
    pub remaining: Duration,
}

impl App {
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

    pub fn phase_duration(&self) -> Duration {
        match self.phase {
            Phase::Work => self.work,
            Phase::Rest => self.rest,
        }
    }

    pub fn remaining_at(&self, now: Instant) -> Duration {
        if self.paused {
            self.remaining
        } else if let Some(end) = self.ends_at {
            end.saturating_duration_since(now)
        } else {
            Duration::ZERO
        }
    }

    /// Returns `true` when a phase transition occurred.
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
