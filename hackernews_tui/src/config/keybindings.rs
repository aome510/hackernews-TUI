use crate::client;
use config_parser2::*;
use cursive::event;
use serde::{de, Deserialize, Deserializer};

#[derive(Default, Debug, Clone, Deserialize, ConfigParse)]
pub struct KeyMap {
    pub edit_keymap: EditKeyMap,
    pub scroll_keymap: ScrollKeyMap,
    pub global_keymap: GlobalKeyMap,
    pub story_view_keymap: StoryViewKeyMap,
    pub search_view_keymap: SearchViewKeyMap,
    pub comment_view_keymap: CommentViewKeyMap,
    pub article_view_keymap: ArticleViewKeyMap,
    pub link_dialog_keymap: LinkDialogKeyMap,

    pub custom_keymaps: Vec<CustomKeyMap>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomKeyMap {
    pub key: Keys,
    pub tag: String,
    pub by_date: bool,
    pub numeric_filters: client::StoryNumericFilters,
}

config_parser_impl!(CustomKeyMap);

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct EditKeyMap {
    pub move_cursor_left: Keys,
    pub move_cursor_right: Keys,
    pub move_cursor_to_begin: Keys,
    pub move_cursor_to_end: Keys,
    pub backward_delete_char: Keys,
}

impl Default for EditKeyMap {
    fn default() -> Self {
        EditKeyMap {
            move_cursor_left: Keys::new(vec![event::Key::Left.into(), event::Event::CtrlChar('b')]),
            move_cursor_right: Keys::new(vec![
                event::Key::Right.into(),
                event::Event::CtrlChar('f'),
            ]),
            move_cursor_to_begin: Keys::new(vec![
                event::Key::Home.into(),
                event::Event::CtrlChar('a'),
            ]),
            move_cursor_to_end: Keys::new(vec![
                event::Key::End.into(),
                event::Event::CtrlChar('e'),
            ]),
            backward_delete_char: Keys::new(vec![event::Key::Backspace.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct ScrollKeyMap {
    pub up: Keys,
    pub down: Keys,
    pub page_up: Keys,
    pub page_down: Keys,
    pub top: Keys,
    pub bottom: Keys,
}

impl Default for ScrollKeyMap {
    fn default() -> Self {
        ScrollKeyMap {
            up: Keys::new(vec!['k'.into(), event::Key::Up.into()]),
            down: Keys::new(vec!['j'.into(), event::Key::Down.into()]),
            page_up: Keys::new(vec!['u'.into(), event::Key::PageUp.into()]),
            page_down: Keys::new(vec!['d'.into(), event::Key::PageDown.into()]),
            top: Keys::new(vec!['g'.into(), event::Key::Home.into()]),
            bottom: Keys::new(vec!['G'.into(), event::Key::End.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct GlobalKeyMap {
    pub open_help_dialog: Keys,
    pub quit: Keys,
    pub close_dialog: Keys,

    // view navigation keymaps
    pub goto_previous_view: Keys,
    pub goto_front_page_view: Keys,
    pub goto_search_view: Keys,
    pub goto_all_stories_view: Keys,
    pub goto_ask_hn_view: Keys,
    pub goto_show_hn_view: Keys,
    pub goto_jobs_view: Keys,
}

impl Default for GlobalKeyMap {
    fn default() -> Self {
        GlobalKeyMap {
            open_help_dialog: Keys::new(vec!['?'.into()]),
            quit: Keys::new(vec!['q'.into(), event::Event::CtrlChar('c')]),
            close_dialog: Keys::new(vec![event::Key::Esc.into()]),

            goto_previous_view: Keys::new(vec![
                event::Key::Backspace.into(),
                event::Event::CtrlChar('p'),
            ]),

            goto_search_view: Keys::new(vec!['/'.into(), event::Event::CtrlChar('s')]),

            goto_front_page_view: Keys::new(vec![event::Key::F1.into()]),
            goto_all_stories_view: Keys::new(vec![event::Key::F2.into()]),
            goto_ask_hn_view: Keys::new(vec![event::Key::F3.into()]),
            goto_show_hn_view: Keys::new(vec![event::Key::F4.into()]),
            goto_jobs_view: Keys::new(vec![event::Key::F5.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct StoryViewKeyMap {
    // story tags navigation keymaps
    pub next_story_tag: Keys,
    pub prev_story_tag: Keys,

    // stories navigation keymaps
    pub next_story: Keys,
    pub prev_story: Keys,
    pub goto_story: Keys,

    // stories paging/filtering keymaps
    pub next_page: Keys,
    pub prev_page: Keys,
    pub toggle_sort_by_date: Keys,

    // link keymaps
    pub open_article_in_browser: Keys,
    pub open_article_in_article_view: Keys,
    pub open_story_in_browser: Keys,

    pub goto_story_comment_view: Keys,
}

impl Default for StoryViewKeyMap {
    fn default() -> Self {
        StoryViewKeyMap {
            next_story_tag: Keys::new(vec!['l'.into()]),
            prev_story_tag: Keys::new(vec!['h'.into()]),
            next_story: Keys::new(vec!['j'.into()]),
            prev_story: Keys::new(vec!['k'.into()]),
            goto_story: Keys::new(vec!['g'.into()]),

            next_page: Keys::new(vec!['n'.into()]),
            prev_page: Keys::new(vec!['p'.into()]),
            toggle_sort_by_date: Keys::new(vec!['d'.into()]),

            open_article_in_browser: Keys::new(vec!['o'.into()]),
            open_article_in_article_view: Keys::new(vec!['O'.into()]),
            open_story_in_browser: Keys::new(vec!['s'.into()]),

            goto_story_comment_view: Keys::new(vec![event::Key::Enter.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct SearchViewKeyMap {
    // switch mode keymaps
    pub to_navigation_mode: Keys,
    pub to_search_mode: Keys,
}

impl Default for SearchViewKeyMap {
    fn default() -> Self {
        SearchViewKeyMap {
            to_navigation_mode: Keys::new(vec![event::Key::Esc.into()]),
            to_search_mode: Keys::new(vec!['i'.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct CommentViewKeyMap {
    // comments navigation keymaps
    pub next_comment: Keys,
    pub prev_comment: Keys,
    pub next_top_level_comment: Keys,
    pub prev_top_level_comment: Keys,
    pub next_leq_level_comment: Keys,
    pub prev_leq_level_comment: Keys,
    pub parent_comment: Keys,

    // link keymaps
    pub open_story_in_browser: Keys,
    pub open_comment_in_browser: Keys,
    pub open_article_in_browser: Keys,
    pub open_article_in_article_view: Keys,
    pub open_link_in_browser: Keys,
    pub open_link_in_article_view: Keys,

    pub toggle_collapse_comment: Keys,
}

impl Default for CommentViewKeyMap {
    fn default() -> Self {
        CommentViewKeyMap {
            next_comment: Keys::new(vec!['j'.into()]),
            prev_comment: Keys::new(vec!['k'.into()]),
            next_top_level_comment: Keys::new(vec!['n'.into()]),
            prev_top_level_comment: Keys::new(vec!['p'.into()]),
            next_leq_level_comment: Keys::new(vec!['l'.into()]),
            prev_leq_level_comment: Keys::new(vec!['h'.into()]),
            parent_comment: Keys::new(vec!['u'.into()]),

            open_comment_in_browser: Keys::new(vec!['c'.into()]),
            open_story_in_browser: Keys::new(vec!['s'.into()]),
            open_article_in_browser: Keys::new(vec!['a'.into()]),
            open_article_in_article_view: Keys::new(vec!['A'.into()]),
            open_link_in_browser: Keys::new(vec!['o'.into()]),
            open_link_in_article_view: Keys::new(vec!['O'.into()]),

            toggle_collapse_comment: Keys::new(vec![event::Key::Tab.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct ArticleViewKeyMap {
    pub open_link_dialog: Keys,
    pub open_article_in_browser: Keys,
    pub open_link_in_browser: Keys,
    pub open_link_in_article_view: Keys,
}

impl Default for ArticleViewKeyMap {
    fn default() -> Self {
        ArticleViewKeyMap {
            open_link_dialog: Keys::new(vec!['l'.into()]),
            open_article_in_browser: Keys::new(vec!['a'.into()]),
            open_link_in_browser: Keys::new(vec!['o'.into()]),
            open_link_in_article_view: Keys::new(vec!['O'.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct LinkDialogKeyMap {
    pub next: Keys,
    pub prev: Keys,
    pub open_link_in_browser: Keys,
    pub open_link_in_article_view: Keys,
}

impl Default for LinkDialogKeyMap {
    fn default() -> Self {
        LinkDialogKeyMap {
            next: Keys::new(vec!['j'.into(), event::Key::Down.into()]),
            prev: Keys::new(vec!['k'.into(), event::Key::Up.into()]),
            open_link_in_browser: Keys::new(vec!['o'.into(), event::Key::Enter.into()]),
            open_link_in_article_view: Keys::new(vec!['O'.into()]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keys {
    events: Vec<event::Event>,
}

impl From<Keys> for event::EventTrigger {
    fn from(k: Keys) -> Self {
        event::EventTrigger::from_fn(move |e| k.has_event(e))
    }
}

impl std::fmt::Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_event(e: &event::Event, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match e {
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
                _ => panic!("unknown event: {:?}", e),
            }
        }

        if self.events.is_empty() {
            return Ok(());
        }

        if self.events.len() == 1 {
            fmt_event(&self.events[0], f)
        } else {
            write!(f, "[")?;
            fmt_event(&self.events[0], f)?;
            for e in &self.events[1..] {
                write!(f, ", ")?;
                fmt_event(e, f)?;
            }
            write!(f, "]")?;
            Ok(())
        }
    }
}

impl Keys {
    pub fn new(events: Vec<event::Event>) -> Self {
        Keys { events }
    }

    pub fn has_event(&self, e: &event::Event) -> bool {
        self.events.iter().any(|x| *x == *e)
    }
}

config_parser_impl!(Keys);

impl<'de> de::Deserialize<'de> for Keys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        /// an enum representing either
        /// - a single key string \[1\]
        /// - an array of multiple key strings
        ///
        /// \[1\]: "key string" denotes the string representation of a key
        enum StringOrVec {
            String(String),
            Vec(Vec<String>),
        }

        /// a helper function that converts a key string into `cursive::event::Event`
        fn from_key_string_to_event(ks: String) -> Result<event::Event> {
            let chars: Vec<char> = ks.chars().collect();

            let event = if chars.len() == 1 {
                // a single character
                event::Event::Char(chars[0])
            } else if chars.len() == 3 && chars[1] == '-' {
                // M-<c> for alt-<c> and C-<c> for ctrl-<c>, with <c> denotes a single character
                match chars[0] {
                    'C' => event::Event::CtrlChar(chars[2]),
                    'M' => event::Event::AltChar(chars[2]),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "failed to parse key: unknown/invalid key {}",
                            ks
                        ))
                    }
                }
            } else {
                let key = match ks.as_str() {
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

                    _ => {
                        return Err(anyhow::anyhow!(
                            "failed to parse key: unknown/invalid key {}",
                            ks
                        ))
                    }
                };

                event::Event::Key(key)
            };

            Ok(event)
        }

        let key_strings = match StringOrVec::deserialize(deserializer)? {
            StringOrVec::String(v) => vec![v],
            StringOrVec::Vec(v) => v,
        };

        let events = key_strings
            .into_iter()
            .map(from_key_string_to_event)
            .collect::<Result<Vec<_>>>()
            .map_err(serde::de::Error::custom)?;

        Ok(Keys::new(events))
    }
}

pub fn get_edit_keymap() -> &'static EditKeyMap {
    &super::get_config().keymap.edit_keymap
}

pub fn get_scroll_keymap() -> &'static ScrollKeyMap {
    &super::get_config().keymap.scroll_keymap
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

pub fn get_link_dialog_keymap() -> &'static LinkDialogKeyMap {
    &super::get_config().keymap.link_dialog_keymap
}
