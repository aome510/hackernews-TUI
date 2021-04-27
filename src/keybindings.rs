use cursive::event::{self, Event, EventTrigger};
use serde::{de, Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Keymap {
    pub global_keymap: GlobalKeyMap,
    pub story_view_keymap: StoryViewKeyMap,
    pub search_view_keymap: SearchViewKeyMap,
    pub comment_view_keymap: CommentViewKeyMap,
}

impl Default for Keymap {
    fn default() -> Self {
        Keymap {
            global_keymap: GlobalKeyMap::default(),
            story_view_keymap: StoryViewKeyMap::default(),
            search_view_keymap: SearchViewKeyMap::default(),
            comment_view_keymap: CommentViewKeyMap::default(),
        }
    }
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct StoryViewKeyMap {
    // stories navigation keymaps
    pub next_story: Key,
    pub prev_story: Key,
    pub goto_story: Key,

    // stories paging/filtering keymaps
    pub next_page: Key,
    pub prev_page: Key,
    pub toggle_sort_by: Key,
    pub filter_past_day: Key,
    pub filter_past_week: Key,
    pub filter_past_month: Key,
    pub filter_past_year: Key,

    // link opening keymaps
    pub open_article_in_browser: Key,
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
            filter_past_day: Key::new('q'),
            filter_past_week: Key::new('w'),
            filter_past_month: Key::new('e'),
            filter_past_year: Key::new('r'),

            open_article_in_browser: Key::new('o'),
            open_story_in_browser: Key::new('s'),

            goto_story_comment_view: Key::new(event::Key::Enter),
        }
    }
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct CommentViewKeyMap {
    // comments navigation keymaps
    pub next_comment: Key,
    pub prev_comment: Key,
    pub next_top_level_comment: Key,
    pub prev_top_level_comment: Key,
    pub next_leq_level_comment: Key,
    pub prev_leq_level_comment: Key,

    // link opening keymaps
    pub open_comment_in_browser: Key,
    pub open_link_in_browser: Key,

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

            open_comment_in_browser: Key::new('c'),
            open_link_in_browser: Key::new('f'),

            reload_comment_view: Key::new('r'),
        }
    }
}

#[derive(Clone)]
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

impl Key {
    pub fn new<T: Into<Event>>(e: T) -> Self {
        Key { event: e.into() }
    }
}

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
