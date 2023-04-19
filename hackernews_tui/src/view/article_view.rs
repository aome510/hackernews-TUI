use super::{async_view, help_view::HasHelpView, link_dialog, traits::*, utils};
use crate::prelude::*;

/// ArticleView is a View used to display the content of a web page in reader mode
pub struct ArticleView {
    article: Article,
    links: Vec<String>,
    width: usize,

    view: ScrollView<LinearLayout>,

    raw_command: String,
}

impl ViewWrapper for ArticleView {
    wrap_impl!(self.view: ScrollView<LinearLayout>);

    fn wrap_layout(&mut self, size: Vec2) {
        if self.width != size.x {
            // got a new width since the last time the article view is rendered,
            // re-parse the article using the new width

            self.width = size.x;

            match self.article.parse(self.width.saturating_sub(5)) {
                Ok(result) => {
                    self.set_article_content(result.s);
                    self.links = result.links;
                }
                Err(err) => {
                    warn!("failed to parse the article: {}", err);
                }
            }
        }

        self.with_view_mut(|v| v.layout(size));
    }

    fn wrap_take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }
}

impl ArticleView {
    pub fn new(article: Article) -> Self {
        let component_style = &config::get_config_theme().component_style;
        let unknown = "[unknown]".to_string();
        let desc = format!(
            "by: {}, date_published: {}",
            article.author.as_ref().unwrap_or(&unknown),
            article.date_published.as_ref().unwrap_or(&unknown),
        );

        let view = LinearLayout::vertical()
            .child(TextView::new(&article.title).center().full_width())
            .child(
                TextView::new(StyledString::styled(desc, component_style.metadata))
                    .center()
                    .full_width(),
            )
            .child(PaddedView::lrtb(1, 1, 1, 1, TextView::new("")))
            .scrollable();

        ArticleView {
            article,
            links: vec![],
            width: 0,

            view,
            raw_command: "".to_string(),
        }
    }

    /// Update the content of the article
    pub fn set_article_content(&mut self, new_content: StyledString) {
        self.view
            .get_inner_mut()
            .get_child_mut(2)
            .expect("The article view should have 3 children")
            .downcast_mut::<PaddedView<TextView>>()
            .expect("The 3rd child of the article view should be a padded text view")
            .get_inner_mut()
            .set_content(new_content)
    }

    inner_getters!(self.view: ScrollView<LinearLayout>);
}

impl ScrollViewContainer for ArticleView {
    type ScrollInner = LinearLayout;

    fn get_inner_scroll_view(&self) -> &ScrollView<LinearLayout> {
        self.get_inner()
    }

    fn get_inner_scroll_view_mut(&mut self) -> &mut ScrollView<LinearLayout> {
        self.get_inner_mut()
    }
}

fn construct_article_main_view(article: Article) -> OnEventView<ArticleView> {
    let is_suffix_key = |c: &Event| -> bool {
        let article_view_keymap = config::get_article_view_keymap();
        article_view_keymap.open_link_in_browser.has_event(c)
            || article_view_keymap.open_link_in_article_view.has_event(c)
    };

    let article_view_keymap = config::get_article_view_keymap().clone();

    OnEventView::new(ArticleView::new(article))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            match *e {
                Event::Char(c) if c.is_ascii_digit() => {
                    s.raw_command.push(c);
                }
                _ => {
                    if !is_suffix_key(e) {
                        s.raw_command.clear();
                    }
                }
            };
            None
        })
        .on_pre_event_inner(article_view_keymap.open_link_dialog, |s, _| {
            Some(EventResult::with_cb({
                let links = s.links.clone();
                move |s| {
                    s.add_layer(link_dialog::get_link_dialog(&links));
                }
            }))
        })
        .on_pre_event_inner(article_view_keymap.open_link_in_browser, |s, _| {
            match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    utils::open_ith_link_in_browser(&s.links, num)
                }
                Err(_) => None,
            }
        })
        .on_pre_event_inner(
            article_view_keymap.open_link_in_article_view,
            |s, _| match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    utils::open_ith_link_in_article_view(&s.links, num)
                }
                Err(_) => None,
            },
        )
        .on_pre_event_inner(article_view_keymap.open_article_in_browser, |s, _| {
            utils::open_url_in_browser(&s.article.url);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(ArticleView::construct_on_event_help_view())
        })
        .on_scroll_events()
}

/// Construct an article view of an article
pub fn construct_article_view(article: Article) -> impl View {
    let desc = format!("Article View - {}", article.title);
    let main_view = construct_article_main_view(article).full_height();

    let mut view = LinearLayout::vertical()
        .child(utils::construct_view_title_bar(&desc))
        .child(main_view)
        .child(utils::construct_footer_view::<ArticleView>());
    view.set_focus_index(1)
        .unwrap_or(EventResult::Consumed(None));

    view
}

/// Retrieve an article from a given `url` and construct an article view of that article
pub fn construct_and_add_new_article_view(s: &mut Cursive, url: &str) {
    let async_view = async_view::construct_article_view_async(s, url);
    s.screen_mut().add_transparent_layer(Layer::new(async_view))
}
