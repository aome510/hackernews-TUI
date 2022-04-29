use crate::prelude::*;

use super::{comment_view::CommentView, story_view::StoryView};

/// ScrollableList is a trait that implements basic methods
/// to interact with a View that wraps a ScrollListView
pub trait ListViewContainer {
    fn get_inner_list(&self) -> &LinearLayout;
    fn get_inner_list_mut(&mut self) -> &mut LinearLayout;

    fn len(&self) -> usize {
        self.get_inner_list().len()
    }

    fn get_focus_index(&self) -> usize {
        self.get_inner_list().get_focus_index()
    }

    fn set_focus_index(&mut self, id: usize) -> Option<EventResult> {
        match self.get_inner_list_mut().set_focus_index(id) {
            Ok(_) => Some(EventResult::Consumed(None)),
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

    // fn get_scroller(&self) -> &scroll::Core;
    // fn get_scroller_mut(&mut self) -> &mut scroll::Core;
    // // Move the scroller to the focused area and adjust the scroller
    // // accordingly depending on the scrolling direction.
    // // (direction = true) stands for going down while false stands for going up
    // fn scroll(&mut self, direction: bool);
}

impl ListViewContainer for StoryView {
    fn get_inner_list(&self) -> &LinearLayout {
        self.get_inner().get_inner()
    }

    fn get_inner_list_mut(&mut self) -> &mut LinearLayout {
        self.get_inner_mut().get_inner_mut()
    }
}

impl ListViewContainer for CommentView {
    fn get_inner_list(&self) -> &LinearLayout {
        self.get_inner().get_inner()
    }

    fn get_inner_list_mut(&mut self) -> &mut LinearLayout {
        self.get_inner_mut().get_inner_mut()
    }
}

// #[macro_export]
// macro_rules! impl_scrollable_list {
//     () => {
//         fn len(&self) -> usize {
//             self.get_inner().get_inner().len()
//         }

//         fn get_focus_index(&self) -> usize {
//             self.get_inner().get_inner().get_focus_index()
//         }

//         fn scroll(&mut self, direction: bool) {
//             if !config::get_config().use_page_scrolling {
//                 self.get_inner_mut().scroll_to_important_area();
//                 return;
//             }

//             let important_area = self
//                 .get_inner()
//                 .get_inner()
//                 .important_area(self.get_scroller().last_outer_size());
//             let scroller = self.get_scroller_mut();

//             // the below implementation is based on scroll_to_rect function
//             // defined in Cursive::view::scroll::core.rs.
//             // The function is modified to add support for the page_scrolling feature.
//             let top_left = (important_area.bottom_right() + (1, 1))
//                 .saturating_sub(scroller.last_available_size());
//             let bottom_right = important_area.top_left();
//             let offset = scroller.content_viewport().top_left();

//             // focused view has larger size than the what is provided to the scroller
//             if (bottom_right.y < top_left.y) {
//                 scroller.set_offset(bottom_right);
//             } else {
//                 if (direction) {
//                     // focused is moved down
//                     if (offset.y < top_left.y) {
//                         scroller.set_offset(bottom_right);
//                     }
//                 } else {
//                     // focus is moved up
//                     if (offset.y > bottom_right.y) {
//                         scroller.set_offset(top_left);
//                     }
//                 }
//             }
//         }

//         fn set_focus_index(&mut self, id: usize) -> Option<EventResult> {
//             let current_id = self.get_focus_index();
//             let direction = (current_id <= id);
//             let linear_layout = self.get_inner_mut().get_inner_mut();

//             match linear_layout.set_focus_index(id) {
//                 Ok(_) => {
//                     self.scroll(direction);
//                     Some(EventResult::Consumed(None))
//                 }
//                 Err(_) => None,
//             }
//         }

//         fn add_item<V: IntoBoxedView + 'static>(&mut self, view: V) {
//             let linear_layout = self.get_inner_mut().get_inner_mut();
//             linear_layout.add_child(view);
//         }

//         fn get_scroller_mut(&mut self) -> &mut scroll::Core {
//             self.get_inner_mut().get_scroller_mut()
//         }

//         fn get_scroller(&self) -> &scroll::Core {
//             self.get_inner().get_scroller()
//         }

//         fn get_item(&self, id: usize) -> Option<&(dyn View + 'static)> {
//             self.get_inner().get_inner().get_child(id)
//         }

//         fn get_item_mut(&mut self, id: usize) -> Option<&mut (dyn View + 'static)> {
//             self.get_inner_mut().get_inner_mut().get_child_mut(id)
//         }
//     };
// }
