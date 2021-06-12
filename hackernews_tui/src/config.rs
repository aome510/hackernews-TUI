use anyhow::Result;
use config_parser::*;
use cursive::theme;
use log::warn;
use once_cell::sync::OnceCell;
use serde::{de, Deserialize, Deserializer};

use super::keybindings::*;

#[derive(Debug, Deserialize, ConfigParse)]
/// Config is a struct storing the application's configurations
pub struct Config {
    pub page_scrolling: bool,
    pub scroll_offset: usize,
    pub url_open_command: String,
    pub article_parse_command: ArticleParseCommand,
    pub client: Client,
    pub theme: Theme,
    pub keymap: KeyMap,
}

#[derive(Debug, Deserialize, Clone, ConfigParse)]
pub struct LazyLoadingComments {
    pub num_comments_init: usize,
    pub num_comments_after: usize,
}

#[derive(Debug, Deserialize, Clone, ConfigParse)]
pub struct ArticleParseCommand {
    pub command: String,
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize, ConfigParse)]
pub struct StoryLimit {
    pub front_page: usize,
    pub story: usize,
    pub ask_hn: usize,
    pub show_hn: usize,
    pub job: usize,
    pub search: usize,
}

#[derive(Debug, Deserialize, ConfigParse)]
pub struct Client {
    pub story_limit: StoryLimit,
    pub lazy_loading_comments: LazyLoadingComments,
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

impl StoryLimit {
    pub fn get_story_limit_by_tag(&self, tag: &str) -> usize {
        match tag {
            "front_page" => self.front_page,
            "story" => self.story,
            "job" => self.job,
            "ask_hn" => self.ask_hn,
            "show_hn" => self.show_hn,
            _ => panic!("unknown tag: {}", tag),
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

config_parser_impl!(Color);

#[derive(Debug, Deserialize, Clone, ConfigParse)]
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

    // additional custom colors
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
        match std::fs::read_to_string(file_path) {
            // if cannot open the file, use the default configurations
            Err(err) => {
                warn!(
                    "failed to open {}: {:#?}\n...Use the default configurations instead",
                    file_path, err
                );
                Ok(Self::default())
            }
            Ok(config_str) => {
                let value = toml::from_str::<toml::Value>(&config_str)?;
                let mut config = Self::default();
                config.parse(value);
                Ok(config)
            }
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background: Color::parse("#f6f6ef").unwrap(),
            shadow: Color::parse("#000000").unwrap(),
            view: Color::parse("#f6f6ef").unwrap(),
            primary: Color::parse("#4a4a48").unwrap(),
            secondary: Color::parse("#a5a5a5").unwrap(),
            tertiary: Color::parse("#ffffff").unwrap(),
            title_primary: Color::parse("#000000").unwrap(),
            title_secondary: Color::parse("#ffff00").unwrap(),
            highlight: Color::parse("#6c6c6c").unwrap(),
            highlight_inactive: Color::parse("#0000ff").unwrap(),
            highlight_text: Color::parse("#c3bbbb").unwrap(),

            link_text: Color::parse("#4fbbfd").unwrap(),
            link_id_bg: Color::parse("#ffff00").unwrap(),
            search_highlight_bg: Color::parse("#ffff00").unwrap(),
            status_bar_bg: Color::parse("#ff6600").unwrap(),
            code_block_bg: Color::parse("#c8c8c8").unwrap(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            page_scrolling: true,
            scroll_offset: 3,
            url_open_command: "xdg-open".to_string(),
            article_parse_command: ArticleParseCommand {
                command: "mercury-parser".to_string(),
                options: vec!["--format".to_string(), "markdown".to_string()],
            },
            client: Client {
                lazy_loading_comments: LazyLoadingComments {
                    num_comments_init: 5,
                    num_comments_after: 10,
                },
                story_limit: StoryLimit {
                    search: 10,
                    front_page: 20,
                    story: 20,
                    ask_hn: 15,
                    show_hn: 15,
                    job: 15,
                },
                client_timeout: 32,
            },
            theme: Theme::default(),
            keymap: KeyMap::default(),
        }
    }
}

static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn init_config(config: Config) {
    CONFIG.set(config).unwrap_or_else(|_| {
        panic!("failed to set up the application's configurations");
    });
}

pub fn get_config() -> &'static Config {
    &CONFIG.get().unwrap()
}

pub fn get_config_theme() -> &'static Theme {
    &get_config().theme
}

pub fn get_custom_keymap() -> &'static CustomKeyMap {
    &get_config().keymap.custom_keymap
}

pub fn get_global_keymap() -> &'static GlobalKeyMap {
    &get_config().keymap.global_keymap
}

pub fn get_story_view_keymap() -> &'static StoryViewKeyMap {
    &get_config().keymap.story_view_keymap
}

pub fn get_search_view_keymap() -> &'static SearchViewKeyMap {
    &get_config().keymap.search_view_keymap
}

pub fn get_comment_view_keymap() -> &'static CommentViewKeyMap {
    &get_config().keymap.comment_view_keymap
}

pub fn get_article_view_keymap() -> &'static ArticleViewKeyMap {
    &get_config().keymap.article_view_keymap
}
