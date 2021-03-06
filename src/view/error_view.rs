use crate::prelude::*;

/// Return a Cursive's View displaying an error
pub fn get_error_view(error_string: String) -> impl View {
    TextView::new(error_string)
}

/// An enum representing a normal View or an error View
pub enum ErrorViewEnum<V: View, E: View> {
    Ok(V),
    Err(E),
}

/// ErrorViewWrapper wraps the ErrorViewEnum and implements View traits for it
pub struct ErrorViewWrapper<V: View, E: View> {
    view: ErrorViewEnum<V, E>,
}

impl<V: View, E: View> ErrorViewWrapper<V, E> {
    pub fn new(view: ErrorViewEnum<V, E>) -> Self {
        ErrorViewWrapper { view }
    }

    fn get_view(&self) -> &dyn View {
        match self.view {
            ErrorViewEnum::Ok(ref v) => v,
            ErrorViewEnum::Err(ref v) => v,
        }
    }

    fn get_view_mut(&mut self) -> &mut dyn View {
        match self.view {
            ErrorViewEnum::Ok(ref mut v) => v,
            ErrorViewEnum::Err(ref mut v) => v,
        }
    }
}

impl<V: View, E: View> View for ErrorViewWrapper<V, E> {
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

    fn take_focus(&mut self, source: Direction) -> bool {
        self.get_view_mut().take_focus(source)
    }

    fn call_on_any<'a>(&mut self, selector: &Selector<'_>, callback: AnyCb<'a>) {
        self.get_view_mut().call_on_any(selector, callback)
    }

    fn needs_relayout(&self) -> bool {
        self.get_view().needs_relayout()
    }

    fn focus_view(&mut self, selector: &Selector<'_>) -> Result<(), ViewNotFound> {
        self.get_view_mut().focus_view(selector)
    }

    fn important_area(&self, size: Vec2) -> Rect {
        self.get_view().important_area(size)
    }
}
