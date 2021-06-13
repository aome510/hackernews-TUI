use core::fmt;

use config_parser2::*;
use cursive::event::{self, Event, EventTrigger};
use serde::{de, Deserialize, Deserializer};

use crate::hn_client;

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct KeyMap {
    #[serde(default)]
    pub custom_keymap: CustomKeyMap,
    pub global_keymap: GlobalKeyMap,
    pub story_view_keymap: StoryViewKeyMap,
    pub search_view_keymap: SearchViewKeyMap,
    pub comment_view_keymap: CommentViewKeyMap,
    pub article_view_keymap: ArticleViewKeyMap,
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap {
            custom_keymap: CustomKeyMap::default(),
            global_keymap: GlobalKeyMap::default(),
            story_view_keymap: StoryViewKeyMap::default(),
            search_view_keymap: SearchViewKeyMap::default(),
            comment_view_keymap: CommentViewKeyMap::default(),
            article_view_keymap: ArticleViewKeyMap::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomViewNavigation {
    pub key: Key,
    pub tag: String,
    pub by_date: bool,
    pub numeric_filters: hn_client::StoryNumericFilters,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomKeyMap {
    pub custom_view_navigation: Vec<CustomViewNavigation>,
}

config_parser_impl!(CustomKeyMap);

impl Default for CustomKeyMap {
    fn default() -> Self {
        CustomKeyMap {
            custom_view_navigation: vec![],
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
            quit: Key::new(Event::CtrlChar('q')),
            close_dialog: Key::new(event::Key::Esc),

            goto_previous_view: Key::new(Event::CtrlChar('p')),
            goto_front_page_view: Key::new(Event::CtrlChar('f')),
            goto_search_view: Key::new(Event::CtrlChar('s')),
            goto_all_stories_view: Key::new(Event::CtrlChar('z')),
            goto_ask_hn_view: Key::new(Event::CtrlChar('x')),
            goto_show_hn_view: Key::new(Event::CtrlChar('c')),
            goto_jobs_view: Key::new(Event::CtrlChar('v')),
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
    pub reload_comment_view: Key,
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
            reload_comment_view: Key::new('r'),
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
    event: Event,
}

impl From<Key> for EventTrigger {
    fn from(k: Key) -> Self {
        k.event.into()
    }
}

impl From<Key> for Event {
    fn from(k: Key) -> Self {
        k.event
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.event {
            Event::Char(c) => write!(f, "{}", c),
            Event::CtrlChar(c) => write!(f, "C-{}", c),
            Event::AltChar(c) => write!(f, "M-{}", c),
            Event::Key(k) => match k {
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
    pub fn new<T: Into<Event>>(e: T) -> Self {
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
            "failed to parse key: unknown key {}",
            s
        )));

        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 1 {
            // a single character
            Ok(Key::new(chars[0]))
        } else if chars.len() > 2 && chars[1] == '-' {
            // M-<c> for alt-<c> and C-<c> for ctrl-C
            match chars[0] {
                'C' => Ok(Key::new(Event::CtrlChar(chars[2]))),
                'M' => Ok(Key::new(Event::AltChar(chars[2]))),
                _ => err,
            }
        } else {
            match s.as_str() {
                "enter" => Ok(Key::new(event::Key::Enter)),
                "tab" => Ok(Key::new(event::Key::Tab)),
                "backspace" => Ok(Key::new(event::Key::Backspace)),
                "esc" => Ok(Key::new(event::Key::Esc)),

                "left" => Ok(Key::new(event::Key::Left)),
                "right" => Ok(Key::new(event::Key::Right)),
                "up" => Ok(Key::new(event::Key::Up)),
                "down" => Ok(Key::new(event::Key::Down)),

                "ins" => Ok(Key::new(event::Key::Ins)),
                "del" => Ok(Key::new(event::Key::Del)),
                "home" => Ok(Key::new(event::Key::Home)),
                "end" => Ok(Key::new(event::Key::End)),
                "page_up" => Ok(Key::new(event::Key::PageUp)),
                "page_down" => Ok(Key::new(event::Key::PageDown)),

                "f1" => Ok(Key::new(event::Key::F1)),
                "f2" => Ok(Key::new(event::Key::F2)),
                "f3" => Ok(Key::new(event::Key::F3)),
                "f4" => Ok(Key::new(event::Key::F4)),
                "f5" => Ok(Key::new(event::Key::F5)),
                "f6" => Ok(Key::new(event::Key::F6)),
                "f7" => Ok(Key::new(event::Key::F7)),
                "f8" => Ok(Key::new(event::Key::F8)),
                "f9" => Ok(Key::new(event::Key::F9)),
                "f10" => Ok(Key::new(event::Key::F10)),
                "f11" => Ok(Key::new(event::Key::F11)),
                "f12" => Ok(Key::new(event::Key::F12)),

                _ => err,
            }
        }
    }
}
