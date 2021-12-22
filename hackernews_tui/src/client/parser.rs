use crate::prelude::*;
use lazy_static::lazy_static;
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

    /// a regex used to parse a HN comment (in HTML format)
    /// It consists of multiple regex(s) representing different elements
    static ref COMMENT_RE: Regex = Regex::new(&format!(
        r"(({})|({})|({})|({})|({})|({}))",
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

/// Story represents a parsed Hacker News story
#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub title: StyledString,
    pub url: String,
    pub author: String,
    pub points: u32,
    pub num_comments: usize,
    pub time: u64,
}

/// Comment represents a parsed Hacker News comment
#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u32,
    pub height: usize,
    pub state: CommentState,
    pub text: StyledString,
    pub minimized_text: StyledString,
    pub links: Vec<String>,
}

#[derive(Debug, Clone)]
/// CommentState represents the state of a single comment component
pub enum CommentState {
    Collapsed,
    PartiallyCollapsed,
    Normal,
}

#[derive(Debug, Clone, Deserialize)]
/// Article represents a web article in a reader mode
/// with its text content in a markdown format.
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

        Story {
            title: parsed_title,
            url: s.url.unwrap_or_default(),
            author: s.author.unwrap_or_else(|| String::from("[deleted]")),
            id: s.id,
            points: s.points,
            num_comments: s.num_comments,
            time: s.time,
        }
    }
}

impl From<CommentResponse> for Vec<Comment> {
    fn from(c: CommentResponse) -> Self {
        let mut children = c
            .children
            .into_iter()
            .filter(|comment| comment.author.is_some() && comment.text.is_some())
            .flat_map(Into::<Vec<Comment>>::into)
            .map(|mut c| {
                c.height += 1; // update the height of every children comments
                c
            })
            .collect::<Vec<_>>();

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
        let (text, links) = parse_raw_html_comment(&c.text.unwrap_or_default(), metadata.clone());

        let comment = Comment {
            id: c.id,
            height: 0,
            state: CommentState::Normal,
            text,
            minimized_text: utils::combine_styled_strings(vec![
                metadata,
                StyledString::styled(
                    format!("({} more)", children.len() + 1),
                    config::get_config_theme().component_style.metadata,
                ),
            ]),
            links,
        };

        let mut comments = vec![comment];
        comments.append(&mut children);
        comments
    }
}

impl Article {
    pub fn parse(&mut self) -> Result<()> {
        // replace a tab character by 4 spaces
        // because it's possible that the terminal cannot render the tab character
        self.content = self.content.replace("\t", "    ");

        debug!("article (url={}), content: {}", self.url, self.content);

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut (self.content.as_bytes()))?;

        let (s, links) = Self::parse_html(dom.document, Style::default(), 0, false, String::new());
        self.parsed_content = s;
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

    fn parse_html(
        node: Handle,
        mut style: Style,
        begin_link_id: usize,
        mut is_pre: bool,
        mut prefix: String,
    ) -> (StyledString, Vec<String>) {
        // TODO: handle parsing <ol>, <table> tags correctly
        debug!("parse node: {:?}", node);

        let mut s = StyledString::new();
        let mut suffix = StyledString::new();
        let mut links = vec![];

        match &node.data {
            NodeData::Text { contents } => {
                let content = contents.borrow().to_string();
                let text = if is_pre {
                    // ident `pre` block by 2 ws character
                    content.trim_matches('\n').to_string().replace("\n", "\n  ")
                } else {
                    // for non-pre element, consecutive whitespaces are ignored.
                    // This is to prevent reader-mode engine to add unneccesary line wraps/idents for each paragraph.
                    WS_RE.replace_all(&content, " ").to_string()
                };

                debug!("parse HTML text: {}", text,);
                suffix.append_styled(decode_html(&text), style);
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
                        style = style.combine(component_style.header);

                        s.append_plain("\n\n");
                    }
                    expanded_name!(html "br") => {
                        suffix.append_plain("\n");
                    }
                    expanded_name!(html "p") => {
                        s.append_styled(format!("\n\n{}", prefix), style);
                    }
                    expanded_name!(html "code") => {
                        if !is_pre {
                            style = style.combine(component_style.single_code_block);
                        }
                    }
                    expanded_name!(html "pre") => {
                        is_pre = true;
                        style = style.combine(component_style.multiline_code_block);

                        // ident `pre` block by 2 ws character
                        s.append_plain("\n\n  ");
                    }
                    expanded_name!(html "table") => {
                        // in the meantime, parsing `table` tag is not supported
                        s.append_styled("\n\n  (table)", component_style.metadata);
                        return (s, links);
                    }
                    expanded_name!(html "ul") | expanded_name!(html "ol") => {
                        prefix = format!("{}  ", prefix);
                    }
                    expanded_name!(html "li") => {
                        s.append_styled(format!("\n{}• ", prefix), style);
                    }
                    expanded_name!(html "blockquote") => {
                        prefix = format!("{}▎ ", prefix);
                        style = style.combine(component_style.quote);
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
                        s.append_styled(format!("\n  {}", img_desc), style);
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
                                format!("[{}]", links.len() + begin_link_id),
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
            let (sub_s, mut sub_links) = Self::parse_html(
                node.clone(),
                style,
                begin_link_id + links.len(),
                is_pre,
                prefix.clone(),
            );

            s.append(sub_s);
            links.append(&mut sub_links);
        });
        s.append(suffix);

        (s, links)
    }
}

/// Decode a HTML encoded string
fn decode_html(s: &str) -> String {
    html_escape::decode_html_entities(s).into()
}

/// Parse a raw HTML comment text into a styled string.
/// The function also returns a list of links in the comment.
fn parse_raw_html_comment(text: &str, metadata: StyledString) -> (StyledString, Vec<String>) {
    debug!("parse raw html comment: {}", text);

    let text = decode_html(text);

    let mut s = utils::combine_styled_strings(vec![metadata, StyledString::plain("\n")]);
    let (s0, links) = parse_comment_text(text, Style::default(), 0);

    s.append(s0);
    (s, links)
}

/// a helper function for parsing comment text that allows recursively parsing sub elements of the text.
fn parse_comment_text(
    text: String,
    style: Style,
    begin_link_id: usize,
) -> (StyledString, Vec<String>) {
    debug!("parse comment text: {}", text);

    let mut curr_pos = 0;
    let mut s = StyledString::new();
    let mut links = vec![];

    // This variable indicates whether we have parsed the first paragraph of the current text.
    // It is used to add a break between 2 consecutive paragraphs.
    let mut seen_first_paragraph = false;

    for caps in COMMENT_RE.captures_iter(&text) {
        let match_s = {
            if let (Some(m_quote), Some(m_text)) = (caps.name("quote"), caps.name("text")) {
                if seen_first_paragraph {
                    s.append_styled("\n", style);
                } else {
                    seen_first_paragraph = true;
                }

                // render quote character `>` as indentation character
                let quote_s = StyledString::styled(
                    "▎"
                        .to_string()
                        .repeat(m_quote.as_str().matches('>').count()),
                    style,
                );

                let (sub_s, mut sub_links) = parse_comment_text(
                    m_text.as_str().to_string(),
                    config::get_config_theme().component_style.quote.into(),
                    links.len(),
                );
                links.append(&mut sub_links);

                utils::combine_styled_strings(vec![quote_s, sub_s, StyledString::plain("\n")])
            } else if let Some(m) = caps.name("paragraph") {
                if seen_first_paragraph {
                    s.append_styled("\n", style);
                } else {
                    seen_first_paragraph = true;
                }

                let (sub_s, mut sub_links) =
                    parse_comment_text(m.as_str().to_string(), style, links.len());
                links.append(&mut sub_links);

                utils::combine_styled_strings(vec![sub_s, StyledString::plain("\n")])
            } else if let Some(m) = caps.name("link") {
                links.push(m.as_str().to_string());

                utils::combine_styled_strings(vec![
                    StyledString::styled(
                        utils::shorten_url(m.as_str()),
                        style.combine(config::get_config_theme().component_style.link),
                    ),
                    StyledString::plain(" "),
                    StyledString::styled(
                        format!("[{}]", links.len() + begin_link_id),
                        style.combine(config::get_config_theme().component_style.link_id),
                    ),
                ])
            } else if let Some(m) = caps.name("multiline_code") {
                StyledString::styled(
                    m.as_str(),
                    style.combine(
                        config::get_config_theme()
                            .component_style
                            .multiline_code_block,
                    ),
                )
            } else if let Some(m) = caps.name("code") {
                StyledString::styled(
                    m.as_str(),
                    style.combine(config::get_config_theme().component_style.single_code_block),
                )
            } else if let Some(m) = caps.name("italic") {
                StyledString::styled(
                    m.as_str(),
                    style.combine(config::get_config_theme().component_style.italic),
                )
            } else {
                unreachable!()
            }
        };

        let whole_match = caps.get(0).unwrap();
        // the part that doesn't match any patterns should be rendered in the default style
        if curr_pos < whole_match.start() {
            s.append_styled(&text[curr_pos..whole_match.start()], style);
        }
        curr_pos = whole_match.end();

        s.append(match_s);
    }

    if curr_pos < text.len() {
        s.append_styled(&text[curr_pos..text.len()], style);
    }
    (s, links)
}
