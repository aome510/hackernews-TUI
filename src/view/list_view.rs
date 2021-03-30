use crate::prelude::*;
use scroll::*;

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
        .on_pre_event_inner(Key::Up, |s, _| {
            s.get_scoller_mut().scroll_up(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(Key::Down, |s, _| {
            s.get_scoller_mut().scroll_down(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(Key::PageUp, |s, _| {
            s.get_scoller_mut().scroll_up(5);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(Key::PageDown, |s, _| {
            s.get_scoller_mut().scroll_down(5);
            Some(EventResult::Consumed(None))
        })
}

/// ScrollableList is a trait that implements basic methods
/// to interact with a View that wraps a ScrollListView
pub trait ScrollableList {
    fn len(&self) -> usize;
    fn get_focus_index(&self) -> usize;
    fn set_focus_index(&mut self, id: usize) -> Option<EventResult>;
    fn get_scoller_mut(&mut self) -> &mut scroll::Core;
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

        fn get_scoller_mut(&mut self) -> &mut scroll::Core {
            self.get_inner_mut().get_scroller_mut()
        }
    };
}

impl ScrollableList for StoryView {
    impl_scrollable_list!();
}

impl ScrollableList for CommentView {
    impl_scrollable_list!();
}
