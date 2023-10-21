use crate::prelude::*;
use crate::utils;
use once_cell::sync::Lazy;
use regex::Regex;

/// A regex to parse a HN text (in HTML).
/// It consists of multiple regexes representing different components.
static HN_TEXT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        "(({})|({})|({})|({})|({})|({}))",
        // a regex matching a HTML paragraph
        r"<p>(?s)(?P<paragraph>(|[^>].*?))</p>",
        // a regex matching a paragraph quote (in markdown format)
        r"<p>(?s)(?P<quote>>[> ]*)(?P<text>.*?)</p>",
        // a regex matching an HTML italic string
        r"<i>(?s)(?P<italic>.*?)</i>",
        // a regex matching a HTML code block (multiline)
        r"<pre><code>(?s)(?P<multiline_code>.*?)[\n]*</code></pre>",
        // a regex matching a single line code block (markdown format)
        r"`(?P<code>[^`]+?)`",
        // a regex matching a HTML link
        r#"<a\s+?href="(?P<link>.*?)"(?s).+?</a>"#,
    ))
    .unwrap()
});

/// Parsed result of a HTML text
#[derive(Debug, Default)]
pub struct HTMLTextParsedResult {
    /// parsed HTML content
    pub content: StyledString,
    /// a list of links inside the HTML document
    pub links: Vec<String>,
}

/// Parsed result of a HTML table
#[derive(Debug, Default)]
pub struct HTMLTableParsedResult {
    /// a list of links inside the HTML document
    pub links: Vec<String>,
    /// parsed table headers
    pub headers: Vec<StyledString>,
    /// parsed table rows
    pub rows: Vec<Vec<StyledString>>,
}

impl HTMLTextParsedResult {
    /// merge two HTML parsed results
    pub fn merge(&mut self, mut other: HTMLTextParsedResult) {
        self.content.append(other.content);
        self.links.append(&mut other.links);
    }
}

/// parse a Hacker News HTML text
pub fn parse_hn_html_text(text: String, style: Style, base_link_id: usize) -> HTMLTextParsedResult {
    debug!("parse hn html text: {}", text);

    // pre-processed the HTML text
    let text = {
        // The item's text returned from HN APIs may have `<p>` tags representing
        // paragraph breaks. Convert `<p>` tags to <p></p> tag pairs to make the text
        // easier to parse.
        if text.is_empty() {
            text
        } else {
            format!("<p>{}</p>", text.replace("<p>", "</p>\n<p>"))
        }
    };

    parse(text, style, base_link_id)
}

/// a helper function of [parse_hn_html_text] for recursively parsing HTML elements inside the text
fn parse(text: String, style: Style, base_link_id: usize) -> HTMLTextParsedResult {
    let mut result = HTMLTextParsedResult::default();
    // an index such that `text[curr_pos..]` represents the slice of the
    // text that hasn't been parsed.
    let mut curr_pos = 0;

    for caps in HN_TEXT_RE.captures_iter(&text) {
        // the part that doesn't match any patterns is rendered in the default style
        let whole_match = caps.get(0).unwrap();
        if curr_pos < whole_match.start() {
            result
                .content
                .append_styled(&text[curr_pos..whole_match.start()], style);
        }
        curr_pos = whole_match.end();

        let component_style = &config::get_config_theme().component_style;

        if let (Some(m_quote), Some(m_text)) = (caps.name("quote"), caps.name("text")) {
            // quoted paragraph
            // render quote character `>` using the `|` indentation character
            result.content.append_styled(
                "▎"
                    .to_string()
                    .repeat(m_quote.as_str().matches('>').count()),
                style,
            );
            result.merge(parse(
                m_text.as_str().to_string(),
                component_style.quote.into(),
                base_link_id + result.links.len(),
            ));

            result.content.append_plain("\n");
        } else if let Some(m) = caps.name("paragraph") {
            // normal paragraph
            result.merge(parse(
                m.as_str().to_string(),
                style,
                base_link_id + result.links.len(),
            ));

            result.content.append_plain("\n");
        } else if let Some(m) = caps.name("link") {
            // HTML link
            result.links.push(m.as_str().to_string());

            result.content.append_styled(
                utils::shorten_url(m.as_str()),
                style.combine(component_style.link),
            );
            result.content.append_styled(" ", style);
            result.content.append_styled(
                format!("[{}]", result.links.len() + base_link_id),
                style.combine(component_style.link_id),
            );
        } else if let Some(m) = caps.name("multiline_code") {
            // HTML code block
            result.content.append_styled(
                m.as_str(),
                style.combine(component_style.multiline_code_block),
            );
            result.content.append_plain("\n");
        } else if let Some(m) = caps.name("code") {
            // markdown single line code block
            result
                .content
                .append_styled(m.as_str(), style.combine(component_style.single_code_block));
        } else if let Some(m) = caps.name("italic") {
            // HTML italic
            result
                .content
                .append_styled(m.as_str(), style.combine(component_style.italic));
        }
    }

    if curr_pos < text.len() {
        result
            .content
            .append_styled(&text[curr_pos..text.len()], style);
    }

    result
}
