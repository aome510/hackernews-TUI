use rayon::prelude::*;
use serde::{de, Deserialize, Deserializer};
use std::{collections::HashMap, sync::Arc, sync::RwLock};
use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};

use crate::prelude::*;

const HN_ALGOLIA_PREFIX: &'static str = "https://hn.algolia.com/api/v1";
const HN_OFFICIAL_PREFIX: &'static str = "https://hacker-news.firebaseio.com/v0";
const HN_SEARCH_QUERY_STRING: &'static str =
    "tags=story&restrictSearchableAttributes=title,url&typoTolerance=false";
pub const HN_HOST_URL: &'static str = "https://news.ycombinator.com";

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
struct StoryResponse {
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
/// CommentResponse represents the comment data received from HN_ALGOLIA APIs
struct CommentResponse {
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
struct StoriesResponse {
    pub hits: Vec<StoryResponse>,
}

impl StoriesResponse {
    pub fn parse_into_stories(self) -> Vec<Story> {
        self.hits
            .into_par_iter()
            .filter(|story| story.highlight_result.is_some() && story.title.is_some())
            .map(|story| story.into())
            .collect()
    }
}

// parsed structs

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

impl From<CommentResponse> for Comment {
    fn from(c: CommentResponse) -> Self {
        let children = c
            .children
            .into_par_iter()
            .filter(|comment| comment.author.is_some() && comment.text.is_some())
            .map(|comment| comment.into())
            .collect();
        Comment {
            id: c.id,
            story_id: c.story_id,
            parent_id: c.parent_id,
            text: c.text.unwrap(),
            author: c.author.unwrap(),
            time: c.time,
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
            author: s.author.unwrap_or(String::from("[deleted]")),
            id: s.id,
            points: s.points,
            num_comments: s.num_comments,
            time: s.time,
            highlight_result,
        }
    }
}

// HN client get Story,Comment data by calling HN_ALGOLIA APIs
// and parsing the result into a corresponding struct

/// StoryCache is a cache storing all comments of a HN story.
/// A story cache will be updated if the number of commments changes.
#[derive(Debug, Clone)]
pub struct StoryCache {
    // num_comments is an approximated number of comments received from HN APIs,
    // it can be different from number of comments in the [comments] field below
    pub num_comments: usize,
    pub comments: Vec<Comment>,
}

impl StoryCache {
    pub fn new(comments: Vec<Comment>, num_comments: usize) -> Self {
        StoryCache {
            num_comments,
            comments,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct FilterInterval<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T: Display + Copy> FilterInterval<T> {
    pub fn query(&self, field: &str) -> String {
        format!(
            "{}{}",
            match self.start {
                Some(x) => format!(",{}>={}", field, x),
                None => "".to_string(),
            },
            match self.end {
                Some(x) => format!(",{}<{}", field, x),
                None => "".to_string(),
            },
        )
    }

    pub fn desc(&self, field: &str) -> String {
        format!(
            "{}: [{}:{}]",
            field,
            match self.start {
                Some(x) => x.to_string(),
                None => "".to_string(),
            },
            match self.end {
                Some(x) => x.to_string(),
                None => "".to_string(),
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct StoryNumericFilters {
    elapsed_days_interval: FilterInterval<u32>,
    points_interval: FilterInterval<u32>,
    num_comments_interval: FilterInterval<usize>,
}

impl StoryNumericFilters {
    fn from_elapsed_days_to_unix_time(elapsed_days: Option<u32>) -> Option<u64> {
        match elapsed_days {
            None => None,
            Some(day_offset) => {
                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Some(current_time - from_day_offset_to_time_offset_in_secs(day_offset))
            }
        }
    }

    pub fn desc(&self) -> String {
        format!(
            "{}, {}, {}",
            self.elapsed_days_interval.desc("elapsed_days"),
            self.points_interval.desc("points"),
            self.num_comments_interval.desc("num_comments")
        )
    }

    pub fn query(&self) -> String {
        // convert elapsed_days to unix time (in seconds)
        let time_interval = FilterInterval {
            end: Self::from_elapsed_days_to_unix_time(self.elapsed_days_interval.start),
            start: Self::from_elapsed_days_to_unix_time(self.elapsed_days_interval.end),
        };

        let mut query = format!(
            "{}{}{}",
            time_interval.query("created_at_i"),
            self.points_interval.query("points"),
            self.num_comments_interval.query("num_comments")
        );
        if query.len() > 0 {
            query.remove(0); // remove trailing ,
            format!("&numericFilters={}", query)
        } else {
            "".to_string()
        }
    }
}

/// HNClient is a HTTP client to communicate with Hacker News APIs.
#[derive(Clone)]
pub struct HNClient {
    client: ureq::Agent,
    story_caches: Arc<RwLock<HashMap<u32, StoryCache>>>,
}

impl HNClient {
    /// Create a new Hacker News Client
    pub fn new() -> Result<HNClient> {
        let timeout = get_config().client.client_timeout;
        Ok(HNClient {
            client: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(timeout))
                .build(),
            story_caches: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Return the story caches stored in the client
    pub fn get_story_caches(&self) -> &Arc<RwLock<HashMap<u32, StoryCache>>> {
        &self.story_caches
    }

    /// Get data of a HN item based on its id then parse the data
    /// to a corresponding struct representing that item
    pub fn get_item_from_id<T>(&self, id: u32) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        let request_url = format!("{}/items/{}", HN_ALGOLIA_PREFIX, id);
        Ok(self.client.get(&request_url).call()?.into_json::<T>()?)
    }

    /// Get all comments from a story.
    /// A cached result is returned if the number of comments doesn't change.
    /// [reload] is a parameter used when we want to re-determine the number of comments in a story.
    pub fn get_comments_from_story(&self, story: &Story, reload: bool) -> Result<Vec<Comment>> {
        let id = story.id;
        let story = if reload {
            self.get_story_from_story_id(id)?
        } else {
            story.clone()
        };

        let num_comments = story.num_comments;
        let comments = match self.story_caches.read().unwrap().get(&id) {
            Some(story_cache) if story_cache.num_comments == num_comments => {
                Some(story_cache.comments.clone())
            }
            _ => None,
        };

        match comments {
            None => match self.get_comments_from_story_id(id) {
                Ok(comments) => {
                    self.story_caches
                        .write()
                        .unwrap()
                        .insert(id, StoryCache::new(comments.clone(), num_comments));
                    info!("insert comments of the story (id={}, num_comments={}) into client's story_caches",
                           id, num_comments);
                    Ok(comments)
                }
                Err(err) => Err(err),
            },
            Some(comments) => Ok(comments),
        }
    }

    /// Get all comments from a story with a given id
    pub fn get_comments_from_story_id(&self, id: u32) -> Result<Vec<Comment>> {
        let time = SystemTime::now();
        let response = self.get_item_from_id::<StoryResponse>(id)?;
        if let Ok(elapsed) = time.elapsed() {
            info!(
                "get comments from story (id={}) took {}ms",
                id,
                elapsed.as_millis()
            );
        }

        Ok(response
            .children
            .into_par_iter()
            .filter(|comment| comment.text.is_some() && comment.author.is_some())
            .map(|comment| comment.into())
            .collect())
    }

    /// Get a story based on its id
    pub fn get_story_from_story_id(&self, id: u32) -> Result<Story> {
        let request_url = format!("{}/search?tags=story,story_{}", HN_ALGOLIA_PREFIX, id);
        let time = SystemTime::now();
        let response = self
            .client
            .get(&request_url)
            .call()?
            .into_json::<StoriesResponse>()?;
        if let Ok(elapsed) = time.elapsed() {
            info!("get story (id={}) took {}ms", id, elapsed.as_millis());
        }

        let stories = response.parse_into_stories();
        Ok(stories.first().unwrap().clone())
    }

    /// Get a list of stories matching certain conditions
    pub fn get_matched_stories(
        &self,
        query: &str,
        by_date: bool,
        page: usize,
    ) -> Result<Vec<Story>> {
        let search_story_limit = get_config().client.story_limit.search;
        let request_url = format!(
            "{}/{}?{}&hitsPerPage={}&page={}",
            HN_ALGOLIA_PREFIX,
            if by_date { "search_by_date" } else { "search" },
            HN_SEARCH_QUERY_STRING,
            search_story_limit,
            page
        );
        let time = SystemTime::now();
        let response = self
            .client
            .get(&request_url)
            .query("query", query)
            .call()?
            .into_json::<StoriesResponse>()?;
        if let Ok(elapsed) = time.elapsed() {
            info!(
                "get matched stories with query {} (by_date={}) took {}ms",
                query,
                by_date,
                elapsed.as_millis()
            );
        }

        Ok(response.parse_into_stories())
    }

    // reorder the front_page stories to follow the same order
    // as in the official Hacker News site.
    // Needs to do this because stories returned by Algolia APIs
    // are sorted by `points`.
    fn reoder_front_page_stories(&self, stories: Vec<Story>, ids: &[u32]) -> Vec<Story> {
        let mut stories = stories;
        stories.sort_by(|story_x, story_y| {
            let story_x_pos = ids
                .iter()
                .enumerate()
                .find(|&(_, story_id)| *story_id == story_x.id)
                .unwrap()
                .0;
            let story_y_pos = ids
                .iter()
                .enumerate()
                .find(|&(_, story_id)| *story_id == story_y.id)
                .unwrap()
                .0;

            story_x_pos.cmp(&story_y_pos)
        });
        stories
    }

    // retrieve a list of front_page story ids using HN Official API then
    // compose a HN Algolia API to retrieve the corresponding stories.
    fn get_front_page_stories(
        &self,
        story_limit: usize,
        page: usize,
        numeric_filters: StoryNumericFilters,
    ) -> Result<Vec<Story>> {
        let request_url = format!("{}/topstories.json", HN_OFFICIAL_PREFIX);
        let time = SystemTime::now();
        let stories = self
            .client
            .get(&request_url)
            .call()?
            .into_json::<Vec<u32>>()?;
        if let Ok(elapsed) = time.elapsed() {
            info!(
                "get front_page stories using {} took {}ms",
                request_url,
                elapsed.as_millis()
            );
        }

        let start_id = story_limit * page;
        if start_id >= stories.len() {
            return Ok(vec![]);
        }

        let end_id = std::cmp::min(start_id + story_limit, stories.len());
        let ids = &stories[start_id..end_id];

        let request_url = format!(
            "{}/search?tags=story,({})&hitsPerPage={}{}",
            HN_ALGOLIA_PREFIX,
            ids.iter().fold("".to_owned(), |tags, story_id| format!(
                "{}story_{},",
                tags, story_id
            )),
            story_limit,
            numeric_filters.query(),
        );

        let response = self
            .client
            .get(&request_url)
            .call()?
            .into_json::<StoriesResponse>()?;
        if let Ok(elapsed) = time.elapsed() {
            info!(
                "get stories (tag=front_page, by_date=false, page={}) took {}ms",
                page,
                elapsed.as_millis()
            );
        }

        Ok(self.reoder_front_page_stories(response.parse_into_stories(), ids))
    }

    /// Get a list of stories filtering on a specific tag
    pub fn get_stories_by_tag(
        &self,
        tag: &str,
        by_date: bool,
        page: usize,
        numeric_filters: StoryNumericFilters,
    ) -> Result<Vec<Story>> {
        let story_limit = get_config().client.story_limit.get_story_limit_by_tag(tag);

        if tag == "front_page" {
            return self.get_front_page_stories(story_limit, page, numeric_filters);
        }
        let request_url = format!(
            "{}/{}?tags={}&hitsPerPage={}&page={}{}",
            HN_ALGOLIA_PREFIX,
            if by_date { "search_by_date" } else { "search" },
            tag,
            story_limit,
            page,
            numeric_filters.query(),
        );

        let time = SystemTime::now();
        let response = self
            .client
            .get(&request_url)
            .call()?
            .into_json::<StoriesResponse>()?;
        if let Ok(elapsed) = time.elapsed() {
            info!(
                "get stories (tag={}, by_date={}, page={}) took {}ms",
                tag,
                by_date,
                page,
                elapsed.as_millis()
            );
        }

        Ok(response.parse_into_stories())
    }
}
