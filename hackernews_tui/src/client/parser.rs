use crate::prelude::*;
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
    pub minimized_text: StyledString,
    pub links: Vec<String>,
}

#[derive(Debug, Clone)]
/// The collapse state of a text component
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

    #[serde(skip)]
    pub parsed_content: StyledString,
    #[serde(skip)]
    pub links: Vec<String>,
}

impl From<StoriesResponse> for Vec<Story> {
    fn from(s: StoriesResponse) -> Vec<Story> {
        s.hits
            .into_iter()
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

        let title = {
            // parse story title based on the post's category
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

        // parse story title that may contain search matches wrapped inside `<em>` tags
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

            // the HTML story text returned by HN Algolia API doesn't wrap a paragraph
            // inside the `<p><\p>` tag pair.
            // Instead, it seems to use `<p>` to indicate a paragraph break.
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
            let mut links = vec![];
            parse_hn_html_text(story_text, Style::default(), &mut text, &mut links);

            HnText {
                id: s.id,
                level: 0,
                state: CollapseState::Normal,
                minimized_text,
                text,
                links,
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
        let mut children = c
            .children
            .into_par_iter()
            .filter(|comment| comment.author.is_some() && comment.text.is_some())
            .flat_map(<Vec<HnText>>::from)
            .map(|mut c| {
                c.level += 1; // update the height of every children comments
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
            let mut links = vec![];

            parse_hn_html_text(
                decode_html(&c.text.unwrap_or_default()),
                Style::default(),
                &mut text,
                &mut links,
            );

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
                links,
            }
        };

        let mut comments = vec![comment];
        comments.append(&mut children);
        comments
    }
}

impl Article {
    /// Parse article's content (in HTML) into a styled text depending on
    /// the application's component styles and the HTML tags
    pub fn parse(&mut self, width: usize) -> Result<()> {
        // replace a tab character by 4 spaces
        // as it's possible that the terminal cannot render the tab character
        self.content = self.content.replace("\t", "    ");

        debug!(
            "parse article (url={}), width: {}, content: {}",
            self.url, width, self.content
        );

        // parse HTML content into DOM node(s)
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut (self.content.as_bytes()))?;

        let mut s = StyledString::new();
        let mut links = vec![];
        Self::parse_dom_node(
            dom.document,
            width,
            &mut s,
            &mut links,
            Style::default(),
            false,
            true,
            String::new(),
        );

        self.parsed_content = s;

        // process the links inside the article
        self.links = links
            .into_iter()
            .map(|l| {
                match url::Url::parse(&l) {
                    // failed to parse the link, possibly a relative link, (e.g `/a/b`)
                    Err(_) => match url::Url::parse(&self.url).unwrap().join(&l) {
                        Ok(url) => url.to_string(),
                        Err(_) => String::new(),
                    },
                    Ok(_) => l,
                }
            })
            .collect();

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_dom_node(
        node: Handle,
        width: usize,
        s: &mut StyledString,
        links: &mut Vec<String>,
        mut style: Style,
        // indicates whether a node is inside a <pre> tag
        mut in_pre: bool,
        // indicates whether a node is the first element of a block tag
        // this is mostly used to add newlines separating a block tag and its sibling tags
        mut is_first_element_in_block: bool,
        // a string added to non-first children of the current block element,
        // the first child must be handled separately
        mut prefix: String,
    ) -> bool {
        // TODO: handle parsing <ol> tags correctly
        debug!("parse dom node: {:?}", node);

        let mut suffix = StyledString::new();
        let mut visit_block_element_cb = || {
            if !is_first_element_in_block {
                s.append_plain("\n\n");
                s.append_styled(&prefix, style);
            }
            is_first_element_in_block = true;
        };

        // `found_text` is the return value of the function.
        // It is used to indicate whether the current node has renderable and non-whitespace text inside.
        let mut found_text = false;

        match &node.data {
            NodeData::Text { contents } => {
                let content = contents.borrow().to_string();

                let text = if in_pre {
                    // add `prefix` to each line of the text inside the `<pre>` tag
                    content.replace("\n", &format!("\n{}", prefix))
                } else {
                    // for non-pre element, consecutive whitespaces are ignored.
                    // This is to prevent reader-mode engine from adding unneccesary line wraps/indents in a paragraph.
                    WS_RE.replace_all(&content, " ").to_string()
                };

                found_text |= !text.trim().is_empty();

                debug!("visit text: {}", text);
                s.append_styled(decode_html(&text), style);
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
                        s.append_styled(format!("\n{}", prefix), style);
                    }
                    expanded_name!(html "p") => visit_block_element_cb(),
                    expanded_name!(html "code") => {
                        if !in_pre {
                            // we don't want to mix the `single_code_block` and `multiline_code_block` styles
                            // if the multiline code block is inside <pre><code>...</code></pre>
                            style = style.combine(component_style.single_code_block);
                        }
                    }
                    expanded_name!(html "pre") => {
                        visit_block_element_cb();

                        in_pre = true;
                        style = style.combine(component_style.multiline_code_block);
                        prefix = format!("{}  ", prefix);
                        s.append_styled("  ", style);
                    }
                    expanded_name!(html "blockquote") => {
                        visit_block_element_cb();

                        style = style.combine(component_style.quote);
                        prefix = format!("{}▎ ", prefix);
                        s.append_styled("▎ ", style);
                    }
                    expanded_name!(html "table") => {
                        let mut headers = vec![];
                        let mut rows = vec![];

                        node.children.borrow().iter().for_each(|node| {
                            Self::parse_html_table(
                                node.clone(),
                                width,
                                links,
                                &mut headers,
                                &mut rows,
                                style,
                                false,
                            );
                        });

                        let mut table = comfy_table::Table::new();
                        table
                            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
                            .set_table_width(width as u16)
                            .load_preset(comfy_table::presets::UTF8_FULL)
                            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
                            .apply_modifier(comfy_table::modifiers::UTF8_SOLID_INNER_BORDERS)
                            .set_header(
                                headers
                                    .into_iter()
                                    .map(|h| comfy_table::Cell::new(h.source()))
                                    .collect::<Vec<_>>(),
                            );

                        for row in rows {
                            table.add_row(row.into_iter().map(|c| c.source().to_owned()));
                        }

                        s.append_styled(format!("\n\n{}", table), style);

                        return true;
                    }
                    expanded_name!(html "menu")
                    | expanded_name!(html "ul")
                    | expanded_name!(html "ol") => {
                        // currently, <ol> tag is treated the same as <ul> tag
                        prefix = format!("{}  ", prefix);
                    }
                    expanded_name!(html "li") => {
                        s.append_styled(format!("\n{}• ", prefix), style);
                        is_first_element_in_block = true;
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

                        if !is_first_element_in_block {
                            s.append_plain("\n\n");
                        }
                        s.append_styled(&img_desc, style);
                        s.append_styled(" (image)", component_style.metadata);
                    }
                    expanded_name!(html "a") => {
                        // find `href` attribute of an <a> tag
                        if let Some(attr) = attrs
                            .borrow()
                            .iter()
                            .find(|&attr| attr.name.expanded() == expanded_name!("", "href"))
                        {
                            links.push(attr.value.clone().to_string());

                            suffix.append_styled(" ", style);
                            suffix.append_styled(
                                format!("[{}]", links.len()),
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
            found_text |= Self::parse_dom_node(
                node.clone(),
                width,
                s,
                links,
                style,
                in_pre,
                is_first_element_in_block,
                prefix.clone(),
            );
            if found_text {
                is_first_element_in_block = false;
            }
        });

        s.append(suffix);
        found_text
    }

    fn parse_html_table(
        node: Handle,
        width: usize,
        links: &mut Vec<String>,
        headers: &mut Vec<StyledString>,
        rows: &mut Vec<Vec<StyledString>>,
        style: Style,
        mut is_header: bool,
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
                        rows.push(vec![]);
                    }
                }
                expanded_name!(html "td") | expanded_name!(html "th") => {
                    let mut s = StyledString::new();

                    node.children.borrow().iter().for_each(|node| {
                        Self::parse_dom_node(
                            node.clone(),
                            width,
                            &mut s,
                            links,
                            style,
                            false,
                            true,
                            String::new(),
                        );
                    });

                    if !is_header {
                        rows.last_mut().unwrap().push(s);
                    } else {
                        headers.push(s);
                    }
                    return;
                }
                _ => {}
            }
        }

        node.children.borrow().iter().for_each(|node| {
            Self::parse_html_table(node.clone(), width, links, headers, rows, style, is_header);
        });
    }
}

/// decode a HTML encoded string
fn decode_html(s: &str) -> String {
    html_escape::decode_html_entities(s).into()
}

/// parse a Hacker News HTML text
fn parse_hn_html_text(text: String, style: Style, s: &mut StyledString, links: &mut Vec<String>) {
    debug!("parse hn html text: {}", text);

    let mut curr_pos = 0;

    // This variable indicates whether we have parsed the first paragraph of the current text.
    // It is used to add a break between 2 consecutive paragraphs.
    let mut seen_first_paragraph = false;

    for caps in HN_TEXT_RE.captures_iter(&text) {
        // the part that doesn't match any patterns is rendered in the default style
        let whole_match = caps.get(0).unwrap();
        if curr_pos < whole_match.start() {
            s.append_styled(&text[curr_pos..whole_match.start()], style);
        }
        curr_pos = whole_match.end();

        let component_style = &config::get_config_theme().component_style;

        if let (Some(m_quote), Some(m_text)) = (caps.name("quote"), caps.name("text")) {
            if seen_first_paragraph {
                s.append_plain("\n");
            } else {
                seen_first_paragraph = true;
            }

            // render quote character `>` as indentation character
            s.append_styled(
                "▎"
                    .to_string()
                    .repeat(m_quote.as_str().matches('>').count()),
                style,
            );
            parse_hn_html_text(
                m_text.as_str().to_string(),
                component_style.quote.into(),
                s,
                links,
            );

            s.append_plain("\n");
        } else if let Some(m) = caps.name("paragraph") {
            if seen_first_paragraph {
                s.append_plain("\n");
            } else {
                seen_first_paragraph = true;
            }

            parse_hn_html_text(m.as_str().to_string(), style, s, links);

            s.append_plain("\n");
        } else if let Some(m) = caps.name("link") {
            links.push(m.as_str().to_string());

            s.append_styled(
                utils::shorten_url(m.as_str()),
                style.combine(component_style.link),
            );
            s.append_styled(" ", style);
            s.append_styled(
                format!("[{}]", links.len()),
                style.combine(component_style.link_id),
            );
        } else if let Some(m) = caps.name("multiline_code") {
            s.append_styled(
                m.as_str(),
                style.combine(component_style.multiline_code_block),
            );
            s.append_plain("\n");
        } else if let Some(m) = caps.name("code") {
            s.append_styled(m.as_str(), style.combine(component_style.single_code_block));
        } else if let Some(m) = caps.name("italic") {
            s.append_styled(m.as_str(), style.combine(component_style.italic));
        }
    }

    if curr_pos < text.len() {
        s.append_styled(&text[curr_pos..text.len()], style);
    }
}
