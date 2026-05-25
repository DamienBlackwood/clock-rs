use std::{
    env::{self, VarError},
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use toml_edit::DocumentMut;

use crate::{color::Color, error::Error, position::Position};

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub position: PositionConfig,
    pub date: DateConfig,
}

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub color: Color,
    pub interval: Option<u64>,
    pub blink: bool,
    pub bold: bool,
}

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct PositionConfig {
    #[serde(rename = "horizontal")]
    pub x: Position,
    #[serde(rename = "vertical")]
    pub y: Position,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct DateConfig {
    pub fmt: String,
    pub use_12h: bool,
    pub utc: bool,
    pub hide_seconds: bool,
}

impl Default for DateConfig {
    fn default() -> Self {
        Self {
            fmt: "%d-%m-%Y".to_string(),
            use_12h: false,
            utc: false,
            hide_seconds: false,
        }
    }
}

impl Config {
    pub fn parse() -> Result<Self, Error> {
        let Some(file_path) = Self::resolve_path()? else {
            return Ok(Config::default());
        };

        let config_str = fs::read_to_string(&file_path).map_err(|err| Error::ReadFile {
            path: file_path.display().to_string(),
            err: err.to_string(),
        })?;

        toml::from_str(&config_str).map_err(|err| Error::ParseToml {
            path: file_path.display().to_string(),
            err: err.to_string(),
        })
    }

    pub fn resolve_path() -> Result<Option<PathBuf>, Error> {
        match env::var("CONF_PATH") {
            Ok(path) if path == "None" => Ok(None),
            Ok(path) => Ok(Some(PathBuf::from(path))),
            Err(VarError::NotUnicode(path)) => Err(Error::NonUnicodePath(path.display().to_string())),
            Err(VarError::NotPresent) => match dirs::config_local_dir() {
                Some(dir) => {
                    let p = dir.join("clock-rs").join("conf.toml");
                    if p.exists() { Ok(Some(p)) } else { Ok(None) }
                }
                None => Ok(None),
            },
        }
    }

    // i was debatign wether i should do it per directory or OS level, but i just chose OS
    pub fn save_path() -> Result<PathBuf, Error> {
        if let Ok(p) = env::var("CONF_PATH") {
            if p != "None" {
                return Ok(PathBuf::from(p));
            }
        }
        match dirs::config_local_dir() {
            Some(dir) => Ok(dir.join("clock-rs").join("conf.toml")),
            None => Err(Error::NonUnicodePath("no OS config dir".into())),
        }
    }
}

pub enum Change<'a> {
    Set(&'a str, &'a str, toml_edit::Value),
    Remove(&'a str, &'a str),
}

pub struct ConfigWriter;

impl ConfigWriter {
    pub fn write(path: &Path, changes: &[Change]) -> Result<(), Error> {
        let mut doc: DocumentMut = if path.exists() {
            let s = fs::read_to_string(path).map_err(|err| Error::ReadFile {
                path: path.display().to_string(),
                err: err.to_string(),
            })?;
            s.parse().map_err(|err: toml_edit::TomlError| Error::ParseToml {
                path: path.display().to_string(),
                err: err.to_string(),
            })?
        } else {
            DocumentMut::new()
        };

        for change in changes {
            match change {
                Change::Set(table, key, new_val) => {
                    let tbl = doc
                        .entry(table)
                        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
                        .as_table_mut()
                        .expect("table entry must be a table");

                    if let Some(existing) = tbl.get_mut(key) {
                        if let Some(v) = existing.as_value_mut() {
                            let decor = v.decor().clone();
                            *v = new_val.clone();
                            *v.decor_mut() = decor;
                            continue;
                        }
                    }
                    tbl.insert(key, toml_edit::Item::Value(new_val.clone()));
                }
                Change::Remove(table, key) => {
                    if let Some(tbl) = doc.get_mut(table).and_then(|i| i.as_table_mut()) {
                        tbl.remove(key);
                    }
                }
            }
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(Error::Io)?;
            }
        }

        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, doc.to_string()).map_err(Error::Io)?;
        fs::rename(&tmp, path).map_err(Error::Io)?;
        Ok(())
    }
}

pub fn toml_str(s: &str) -> toml_edit::Value {
    toml_edit::Value::from(s)
}
pub fn toml_bool(b: bool) -> toml_edit::Value {
    toml_edit::Value::from(b)
}
pub fn toml_int(n: i64) -> toml_edit::Value {
    toml_edit::Value::from(n)
}
