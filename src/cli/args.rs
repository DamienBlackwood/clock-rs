use clap::{
    builder::styling::{AnsiColor, Styles},
    Parser, Subcommand,
};
use serde::Deserialize;

use crate::{color::Color, position::Position};

#[derive(Parser)]
#[clap(version = "v0.2.0, (c) 2024 Oughie, fork (c) 2026 Damien Blackwood", hide_possible_values = true, styles = Self::STYLES)]
pub struct Args {
    #[clap(subcommand)]
    pub mode: Option<Mode>,
    #[doc = "Specify the clock color"]
    #[clap(long, short)]
    pub color: Option<Color>,
    #[doc = "Set the polling interval in milliseconds"]
    #[clap(long, short)]
    pub interval: Option<u64>,
    #[doc = "Set the colon to blink"]
    #[clap(long, short = 'B')]
    pub blink: bool,
    #[doc = "Use bold text"]
    #[clap(long, short)]
    pub bold: bool,
    #[doc = "Set the position along the horizontal axis"]
    #[clap(long, short)]
    pub x_pos: Option<Position>,
    #[doc = "Set the position along the vertical axis"]
    #[clap(long, short)]
    pub y_pos: Option<Position>,
    #[doc = "Set the date format"]
    #[clap(long)]
    pub fmt: Option<String>,
    #[doc = "Use the 12h format"]
    #[clap(short = 't')]
    pub use_12h: bool,
    #[doc = "Use UTC time"]
    #[clap(long)]
    pub utc: bool,
    #[doc = "Do not show seconds"]
    #[clap(long, short = 's')]
    pub hide_seconds: bool,
    #[doc = "Hide the status bar (just the time)"]
    #[clap(long, short = 'p')]
    pub plain: bool,
}

impl Args {
    const STYLES: Styles = Styles::styled()
        .header(AnsiColor::Green.on_default().bold().underline())
        .usage(AnsiColor::Green.on_default().bold().underline())
        .literal(AnsiColor::Blue.on_default().bold())
        .placeholder(AnsiColor::Yellow.on_default().italic());
}

#[derive(Clone, Subcommand, Deserialize)]
pub enum Mode {
    #[doc = "Display the current time (default)"]
    Clock,
    #[doc = "Create a timer (5 minutes if no time is specified)"]
    Timer(TimerArgs),
    #[doc = "Start a stopwatch"]
    Stopwatch,
}

#[derive(clap::Args, Clone, Deserialize)]
pub struct TimerArgs {
    #[doc = "Add seconds to the timer"]
    #[clap(long, short = 'S')]
    pub seconds: Option<u64>,
    #[doc = "Add minutes to the timer"]
    #[clap(long, short = 'M')]
    pub minutes: Option<u64>,
    #[doc = "Add hours to the timer"]
    #[clap(long, short = 'H')]
    pub hours: Option<u64>,
    #[doc = "Terminate the application when the timer finishes"]
    #[clap(long, short)]
    pub kill: bool,
}
