use anyhow::Result;
use log::warn;
use once_cell::sync::OnceCell;
use serde::{de, Deserialize, Deserializer};
use std::fs;

use cursive::theme;

#[derive(Deserialize, Debug, Clone)]
/// Config is a struct storing the application's configurations
pub struct Config {
    pub story_pooling: bool,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub color: theme::Color,
}

impl Color {
    fn parse(s: &str) -> Option<Self> {
        match theme::Color::parse(s) {
            None => None,
            Some(color) => Some(Color { color }),
        }
    }
}

impl<'de> de::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match Color::parse(&s) {
            None => Err(de::Error::custom(format!("failed to parse color: {}", s))),
            Some(color) => Ok(color),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub focus: Color,
    pub primary: Color,
}

impl Config {
    // parse config struct from a file
    pub fn from_config_file(file_path: &str) -> Result<Self> {
        match fs::read_to_string(file_path) {
            // if cannot open the file, use the default configurations
            Err(err) => {
                warn!(
                    "failed to open {}: {:#?}\nUse the default configurations instead",
                    file_path, err
                );
                Ok(Self::default())
            }
            Ok(config_str) => Ok(toml::from_str(&config_str)?),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            story_pooling: true,
            theme: Theme {
                background: Color::parse("#f6f6ef").unwrap(),
                focus: Color::parse("#6c6c6c").unwrap(),
                primary: Color::parse("#4a4a48").unwrap(),
            },
        }
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();
