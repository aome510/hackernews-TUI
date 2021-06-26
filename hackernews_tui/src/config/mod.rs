// modules
mod keybindings;
mod theme;

// re-export
pub use keybindings::*;
pub use theme::*;

use config_parser2::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, ConfigParse)]
/// Config is a struct storing the application's configurations
pub struct Config {
    pub allow_unicode: bool,
    pub page_scrolling: bool,
    pub scroll_offset: usize,
    pub url_open_command: String,
    pub article_parse_command: ArticleParseCommand,
    pub client: Client,
    pub theme: theme::Theme,
    pub keymap: keybindings::KeyMap,
}

impl Config {
    // parse config struct from a file
    pub fn from_config_file(file_path: &str) -> anyhow::Result<Self> {
        match std::fs::read_to_string(file_path) {
            // if cannot open the file, use the default configurations
            Err(err) => {
                log::warn!(
                    "failed to open {}: {:#?}\n...Use the default configurations instead",
                    file_path,
                    err
                );
                Ok(Self::default())
            }
            Ok(config_str) => {
                let value = toml::from_str::<toml::Value>(&config_str)?;
                let mut config = Self::default();
                config.parse(value)?;
                Ok(config)
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            allow_unicode: false,
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
            theme: theme::Theme::default(),
            keymap: keybindings::KeyMap::default(),
        }
    }
}

#[derive(Debug, Deserialize, ConfigParse)]
pub struct LazyLoadingComments {
    pub num_comments_init: usize,
    pub num_comments_after: usize,
}

#[derive(Debug, Deserialize, ConfigParse, Clone)]
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

#[derive(Debug, Deserialize, ConfigParse)]
pub struct Client {
    pub story_limit: StoryLimit,
    pub lazy_loading_comments: LazyLoadingComments,
    pub client_timeout: u64,
}

static CONFIG: once_cell::sync::OnceCell<Config> = once_cell::sync::OnceCell::new();

pub fn init_config(config: Config) {
    CONFIG.set(config).unwrap_or_else(|_| {
        panic!("failed to set up the application's configurations");
    });
}

pub fn get_config() -> &'static Config {
    &CONFIG.get().unwrap()
}
