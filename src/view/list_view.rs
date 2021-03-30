use crate::prelude::*;

pub type ScrollListView = ScrollView<LinearLayout>;

/// Construct a new OnEventView wrapping a ScrollListView
pub fn construct_scroll_list_event_view<T: ScrollableList>(view: T) -> OnEventView<T> {
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| {
            let id = s.get_focus_index();
            if id == 0 {
                None
            } else {
                s.set_focus_index(id - 1)
            }
        })
        .on_pre_event_inner('j', |s, _| {
            let id = s.get_focus_index();
            s.set_focus_index(id + 1)
        })
        .on_pre_event_inner('t', |s, _| s.set_focus_index(0))
        .on_pre_event_inner('b', |s, _| {
            let len = s.len();
            if len == 0 {
                None
            } else {
                s.set_focus_index(len - 1)
            }
        })
    // event handlers for parsing numbers
    // .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| match *e {
    //     Event::Char(c) => s.handle_digit(c),
    //     _ => None,
    // })
    // ignore up,down,pageUp,pageDown keys. Rely on main scrollView to handle those keys
    // .on_pre_event_inner(EventTrigger::from_fn(|_| true), |_, e| match *e {
    //     Event::Key(Key::Up)
    //     | Event::Key(Key::Down)
    //     | Event::Key(Key::PageUp)
    //     | Event::Key(Key::PageDown) => Some(EventResult::Ignored),
    //     _ => None,
    // })
}

/// ScrollableList is a trait that implements basic methods
/// to interact with a View that wraps a ScrollListView
pub trait ScrollableList {
    fn len(&self) -> usize;
    fn get_focus_index(&self) -> usize;
    fn set_focus_index(&mut self, id: usize) -> Option<EventResult>;
}

#[macro_export]
macro_rules! impl_scrollable_list {
    () => {
        fn len(&self) -> usize {
            self.get_inner().get_inner().len()
        }

        fn get_focus_index(&self) -> usize {
            self.get_inner().get_inner().get_focus_index()
        }

        fn set_focus_index(&mut self, id: usize) -> Option<EventResult> {
            let scroll_view = self.get_inner_mut();
            let linear_layout = scroll_view.get_inner_mut();
            match linear_layout.set_focus_index(id) {
                Ok(_) => {
                    scroll_view.scroll_to_important_area();
                    Some(EventResult::Consumed(None))
                }
                Err(_) => None,
            }
        }
    };
}

impl ScrollableList for StoryView {
    impl_scrollable_list!();
}

impl ScrollableList for CommentView {
    impl_scrollable_list!();
}
