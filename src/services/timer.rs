use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerPhase {
    Work,
    ShortBreak,
    LongBreak,
    Custom,
}

impl TimerPhase {
    pub fn label(&self) -> &'static str {
        match self {
            TimerPhase::Work => "Work Focus",
            TimerPhase::ShortBreak => "Short Break",
            TimerPhase::LongBreak => "Long Break",
            TimerPhase::Custom => "Countdown",
        }
    }

    pub fn default_duration_secs(&self) -> f32 {
        match self {
            TimerPhase::Work => 25.0 * 60.0,
            TimerPhase::ShortBreak => 5.0 * 60.0,
            TimerPhase::LongBreak => 15.0 * 60.0,
            TimerPhase::Custom => 10.0 * 60.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PomodoroTimer {
    pub phase: TimerPhase,
    pub is_running: bool,
    pub remaining_secs: f32,
    pub total_duration_secs: f32,
    pub completed_sessions: u32,
    last_tick: Instant,
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl PomodoroTimer {
    pub fn new() -> Self {
        let default_secs = TimerPhase::Work.default_duration_secs();
        Self {
            phase: TimerPhase::Work,
            is_running: false,
            remaining_secs: default_secs,
            total_duration_secs: default_secs,
            completed_sessions: 0,
            last_tick: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;

        if self.is_running {
            if self.remaining_secs > dt {
                self.remaining_secs -= dt;
            } else {
                self.remaining_secs = 0.0;
                self.is_running = false;
                self.on_phase_complete();
            }
        }
    }

    fn on_phase_complete(&mut self) {
        match self.phase {
            TimerPhase::Work => {
                self.completed_sessions += 1;
                if self.completed_sessions % 4 == 0 {
                    self.set_phase(TimerPhase::LongBreak);
                } else {
                    self.set_phase(TimerPhase::ShortBreak);
                }
            }
            TimerPhase::ShortBreak | TimerPhase::LongBreak | TimerPhase::Custom => {
                self.set_phase(TimerPhase::Work);
            }
        }
    }

    pub fn toggle_start_pause(&mut self) {
        self.is_running = !self.is_running;
        self.last_tick = Instant::now();
    }

    pub fn start(&mut self) {
        self.is_running = true;
        self.last_tick = Instant::now();
    }

    pub fn pause(&mut self) {
        self.is_running = false;
    }

    pub fn reset(&mut self) {
        self.is_running = false;
        self.remaining_secs = self.total_duration_secs;
        self.last_tick = Instant::now();
    }

    pub fn set_phase(&mut self, phase: TimerPhase) {
        self.phase = phase;
        let secs = phase.default_duration_secs();
        self.total_duration_secs = secs;
        self.remaining_secs = secs;
        self.is_running = false;
        self.last_tick = Instant::now();
    }

    pub fn add_minutes(&mut self, mins: f32) {
        let add_secs = mins * 60.0;
        self.remaining_secs += add_secs;
        self.total_duration_secs = self.total_duration_secs.max(self.remaining_secs);
        self.last_tick = Instant::now();
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.total_duration_secs <= 0.0 {
            0.0
        } else {
            1.0 - (self.remaining_secs / self.total_duration_secs).clamp(0.0, 1.0)
        }
    }

    pub fn formatted_remaining(&self) -> String {
        let s = self.remaining_secs.round() as u32;
        let m = s / 60;
        let rem_s = s % 60;
        format!("{:02}:{:02}", m, rem_s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pomodoro_timer_lifecycle() {
        let mut timer = PomodoroTimer::new();
        assert_eq!(timer.phase, TimerPhase::Work);
        assert_eq!(timer.formatted_remaining(), "25:00");
        assert_eq!(timer.progress_ratio(), 0.0);

        timer.toggle_start_pause();
        assert!(timer.is_running);

        timer.add_minutes(5.0);
        assert_eq!(timer.formatted_remaining(), "30:00");

        timer.set_phase(TimerPhase::ShortBreak);
        assert_eq!(timer.phase, TimerPhase::ShortBreak);
        assert_eq!(timer.formatted_remaining(), "05:00");
    }
}
