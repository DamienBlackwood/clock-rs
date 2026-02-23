use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "the timer duration is too long: {hours}h, {minutes}m and {seconds}s exceed the maximum duration of 99h, 59m and 59s"
    )]
    TimerDurationTooLong {
        hours: u64,
        minutes: u64,
        seconds: u64,
    },
    #[error("the formatted date exceeds the clock's width: {fmt_len} > {max_len}")]
    DateFormatTooWide { fmt_len: u16, max_len: u16 },
    #[error("failed to format the date string `{fmt}`: {err}")]
    DateFormatInvalid { fmt: String, err: String },
    #[error("configuration path is invalid unicode: `{0}`")]
    NonUnicodePath(String),
    #[error("failed to read file `{path}`: {err}")]
    ReadFile { path: String, err: String },
    #[error("failed to parse configuration file `{path}`:\n{err}")]
    ParseToml { path: String, err: String },
    #[error("interval must be greater than 0")]
    ZeroInterval,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
