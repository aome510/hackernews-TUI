use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub story_pooling: bool,
}

impl Config {
    pub fn from_config_file(file_path: &str) -> Result<Self> {
        let config_str = fs::read_to_string(file_path)?;
        Ok(toml::from_str(&config_str)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            story_pooling: true,
        }
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();
