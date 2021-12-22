use super::help_view::HasHelpView;
use super::{async_view, text_view};
use crate::prelude::*;

/// ArticleView is a View used to display the content of a web page in reader mode
pub struct ArticleView {
    article: client::Article,
    view: ScrollView<LinearLayout>,

    raw_command: String,
}

impl ViewWrapper for ArticleView {
    wrap_impl!(self.view: ScrollView<LinearLayout>);

    fn wrap_take_focus(&mut self, _: Direction) -> bool {
        true
    }
}

impl ArticleView {
    pub fn new(article: client::Article) -> Self {
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
            .child(PaddedView::lrtb(
                1,
                1,
                0,
                0,
                TextView::new(article.parsed_content.clone()),
            ))
            .scrollable();

        ArticleView {
            article,
            view,
            raw_command: "".to_string(),
        }
    }

    inner_getters!(self.view: ScrollView<LinearLayout>);
}

/// Construct a help dialog from a list of URLs
pub fn get_link_dialog(links: &[String]) -> impl View {
    let article_view_keymap = config::get_article_view_keymap().clone();

    let links_view = OnEventView::new(LinearLayout::vertical().with(|v| {
        links.iter().enumerate().for_each(|(id, link)| {
            let mut link_styled_string = StyledString::plain(format!("{}. ", id + 1));
            link_styled_string.append_styled(
                utils::shorten_url(link),
                config::get_config_theme().component_style.link,
            );
            v.add_child(text_view::TextView::new(link_styled_string));
        })
    }))
    .on_pre_event_inner(article_view_keymap.link_dialog_focus_next, |s, _| {
        let focus_id = s.get_focus_index();
        s.set_focus_index(focus_id + 1).unwrap_or_else(|_| {});
        Some(EventResult::Consumed(None))
    })
    .on_pre_event_inner(article_view_keymap.link_dialog_focus_prev, |s, _| {
        let focus_id = s.get_focus_index();
        if focus_id > 0 {
            s.set_focus_index(focus_id - 1).unwrap_or_else(|_| {});
        }
        Some(EventResult::Consumed(None))
    })
    .on_pre_event_inner(article_view_keymap.open_link_in_browser, {
        let links = links.to_owned();
        move |s, _| {
            let focus_id = s.get_focus_index();
            utils::open_url_in_browser(&links[focus_id]);
            Some(EventResult::Consumed(None))
        }
    })
    .on_pre_event_inner(article_view_keymap.open_link_in_article_view, {
        let links = links.to_owned();
        move |s, _| {
            let focus_id = s.get_focus_index();
            let url = links[focus_id].clone();
            Some(EventResult::with_cb({
                move |s| add_article_view_layer(s, &url)
            }))
        }
    })
    .scrollable();

    OnEventView::new(Dialog::around(links_view).title("Link Dialog"))
        .on_event(config::get_global_keymap().close_dialog.clone(), |s| {
            s.pop_layer();
        })
        .on_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(ArticleView::construct_help_view())
        })
        .max_height(32)
        .max_width(64)
}

/// Return a main view of a ArticleView displaying an article in reader mode.
/// The main view of a ArticleView is a View without status bar or footer.
pub fn get_article_main_view(article: client::Article) -> OnEventView<ArticleView> {
    let article_view_keymap = config::get_article_view_keymap().clone();

    let is_suffix_key = |c: &Event| -> bool {
        let article_view_keymap = config::get_article_view_keymap().clone();
        *c == article_view_keymap.open_link_in_browser.into()
            || *c == article_view_keymap.open_link_in_article_view.into()
    };

    OnEventView::new(ArticleView::new(article))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            match *e {
                Event::Char(c) if ('0'..='9').contains(&c) => {
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
        .on_pre_event_inner(article_view_keymap.down, |s, _| {
            s.get_inner_mut().get_scroller_mut().scroll_down(3);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(article_view_keymap.up, |s, _| {
            s.get_inner_mut().get_scroller_mut().scroll_up(3);
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
        .on_pre_event_inner(article_view_keymap.top, |s, _| {
            s.get_inner_mut().get_scroller_mut().scroll_to_top();
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(article_view_keymap.bottom, |s, _| {
            s.get_inner_mut().get_scroller_mut().scroll_to_bottom();
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(article_view_keymap.open_link_dialog, |s, _| {
            Some(EventResult::with_cb({
                let links = s.article.links.clone();
                move |s| {
                    s.add_layer(get_link_dialog(&links));
                }
            }))
        })
        .on_pre_event_inner(article_view_keymap.open_link_in_browser, |s, _| {
            match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    if num > 0 && num <= s.article.links.len() {
                        utils::open_url_in_browser(&s.article.links[num - 1]);
                    }
                    Some(EventResult::Consumed(None))
                }
                Err(_) => None,
            }
        })
        .on_pre_event_inner(
            article_view_keymap.open_link_in_article_view,
            |s, _| match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    if num > 0 && num <= s.article.links.len() {
                        let url = s.article.links[num - 1].clone();
                        Some(EventResult::with_cb({
                            move |s| add_article_view_layer(s, &url)
                        }))
                    } else {
                        Some(EventResult::Consumed(None))
                    }
                }
                Err(_) => None,
            },
        )
}

/// Return a ArticleView constructed from a Article struct
pub fn get_article_view(article: client::Article) -> impl View {
    let desc = format!("Article View - {}", article.title);
    let main_view = get_article_main_view(article.clone()).full_height();
    let mut view = LinearLayout::vertical()
        .child(utils::construct_view_title_bar(&desc))
        .child(main_view)
        .child(utils::construct_footer_view::<ArticleView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let article_view_keymap = config::get_article_view_keymap().clone();

    OnEventView::new(view)
        .on_event(article_view_keymap.open_article_in_browser, {
            move |_| {
                utils::open_url_in_browser(&article.url);
            }
        })
        .on_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(ArticleView::construct_help_view())
        })
}

/// Add a ArticleView as a new layer to the main Cursive View
pub fn add_article_view_layer(s: &mut Cursive, url: &str) {
    let async_view = async_view::get_article_view_async(s, url);
    s.screen_mut().add_transparent_layer(Layer::new(async_view))
}
