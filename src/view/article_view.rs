use crate::prelude::*;

pub struct ArticleView {
    view: ScrollView<TextView>,
    links: Vec<String>,
}

impl ViewWrapper for ArticleView {
    wrap_impl!(self.view: ScrollView<TextView>);
}

impl ArticleView {
    pub fn new(content: String) -> Self {
        let (parsed_text, links) = Self::parse_markdown(content);
        ArticleView {
            view: TextView::new(parsed_text).scrollable(),
            links,
        }
    }

    fn parse_markdown(raw_text: String) -> (StyledString, Vec<String>) {
        (StyledString::plain(raw_text), vec![])
    }
}
