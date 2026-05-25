use std::{
    io::{self, BufWriter, Write},
    time::Duration,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

#[cfg(unix)]
use signal_hook::{consts, flag};

use crate::{
    cli::args::{Args, Mode, TimerArgs},
    clock::{
        counter::{Counter, CounterType},
        mode::ClockMode,
        time_zone::TimeZone,
        Clock,
    },
    color::{next_color, prev_color},
    config::{toml_bool, toml_int, toml_str, Change, Config, ConfigWriter},
    error::Error,
};

pub struct State {
    clock: Clock,
    plain: bool,
    baseline: Config,
}

impl State {
    pub fn new() -> Result<Self, Error> {
        let args = Args::parse();
        let mut config = Config::parse()?;
        let mode = args.mode.clone();
        let plain = args.plain;

        args.overwrite(&mut config)?;

        let baseline = config.clone();
        let clock_mode = Self::clock_mode(mode, &config)?;
        let mut clock = Clock::new(config, clock_mode);

        let (width, height) = terminal::size().map_err(Error::Io)?;
        clock.update_padding(width, height)?;

        Ok(Self { clock, plain, baseline })
    }

    fn clock_mode(mode: Option<Mode>, config: &Config) -> Result<ClockMode, Error> {
        let TimerArgs {
            seconds,
            minutes,
            hours,
            kill,
        } = match mode {
            Some(Mode::Clock) | None => {
                return Ok(ClockMode::Time {
                    time_zone: TimeZone::from_utc(config.date.utc),
                    date_format: config.date.fmt.clone(),
                });
            }
            Some(Mode::Stopwatch) => {
                return Ok(ClockMode::Counter(Counter::new(CounterType::Stopwatch)))
            }
            Some(Mode::Timer(timer_args)) => timer_args,
        };

        let total_seconds = match (seconds, minutes, hours) {
            (None, None, None) => Counter::DEFAULT_TIMER_DURATION,
            _ => {
                let seconds = seconds.unwrap_or_default();
                let minutes = minutes.unwrap_or_default();
                let hours = hours.unwrap_or_default();
                let total_seconds = hours * 3600 + minutes * 60 + seconds;

                if total_seconds > Counter::MAX_TIMER_DURATION {
                    return Err(Error::TimerDurationTooLong {
                        hours,
                        minutes,
                        seconds,
                    });
                }

                total_seconds
            }
        };

        Ok(ClockMode::Counter(Counter::new(CounterType::Timer {
            duration: Duration::from_secs(total_seconds),
            kill,
        })))
    }

    pub fn run(mut self) -> Result<(), Error> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;

        let reload_config = Arc::new(AtomicBool::new(false));

        #[cfg(unix)]
        flag::register(consts::SIGUSR1, Arc::clone(&reload_config))?;

        loop {
            if reload_config.swap(false, Ordering::Relaxed) {
                self.reload_config()?;
            }

            self.render()?;

            if !event::poll(self.clock.interval)? {
                continue;
            }

            match event::read()? {
                Event::Key(key_event) => match key_event {
                    KeyEvent {
                        code: KeyCode::Esc | KeyCode::Char('Q' | 'q'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => return Ok(()),
                    KeyEvent {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => reload_config.store(true, Ordering::Relaxed),
                    KeyEvent {
                        code: KeyCode::Char(character @ ('P' | 'p' | 'R' | 'r')),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                        ..
                    } => {
                        let ClockMode::Counter(counter) = &mut self.clock.mode else {
                            continue;
                        };

                        match character {
                            'P' | 'p' => counter.toggle_pause(),
                            _ => counter.restart(),
                        }

                        let (width, height) = terminal::size()?;
                        self.refresh_display(width, height)?;
                    }
                    // keybinds for all of the commands
                    KeyEvent {
                        code: KeyCode::Char('-'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        let ms = self.clock.interval.as_millis() as u64;
                        let raw = ms.saturating_sub(100).max(100);
                        let auto = Clock::auto_interval(self.clock.blink);
                        // snap to auto if step crossed it (going down through auto)
                        let new_ms = if ms > auto && raw < auto { auto } else { raw };
                        self.clock.interval = Duration::from_millis(new_ms);
                        self.clock.interval_auto = new_ms == auto;
                    }
                    KeyEvent {
                        code: KeyCode::Char('+' | '='),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        let ms = self.clock.interval.as_millis() as u64;
                        let raw = (ms + 100).min(9900);
                        let auto = Clock::auto_interval(self.clock.blink);
                        // snap to auto if step crossed it (going up through auto)
                        let new_ms = if ms < auto && raw > auto { auto } else { raw };
                        self.clock.interval = Duration::from_millis(new_ms);
                        self.clock.interval_auto = new_ms == auto;
                    }
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        self.clock.color = next_color(&self.clock.color);
                    }
                    KeyEvent {
                        code: KeyCode::Char('C'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } => {
                        self.clock.color = prev_color(&self.clock.color);
                    }
                    KeyEvent {
                        code: KeyCode::Char('b' | 'B'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                        ..
                    } => {
                        self.clock.blink = !self.clock.blink;
                        if self.clock.interval_auto {
                            self.clock.interval =
                                Duration::from_millis(Clock::auto_interval(self.clock.blink));
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char('s' | 'S'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                        ..
                    } => {
                        self.clock.hide_seconds = !self.clock.hide_seconds;
                        let (width, height) = terminal::size()?;
                        self.refresh_display(width, height)?;
                    }
                    KeyEvent {
                        code: KeyCode::Char('h' | 'H'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                        ..
                    } => {
                        self.plain = !self.plain;
                        let (width, height) = terminal::size()?;
                        self.refresh_display(width, height)?;
                    }
                    _ => (),
                },
                Event::Resize(width, height) => self.refresh_display(width, height)?,

                _ => (),
            }
        }
    }

    pub fn exit() {
        execute!(io::stdout(), LeaveAlternateScreen, Show).expect(
            "error: failed to leave alternate screen, you might have to restart your terminal",
        );
        terminal::disable_raw_mode()
            .expect("error: failed to disable raw mode, you might have to restart your terminal");
    }

    fn refresh_display(&mut self, width: u16, height: u16) -> Result<(), Error> {
        execute!(io::stdout(), Clear(ClearType::All))?;
        self.clock.update_padding(width, height)
    }

    fn reload_config(&mut self) -> Result<(), Error> {
        // flush any pending tweaks first so they aren't wiped by reload
        self.persist_changes();

        let config = Config::parse()?;
        self.baseline = config.clone();
        let clock = &mut self.clock;

        clock.color = config.general.color;
        clock.blink = config.general.blink;
        clock.bold = config.general.bold;

        clock.interval_auto = config.general.interval.is_none();
        clock.interval = Duration::from_millis(
            config
                .general
                .interval
                .unwrap_or_else(|| Clock::auto_interval(clock.blink)),
        );

        clock.x_pos = config.position.x;
        clock.y_pos = config.position.y;

        clock.use_12h = config.date.use_12h;
        clock.hide_seconds = config.date.hide_seconds;

        if let ClockMode::Time {
            time_zone,
            date_format,
        } = &mut self.clock.mode
        {
            *time_zone = TimeZone::from_utc(config.date.utc);
            *date_format = config.date.fmt;
        }

        let (width, height) = terminal::size()?;
        self.refresh_display(width, height)
    }

    fn render(&self) -> Result<(), Error> {
        let (width, height) = terminal::size()?;

        if self.clock.is_too_large(width, height) {
            return Ok(());
        }

        let mut stdout = io::stdout();

        execute!(stdout, MoveTo(0, self.clock.padding.top))?;

        let lock = stdout.lock();
        let mut buffered_writer = BufWriter::new(lock);

        self.clock.fmt(&mut buffered_writer)?;

        if !self.plain {
            self.render_statusbar(&mut buffered_writer, width, height)?;
        }

        buffered_writer.flush()?;

        Ok(())
    }

    fn render_statusbar(&self, w: &mut BufWriter<io::StdoutLock<'_>>, width: u16, height: u16) -> Result<(), Error> {
        let right = if self.clock.interval_auto {
            " auto \u{2014}".to_string()
        } else {
            format!(" {}ms \u{2014}", self.clock.interval.as_millis())
        };
        let left = "\u{2014} b: Blink | s: Secs | c: Color | -/+: Interval | h: Hide "; // "— b: ..."

        let left_len = left.chars().count();
        let right_len = right.chars().count();
        let total = width as usize;

        // fill dashes between left and right and clamp we never go negative
        let fill_count = total.saturating_sub(left_len + right_len);
        let fill = "\u{2014}".repeat(fill_count);

        // move to the bottom row and write everything
        write!(
            w,
            "\x1B[{};1H\x1B[2m{left}{fill}{right}\x1B[0m",
            height,
        )?;

        Ok(())
    }
}

impl State {
    fn persist_changes(&self) {
        let path = match Config::save_path() {
            Ok(p) => p,
            Err(_) => return,
        };

        let base = &self.baseline;
        let clock = &self.clock;
        let color_str = clock.color.as_toml_string();
        let mut changes: Vec<Change> = Vec::new();

        if clock.color != base.general.color {
            changes.push(Change::Set("general", "color", toml_str(&color_str)));
        }
        if clock.blink != base.general.blink {
            changes.push(Change::Set("general", "blink", toml_bool(clock.blink)));
        }
        if clock.bold != base.general.bold {
            changes.push(Change::Set("general", "bold", toml_bool(clock.bold)));
        }

        let current_interval = if clock.interval_auto {
            None
        } else {
            Some(clock.interval.as_millis() as u64)
        };
        if current_interval != base.general.interval {
            match current_interval {
                Some(ms) => changes.push(Change::Set("general", "interval", toml_int(ms as i64))),
                None => changes.push(Change::Remove("general", "interval")),
            }
        }

        if clock.x_pos != base.position.x {
            changes.push(Change::Set("position", "horizontal", toml_str(clock.x_pos.as_toml_str())));
        }
        if clock.y_pos != base.position.y {
            changes.push(Change::Set("position", "vertical", toml_str(clock.y_pos.as_toml_str())));
        }
        if clock.use_12h != base.date.use_12h {
            changes.push(Change::Set("date", "use_12h", toml_bool(clock.use_12h)));
        }
        if clock.hide_seconds != base.date.hide_seconds {
            changes.push(Change::Set("date", "hide_seconds", toml_bool(clock.hide_seconds)));
        }

        if changes.is_empty() {
            return;
        }
        let _ = ConfigWriter::write(&path, &changes);
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.persist_changes();
        Self::exit();
    }
}
