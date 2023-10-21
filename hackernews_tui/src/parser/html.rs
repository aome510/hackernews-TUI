use crate::prelude::*;
use crate::utils;
use once_cell::sync::Lazy;
use regex::Regex;

/// A regex to parse a HN text (in HTML).
/// It consists of multiple regex(s) representing different elements.
static HN_TEXT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        "(({})|({})|({})|({})|({})|({}))",
        // a regex that matches a HTML paragraph
        r"<p>(?s)(?P<paragraph>(|[^>].*?))</p>",
        // a regex that matches a paragraph quote (in markdown format)
        r"<p>(?s)(?P<quote>>[> ]*)(?P<text>.*?)</p>",
        // a regex that matches an HTML italic string
        r"<i>(?s)(?P<italic>.*?)</i>",
        // a regex that matches a HTML code block (multiline)
        r"<pre><code>(?s)(?P<multiline_code>.*?)[\n]*</code></pre>",
        // a regex that matches a single line code block (markdown format)
        r"`(?P<code>[^`]+?)`",
        // a regex that matches a HTML link
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
pub fn parse_hn_html_text(text: String, style: Style, base_link_id: usize) -> HTMLParsedResult {
    debug!("parse hn html text: {}", text);

    let mut result = HTMLParsedResult::default();
    // an index such that `text[curr_pos..]` represents the part of the
    // text that hasn't been parsed.
    let mut curr_pos = 0;

    // This variable indicates whether we have parsed the first paragraph of the current text.
    // It is used to add a break between 2 consecutive paragraphs.
    let mut seen_first_paragraph = false;

    for caps in HN_TEXT_RE.captures_iter(&text) {
        // the part that doesn't match any patterns is rendered in the default style
        let whole_match = caps.get(0).unwrap();
        if curr_pos < whole_match.start() {
            result
                .s
                .append_styled(&text[curr_pos..whole_match.start()], style);
        }
        curr_pos = whole_match.end();

        let component_style = &config::get_config_theme().component_style;

        if let (Some(m_quote), Some(m_text)) = (caps.name("quote"), caps.name("text")) {
            if seen_first_paragraph {
                result.s.append_plain("\n");
            } else {
                seen_first_paragraph = true;
            }

            // render quote character `>` as indentation character
            result.s.append_styled(
                "▎"
                    .to_string()
                    .repeat(m_quote.as_str().matches('>').count()),
                style,
            );
            result.merge(parse_hn_html_text(
                m_text.as_str().to_string(),
                component_style.quote.into(),
                base_link_id + result.links.len(),
            ));

            result.s.append_plain("\n");
        } else if let Some(m) = caps.name("paragraph") {
            if seen_first_paragraph {
                result.s.append_plain("\n");
            } else {
                seen_first_paragraph = true;
            }

            result.merge(parse_hn_html_text(
                m.as_str().to_string(),
                style,
                base_link_id + result.links.len(),
            ));

            result.s.append_plain("\n");
        } else if let Some(m) = caps.name("link") {
            result.links.push(m.as_str().to_string());

            result.s.append_styled(
                utils::shorten_url(m.as_str()),
                style.combine(component_style.link),
            );
            result.s.append_styled(" ", style);
            result.s.append_styled(
                format!("[{}]", result.links.len() + base_link_id),
                style.combine(component_style.link_id),
            );
        } else if let Some(m) = caps.name("multiline_code") {
            result.s.append_styled(
                m.as_str(),
                style.combine(component_style.multiline_code_block),
            );
            result.s.append_plain("\n");
        } else if let Some(m) = caps.name("code") {
            result
                .s
                .append_styled(m.as_str(), style.combine(component_style.single_code_block));
        } else if let Some(m) = caps.name("italic") {
            result
                .s
                .append_styled(m.as_str(), style.combine(component_style.italic));
        }
    }

    if curr_pos < text.len() {
        result.s.append_styled(&text[curr_pos..text.len()], style);
    }

    result
}
