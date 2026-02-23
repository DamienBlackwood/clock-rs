use std::{
    process,
    time::{Duration, Instant},
};

use crate::state::State;

pub struct Counter {
    pub text: &'static str,
    ty: CounterType,
    start: Instant,
    last_pause: Option<Instant>,
    paused: bool,
}

pub enum CounterType {
    Stopwatch,
    Timer { duration: Duration, kill: bool },
}

impl Counter {
    pub const DEFAULT_TIMER_DURATION: u64 = 5 * 60;
    pub const MAX_TIMER_DURATION: u64 = 99 * 3600 + 59 * 60 + 59;
    const TEXT: &'static str = "P: Toggle Pause, R: Restart";
    const TEXT_PAUSED: &'static str = "P: Toggle Pause, R: Restart [Paused]";

    pub fn new(ty: CounterType) -> Self {
        Self {
            text: Self::TEXT,
            ty,
            start: Instant::now(),
            last_pause: None,
            paused: false,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.text = if self.paused {
            if let Some(last_pause) = self.last_pause {
                self.start += last_pause.elapsed();
                self.last_pause = None;
            }
            Self::TEXT
        } else {
            self.last_pause = Some(Instant::now());
            Self::TEXT_PAUSED
        };

        self.paused = !self.paused;
    }

    pub fn restart(&mut self) {
        self.start = Instant::now();
        self.last_pause = None;

        if self.paused {
            self.toggle_pause();
        }
    }

    pub fn get_time(&self) -> (u32, u32, u32) {
        let mut elapsed = if self.paused {
            match self.last_pause {
                Some(last_pause) => last_pause.duration_since(self.start),
                _ => Duration::from_secs(0),
            }
        } else {
            self.start.elapsed()
        };

        let mut secs = elapsed.as_secs() as u32;

        if let CounterType::Timer { duration, kill } = self.ty {
            elapsed = duration.saturating_sub(elapsed.saturating_sub(Duration::from_secs(1)));
            secs = elapsed.as_secs() as u32;

            if secs == 0 && kill {
                State::exit();
                process::exit(0);
            }
        }

        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        (hours, minutes, seconds)
    }
}
