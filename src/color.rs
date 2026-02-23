use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};

#[macro_export]
macro_rules! esc {
    ($code:tt) => {
        concat!("\x1B[", $code, "m")
    };
}

#[derive(Clone, Default)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    #[default]
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
}

impl Color {
    pub const RESET: &'static str = esc!(0);
    pub const BOLD: &'static str = esc!(1);
    pub const POSSIBLE_VALUES: &[&'static str] = &[
        "black",
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        "white",
        "bright-black",
        "bright-red",
        "bright-green",
        "bright-yellow",
        "bright-blue",
        "bright-magenta",
        "bright-cyan",
        "bright-white",
        "'#rrggbb'",
    ];

    pub fn foreground(&self) -> String {
        match self {
            Self::Black => esc!(30),
            Self::Red => esc!(31),
            Self::Green => esc!(32),
            Self::Yellow => esc!(33),
            Self::Blue => esc!(34),
            Self::Magenta => esc!(35),
            Self::Cyan => esc!(36),
            Self::White => esc!(37),
            Self::BrightBlack => esc!(90),
            Self::BrightRed => esc!(91),
            Self::BrightGreen => esc!(92),
            Self::BrightYellow => esc!(93),
            Self::BrightBlue => esc!(94),
            Self::BrightMagenta => esc!(95),
            Self::BrightCyan => esc!(96),
            Self::BrightWhite => esc!(97),
            Self::Rgb { r, g, b } => return format!("\x1B[38;2;{r};{g};{b}m"),
        }
        .to_string()
    }

    pub fn background(&self) -> String {
        match self {
            Self::Black => esc!(40),
            Self::Red => esc!(41),
            Self::Green => esc!(42),
            Self::Yellow => esc!(43),
            Self::Blue => esc!(44),
            Self::Magenta => esc!(45),
            Self::Cyan => esc!(46),
            Self::White => esc!(47),
            Self::BrightBlack => esc!(100),
            Self::BrightRed => esc!(101),
            Self::BrightGreen => esc!(102),
            Self::BrightYellow => esc!(103),
            Self::BrightBlue => esc!(104),
            Self::BrightMagenta => esc!(105),
            Self::BrightCyan => esc!(106),
            Self::BrightWhite => esc!(107),
            Self::Rgb { r, g, b } => return format!("\x1B[48;2;{r};{g};{b}m"),
        }
        .to_string()
    }

    fn parse_rgb_component(hex: &str) -> Result<u8, String> {
        u8::from_str_radix(hex, 16).map_err(|err| err.to_string())
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "black" => Self::Black,
            "red" => Self::Red,
            "green" => Self::Green,
            "yellow" => Self::Yellow,
            "blue" => Self::Blue,
            "magenta" => Self::Magenta,
            "cyan" => Self::Cyan,
            "white" => Self::White,
            "bright-black" => Self::BrightBlack,
            "bright-red" => Self::BrightRed,
            "bright-green" => Self::BrightGreen,
            "bright-yellow" => Self::BrightYellow,
            "bright-blue" => Self::BrightBlue,
            "bright-magenta" => Self::BrightMagenta,
            "bright-cyan" => Self::BrightCyan,
            "bright-white" => Self::BrightWhite,
            _ if s.starts_with('#') => {
                if s.len() != 7 {
                    return Err(format!("expected format `#rrggbb`, found `{s}`"));
                }

                let r = Self::parse_rgb_component(&s[1..3])?;
                let g = Self::parse_rgb_component(&s[3..5])?;
                let b = Self::parse_rgb_component(&s[5..7])?;

                Self::Rgb { r, g, b }
            }
            _ => {
                let green_fg = Color::Green.foreground();

                return Err(format!(
                    "color `{s}` is neither a recognized color nor a valid hex color code.\n  [possible values: {}{}{}]",
                    green_fg,
                    Self::POSSIBLE_VALUES.join(&format!("{},{} ", Color::RESET, green_fg)),
                    Color::RESET
                ));
            }
        })
    }
}

pub fn next_color(current: &Color) -> Color {
    match current {
        Color::Black => Color::Red,
        Color::Red => Color::Green,
        Color::Green => Color::Yellow,
        Color::Yellow => Color::Blue,
        Color::Blue => Color::Magenta,
        Color::Magenta => Color::Cyan,
        Color::Cyan => Color::White,
        Color::White => Color::BrightBlack,
        Color::BrightBlack => Color::BrightRed,
        Color::BrightRed => Color::BrightGreen,
        Color::BrightGreen => Color::BrightYellow,
        Color::BrightYellow => Color::BrightBlue,
        Color::BrightBlue => Color::BrightMagenta,
        Color::BrightMagenta => Color::BrightCyan,
        Color::BrightCyan => Color::BrightWhite,
        Color::BrightWhite => Color::Black,
        Color::Rgb { .. } => Color::White,
    }
}

pub fn prev_color(current: &Color) -> Color {
    match current {
        Color::Black => Color::BrightWhite,
        Color::Red => Color::Black,
        Color::Green => Color::Red,
        Color::Yellow => Color::Green,
        Color::Blue => Color::Yellow,
        Color::Magenta => Color::Blue,
        Color::Cyan => Color::Magenta,
        Color::White => Color::Cyan,
        Color::BrightBlack => Color::White,
        Color::BrightRed => Color::BrightBlack,
        Color::BrightGreen => Color::BrightRed,
        Color::BrightYellow => Color::BrightGreen,
        Color::BrightBlue => Color::BrightYellow,
        Color::BrightMagenta => Color::BrightBlue,
        Color::BrightCyan => Color::BrightMagenta,
        Color::BrightWhite => Color::BrightCyan,
        Color::Rgb { .. } => Color::White,
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        Color::from_str(&string).map_err(de::Error::custom)
    }
}
