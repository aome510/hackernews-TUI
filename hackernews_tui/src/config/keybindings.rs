use crate::client;
use config_parser2::*;
use cursive::event;
use serde::{de, Deserialize, Deserializer};

#[derive(Default, Debug, Clone, Deserialize, ConfigParse)]
pub struct KeyMap {
    pub edit_keymap: EditKeyMap,
    pub global_keymap: GlobalKeyMap,
    pub story_view_keymap: StoryViewKeyMap,
    pub search_view_keymap: SearchViewKeyMap,
    pub comment_view_keymap: CommentViewKeyMap,
    pub article_view_keymap: ArticleViewKeyMap,

    pub custom_keymaps: Vec<CustomKeyMap>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomKeyMap {
    pub key: Key,
    pub tag: String,
    pub by_date: bool,
    pub numeric_filters: client::StoryNumericFilters,
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
            move_cursor_left: Key::new(vec![event::Key::Left.into()]),
            move_cursor_right: Key::new(vec![event::Key::Right.into()]),
            move_cursor_to_begin: Key::new(vec![event::Key::Home.into()]),
            move_cursor_to_end: Key::new(vec![event::Key::End.into()]),
            backward_delete_char: Key::new(vec![event::Key::Backspace.into()]),
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
            open_help_dialog: Key::new(vec!['?'.into()]),
            quit: Key::new(vec![event::Event::CtrlChar('q')]),
            close_dialog: Key::new(vec![event::Key::Esc.into()]),

            goto_previous_view: Key::new(vec![event::Event::CtrlChar('p')]),
            goto_search_view: Key::new(vec![event::Event::CtrlChar('s')]),
            goto_front_page_view: Key::new(vec![event::Key::F1.into()]),
            goto_all_stories_view: Key::new(vec![event::Key::F2.into()]),
            goto_ask_hn_view: Key::new(vec![event::Key::F3.into()]),
            goto_show_hn_view: Key::new(vec![event::Key::F4.into()]),
            goto_jobs_view: Key::new(vec![event::Key::F5.into()]),
        }
    }
}

#[derive(Debug, Clone, Deserialize, ConfigParse)]
pub struct StoryViewKeyMap {
    // story tags navigation keymaps
    pub next_story_tag: Key,
    pub prev_story_tag: Key,

    // stories navigation keymaps
    pub next_story: Key,
    pub prev_story: Key,
    pub goto_story: Key,

    // stories paging/filtering keymaps
    pub next_page: Key,
    pub prev_page: Key,
    pub toggle_sort_by_date: Key,

    // link opening keymaps
    pub open_article_in_browser: Key,
    pub open_article_in_article_view: Key,
    pub open_story_in_browser: Key,

    pub goto_story_comment_view: Key,
}

impl Default for StoryViewKeyMap {
    fn default() -> Self {
        StoryViewKeyMap {
            next_story_tag: Key::new(vec!['l'.into()]),
            prev_story_tag: Key::new(vec!['h'.into()]),
            next_story: Key::new(vec!['j'.into()]),
            prev_story: Key::new(vec!['k'.into()]),
            goto_story: Key::new(vec!['g'.into()]),

            next_page: Key::new(vec!['n'.into()]),
            prev_page: Key::new(vec!['p'.into()]),
            toggle_sort_by_date: Key::new(vec!['d'.into()]),

            open_article_in_browser: Key::new(vec!['o'.into()]),
            open_article_in_article_view: Key::new(vec!['O'.into()]),
            open_story_in_browser: Key::new(vec!['s'.into()]),

            goto_story_comment_view: Key::new(vec![event::Key::Enter.into()]),
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
            to_navigation_mode: Key::new(vec![event::Key::Esc.into()]),
            to_search_mode: Key::new(vec!['i'.into()]),
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
            next_comment: Key::new(vec!['j'.into()]),
            prev_comment: Key::new(vec!['k'.into()]),
            next_top_level_comment: Key::new(vec!['n'.into()]),
            prev_top_level_comment: Key::new(vec!['p'.into()]),
            next_leq_level_comment: Key::new(vec!['l'.into()]),
            prev_leq_level_comment: Key::new(vec!['h'.into()]),
            parent_comment: Key::new(vec!['u'.into()]),

            open_comment_in_browser: Key::new(vec!['c'.into()]),
            open_link_in_browser: Key::new(vec!['f'.into()]),
            open_link_in_article_view: Key::new(vec!['F'.into()]),

            up: Key::new(vec![event::Key::Up.into()]),
            down: Key::new(vec![event::Key::Down.into()]),
            page_up: Key::new(vec![event::Key::PageUp.into()]),
            page_down: Key::new(vec![event::Key::PageDown.into()]),

            toggle_collapse_comment: Key::new(vec![event::Key::Tab.into()]),
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
}

impl Default for ArticleViewKeyMap {
    fn default() -> Self {
        ArticleViewKeyMap {
            down: Key::new(vec!['j'.into()]),
            up: Key::new(vec!['k'.into()]),
            page_down: Key::new(vec!['d'.into()]),
            page_up: Key::new(vec!['u'.into()]),
            top: Key::new(vec!['g'.into()]),
            bottom: Key::new(vec!['G'.into()]),

            open_link_dialog: Key::new(vec!['l'.into()]),
            link_dialog_focus_next: Key::new(vec!['j'.into()]),
            link_dialog_focus_prev: Key::new(vec!['k'.into()]),

            open_article_in_browser: Key::new(vec!['o'.into()]),
            open_link_in_browser: Key::new(vec!['f'.into()]),
            open_link_in_article_view: Key::new(vec!['F'.into()]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Key {
    events: Vec<event::Event>,
}

impl From<Key> for event::EventTrigger {
    fn from(k: Key) -> Self {
        event::EventTrigger::from_fn(move |e| k.has_event(e))
    }
}

impl std::fmt::Display for Key {
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

        if self.events.len() == 1 {
            fmt_event(&self.events[0], f)
        } else {
            write!(f, "[")?;
            for e in &self.events {
                fmt_event(e, f)?;
                write!(f, ", ")?;
            }
            write!(f, "]")?;
            Ok(())
        }
    }
}

impl Key {
    pub fn new(events: Vec<event::Event>) -> Self {
        Key { events }
    }

    pub fn has_event(&self, e: &event::Event) -> bool {
        self.events.iter().any(|x| *x == *e)
    }
}

config_parser_impl!(Key);

impl<'de> de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
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
                        return Err(anyhow::anyhow!(format!(
                            "failed to parse key: unknown/invalid key {}",
                            ks
                        )))
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
                        return Err(anyhow::anyhow!(format!(
                            "failed to parse key: unknown/invalid key {}",
                            ks
                        )))
                    }
                };

                event::Event::Key(key)
            };

            Ok(event)
        }

        let v = match StringOrVec::deserialize(deserializer)? {
            StringOrVec::String(v) => vec![v],
            StringOrVec::Vec(v) => v,
        };

        let events = v
            .into_iter()
            .map(from_key_string_to_event)
            .collect::<Result<Vec<_>>>()
            .map_err(serde::de::Error::custom)?;

        Ok(Key::new(events))
    }
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
