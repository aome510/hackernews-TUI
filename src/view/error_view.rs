use super::fn_view_wrapper::*;
use crate::{impl_view_for_fn_wrapper, prelude::*};

/// Return a Cursive's View displaying an error
pub fn get_error_view(err_desc: String, err: Error, client: &hn_client::HNClient) -> impl View {
    OnEventView::new(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(err_desc))
                .child(TextView::new(format!("{:#?}", err))),
        )
        .button("front page", {
            let client = client.clone();
            move |s| {
                let async_view = async_view::get_story_view_async(s, &client);
                s.pop_layer();
                s.screen_mut().add_transparent_layer(Layer::new(async_view));
            }
        })
        .button("quit", |s| s.quit()),
    )
    .on_event(Event::AltChar('f'), {
        let client = client.clone();
        move |s| {
            let async_view = async_view::get_story_view_async(s, &client);
            s.pop_layer();
            s.add_layer(async_view);
        }
    })
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
}

impl<V: View, E: View> FnViewWrapper for ErrorViewWrapper<V, E> {
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
    impl_view_for_fn_wrapper!();
}
