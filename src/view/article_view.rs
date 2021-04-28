use crate::prelude::*;

use super::async_view;
use super::utils::*;

use serde::Deserialize;

pub struct ArticleView {
    article: Article,
    links: Vec<String>,
    view: ScrollView<TextView>,
}

#[derive(Clone, Deserialize)]
pub struct Article {
    title: String,
    url: String,
    content: String,
    author: Option<String>,
    date_published: Option<String>,
    excerpt: Option<String>,
    word_count: usize,
}

impl ViewWrapper for ArticleView {
    wrap_impl!(self.view: ScrollView<TextView>);

    fn wrap_take_focus(&mut self, _: Direction) -> bool {
        true
    }
}

impl ArticleView {
    pub fn new(article: Article) -> Self {
        let view = TextView::new(article.content.clone()).scrollable();

        ArticleView {
            article,
            view,
            links: vec![],
        }
    }

    inner_getters!(self.view: ScrollView<TextView>);
}

/// Return a main view of a ArticleView displaying an article in reader mode.
/// The main view of a ArticleView is a View without status bar or footer.
pub fn get_article_main_view(article: Article) -> OnEventView<ArticleView> {
    let article_view_keymap = get_article_view_keymap().clone();
    OnEventView::new(ArticleView::new(article))
        .on_pre_event_inner(article_view_keymap.down, |s, _| {
            s.get_inner_mut().get_scroller_mut().scroll_down(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(article_view_keymap.up, |s, _| {
            s.get_inner_mut().get_scroller_mut().scroll_up(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(article_view_keymap.page_down, |s, _| {
            let height = s.get_inner().get_scroller().last_available_size().y;
            s.get_inner_mut().get_scroller_mut().scroll_down(height / 2);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(article_view_keymap.page_up, |s, _| {
            let height = s.get_inner().get_scroller().last_available_size().y;
            s.get_inner_mut().get_scroller_mut().scroll_up(height / 2);
            Some(EventResult::Consumed(None))
        })
}

/// Return a ArticleView constructed from a Article struct
pub fn get_article_view(article: Article) -> impl View {
    let desc = format!("Article View - {}", article.title);
    let main_view = get_article_main_view(article).full_height();
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(&desc))
        .child(main_view)
        .child(construct_footer_view::<ArticleView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    view
}

/// Add a ArticleView as a new layer to the main Cursive View
pub fn add_article_view_layer(s: &mut Cursive, url: String) {
    let async_view = async_view::get_article_view_async(s, url);
    s.screen_mut().add_transparent_layer(Layer::new(async_view))
}
