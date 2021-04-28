use crate::prelude::*;

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
}

impl ArticleView {
    pub fn new(data: Vec<u8>) -> Self {
        let article: Article = serde_json::from_slice(&data).unwrap();
        let view = TextView::new(article.content.clone()).scrollable();

        ArticleView {
            article,
            view,
            links: vec![],
        }
    }
}
