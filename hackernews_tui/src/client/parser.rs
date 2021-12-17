use crate::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de, Deserialize, Deserializer};

lazy_static! {
    /// a regex that matches a search match in the response from HN Algolia search API
    static ref MATCH_RE: Regex = Regex::new(r"<em>(?P<match>.*?)</em>").unwrap();

    /// a regex used to parse a HN comment (in HTML format)
    /// It consists of multiple regex(s) representing different elements
    static ref COMMENT_RE: Regex = Regex::new(&format!(
        "(({})|({})|({})|({})|({}))",
        // a regex that matches a HTML paragraph
        r"<p>(?s)(?P<paragraph>(|[^>].*?))</p>",
        // a regex that matches a paragraph quote (in markdown format)
        r"<p>(?s)(?P<quote>>[> ]*)(?P<text>.*?)</p>",
        // a regex that matches an HTML italic string
        r"<i>(?s)(?P<italic>.*?)</i>",
        // a regex that matches a HTML code block (multiline)
        r"<pre><code>(?s)(?P<code>.*?)[\n]*</code></pre>",
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
    url: Option<MatchResult>,
    author: Option<MatchResult>,
}

#[derive(Debug, Deserialize)]
/// StoryResponse represents the story data received from HN_ALGOLIA APIs
pub struct StoryResponse {
    #[serde(default)]
    #[serde(rename(deserialize = "objectID"))]
    #[serde(deserialize_with = "parse_id")]
    id: u32,

    title: Option<String>,
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

// HN client get Story,Comment data by calling HN_ALGOLIA APIs
// and parsing the result into a corresponding struct

/// HighlightResult represents matched results when
/// searching stories matching certain conditions
#[derive(Debug, Clone)]
pub struct HighlightResult {
    pub title: String,
    pub url: String,
    pub author: String,
}

/// Story represents a Hacker News story
#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub author: String,
    pub points: u32,
    pub num_comments: usize,
    pub time: u64,
    pub highlight_result: HighlightResult,
}

/// Comment represents a Hacker News comment
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

impl From<StoriesResponse> for Vec<Story> {
    fn from(s: StoriesResponse) -> Vec<Story> {
        s.hits
            .into_iter()
            .filter(|story| story.highlight_result.is_some() && story.title.is_some())
            .map(|story| story.into())
            .collect()
    }
}

impl From<StoryResponse> for Story {
    fn from(s: StoryResponse) -> Self {
        // need to make sure that highlight_result is not none,
        // and its title field is not none,
        let highlight_result_raw = s.highlight_result.unwrap();
        let highlight_result = HighlightResult {
            title: decode_html(&highlight_result_raw.title.unwrap().value),
            url: match highlight_result_raw.url {
                None => String::new(),
                Some(url) => url.value,
            },
            author: match highlight_result_raw.author {
                None => String::from("[deleted]"),
                Some(author) => author.value,
            },
        };
        Story {
            title: MATCH_RE
                .replace_all(&s.title.unwrap(), "${match}")
                .to_string(),
            url: s.url.unwrap_or_default(),
            author: s.author.unwrap_or_else(|| String::from("[deleted]")),
            id: s.id,
            points: s.points,
            num_comments: s.num_comments,
            time: s.time,
            highlight_result,
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

/// Decode a HTML encoded string
fn decode_html(s: &str) -> String {
    htmlescape::decode_html(s).unwrap_or_else(|_| s.to_string())
}

/// Parse a raw HTML comment text into a styled string.
/// The function also returns a list of links in the comment.
fn parse_raw_html_comment(text: &str, metadata: StyledString) -> (StyledString, Vec<String>) {
    let text = decode_html(text);

    let mut s = utils::combine_styled_strings(vec![metadata, StyledString::plain("\n")]);
    let (s0, links) = parse(text, Style::default(), 0);

    s.append(s0);
    (s, links)
}

/// a helper function for parsing comment text that allows recursively parsing sub elements of the text.
fn parse(text: String, style: Style, begin_link_id: usize) -> (StyledString, Vec<String>) {
    debug!("parse {}", text);

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
                info!("{}", m_quote.as_str());
                let quote_s = StyledString::styled(
                    "▎"
                        .to_string()
                        .repeat(m_quote.as_str().matches('>').count()),
                    style,
                );

                let (sub_s, mut sub_links) = parse(
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

                let (sub_s, mut sub_links) = parse(m.as_str().to_string(), style, links.len());
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
            } else if let Some(m) = caps.name("code") {
                StyledString::styled(
                    m.as_str(),
                    style.combine(
                        config::get_config_theme()
                            .component_style
                            .multiline_code_block,
                    ),
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
