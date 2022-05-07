use crate::prelude::*;

/// A trait that represents a wrapper view.
/// It requires to implement functions that return (mutable) pointer
/// to the inner view object.
pub trait FnViewWrapper {
    fn get_view(&self) -> &dyn View;
    fn get_view_mut(&mut self) -> &mut dyn View;
}

#[macro_export]
macro_rules! impl_view_fns_for_fn_view_wrapper {
    () => {
        fn draw(&self, printer: &Printer) {
            self.get_view().draw(printer);
        }

        fn required_size(&mut self, req: Vec2) -> Vec2 {
            self.get_view_mut().required_size(req)
        }

        fn on_event(&mut self, ch: Event) -> EventResult {
            self.get_view_mut().on_event(ch)
        }

        fn layout(&mut self, size: Vec2) {
            self.get_view_mut().layout(size);
        }

        fn take_focus(&mut self, source: Direction) -> Result<EventResult, CannotFocus> {
            self.get_view_mut().take_focus(source)
        }

        fn call_on_any<'a>(&mut self, selector: &Selector<'_>, callback: AnyCb<'a>) {
            self.get_view_mut().call_on_any(selector, callback)
        }

        fn needs_relayout(&self) -> bool {
            self.get_view().needs_relayout()
        }

        fn focus_view(&mut self, selector: &Selector<'_>) -> Result<EventResult, ViewNotFound> {
            self.get_view_mut().focus_view(selector)
        }

        fn important_area(&self, size: Vec2) -> Rect {
            self.get_view().important_area(size)
        }
    };
}
