use crate::prelude::*;

/// Construct a new Event view from a LinearLayout by adding
/// event handlers for a key pressed
pub fn construct_event_view(view: LinearLayout) -> OnEventView<LinearLayout> {
    // add "j" and "k" for moving down and up the story list
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| {
            let id = s.get_focus_index();
            if id > 0 {
                match s.set_focus_index(id - 1) {
                    Ok(_) => Some(EventResult::Consumed(None)),
                    Err(_) => Some(EventResult::Ignored),
                }
            } else {
                Some(EventResult::Ignored)
            }
        })
        .on_pre_event_inner('j', |s, _| {
            let id = s.get_focus_index();
            if id + 1 < s.len() {
                match s.set_focus_index(id + 1) {
                    Ok(_) => Some(EventResult::Consumed(None)),
                    Err(_) => Some(EventResult::Ignored),
                }
            } else {
                Some(EventResult::Ignored)
            }
        })
}
