use super::{fn_view_wrapper::FnViewWrapper, help_view, utils};
use crate::{impl_view_fns_for_fn_view_wrapper, prelude::*};

/// A result view
pub enum ResultView<V: View> {
    Err(ErrorView),
    Ok(V),
}

/// A view displaying an application error
pub struct ErrorView {
    view: LinearLayout,
}

impl<V: View> ResultView<V> {
    /// constructs a result view from an `anyhow::Result`.
    ///
    /// The function also requires to specify a callback function `cb`
    /// that converts the result `Ok` value into the corresponding `Ok` view.
    pub fn new<T, F>(result: anyhow::Result<T>, cb: F) -> Self
    where
        V: View,
        F: Fn(T) -> V,
    {
        match result {
            Ok(x) => Self::Ok(cb(x)),
            Err(err) => Self::Err(ErrorView::new(err)),
        }
    }
}

impl<V: View> FnViewWrapper for ResultView<V> {
    fn get_view(&self) -> &dyn View {
        match self {
            Self::Ok(v) => v,
            Self::Err(e) => e,
        }
    }

    fn get_view_mut(&mut self) -> &mut dyn View {
        match self {
            Self::Ok(v) => v,
            Self::Err(e) => e,
        }
    }
}

impl<V: View> View for ResultView<V> {
    impl_view_fns_for_fn_view_wrapper!();
}

impl ErrorView {
    /// constructs an error view from an `anyhow::Error`
    pub fn new(err: anyhow::Error) -> Self {
        Self {
            view: LinearLayout::vertical()
                .child(utils::construct_view_title_bar("Error View"))
                .child(Dialog::around(
                    TextView::new(format!("Error: {:?}", err))
                        .scrollable()
                        .full_height(),
                ))
                .child(utils::construct_footer_view::<help_view::DefaultHelpView>()),
        }
    }
}

impl ViewWrapper for ErrorView {
    wrap_impl!(self.view: LinearLayout);
}
