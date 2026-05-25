use clap::ValueEnum;
use serde::Deserialize;

#[derive(Clone, Default, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Start,
    #[default]
    Center,
    End,
}

impl Position {
    pub fn calculate(&self, len: u16, offset: u16) -> u16 {
        match self {
            Self::Start => 1,
            Self::Center => (len / 2).saturating_sub(offset),
            Self::End => len.saturating_sub(offset * 2 + 2),
        }
    }

    pub fn as_toml_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}
