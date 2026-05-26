use std::time::{Duration, Instant};

/// A phase of the Pomodoro cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn phase_label() {
        assert_eq!(Phase::Work.label(), "Work");
        assert_eq!(Phase::Rest.label(), "Rest");
    }

    #[test]
    fn new_app_starts_work_phase() {
        let app = App::new(Duration::from_secs(25 * 60), Duration::from_secs(5 * 60));
        assert_eq!(app.phase, Phase::Work);
        assert!(!app.paused);
        assert!(app.ends_at.is_some());
        assert_eq!(app.remaining, Duration::from_secs(25 * 60));
    }

    #[test]
    fn phase_duration_returns_current_phase_length() {
        let work = Duration::from_secs(10);
        let rest = Duration::from_secs(5);
        let mut app = App::new(work, rest);
        assert_eq!(app.phase_duration(), work);

        let start = Instant::now();
        let end = start + work + Duration::from_millis(1);
        app.tick(end);
        assert_eq!(app.phase_duration(), rest);
    }

    #[test]
    fn remaining_at_counts_down_when_running() {
        let app = App::new(Duration::from_secs(60), Duration::from_secs(30));
        let start = Instant::now();

        let rem = app.remaining_at(start);
        assert!(rem > Duration::from_secs(59));

        let later = start + Duration::from_secs(1);
        let rem = app.remaining_at(later);
        assert!(rem > Duration::from_secs(58) && rem < Duration::from_secs(60));
    }

    #[test]
    fn tick_returns_true_on_phase_transition() {
        let mut app = App::new(Duration::from_secs(1), Duration::from_secs(1));
        let start = Instant::now();
        assert!(app.tick(start + Duration::from_secs(2)));
    }

    #[test]
    fn tick_returns_false_before_deadline() {
        let mut app = App::new(Duration::from_secs(60), Duration::from_secs(30));
        let start = Instant::now();
        assert!(!app.tick(start));
    }

    #[test]
    fn tick_returns_false_when_paused() {
        let mut app = App::new(Duration::from_secs(10), Duration::from_secs(5));
        let start = Instant::now();
        app.toggle_pause(start);
        assert!(!app.tick(start + Duration::from_secs(20)));
    }

    #[test]
    fn tick_transitions_work_to_rest() {
        let work = Duration::from_secs(10);
        let rest = Duration::from_secs(5);
        let mut app = App::new(work, rest);
        let start = Instant::now();

        let end = start + work + Duration::from_millis(1);
        assert!(app.tick(end));
        assert_eq!(app.phase, Phase::Rest);
        assert_eq!(app.remaining, rest);
    }

    #[test]
    fn tick_transitions_rest_to_work() {
        let work = Duration::from_secs(10);
        let rest = Duration::from_secs(5);
        let mut app = App::new(work, rest);
        let start = Instant::now();

        let work_end = start + work + Duration::from_millis(1);
        app.tick(work_end);
        assert_eq!(app.phase, Phase::Rest);

        let rest_end = work_end + rest + Duration::from_millis(1);
        assert!(app.tick(rest_end));
        assert_eq!(app.phase, Phase::Work);
        assert_eq!(app.remaining, work);
    }

    #[test]
    fn tick_large_jump_only_transitions_once() {
        let mut app = App::new(Duration::from_secs(10), Duration::from_secs(5));
        let start = Instant::now();

        let far_future = start + Duration::from_secs(1000);
        assert!(app.tick(far_future));
        assert_eq!(app.phase, Phase::Rest);
        assert_eq!(app.remaining, Duration::from_secs(5));
    }

    #[test]
    fn zero_duration_phase_transitions_immediately() {
        let mut app = App::new(Duration::ZERO, Duration::from_secs(30));
        let start = Instant::now();

        assert!(app.tick(start));
        assert_eq!(app.phase, Phase::Rest);
    }

    #[test]
    fn toggle_pause_pauses_and_records_remaining() {
        let mut app = App::new(Duration::from_secs(60), Duration::from_secs(30));
        let start = Instant::now();

        let later = start + Duration::from_secs(5);
        app.toggle_pause(later);

        assert!(app.paused);
        assert!(app.ends_at.is_none());
        assert!(app.remaining > Duration::from_secs(54) && app.remaining <= Duration::from_secs(55));
    }

    #[test]
    fn toggle_pause_resume_continues_from_saved_time() {
        let mut app = App::new(Duration::from_secs(60), Duration::from_secs(30));
        let start = Instant::now();

        let pause_time = start + Duration::from_secs(5);
        app.toggle_pause(pause_time);

        let resume_time = pause_time + Duration::from_secs(10);
        app.toggle_pause(resume_time);

        assert!(!app.paused);
        assert!(app.ends_at.is_some());
        assert!(app.remaining_at(resume_time) > Duration::from_secs(50));
    }

    #[test]
    fn toggle_pause_idempotent() {
        let mut app = App::new(Duration::from_secs(60), Duration::from_secs(30));
        let start = Instant::now();

        app.toggle_pause(start);
        app.toggle_pause(start);
        app.toggle_pause(start);

        assert!(app.paused);
        assert!(app.ends_at.is_none());
    }

    #[test]
    fn remaining_at_uses_stored_value_when_paused() {
        let mut app = App::new(Duration::from_secs(60), Duration::from_secs(30));
        let start = Instant::now();

        let pause_time = start + Duration::from_secs(10);
        app.toggle_pause(pause_time);

        let much_later = pause_time + Duration::from_secs(3600);
        let rem = app.remaining_at(much_later);
        assert!(rem > Duration::from_secs(49) && rem <= Duration::from_secs(50));
    }
}
