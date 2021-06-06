use regex::Regex;
use serde::Deserialize;

use super::{async_view, text_view};

use crate::prelude::*;

/// ArticleView is a View used to display the content of a web page in reader mode
pub struct ArticleView {
    links: Vec<String>,
    view: ScrollView<LinearLayout>,

    raw_command: String,
}

/// Article is a struct representing the data of a web page
#[derive(Debug, Clone, Deserialize)]
pub struct Article {
    title: String,
    url: String,
    content: String,
    author: Option<String>,
    date_published: Option<String>,
    #[serde(default)]
    word_count: usize,
}

impl Article {
    pub fn update_url(&mut self, url: String) {
        self.url = url;
    }
}

impl Article {
    /// parse links from the article's content (in markdown format)
    pub fn parse_link(&self, raw_md: bool) -> (StyledString, Vec<String>) {
        // escape characters in markdown: \ ` * _ { } [ ] ( ) # + - . ! =
        let md_escape_char_re = Regex::new(r"\\(?P<char>[\\`\*_\{\}\[\]\(\)#\+\-\.!=])").unwrap();

        // if raw_md is true, don't parse link
        if raw_md {
            let content = md_escape_char_re
                .replace_all(&self.content, "${char}")
                .to_string();
            return (StyledString::plain(content), vec![]);
        }

        let md_img_re = Regex::new(r"!\[(?P<desc>.*?)\]\((?P<link>.*?)\)").unwrap();
        let mut s = md_img_re
            .replace_all(&self.content, "!\\[${desc}\\]\\(image\\)")
            .to_string();

        let md_link_re =
            Regex::new(r"(?P<prefix_char>[^\\]|^)\[(?P<desc>.*?)\]\((?P<link>.*?)\)").unwrap();
        let mut styled_s = StyledString::new();
        let mut links: Vec<String> = vec![];

        loop {
            match md_link_re.captures(&s.clone()) {
                None => break,
                Some(c) => {
                    let m = c.get(0).unwrap();
                    let prefix_char = c.name("prefix_char").unwrap().as_str();
                    let link = md_escape_char_re
                        .replace_all(c.name("link").unwrap().as_str(), "${char}")
                        .to_string();
                    let desc = md_escape_char_re
                        .replace_all(c.name("desc").unwrap().as_str(), "${char}")
                        .to_string();

                    let link = if url::Url::parse(&link).is_err() {
                        // not an absolute link
                        match url::Url::parse(&self.url).unwrap().join(&link) {
                            Ok(url) => url.to_string(),
                            Err(err) => {
                                warn!("{} is not a valid path/url: {:#?}", link, err);
                                "".to_owned()
                            }
                        }
                    } else {
                        link.to_string()
                    };
                    let desc = if desc.is_empty() {
                        format!("\"{}\"", shorten_url(&link))
                    } else {
                        desc
                    };

                    let range = m.range();
                    let mut prefix: String = s
                        .drain(std::ops::Range {
                            start: 0,
                            end: m.end(),
                        })
                        .collect();
                    prefix.drain(range);

                    prefix += prefix_char;
                    if !prefix.is_empty() {
                        styled_s.append_plain(
                            md_escape_char_re
                                .replace_all(&&prefix, "${char}")
                                .to_string(),
                        );
                    }

                    styled_s.append_styled(
                        format!("{} ", desc),
                        Style::from(get_config_theme().link_text.color),
                    );

                    let valid_url = !link.is_empty();
                    styled_s.append_styled(
                        if valid_url {
                            format!("[{}]", links.len())
                        } else {
                            "[X]".to_owned()
                        },
                        ColorStyle::new(
                            PaletteColor::TitlePrimary,
                            get_config_theme().link_id_bg.color,
                        ),
                    );

                    if !link.is_empty() {
                        // valid url
                        links.push(link.to_string());
                    }
                    continue;
                }
            }
        }
        if !s.is_empty() {
            styled_s.append_plain(md_escape_char_re.replace_all(&s, "${char}").to_string());
        }
        (styled_s, links)
    }
}

impl ViewWrapper for ArticleView {
    wrap_impl!(self.view: ScrollView<LinearLayout>);

    fn wrap_take_focus(&mut self, _: Direction) -> bool {
        true
    }
}

impl ArticleView {
    pub fn new(article: Article, raw_md: bool) -> Self {
        let (content, links) = article.parse_link(raw_md);
        let title = article.title + "\n";

        let desc = format!(
            "by: {}, date_published: {}, word_count: {}\n\n",
            article.author.unwrap_or_else(|| "[unknown]".to_string()),
            article
                .date_published
                .unwrap_or_else(|| "[unknown]".to_string()),
            if article.word_count > 1 {
                article.word_count.to_string()
            } else {
                "[unknown]".to_string()
            }
        );

        let view = LinearLayout::vertical()
            .child(
                TextView::new(StyledString::styled(
                    title,
                    ColorStyle::front(PaletteColor::TitlePrimary),
                ))
                .center()
                .full_width(),
            )
            .child(
                TextView::new(StyledString::styled(
                    desc,
                    ColorStyle::front(PaletteColor::Secondary),
                ))
                .center()
                .full_width(),
            )
            .child(TextView::new(content).full_width())
            .scrollable();

        ArticleView {
            view,
            links,
            raw_command: "".to_string(),
        }
    }

    inner_getters!(self.view: ScrollView<LinearLayout>);
}

/// Construct a help dialog from a list of URLs
pub fn get_link_dialog(links: &[String]) -> impl View {
    let article_view_keymap = get_article_view_keymap().clone();

    let links_view = OnEventView::new(LinearLayout::vertical().with(|v| {
        links.iter().enumerate().for_each(|(id, link)| {
            let mut link_styled_string = StyledString::plain(format!("{}. ", id));
            link_styled_string.append_styled(
                shorten_url(link),
                ColorStyle::front(get_config_theme().link_text.color),
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
            open_url_in_browser(&links[focus_id]);
            Some(EventResult::Consumed(None))
        }
    })
    .on_pre_event_inner(article_view_keymap.open_link_in_article_view, {
        let links = links.to_owned();
        move |s, _| {
            let focus_id = s.get_focus_index();
            let url = links[focus_id].clone();
            Some(EventResult::with_cb({
                move |s| add_article_view_layer(s, url.clone())
            }))
        }
    })
    .scrollable();

    OnEventView::new(Dialog::around(links_view).title("Link Dialog"))
        .on_event(get_global_keymap().close_dialog.clone(), |s| {
            s.pop_layer();
        })
        .on_event(get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(ArticleView::construct_help_view())
        })
        .max_height(32)
        .max_width(64)
}

/// Return a main view of a ArticleView displaying an article in reader mode.
/// The main view of a ArticleView is a View without status bar or footer.
pub fn get_article_main_view(article: Article, raw_md: bool) -> OnEventView<ArticleView> {
    let article_view_keymap = get_article_view_keymap().clone();

    let is_suffix_key = |c: &Event| -> bool {
        let article_view_keymap = get_article_view_keymap().clone();
        *c == article_view_keymap.open_link_in_browser.into()
            || *c == article_view_keymap.open_link_in_article_view.into()
    };

    OnEventView::new(ArticleView::new(article, raw_md))
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
            s.get_inner_mut()
                .get_scroller_mut()
                .scroll_down(get_config().scroll_offset);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(article_view_keymap.up, |s, _| {
            s.get_inner_mut()
                .get_scroller_mut()
                .scroll_up(get_config().scroll_offset);
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
                let links = s.links.clone();
                move |s| {
                    s.add_layer(get_link_dialog(&links));
                }
            }))
        })
        .on_pre_event_inner(article_view_keymap.open_link_in_browser, |s, _| {
            match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    if num < s.links.len() {
                        open_url_in_browser(&s.links[num]);
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
                    if num < s.links.len() {
                        let url = s.links[num].clone();
                        Some(EventResult::with_cb({
                            move |s| add_article_view_layer(s, url.clone())
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
pub fn get_article_view(article: Article, raw_md: bool) -> impl View {
    let desc = format!("Article View - {}", article.title);
    let main_view = get_article_main_view(article.clone(), raw_md).full_height();
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(&desc))
        .child(main_view)
        .child(construct_footer_view::<ArticleView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let article_view_keymap = get_article_view_keymap().clone();

    OnEventView::new(view)
        .on_event(article_view_keymap.open_article_in_browser, {
            let url = article.url.clone();
            move |_| {
                open_url_in_browser(&url);
            }
        })
        .on_event(article_view_keymap.toggle_raw_markdown_mode, {
            move |s| {
                let view = get_article_view(article.clone(), !raw_md);
                s.pop_layer();
                s.screen_mut().add_transparent_layer(Layer::new(view))
            }
        })
        .on_event(get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(ArticleView::construct_help_view())
        })
}

/// Add a ArticleView as a new layer to the main Cursive View
pub fn add_article_view_layer(s: &mut Cursive, url: String) {
    let async_view = async_view::get_article_view_async(s, url);
    s.screen_mut().add_transparent_layer(Layer::new(async_view))
}
