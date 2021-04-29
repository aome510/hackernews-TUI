use regex::Regex;
use serde::Deserialize;
use std::thread;

use super::async_view;
use super::utils::*;

use crate::prelude::*;

pub struct ArticleView {
    article: Article,
    links: Vec<String>,
    view: ScrollView<TextView>,

    raw_command: String,
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

impl Article {
    /// parse links from the article's content (in markdown format)
    pub fn parse_link(&self) -> (StyledString, Vec<String>) {
        // escape characters in markdown: \ ` * _ { } [ ] ( ) # + - . !
        let md_escape_char_re = Regex::new(r"\\(?P<char>[\\`\*_\{\}\[\]\(\)#\+\-\.!`])").unwrap();

        let md_img_re = Regex::new(r"!\[(?P<desc>.*?)\]\((?P<link>[^\[\]\s]*)\)").unwrap();
        let mut s = md_img_re
            .replace_all(&self.content, "!\\[${desc}\\]\\(image\\)")
            .to_string();

        let md_link_re =
            Regex::new(r"(?P<prefix_char>[^\\]|^)\[(?P<desc>.*?)\]\((?P<link>[^\[\]\s]*)\)")
                .unwrap();
        let mut styled_s = StyledString::new();
        let mut links: Vec<String> = vec![];

        loop {
            match md_link_re.captures(&s.clone()) {
                None => break,
                Some(c) => {
                    let m = c.get(0).unwrap();
                    let prefix_char = c.name("prefix_char").unwrap().as_str();
                    let link = c.name("link").unwrap().as_str();
                    let desc = c.name("desc").unwrap().as_str();

                    let link = if url::Url::parse(link).is_err() {
                        // not an absolute link
                        url::Url::parse(&self.url)
                            .unwrap()
                            .join(link)
                            .unwrap()
                            .to_string()
                    } else {
                        link.to_string()
                    };
                    let desc = if desc.len() == 0 {
                        format!("\"{}\"", shorten_url(&link))
                    } else {
                        md_escape_char_re.replace_all(&desc, "${char}").to_string()
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
                    if prefix.len() > 0 {
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
                    styled_s.append_styled(
                        format!("[{}]", links.len()),
                        ColorStyle::new(
                            PaletteColor::TitlePrimary,
                            get_config_theme().link_id_bg.color,
                        ),
                    );
                    links.push(link.to_string());
                    continue;
                }
            }
        }
        if s.len() > 0 {
            styled_s.append_plain(md_escape_char_re.replace_all(&s, "${char}").to_string());
        }
        (styled_s, links)
    }
}

impl ViewWrapper for ArticleView {
    wrap_impl!(self.view: ScrollView<TextView>);

    fn wrap_take_focus(&mut self, _: Direction) -> bool {
        true
    }
}

impl ArticleView {
    pub fn new(article: Article) -> Self {
        let (content, links) = article.parse_link();
        let view = TextView::new(content).scrollable();

        ArticleView {
            article,
            view,
            links,
            raw_command: "".to_string(),
        }
    }

    inner_getters!(self.view: ScrollView<TextView>);
}

/// Return a main view of a ArticleView displaying an article in reader mode.
/// The main view of a ArticleView is a View without status bar or footer.
pub fn get_article_main_view(article: Article) -> OnEventView<ArticleView> {
    let article_view_keymap = get_article_view_keymap().clone();

    let is_suffix_key = |c: &Event| -> bool {
        let article_view_keymap = get_article_view_keymap().clone();
        *c == article_view_keymap.open_link_in_browser.into()
            || *c == article_view_keymap.open_link_in_article_view.into()
    };

    OnEventView::new(ArticleView::new(article))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            match *e {
                Event::Char(c) if '0' <= c && c <= '9' => {
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
        .on_pre_event_inner(article_view_keymap.open_link_in_browser, |s, _| {
            match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    if num < s.links.len() {
                        let url = s.links[num].clone();
                        thread::spawn(move || {
                            if let Err(err) = webbrowser::open(&url) {
                                warn!("failed to open link {}: {}", url, err);
                            }
                        });
                        Some(EventResult::Consumed(None))
                    } else {
                        Some(EventResult::Consumed(None))
                    }
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
pub fn get_article_view(article: Article) -> impl View {
    let desc = format!("Article View - {}", article.title);
    let main_view = get_article_main_view(article.clone()).full_height();
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(&desc))
        .child(main_view)
        .child(construct_footer_view::<ArticleView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let article_view_keymap = get_article_view_keymap().clone();

    OnEventView::new(view).on_event(article_view_keymap.open_article_in_browser, {
        let url = article.url.clone();
        move |_| {
            if url.len() > 0 {
                let url = url.clone();
                thread::spawn(move || {
                    if let Err(err) = webbrowser::open(&url) {
                        warn!("failed to open link {}: {}", url, err);
                    }
                });
            }
        }
    })
}

/// Add a ArticleView as a new layer to the main Cursive View
pub fn add_article_view_layer(s: &mut Cursive, url: String) {
    let async_view = async_view::get_article_view_async(s, url);
    s.screen_mut().add_transparent_layer(Layer::new(async_view))
}
