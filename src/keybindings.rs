use cursive::event::{self, Event, EventTrigger};
use serde::{de, Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Keymap {
    pub global_keymap: GlobalKeyMap,
    pub story_view_keymap: StoryViewKeyMap,
    pub search_view_keymap: SearchViewKeyMap,
    pub comment_view_keymap: CommentViewKeyMap,
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

#[derive(Deserialize)]
pub struct StoryViewKeyMap {
    // stories navigation keymaps
    pub next_story: Key,
    pub prev_story: Key,
    pub goto_story: Key,

    // stories paging/filtering keymaps
    pub next_page: Key,
    pub prev_page: Key,
    pub toggle_sory_by: Key,
    pub filter_past_day: Key,
    pub filter_past_week: Key,
    pub filter_past_month: Key,
    pub filter_past_year: Key,

    // link opening keymaps
    pub open_article_in_browser: Key,
    pub open_story_in_browser: Key,

    pub goto_story_comment_view: Key,
}

#[derive(Deserialize)]
pub struct SearchViewKeyMap {
    // switch mode keymaps
    pub to_navigation_mode: Key,
    pub to_search_mode: Key,
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

pub struct Key {
    event_trigger: EventTrigger,
}

impl From<Key> for EventTrigger {
    fn from(k: Key) -> Self {
        k.event_trigger
    }
}

impl Key {
    pub fn new(event_trigger: EventTrigger) -> Self {
        Key { event_trigger }
    }

    pub fn from<T: Into<EventTrigger>>(e: T) -> Self {
        Key {
            event_trigger: e.into(),
        }
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
            Ok(Key::from(chars[0]))
        } else if chars.len() > 3 && chars[1] == '-' {
            // M-<c> for alt-<c> and C-<c> for ctrl-C
            match chars[1] {
                'C' => Ok(Key::from(Event::CtrlChar(chars[2]))),
                'M' => Ok(Key::from(Event::AltChar(chars[2]))),
                _ => err,
            }
        } else {
            match s.as_str() {
                "enter" => Ok(Key::from(Event::Key(event::Key::Enter))),
                "tab" => Ok(Key::from(Event::Key(event::Key::Tab))),
                "backspace" => Ok(Key::from(Event::Key(event::Key::Backspace))),
                "esc" => Ok(Key::from(Event::Key(event::Key::Esc))),

                "left" => Ok(Key::from(Event::Key(event::Key::Left))),
                "right" => Ok(Key::from(Event::Key(event::Key::Right))),
                "up" => Ok(Key::from(Event::Key(event::Key::Up))),
                "down" => Ok(Key::from(Event::Key(event::Key::Down))),

                "ins" => Ok(Key::from(Event::Key(event::Key::Ins))),
                "del" => Ok(Key::from(Event::Key(event::Key::Del))),
                "home" => Ok(Key::from(Event::Key(event::Key::Home))),
                "end" => Ok(Key::from(Event::Key(event::Key::End))),
                "page_up" => Ok(Key::from(Event::Key(event::Key::PageUp))),
                "page_down" => Ok(Key::from(Event::Key(event::Key::PageDown))),

                "f1" => Ok(Key::from(Event::Key(event::Key::F1))),
                "f2" => Ok(Key::from(Event::Key(event::Key::F2))),
                "f3" => Ok(Key::from(Event::Key(event::Key::F3))),
                "f4" => Ok(Key::from(Event::Key(event::Key::F4))),
                "f5" => Ok(Key::from(Event::Key(event::Key::F5))),
                "f6" => Ok(Key::from(Event::Key(event::Key::F6))),
                "f7" => Ok(Key::from(Event::Key(event::Key::F7))),
                "f8" => Ok(Key::from(Event::Key(event::Key::F8))),
                "f9" => Ok(Key::from(Event::Key(event::Key::F9))),
                "f10" => Ok(Key::from(Event::Key(event::Key::F10))),
                "f11" => Ok(Key::from(Event::Key(event::Key::F11))),
                "f12" => Ok(Key::from(Event::Key(event::Key::F12))),

                _ => err,
            }
        }
    }
}
