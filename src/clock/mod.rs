pub mod counter;
pub mod mode;
pub mod time_zone;

use std::{
    io::{BufWriter, StdoutLock, Write},
    time::Duration,
};

use crate::{
    character::Character, clock::mode::ClockMode, color::Color, config::Config, error::Error,
    position::Position,
};

#[derive(Default)]
pub struct Padding {
    pub top: u16,
    clock: String,
    text: String,
}

pub struct Clock {
    pub mode: ClockMode,
    pub padding: Padding,
    pub interval: Duration,
    pub x_pos: Position,
    pub y_pos: Position,
    pub color: Color,
    pub use_12h: bool,
    pub hide_seconds: bool,
    pub blink: bool,
    pub bold: bool,
}

impl Clock {
    const WIDTH: u16 = 51;
    const WIDTH_NO_SECONDS: u16 = 32;
    const HEIGHT: u16 = 9;
    const SUFFIX_LEN: u16 = 5;
    const AM_SUFFIX: &'static str = " [AM]";
    const PM_SUFFIX: &'static str = " [PM]";

    pub fn new(config: Config, mode: ClockMode) -> Self {
        Self {
            mode,
            padding: Padding::default(),
            interval: Duration::from_millis(config.general.interval),
            x_pos: config.position.x,
            y_pos: config.position.y,
            color: config.general.color,
            use_12h: config.date.use_12h,
            hide_seconds: config.date.hide_seconds,
            blink: config.general.blink,
            bold: config.general.bold,
        }
    }

    pub fn update_padding(&mut self, width: u16, height: u16) -> Result<(), Error> {
        let clock_width = self.width();
        let text_len = self.mode.text(clock_width)?.len() as u16
            + if self.use_12h { Self::SUFFIX_LEN } else { 0 };

        let half_width = clock_width / 2;

        let column = self.x_pos.calculate(width, half_width);
        self.padding.top = self.y_pos.calculate(height, Self::HEIGHT / 2);

        self.padding.clock = " ".repeat(column as usize);
        self.padding.text = format!(
            "{}{}",
            self.padding.clock,
            " ".repeat(half_width.saturating_sub(text_len / 2) as usize)
        );

        Ok(())
    }

    pub fn is_too_large(&self, width: u16, height: u16) -> bool {
        self.width() + 1 >= width || Self::HEIGHT + 1 >= height
    }

    fn width(&self) -> u16 {
        if self.hide_seconds {
            return Self::WIDTH_NO_SECONDS;
        }

        Self::WIDTH
    }

    pub fn fmt(&self, w: &mut BufWriter<StdoutLock<'_>>) -> Result<(), Error> {
        let mut text = self.mode.text(self.width())?;
        let (mut hour, minute, second) = self.mode.get_time();

        if matches!(self.mode, ClockMode::Time { .. }) && self.use_12h {
            let suffix = if hour < 12 {
                Self::AM_SUFFIX
            } else {
                Self::PM_SUFFIX
            };

            text.push_str(suffix);

            if hour > 12 {
                hour -= 12;
            } else if hour == 0 {
                hour = 12;
            }
        }

        let color = &self.color;

        for row in 0..5 {
            let colon_character = if self.blink && (second & 1 == 1) {
                Character::Empty
            } else {
                Character::Colon
            };

            let colon = colon_character.fmt(color, row);
            let h0 = Character::Num(hour / 10).fmt(color, row);
            let h1 = Character::Num(hour % 10).fmt(color, row);
            let m0 = Character::Num(minute / 10).fmt(color, row);
            let m1 = Character::Num(minute % 10).fmt(color, row);

            write!(w, "{}{h0}{h1}{colon}{m0}{m1}", self.padding.clock)?;

            if !self.hide_seconds {
                let s0 = Character::Num(second / 10).fmt(color, row);
                let s1 = Character::Num(second % 10).fmt(color, row);

                write!(w, "{colon}{s0}{s1}")?;
            }

            writeln!(w, "\r")?;
        }

        let bold_escape_str = if self.bold { Color::BOLD } else { "" };

        writeln!(
            w,
            "\n{bold_escape_str}{}{}{text}",
            self.padding.text,
            self.color.foreground()
        )?;

        Ok(())
    }
}
