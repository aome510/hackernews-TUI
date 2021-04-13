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
        .on_pre_event_inner(Key::Up, |s, _| {
            s.get_scroller_mut().scroll_up(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(Key::Down, |s, _| {
            s.get_scroller_mut().scroll_down(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(Key::PageUp, |s, _| {
            s.get_scroller_mut().scroll_up(5);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(Key::PageDown, |s, _| {
            s.get_scroller_mut().scroll_down(5);
            Some(EventResult::Consumed(None))
        })
}

/// ScrollableList is a trait that implements basic methods
/// to interact with a View that wraps a ScrollListView
pub trait ScrollableList {
    fn len(&self) -> usize;
    fn get_focus_index(&self) -> usize;
    fn set_focus_index(&mut self, id: usize) -> Option<EventResult>;
    fn get_scroller(&self) -> &scroll::Core;
    fn get_scroller_mut(&mut self) -> &mut scroll::Core;
    // Move the scroller to the focused area and adjust the scroller
    // accordingly depending on the scrolling direction.
    // (direction = true) stands for going down while false stands for going up
    fn scroll(&mut self, direction: bool);
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

        fn scroll(&mut self, direction: bool) {
            let important_area = self
                .get_inner()
                .get_inner()
                .important_area(self.get_scroller().last_outer_size());
            let scroller = self.get_scroller_mut();

            // the below implementation is based on scroll_to_rect function
            // defined in Cursive::view::scroll::core.rs
            let top_left = (important_area.bottom_right() + (1, 1))
                .saturating_sub(scroller.last_available_size());
            let bottom_right = important_area.top_left();
            let offset = scroller.content_viewport().top_left();

            // debug!(
            //     "top_left: {:#?},\n bottom_right: {:#?},\n offset: {:#?}",
            //     top_left, bottom_right, offset
            // );

            // focused view has larger size than the what is provided to the scroller
            if (bottom_right.y < top_left.y) {
                scroller.set_offset(bottom_right);
            } else {
                if (direction) {
                    // focused is moved down
                    if (offset.y < top_left.y) {
                        scroller.set_offset(bottom_right);
                    }
                } else {
                    // focus is moved up
                    if (offset.y > bottom_right.y) {
                        scroller.set_offset(top_left);
                    }
                }
            }
        }

        fn set_focus_index(&mut self, id: usize) -> Option<EventResult> {
            let current_id = self.get_focus_index();
            let direction = if (current_id <= id) { true } else { false };
            let linear_layout = self.get_inner_mut().get_inner_mut();

            match linear_layout.set_focus_index(id) {
                Ok(_) => {
                    self.scroll(direction);
                    Some(EventResult::Consumed(None))
                }
                Err(_) => None,
            }
        }

        fn get_scroller_mut(&mut self) -> &mut scroll::Core {
            self.get_inner_mut().get_scroller_mut()
        }

        fn get_scroller(&self) -> &scroll::Core {
            self.get_inner().get_scroller()
        }
    };
}

impl ScrollableList for StoryView {
    impl_scrollable_list!();
}

impl ScrollableList for CommentView {
    impl_scrollable_list!();
}
