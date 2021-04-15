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
    pub page_scrolling: bool,
    pub client: Client,
    pub theme: Theme,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StoryLimit {
    pub front_page: usize,
    pub search: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Client {
    pub story_limit: StoryLimit,
    pub client_timeout: u64,
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
    // cursive's palette colors
    pub background: Color,
    pub view: Color,
    pub shadow: Color,
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub title_primary: Color,
    pub title_secondary: Color,
    pub highlight: Color,
    pub highlight_inactive: Color,
    pub highlight_text: Color,

    pub link_text: Color,
    pub link_id_bg: Color,
    pub search_highlight_bg: Color,
    pub status_bar_bg: Color,
    pub code_block_bg: Color,
}

impl Theme {
    pub fn update_theme(&self, theme: &mut theme::Theme) {
        theme.palette.set_color("background", self.background.color);
        theme.palette.set_color("view", self.view.color);
        theme.palette.set_color("shadow", self.shadow.color);
        theme.palette.set_color("primary", self.primary.color);
        theme.palette.set_color("secondary", self.secondary.color);
        theme.palette.set_color("tertiary", self.tertiary.color);
        theme
            .palette
            .set_color("title_primary", self.title_primary.color);
        theme
            .palette
            .set_color("title_secondary", self.title_secondary.color);
        theme.palette.set_color("highlight", self.highlight.color);
        theme
            .palette
            .set_color("highlight_inactive", self.highlight_inactive.color);
        theme
            .palette
            .set_color("highlight_text", self.highlight_text.color);
    }
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
            page_scrolling: true,
            client: Client {
                story_limit: StoryLimit {
                    front_page: 25,
                    search: 10,
                },
                client_timeout: 16,
            },
            theme: Theme {
                background: Color::parse("#f6f6ef").unwrap(),
                shadow: Color::parse("black").unwrap(),
                view: Color::parse("#f6f6ef").unwrap(),
                primary: Color::parse("#4a4a48").unwrap(),
                secondary: Color::parse("#a5a5a5").unwrap(),
                tertiary: Color::parse("white").unwrap(),
                title_primary: Color::parse("black").unwrap(),
                title_secondary: Color::parse("yellow").unwrap(),
                highlight: Color::parse("#6c6c6c").unwrap(),
                highlight_inactive: Color::parse("blue").unwrap(),
                highlight_text: Color::parse("white").unwrap(),

                link_text: Color::parse("#4fbbfd").unwrap(),
                link_id_bg: Color::parse("#ffff00").unwrap(),
                search_highlight_bg: Color::parse("#ffff00").unwrap(),
                status_bar_bg: Color::parse("#ff6600").unwrap(),
                code_block_bg: Color::parse("#c8c8c8").unwrap(),
            },
        }
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn get_config_theme() -> &'static Theme {
    &CONFIG.get().unwrap().theme
}
