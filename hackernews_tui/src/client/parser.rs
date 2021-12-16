use crate::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de, Deserialize, Deserializer};

lazy_static! {
    static ref MATCH_RE: Regex = Regex::new(r"<em>(?P<match>.*?)</em>").unwrap();
    static ref PARAGRAPH_RE: Regex = Regex::new(r"<p>(?s)(?P<paragraph>.*?)</p>").unwrap();
    static ref ITALIC_RE: Regex = Regex::new(r"<i>(?s)(?P<text>.+?)</i>").unwrap();
    static ref CODE_RE: Regex =
        Regex::new(r"<pre><code>(?s)(?P<code>.+?)[\n]*</code></pre>").unwrap();
    static ref LINK_RE: Regex = Regex::new(r#"<a\s+?href="(?P<link>.+?)"(?s).+?</a>"#).unwrap();
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

/// Parse a raw HTML comment text into a markdown text with colors.
/// The function returns the parsed text and a vector of links in the comment.
///
/// Links inside the parsed text are colored.
fn parse_raw_html_comment(text: &str, metadata: StyledString) -> (StyledString, Vec<String>) {
    // insert newlines as a separator between paragraphs
    let mut s = PARAGRAPH_RE
        .replace_all(text, "${paragraph}\n\n")
        .to_string();
    // we already have bottom margin between two consecutive comments,
    // so no need to add an additional newline at the end of a comment.
    if s.ends_with("\n\n") {
        s.remove(s.len() - 1);
    }

    s = ITALIC_RE.replace_all(&s, "*${text}*").to_string();
    s = CODE_RE.replace_all(&s, "```\n${code}\n```").to_string();

    let mut result = metadata;
    result.append_plain("\n");

    // parse links in the comment, color them in the parsed text as well
    let mut links: Vec<String> = vec![];

    // replace the `<a href="${link}">...</a>` pattern one-by-one with "${link}".
    // cannot use `replace_all` because we want to replace a matched string with a `StyledString` (not a raw string)
    loop {
        match LINK_RE.captures(&s.clone()) {
            None => break,
            Some(c) => {
                let m = c.get(0).unwrap();
                let link = decode_html(c.name("link").unwrap().as_str());

                let range = m.range();
                let mut prefix: String = s
                    .drain(std::ops::Range {
                        start: 0,
                        end: m.end(),
                    })
                    .collect();
                prefix.drain(range);

                if !prefix.is_empty() {
                    result.append_plain(decode_html(&prefix));
                }

                result.append_styled(
                    format!("\"{}\" ", utils::shorten_url(&link)),
                    config::get_config_theme().component_style.link,
                );
                result.append_styled(
                    format!("[{}]", links.len()),
                    config::get_config_theme().component_style.link_id,
                );
                links.push(link);
                continue;
            }
        }
    }
    if !s.is_empty() {
        result.append_plain(decode_html(&s));
    }
    (result, links)
}
