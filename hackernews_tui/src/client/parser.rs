use std::borrow::Cow;

use crate::prelude::*;
use crate::utils;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use serde::{de, Deserialize, Deserializer};

use html5ever::tendril::TendrilSink;
use html5ever::*;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

lazy_static! {
    /// a regex that matches a search match in the response from HN Algolia search API
    static ref MATCH_RE: Regex = Regex::new(r"<em>(?P<match>.*?)</em>").unwrap();

    /// a regex that matches whitespace character(s)
    static ref WS_RE: Regex = Regex::new(r"\s+").unwrap();

    /// a regex used to parse a HN text (in HTML format)
    /// It consists of multiple regex(s) representing different elements
    static ref HN_TEXT_RE: Regex = Regex::new(&format!(
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
    )).unwrap();
}

// serde helper functions

fn parse_id<'de, D>(d: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse::<u32>().map_err(de::Error::custom)
}

fn parse_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

// API response structs

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct MatchResult {
    value: String,
}

#[derive(Debug, Deserialize)]
struct HighlightResultResponse {
    title: Option<MatchResult>,
}

#[derive(Debug, Deserialize)]
/// StoryResponse represents the story data received from HN_ALGOLIA APIs
pub struct StoryResponse {
    #[serde(default)]
    #[serde(rename(deserialize = "objectID"))]
    #[serde(deserialize_with = "parse_id")]
    id: u32,

    author: Option<String>,
    url: Option<String>,
    #[serde(rename(deserialize = "story_text"))]
    text: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "parse_null_default")]
    points: u32,
    #[serde(default)]
    #[serde(deserialize_with = "parse_null_default")]
    num_comments: usize,

    #[serde(rename(deserialize = "created_at_i"))]
    time: u64,

    // search result
    #[serde(rename(deserialize = "_highlightResult"))]
    highlight_result: Option<HighlightResultResponse>,
}

#[derive(Debug, Deserialize)]
/// HNStoryResponse represents the story data received from the official HackerNews APIs
pub struct HNStoryResponse {
    #[serde(default)]
    pub kids: Vec<u32>,
}

#[derive(Debug, Deserialize)]
/// CommentResponse represents the comment data received from HN_ALGOLIA APIs
pub struct CommentResponse {
    id: u32,

    #[serde(default)]
    children: Vec<CommentResponse>,

    text: Option<String>,
    author: Option<String>,

    #[serde(rename(deserialize = "created_at_i"))]
    time: u64,
}

#[derive(Debug, Deserialize)]
/// StoriesResponse represents the stories data received from HN_ALGOLIA APIs
pub struct StoriesResponse {
    pub hits: Vec<StoryResponse>,
}

// parsed structs

/// A parsed Hacker News story
#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub title: StyledString,
    pub url: String,
    pub author: String,
    pub text: HnText,
    pub points: u32,
    pub num_comments: usize,
    pub time: u64,
}

/// A parsed Hacker News text
#[derive(Debug, Clone)]
pub struct HnText {
    pub id: u32,
    pub level: usize,
    pub state: CollapseState,
    pub text: StyledString,
    /// The minimized version of the text used to display the text component when it's collapsed.
    pub minimized_text: StyledString,
    pub links: Vec<String>,
}

#[derive(Debug, Clone)]
/// The collapse state of a HN text component
pub enum CollapseState {
    Collapsed,
    PartiallyCollapsed,
    Normal,
}

#[derive(Debug, Clone, Deserialize)]
/// A web article in a reader mode
pub struct Article {
    pub title: String,
    pub url: String,
    pub content: String,
    pub author: Option<String>,
    pub date_published: Option<String>,
}

impl Story {
    /// get the story's article URL.
    /// If the article URL is empty (in case of "AskHN" stories), fallback to the HN story's URL
    pub fn get_url(&self) -> Cow<str> {
        if self.url.is_empty() {
            Cow::from(self.story_url())
        } else {
            Cow::from(&self.url)
        }
    }

    pub fn story_url(&self) -> String {
        format!("{}/item?id={}", client::HN_HOST_URL, self.id)
    }
}

impl From<StoriesResponse> for Vec<Story> {
    fn from(s: StoriesResponse) -> Vec<Story> {
        s.hits
            .into_par_iter()
            .filter(|story| story.highlight_result.is_some())
            .map(|story| story.into())
            .collect()
    }
}

impl From<StoryResponse> for Story {
    fn from(s: StoryResponse) -> Self {
        let title = s
            .highlight_result
            .unwrap()
            .title
            .map(|r| r.value)
            .unwrap_or_default();
        let title = decode_html(&title);

        let mut parsed_title = StyledString::new();

        // parse the story title and decorate it based on the story category
        let title = {
            if let Some(title) = title.strip_prefix("Ask HN") {
                parsed_title
                    .append_styled("Ask HN", config::get_config_theme().component_style.ask_hn);
                title
            } else if let Some(title) = title.strip_prefix("Tell HN") {
                parsed_title.append_styled(
                    "Tell HN",
                    config::get_config_theme().component_style.tell_hn,
                );
                title
            } else if let Some(title) = title.strip_prefix("Show HN") {
                parsed_title.append_styled(
                    "Show HN",
                    config::get_config_theme().component_style.show_hn,
                );
                title
            } else if let Some(title) = title.strip_prefix("Launch HN") {
                parsed_title.append_styled(
                    "Launch HN",
                    config::get_config_theme().component_style.launch_hn,
                );
                title
            } else {
                &title
            }
        };

        // parse the story title that may contain search matches wrapped inside `<em>` tags
        // The matches are decorated with a corresponding style.
        {
            // an index such that `title[curr_pos..]` represents the part of the
            // text that hasn't been parsed.
            let mut curr_pos = 0;
            for caps in MATCH_RE.captures_iter(title) {
                let whole_match = caps.get(0).unwrap();
                // the part that doesn't match any patterns should be rendered in the default style
                if curr_pos < whole_match.start() {
                    parsed_title.append_plain(&title[curr_pos..whole_match.start()]);
                }
                curr_pos = whole_match.end();

                parsed_title.append_styled(
                    caps.name("match").unwrap().as_str(),
                    config::get_config_theme().component_style.matched_highlight,
                );
            }
            if curr_pos < title.len() {
                parsed_title.append_plain(&title[curr_pos..title.len()]);
            }
        }

        let author = s.author.unwrap_or_else(|| String::from("[deleted]"));

        // parse story's text
        let text = {
            let metadata = utils::combine_styled_strings(vec![
                StyledString::plain(parsed_title.source()),
                StyledString::plain("\n"),
                StyledString::styled(
                    format!(
                        "{} points | by {} | {} ago | {} comments\n",
                        s.points,
                        author,
                        utils::get_elapsed_time_as_text(s.time),
                        s.num_comments,
                    ),
                    config::get_config_theme().component_style.metadata,
                ),
            ]);

            // the HTML story text returned by HN Algolia API doesn't wrap a
            // paragraph inside a `<p><\p>` tag pair.
            // Instead, it seems to use `<p>` to represent a paragraph break.
            let mut story_text = decode_html(&s.text.unwrap_or_default()).replace("<p>", "\n\n");

            let minimized_text = if story_text.is_empty() {
                metadata.clone()
            } else {
                story_text = format!("\n{}", story_text);

                utils::combine_styled_strings(vec![
                    metadata.clone(),
                    StyledString::plain("... (more)"),
                ])
            };

            let mut text = metadata;
            let result = parse_hn_html_text(story_text, Style::default(), 0);
            text.append(result.s);

            HnText {
                id: s.id,
                level: 0,
                state: CollapseState::Normal,
                minimized_text,
                text,
                links: result.links,
            }
        };

        Story {
            title: parsed_title,
            url: s.url.unwrap_or_default(),
            author,
            id: s.id,
            points: s.points,
            num_comments: s.num_comments,
            time: s.time,
            text,
        }
    }
}

impl From<CommentResponse> for Vec<HnText> {
    fn from(c: CommentResponse) -> Self {
        // recursively parse child comments of the current comment
        let children = c
            .children
            .into_par_iter()
            .filter(|comment| comment.author.is_some() && comment.text.is_some())
            .flat_map(<Vec<HnText>>::from)
            .map(|mut c| {
                c.level += 1; // update the level of every children comments
                c
            })
            .collect::<Vec<_>>();

        // parse current comment
        let comment = {
            let metadata = utils::combine_styled_strings(vec![
                StyledString::styled(
                    c.author.unwrap_or_default(),
                    config::get_config_theme().component_style.username,
                ),
                StyledString::styled(
                    format!(" {} ago ", utils::get_elapsed_time_as_text(c.time)),
                    config::get_config_theme().component_style.metadata,
                ),
            ]);

            let mut text =
                utils::combine_styled_strings(vec![metadata.clone(), StyledString::plain("\n")]);

            let result = parse_hn_html_text(
                decode_html(&c.text.unwrap_or_default()),
                Style::default(),
                0,
            );
            text.append(result.s);

            HnText {
                id: c.id,
                level: 0,
                state: CollapseState::Normal,
                minimized_text: utils::combine_styled_strings(vec![
                    metadata,
                    StyledString::styled(
                        format!("({} more)", children.len() + 1),
                        config::get_config_theme().component_style.metadata,
                    ),
                ]),
                text,
                links: result.links,
            }
        };

        [vec![comment], children].concat()
    }
}

/// A HTML parsed result.
#[derive(Debug, Default)]
pub struct HTMLParsedResult {
    /// a styled string representing the decorated HTML content
    pub s: StyledString,
    /// a list of links inside the HTML document
    pub links: Vec<String>,
}

/// A HTML table parsed result.
#[derive(Debug, Default)]
pub struct HTMLTableParsedResult {
    /// a list of links inside the HTML document
    pub links: Vec<String>,
    /// parsed table headers
    pub headers: Vec<StyledString>,
    /// parsed table rows
    pub rows: Vec<Vec<StyledString>>,
}

impl HTMLParsedResult {
    pub fn merge(&mut self, mut other: HTMLParsedResult) {
        self.s.append(other.s);
        self.links.append(&mut other.links);
    }
}

#[derive(Debug, Clone)]
/// Additional arguments of the article parse function [`Article::parse()`]
struct ArticleParseArgs {
    /// A value indicates whether the current node is inside a `<pre>` tag.
    pub in_pre_node: bool,
    /// A value indicates whether a node is the first element of a block tag.
    /// This is mostly used to add newlines separating two consecutive elements in a block node.
    pub is_first_element_in_block: bool,
    /// A prefix string appended to each line of the current node's inner text.
    /// This is mostly used to decorate or indent elements inside specific nodes.
    pub prefix: String,
}

impl Default for ArticleParseArgs {
    fn default() -> Self {
        Self {
            in_pre_node: false,
            is_first_element_in_block: true,
            prefix: String::new(),
        }
    }
}

impl Article {
    /// Parses the article's HTML content
    ///
    /// # Arguments:
    /// * `max_width`: the maximum width of the parsed content. This is mostly used
    /// to construct a HTML table using `comfy_table`.
    pub fn parse(&self, max_width: usize) -> Result<HTMLParsedResult> {
        debug!("parse article ({:?})", self);

        // parse HTML content into DOM node(s)
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut (self.content.as_bytes()))?;

        let (mut result, _) = Self::parse_dom_node(
            dom.document,
            max_width,
            0,
            Style::default(),
            ArticleParseArgs::default(),
        );

        // process the links
        result.links = result
            .links
            .into_iter()
            .map(|l| {
                match url::Url::parse(&l) {
                    // Failed to parse the link, possibly because it's a relative link, (e.g `/a/b`).
                    // Try to convert the relative link into an absolute link.
                    Err(err) => {
                        debug!("failed to parse url {l}: {err}");
                        match url::Url::parse(&self.url).unwrap().join(&l) {
                            Ok(url) => url.to_string(),
                            Err(_) => l,
                        }
                    }
                    Ok(_) => l,
                }
            })
            .collect();

        Ok(result)
    }

    /// Parses a HTML DOM node.
    ///
    /// # Returns
    /// The function returns a HTML parsed result and a boolean value
    /// indicating whether the current node has a non-whitespace text.
    fn parse_dom_node(
        node: Handle,
        max_width: usize,
        base_link_id: usize,
        mut style: Style,
        mut args: ArticleParseArgs,
    ) -> (HTMLParsedResult, bool) {
        // TODO: handle parsing <ol> tags correctly

        debug!(
            "parse dom node: {:?}, style: {:?}, args: {:?}",
            node, style, args
        );

        let mut result = HTMLParsedResult::default();
        let mut suffix = StyledString::new();

        let mut visit_block_element_cb = || {
            if !args.is_first_element_in_block {
                result.s.append_plain("\n\n");
                result.s.append_styled(&args.prefix, style);
            }
            args.is_first_element_in_block = true;
        };

        let mut has_non_ws_text = false;

        match &node.data {
            NodeData::Text { contents } => {
                let content = contents.borrow().to_string();

                let text = if args.in_pre_node {
                    // add `prefix` to each line of the text inside the `<pre>` tag
                    content.replace('\n', &format!("\n{}", args.prefix))
                } else {
                    // Otherwise, consecutive whitespaces are ignored for non-pre elements.
                    // This is to prevent reader-mode engine from adding unneccesary line wraps/indents in a paragraph.
                    WS_RE.replace_all(&content, " ").to_string()
                };
                let text = decode_html(&text);
                debug!("visit text: {}", text);

                has_non_ws_text |= !text.trim().is_empty();

                result.s.append_styled(text, style);
            }
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                debug!("visit element: name={:?}, attrs: {:?}", name, attrs);

                let component_style = &config::get_config_theme().component_style;

                match name.expanded() {
                    expanded_name!(html "h1")
                    | expanded_name!(html "h2")
                    | expanded_name!(html "h3")
                    | expanded_name!(html "h4")
                    | expanded_name!(html "h5")
                    | expanded_name!(html "h6") => {
                        visit_block_element_cb();

                        style = style.combine(component_style.header);
                    }
                    expanded_name!(html "br") => {
                        result.s.append_styled(format!("\n{}", args.prefix), style);
                    }
                    expanded_name!(html "p") => visit_block_element_cb(),
                    expanded_name!(html "code") => {
                        if !args.in_pre_node {
                            // this assumes that `<code>` element that is not inside a pre node
                            // is a single-line code block.
                            style = style.combine(component_style.single_code_block);
                        }
                    }
                    expanded_name!(html "pre") => {
                        visit_block_element_cb();

                        args.in_pre_node = true;
                        args.prefix = format!("{}  ", args.prefix);

                        style = style.combine(component_style.multiline_code_block);

                        result.s.append_styled("  ", style);
                    }
                    expanded_name!(html "blockquote") => {
                        visit_block_element_cb();

                        args.prefix = format!("{}▎ ", args.prefix);
                        style = style.combine(component_style.quote);

                        result.s.append_styled("▎ ", style);
                    }
                    expanded_name!(html "table") => {
                        let mut table_result = HTMLTableParsedResult::default();
                        Self::parse_html_table(
                            node.clone(),
                            max_width,
                            base_link_id + result.links.len(),
                            style,
                            false,
                            &mut table_result,
                        );

                        result.links.append(&mut table_result.links);

                        let mut table = comfy_table::Table::new();
                        table
                            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
                            .set_table_width(max_width as u16)
                            .load_preset(comfy_table::presets::UTF8_FULL)
                            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
                            .apply_modifier(comfy_table::modifiers::UTF8_SOLID_INNER_BORDERS)
                            .set_header(
                                table_result
                                    .headers
                                    .into_iter()
                                    .map(|h| comfy_table::Cell::new(h.source()))
                                    .collect::<Vec<_>>(),
                            );

                        for row in table_result.rows {
                            table.add_row(row.into_iter().map(|c| c.source().to_owned()));
                        }

                        result.s.append_styled(format!("\n\n{}", table), style);

                        return (result, true);
                    }
                    expanded_name!(html "menu")
                    | expanded_name!(html "ul")
                    | expanded_name!(html "ol") => {
                        // currently, <ol> tag is treated the same as <ul> tag
                        args.prefix = format!("{}  ", args.prefix);
                    }
                    expanded_name!(html "li") => {
                        args.is_first_element_in_block = true;

                        result
                            .s
                            .append_styled(format!("\n{}• ", args.prefix), style);
                    }
                    expanded_name!(html "img") => {
                        let img_desc = if let Some(attr) = attrs
                            .borrow()
                            .iter()
                            .find(|&attr| attr.name.expanded() == expanded_name!("", "alt"))
                        {
                            attr.value.to_string()
                        } else {
                            String::new()
                        };

                        if !args.is_first_element_in_block {
                            result.s.append_plain("\n\n");
                        }
                        result.s.append_styled(&img_desc, style);
                        result.s.append_styled(" (image)", component_style.metadata);
                    }
                    expanded_name!(html "a") => {
                        // find `href` attribute of an <a> tag
                        if let Some(attr) = attrs
                            .borrow()
                            .iter()
                            .find(|&attr| attr.name.expanded() == expanded_name!("", "href"))
                        {
                            result.links.push(attr.value.clone().to_string());

                            suffix.append_styled(" ", style);
                            suffix.append_styled(
                                format!("[{}]", result.links.len() + base_link_id),
                                component_style.link_id,
                            );
                        }

                        style = style.combine(component_style.link);
                    }
                    expanded_name!(html "strong") => {
                        style = style.combine(component_style.bold);
                    }
                    expanded_name!(html "em") => {
                        style = style.combine(component_style.italic);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        node.children.borrow().iter().for_each(|node| {
            let (child_result, child_has_non_ws_text) = Self::parse_dom_node(
                node.clone(),
                max_width,
                base_link_id + result.links.len(),
                style,
                args.clone(),
            );

            result.merge(child_result);
            has_non_ws_text |= child_has_non_ws_text;
            if has_non_ws_text {
                args.is_first_element_in_block = false;
            }
        });

        result.s.append(suffix);
        (result, has_non_ws_text)
    }

    fn parse_html_table(
        node: Handle,
        max_width: usize,
        base_link_id: usize,
        style: Style,
        mut is_header: bool,
        result: &mut HTMLTableParsedResult,
    ) {
        debug!("parse html table: {:?}", node);

        if let NodeData::Element { name, .. } = &node.data {
            match name.expanded() {
                expanded_name!(html "thead") => {
                    is_header = true;
                }
                expanded_name!(html "tbody") => {
                    is_header = false;
                }
                expanded_name!(html "tr") => {
                    if !is_header {
                        result.rows.push(vec![]);
                    }
                }
                expanded_name!(html "td") | expanded_name!(html "th") => {
                    let mut s = StyledString::new();

                    node.children.borrow().iter().for_each(|node| {
                        let (mut child_result, _) = Self::parse_dom_node(
                            node.clone(),
                            max_width,
                            base_link_id + result.links.len(),
                            style,
                            ArticleParseArgs::default(),
                        );

                        result.links.append(&mut child_result.links);
                        s.append(child_result.s);
                    });

                    if !is_header {
                        result.rows.last_mut().unwrap().push(s);
                    } else {
                        result.headers.push(s);
                    }

                    return;
                }
                _ => {}
            }
        }

        node.children.borrow().iter().for_each(|node| {
            Self::parse_html_table(
                node.clone(),
                max_width,
                base_link_id + result.links.len(),
                style,
                is_header,
                result,
            );
        });
    }
}

/// decode a HTML encoded string
fn decode_html(s: &str) -> String {
    html_escape::decode_html_entities(s).into()
}

/// parse a Hacker News HTML text
fn parse_hn_html_text(text: String, style: Style, base_link_id: usize) -> HTMLParsedResult {
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
