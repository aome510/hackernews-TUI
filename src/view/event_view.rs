use super::comment_view::CommentView;
use super::story_view::StoryView;
use crate::prelude::*;

/// Construct a new Event view from a LinearLayout by adding
/// event handlers for a key pressed
pub fn construct_event_view<T: ListEventView>(view: T) -> OnEventView<T> {
    // add "j" and "k" for moving down and up the story list
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| s.focus_up())
        .on_pre_event_inner('j', |s, _| s.focus_down())
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| match *e {
            Event::Char(c) => s.handle_digit(c),
            _ => None,
        })
}

/// ListEventView is a trait that implements method interfaces
/// to interact with a List View
pub trait ListEventView {
    fn focus_up(&mut self) -> Option<EventResult> {
        None
    }
    fn focus_down(&mut self) -> Option<EventResult> {
        None
    }
    fn handle_digit(&mut self, _: char) -> Option<EventResult> {
        None
    }
}

impl ListEventView for LinearLayout {
    fn focus_up(&mut self) -> Option<EventResult> {
        let id = self.get_focus_index();
        if id > 0 {
            match self.set_focus_index(id - 1) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        } else {
            Some(EventResult::Ignored)
        }
    }

    fn focus_down(&mut self) -> Option<EventResult> {
        let id = self.get_focus_index();
        if id + 1 < self.len() {
            match self.set_focus_index(id + 1) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        } else {
            Some(EventResult::Ignored)
        }
    }
}

#[macro_export]
macro_rules! raw_command {
    () => {
        pub fn add_raw_command_char(&mut self, c: char) {
            self.raw_command.push(c);
        }

        pub fn clear_raw_command(&mut self) {
            self.raw_command.clear();
        }

        pub fn get_raw_command_as_number(&self) -> Result<usize> {
            Ok(self.raw_command.parse::<usize>()?)
        }
    };
}

#[macro_export]
macro_rules! list_event_view_wrapper {
    () => {
        fn focus_up(&mut self) -> Option<EventResult> {
            self.get_inner_mut().focus_up()
        }
        fn focus_down(&mut self) -> Option<EventResult> {
            self.get_inner_mut().focus_down()
        }
    };
}

impl ListEventView for CommentView {
    crate::list_event_view_wrapper!();
    fn handle_digit(&mut self, c: char) -> Option<EventResult> {
        if '0' <= c && c <= '9' {
            self.add_raw_command_char(c);
            Some(EventResult::Consumed(None))
        } else {
            if c != 'f' {
                self.clear_raw_command();
            }
            None
        }
    }
}

impl ListEventView for StoryView {
    crate::list_event_view_wrapper!();
    fn handle_digit(&mut self, c: char) -> Option<EventResult> {
        if '0' <= c && c <= '9' {
            self.add_raw_command_char(c);
            Some(EventResult::Consumed(None))
        } else {
            if c != 'g' {
                self.clear_raw_command();
            }
            None
        }
    }
}
