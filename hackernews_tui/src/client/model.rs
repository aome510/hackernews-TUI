use crate::utils::decode_html;

use super::*;
use serde::{de, Deserialize, Deserializer};

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

        let content = decode_html(&s.text.unwrap_or_default());

        Story {
            url: s.url.unwrap_or_default(),
            author: s.author.unwrap_or_default(),
            id: s.id,
            points: s.points,
            num_comments: s.num_comments,
            time: s.time,
            title,
            content,
        }
    }
}

impl From<CommentResponse> for Vec<Comment> {
    fn from(c: CommentResponse) -> Self {
        // recursively parse child comments of the current comment
        let children = c
            .children
            .into_par_iter()
            .filter(|comment| comment.author.is_some() && comment.text.is_some())
            .flat_map(<Vec<Comment>>::from)
            .map(|mut c| {
                c.level += 1; // update the level of every children comments
                c
            })
            .collect::<Vec<_>>();

        // parse current comment
        let comment = {
            Comment {
                id: c.id,
                level: 0,
                n_children: children.len(),
                time: c.time,
                author: c.author.unwrap_or_default(),
                content: decode_html(&c.text.unwrap_or_default()),
            }
        };

        [vec![comment], children].concat()
    }
}
