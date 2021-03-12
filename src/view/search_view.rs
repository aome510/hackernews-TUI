use crate::prelude::*;

/// SearchView is a view used to search for stories
pub struct SearchView {
    view: LinearLayout,
}

impl ViewWrapper for SearchView {
    wrap_impl!(self.view: LinearLayout);
}

impl SearchView {
    pub fn new() -> Self {
        SearchView {
            view: LinearLayout::vertical().child(
                TextArea::new()
                    .with_name("Search Bar")
                    .full_width()
                    .max_height(1),
            ),
        }
    }
}

pub fn get_search_view(client: &hn_client::HNClient) -> impl View {
    let client = client.clone();
    OnEventView::new(SearchView::new())
        .on_event(Event::AltChar('q'), |s| {
            s.quit();
        })
        .on_event(Event::AltChar('f'), move |s| {
            s.pop_layer();
            let async_view = async_view::get_story_view_async(s, &client);
            s.screen_mut().add_transparent_layer(Layer::new(async_view));
        })
}
