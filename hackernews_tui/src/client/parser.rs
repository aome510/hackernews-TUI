use crate::{config, utils};
use serde::{de, Deserialize, Deserializer};

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
    #[serde(default)]
    matched_words: Vec<String>,
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

    #[serde(default)]
    children: Vec<CommentResponse>,

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
    #[serde(deserialize_with = "parse_null_default")]
    parent_id: u32,
    #[serde(default)]
    #[serde(deserialize_with = "parse_null_default")]
    story_id: u32,

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
    pub story_id: u32,
    pub parent_id: u32,
    pub text: String,
    pub author: String,
    pub time: u64,
    pub children: Vec<Comment>,
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

impl From<CommentResponse> for Comment {
    fn from(c: CommentResponse) -> Self {
        let children = c
            .children
            .into_iter()
            .filter(|comment| comment.author.is_some() && comment.text.is_some())
            .map(|comment| comment.into())
            .collect();
        let text: String = if !config::get_config().allow_unicode {
            c.text
                .unwrap()
                .chars()
                .filter(|c| utils::allow_unicode_char(c))
                .collect()
        } else {
            c.text.unwrap()
        };
        Comment {
            id: c.id,
            story_id: c.story_id,
            parent_id: c.parent_id,
            author: c.author.unwrap(),
            time: c.time,
            text,
            children,
        }
    }
}

impl From<StoryResponse> for Story {
    fn from(s: StoryResponse) -> Self {
        // need to make sure that highlight_result is not none,
        // and its title field is not none,
        let highlight_result_raw = s.highlight_result.unwrap();
        let highlight_result = HighlightResult {
            title: highlight_result_raw.title.unwrap().value,
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
            title: s.title.unwrap(),
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
