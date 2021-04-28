use regex::Regex;
use serde::Deserialize;

use super::async_view;
use super::utils::*;

use crate::prelude::*;

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

impl Article {
    /// parse links from the article's content (in markdown format)
    pub fn parse_link(&self) -> (StyledString, Vec<String>) {
        // escape characters in markdown
        let md_escape_char_re = Regex::new(r"\\(?P<char>[*_\[\]\(\)])").unwrap();

        let md_img_re = Regex::new(r"!\[(?P<desc>.*?)\]\((?P<link>[^\[\]]*)\)").unwrap();
        let mut s = md_img_re
            .replace_all(&self.content, "${desc}\\(image\\)")
            .to_string();

        let md_link_re = Regex::new(r"[^\\]\[(?P<desc>.*?)\]\((?P<link>[^\[\]]*)\)").unwrap();
        let mut styled_s = StyledString::new();
        let mut links: Vec<String> = vec![];

        loop {
            match md_link_re.captures(&s.clone()) {
                None => break,
                Some(c) => {
                    let m = c.get(0).unwrap();
                    let link = c.name("link").unwrap().as_str();
                    let desc = c.name("desc").unwrap().as_str();

                    let link = if !link.starts_with("http") {
                        self.url.clone() + link
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

                    if prefix.len() > 0 {
                        styled_s.append_plain(
                            md_escape_char_re
                                .replace_all(&&prefix, "${char}")
                                .to_string(),
                        );
                    }

                    styled_s.append_styled(
                        format!(" {} ", desc),
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
