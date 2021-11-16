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
    pub page_scrolling: bool,
    pub scroll_offset: usize,
    pub url_open_command: String,
    pub article_parse_command: ArticleParseCommand,
    pub client: Client,
    pub theme: theme::Theme,
    pub keymap: keybindings::KeyMap,
}

impl Config {
    /// parse config from a file
    pub fn from_config_file(file_path: &str) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(file_path)?;
        let value = toml::from_str::<toml::Value>(&config_str)?;
        let mut config = Self::default();
        config.parse(value)?;
        Ok(config)
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
    pub client_timeout: u64,
}

static CONFIG: once_cell::sync::OnceCell<Config> = once_cell::sync::OnceCell::new();

/// loads the configuration options from a config file.
/// If failed to find/process the file, loads the default options.
pub fn load_config(config_file_path: Option<&str>) {
    // if no config file is specified, use the default value ($HOME/.config/hn-tui.toml)
    let config_file_path = if let Some(path) = config_file_path {
        Some(path.to_string())
    } else {
        dirs_next::home_dir().map(|path| format!("{}/.config/hn-tui.toml", path.to_str().unwrap()))
    };

    let config = match config_file_path {
        None => Config::default(),
        Some(config_file_path) => match Config::from_config_file(&config_file_path) {
            Err(err) => {
                tracing::error!(
                    "failed to load configurations from the file {}: {} \
                     \n...Use the default configurations instead",
                    config_file_path,
                    err
                );
                Config::default()
            }
            Ok(config) => config,
        },
    };

    init_config(config);
}

fn init_config(config: Config) {
    CONFIG.set(config).unwrap_or_else(|_| {
        panic!("failed to set up the application's configurations");
    });
}

pub fn get_config() -> &'static Config {
    CONFIG.get().unwrap()
}
