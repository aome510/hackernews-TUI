use crate::client;
use config_parser2::*;
use cursive::event;
use serde::{de, Deserialize, Deserializer};

#[derive(Default, Debug, Clone, Deserialize, ConfigParse)]
pub struct KeyMap {
    pub custom_keymap: CustomKeyMap,
    pub edit_keymap: EditKeyMap,
    pub global_keymap: GlobalKeyMap,
    pub story_view_keymap: StoryViewKeyMap,
    pub search_view_keymap: SearchViewKeyMap,
    pub comment_view_keymap: CommentViewKeyMap,
    pub article_view_keymap: ArticleViewKeyMap,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomViewNavigation {
    pub key: Key,
    pub tag: String,
    pub by_date: bool,
    pub numeric_filters: client::StoryNumericFilters,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct CustomKeyMap {
    pub custom_view_navigation: Vec<CustomViewNavigation>,
}

config_parser_impl!(CustomKeyMap);

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct EditKeyMap {
    pub move_cursor_left: Key,
    pub move_cursor_right: Key,
    pub move_cursor_to_begin: Key,
    pub move_cursor_to_end: Key,
    pub backward_delete_char: Key,
}

impl Default for EditKeyMap {
    fn default() -> Self {
        EditKeyMap {
            move_cursor_left: Key::new(event::Key::Left),
            move_cursor_right: Key::new(event::Key::Right),
            move_cursor_to_begin: Key::new(event::Key::Home),
            move_cursor_to_end: Key::new(event::Key::End),
            backward_delete_char: Key::new(event::Key::Backspace),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct GlobalKeyMap {
    pub open_help_dialog: Key,
    pub quit: Key,
    pub close_dialog: Key,

    // view navigation keymaps
    pub goto_previous_view: Key,
    pub goto_front_page_view: Key,
    pub goto_search_view: Key,
    pub goto_all_stories_view: Key,
    pub goto_ask_hn_view: Key,
    pub goto_show_hn_view: Key,
    pub goto_jobs_view: Key,
}

impl Default for GlobalKeyMap {
    fn default() -> Self {
        GlobalKeyMap {
            open_help_dialog: Key::new('?'),
            quit: Key::new(event::Event::CtrlChar('q')),
            close_dialog: Key::new(event::Key::Esc),

            goto_previous_view: Key::new(event::Event::CtrlChar('p')),
            goto_search_view: Key::new(event::Event::CtrlChar('s')),
            goto_front_page_view: Key::new(event::Key::F1),
            goto_all_stories_view: Key::new(event::Key::F2),
            goto_ask_hn_view: Key::new(event::Key::F3),
            goto_show_hn_view: Key::new(event::Key::F4),
            goto_jobs_view: Key::new(event::Key::F5),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct StoryViewKeyMap {
    // stories navigation keymaps
    pub next_story: Key,
    pub prev_story: Key,
    pub goto_story: Key,

    // stories paging/filtering keymaps
    pub next_page: Key,
    pub prev_page: Key,
    pub toggle_sort_by: Key,

    // link opening keymaps
    pub open_article_in_browser: Key,
    pub open_article_in_article_view: Key,
    pub open_story_in_browser: Key,

    pub goto_story_comment_view: Key,
}

impl Default for StoryViewKeyMap {
    fn default() -> Self {
        StoryViewKeyMap {
            next_story: Key::new('j'),
            prev_story: Key::new('k'),
            goto_story: Key::new('g'),

            next_page: Key::new('n'),
            prev_page: Key::new('p'),
            toggle_sort_by: Key::new('d'),

            open_article_in_browser: Key::new('o'),
            open_article_in_article_view: Key::new('O'),
            open_story_in_browser: Key::new('s'),

            goto_story_comment_view: Key::new(event::Key::Enter),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct SearchViewKeyMap {
    // switch mode keymaps
    pub to_navigation_mode: Key,
    pub to_search_mode: Key,
}

impl Default for SearchViewKeyMap {
    fn default() -> Self {
        SearchViewKeyMap {
            to_navigation_mode: Key::new(event::Key::Esc),
            to_search_mode: Key::new('i'),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct CommentViewKeyMap {
    // comments navigation keymaps
    pub next_comment: Key,
    pub prev_comment: Key,
    pub next_top_level_comment: Key,
    pub prev_top_level_comment: Key,
    pub next_leq_level_comment: Key,
    pub prev_leq_level_comment: Key,
    pub parent_comment: Key,

    // link opening keymaps
    pub open_comment_in_browser: Key,
    pub open_link_in_browser: Key,
    pub open_link_in_article_view: Key,

    // scrolling
    pub down: Key,
    pub up: Key,
    pub page_down: Key,
    pub page_up: Key,

    pub toggle_collapse_comment: Key,
}

impl Default for CommentViewKeyMap {
    fn default() -> Self {
        CommentViewKeyMap {
            next_comment: Key::new('j'),
            prev_comment: Key::new('k'),
            next_top_level_comment: Key::new('n'),
            prev_top_level_comment: Key::new('p'),
            next_leq_level_comment: Key::new('l'),
            prev_leq_level_comment: Key::new('h'),
            parent_comment: Key::new('u'),

            open_comment_in_browser: Key::new('c'),
            open_link_in_browser: Key::new('f'),
            open_link_in_article_view: Key::new('F'),

            up: Key::new(event::Key::Up),
            down: Key::new(event::Key::Down),
            page_up: Key::new(event::Key::PageUp),
            page_down: Key::new(event::Key::PageDown),

            toggle_collapse_comment: Key::new(event::Key::Tab),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct ArticleViewKeyMap {
    pub down: Key,
    pub up: Key,
    pub page_down: Key,
    pub page_up: Key,
    pub top: Key,
    pub bottom: Key,

    pub open_link_dialog: Key,
    pub link_dialog_focus_next: Key,
    pub link_dialog_focus_prev: Key,

    pub open_article_in_browser: Key,
    pub open_link_in_browser: Key,
    pub open_link_in_article_view: Key,

    pub toggle_raw_markdown_mode: Key,
}

impl Default for ArticleViewKeyMap {
    fn default() -> Self {
        ArticleViewKeyMap {
            down: Key::new('j'),
            up: Key::new('k'),
            page_down: Key::new('d'),
            page_up: Key::new('u'),
            top: Key::new('t'),
            bottom: Key::new('b'),

            open_link_dialog: Key::new('l'),
            link_dialog_focus_next: Key::new('j'),
            link_dialog_focus_prev: Key::new('k'),

            open_article_in_browser: Key::new('o'),
            open_link_in_browser: Key::new('f'),
            open_link_in_article_view: Key::new('F'),

            toggle_raw_markdown_mode: Key::new('T'),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Key {
    event: event::Event,
}

impl From<Key> for event::EventTrigger {
    fn from(k: Key) -> Self {
        k.event.into()
    }
}

impl From<Key> for event::Event {
    fn from(k: Key) -> Self {
        k.event
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.event {
            event::Event::Char(c) => write!(f, "{}", c),
            event::Event::CtrlChar(c) => write!(f, "C-{}", c),
            event::Event::AltChar(c) => write!(f, "M-{}", c),
            event::Event::Key(k) => match k {
                event::Key::Enter => write!(f, "enter"),
                event::Key::Tab => write!(f, "tab"),
                event::Key::Backspace => write!(f, "backspace"),
                event::Key::Esc => write!(f, "esc"),

                event::Key::Left => write!(f, "left"),
                event::Key::Right => write!(f, "right"),
                event::Key::Up => write!(f, "up"),
                event::Key::Down => write!(f, "down"),

                event::Key::Ins => write!(f, "ins"),
                event::Key::Del => write!(f, "del"),
                event::Key::Home => write!(f, "home"),
                event::Key::End => write!(f, "end"),
                event::Key::PageUp => write!(f, "page_up"),
                event::Key::PageDown => write!(f, "page_down"),

                event::Key::F1 => write!(f, "f1"),
                event::Key::F2 => write!(f, "f2"),
                event::Key::F3 => write!(f, "f3"),
                event::Key::F4 => write!(f, "f4"),
                event::Key::F5 => write!(f, "f5"),
                event::Key::F6 => write!(f, "f6"),
                event::Key::F7 => write!(f, "f7"),
                event::Key::F8 => write!(f, "f8"),
                event::Key::F9 => write!(f, "f9"),
                event::Key::F10 => write!(f, "f10"),
                event::Key::F11 => write!(f, "f11"),
                event::Key::F12 => write!(f, "f12"),

                _ => panic!("unknown key: {:?}", k),
            },
            _ => panic!("unknown event: {:?}", self.event),
        }
    }
}

impl Key {
    pub fn new<T: Into<event::Event>>(e: T) -> Self {
        Key { event: e.into() }
    }
}

config_parser_impl!(Key);

impl<'de> de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let err = Err(de::Error::custom(format!(
            "failed to parse key: unknown/invalid key {}",
            s
        )));

        let chars: Vec<char> = s.chars().collect();
        let key = {
            if chars.len() == 1 {
                // a single character
                Key::new(chars[0])
            } else if chars.len() == 3 && chars[1] == '-' {
                // M-<c> for alt-<c> and C-<c> for ctrl-<c>, with <c> denotes a single character
                match chars[0] {
                    'C' => Key::new(event::Event::CtrlChar(chars[2])),
                    'M' => Key::new(event::Event::AltChar(chars[2])),
                    _ => return err,
                }
            } else {
                let key = match s.as_str() {
                    "enter" => event::Key::Enter,
                    "tab" => event::Key::Tab,
                    "backspace" => event::Key::Backspace,
                    "esc" => event::Key::Esc,

                    "left" => event::Key::Left,
                    "right" => event::Key::Right,
                    "up" => event::Key::Up,
                    "down" => event::Key::Down,

                    "ins" => event::Key::Ins,
                    "del" => event::Key::Del,
                    "home" => event::Key::Home,
                    "end" => event::Key::End,
                    "page_up" => event::Key::PageUp,
                    "page_down" => event::Key::PageDown,

                    "f1" => event::Key::F1,
                    "f2" => event::Key::F2,
                    "f3" => event::Key::F3,
                    "f4" => event::Key::F4,
                    "f5" => event::Key::F5,
                    "f6" => event::Key::F6,
                    "f7" => event::Key::F7,
                    "f8" => event::Key::F8,
                    "f9" => event::Key::F9,
                    "f10" => event::Key::F10,
                    "f11" => event::Key::F11,
                    "f12" => event::Key::F12,

                    _ => return err,
                };

                Key::new(key)
            }
        };

        Ok(key)
    }
}

pub fn get_custom_keymap() -> &'static CustomKeyMap {
    &super::get_config().keymap.custom_keymap
}

pub fn get_edit_keymap() -> &'static EditKeyMap {
    &super::get_config().keymap.edit_keymap
}

pub fn get_global_keymap() -> &'static GlobalKeyMap {
    &super::get_config().keymap.global_keymap
}

pub fn get_story_view_keymap() -> &'static StoryViewKeyMap {
    &super::get_config().keymap.story_view_keymap
}

pub fn get_search_view_keymap() -> &'static SearchViewKeyMap {
    &super::get_config().keymap.search_view_keymap
}

pub fn get_comment_view_keymap() -> &'static CommentViewKeyMap {
    &super::get_config().keymap.comment_view_keymap
}

pub fn get_article_view_keymap() -> &'static ArticleViewKeyMap {
    &super::get_config().keymap.article_view_keymap
}
