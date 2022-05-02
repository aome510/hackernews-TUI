use crate::prelude::*;

/// A trait defines methods to interact with the inner `LinearLayout` list
pub trait ListViewContainer {
    fn get_inner_list(&self) -> &LinearLayout;
    fn get_inner_list_mut(&mut self) -> &mut LinearLayout;

    // default to be no-op
    fn on_set_focus_index(&mut self, _old_id: usize, _new_id: usize) {}

    fn len(&self) -> usize {
        self.get_inner_list().len()
    }

    fn get_focus_index(&self) -> usize {
        self.get_inner_list().get_focus_index()
    }

    fn set_focus_index(&mut self, id: usize) -> Option<EventResult> {
        let old_id = self.get_focus_index();

        match self.get_inner_list_mut().set_focus_index(id) {
            Ok(_) => {
                self.on_set_focus_index(old_id, id);
                Some(EventResult::Consumed(None))
            }
            Err(_) => None,
        }
    }

    fn get_item(&self, id: usize) -> Option<&(dyn View + 'static)> {
        self.get_inner_list().get_child(id)
    }

    fn get_item_mut(&mut self, id: usize) -> Option<&mut (dyn View + 'static)> {
        self.get_inner_list_mut().get_child_mut(id)
    }

    fn add_item<V: IntoBoxedView + 'static>(&mut self, view: V) {
        self.get_inner_list_mut().add_child(view);
    }
}

/// A trait defines a `ScrollView` container view.
/// It defines methods to get the inner scroller view.
pub trait ScrollViewContainer {
    type ScrollInner: View;
    fn get_inner_scroller_view(&self) -> &ScrollView<Self::ScrollInner>;
    fn get_inner_scroller_view_mut(&mut self) -> &mut ScrollView<Self::ScrollInner>;
}

/// A trait defines the `on_scroll_events` method that adds
/// callbacks to handle scrolling events for the `self` View.
pub trait OnScrollEventView {
    /// adds callbacks handling scrolling events
    fn on_scroll_events(self) -> Self;
}

impl<T> OnScrollEventView for OnEventView<T>
where
    T: ScrollViewContainer,
{
    fn on_scroll_events(self) -> Self {
        let scroll_keymap = config::get_scroll_keymap().clone();

        // Scrolling events are handled using the `on_event_inner` callbacks,
        // which is triggered when the event is not handled by neither the children view
        // nor the `self` view's `pre_on_event_inner` callbacks.
        //
        // The `on_scroll_events` is often placed at the end of an `OnEventView` construction
        // after specifying all `on_pre_event` callbacks.
        self.on_event_inner(scroll_keymap.up, |s, _| {
            s.get_inner_scroller_view_mut()
                .get_scroller_mut()
                .scroll_up(3);
            Some(EventResult::Consumed(None))
        })
        .on_event_inner(scroll_keymap.down, |s, _| {
            s.get_inner_scroller_view_mut()
                .get_scroller_mut()
                .scroll_down(3);
            Some(EventResult::Consumed(None))
        })
        .on_event_inner(scroll_keymap.page_up, |s, _| {
            let height = s
                .get_inner_scroller_view()
                .get_scroller()
                .last_available_size()
                .y;
            s.get_inner_scroller_view_mut()
                .get_scroller_mut()
                .scroll_up(height / 2);
            Some(EventResult::Consumed(None))
        })
        .on_event_inner(scroll_keymap.page_down, |s, _| {
            let height = s
                .get_inner_scroller_view()
                .get_scroller()
                .last_available_size()
                .y;
            s.get_inner_scroller_view_mut()
                .get_scroller_mut()
                .scroll_down(height / 2);
            Some(EventResult::Consumed(None))
        })
        .on_event_inner(scroll_keymap.top, |s, _| {
            s.get_inner_scroller_view_mut().scroll_to_top();
            Some(EventResult::Consumed(None))
        })
        .on_event_inner(scroll_keymap.bottom, |s, _| {
            s.get_inner_scroller_view_mut().scroll_to_bottom();
            Some(EventResult::Consumed(None))
        })
    }
}

/// A trait defines the `scroll` method that scrolls the `self` view to
/// the inner view's important area.
///
/// This trait should be only implemented for a view that satisfies the `ScrollViewContainer` trait
/// or any traits that give access to the inner `ScrollView` of the `self` view.
pub trait AutoScrolling {
    /// scrolls to the inner view's important area
    fn scroll(&mut self, direction: bool);
}

impl<T> AutoScrolling for T
where
    T: ScrollViewContainer,
{
    fn scroll(&mut self, direction: bool) {
        {
            if !config::get_config().use_page_scrolling {
                self.get_inner_scroller_view_mut()
                    .scroll_to_important_area();
                return;
            }

            let important_area = self.get_inner_scroller_view().get_inner().important_area(
                self.get_inner_scroller_view()
                    .get_scroller()
                    .last_outer_size(),
            );
            let scroller = self.get_inner_scroller_view_mut().get_scroller_mut();

            // the below implementation is based on `scroll_to_rect` function
            // defined in `Cursive::view::scroll::core.rs`.
            // The function is modified to add support for the `page_scrolling` feature.
            let top_left = (important_area.bottom_right() + (1, 1))
                .saturating_sub(scroller.last_available_size());
            let bottom_right = important_area.top_left();
            let offset = scroller.content_viewport().top_left();

            // focused view has larger size than the what is provided to the scroller
            if bottom_right.y < top_left.y {
                scroller.set_offset(bottom_right);
            } else if direction {
                // focused is moved down
                if offset.y < top_left.y {
                    scroller.set_offset(bottom_right);
                }
            } else {
                // focus is moved up
                if offset.y > bottom_right.y {
                    scroller.set_offset(top_left);
                }
            }
        }
    }
}
