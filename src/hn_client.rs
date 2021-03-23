use crate::prelude::*;

const HN_ALGOLIA_PREFIX: &'static str = "https://hn.algolia.com/api/v1";
const HN_SEARCH_QUERY_STRING: &'static str =
    "tags=story&restrictSearchableAttributes=title&typoTolerance=false&hitsPerPage=16";
const CLIENT_TIMEOUT: Duration = Duration::from_secs(16);
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
struct HighlightResult {
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

    #[serde(deserialize_with = "parse_null_default")]
    points: u32,
    #[serde(default)]
    #[serde(deserialize_with = "parse_null_default")]
    num_comments: u32,

    #[serde(rename(deserialize = "created_at_i"))]
    time: u64,

    // search result
    #[serde(rename(deserialize = "_highlightResult"))]
    highlight_result: Option<HighlightResult>,
}

#[derive(Debug, Deserialize)]
/// CommentResponse represents the story data received from HN_ALGOLIA APIs
struct CommentResponse {
    id: u32,
    #[serde(deserialize_with = "parse_null_default")]
    parent_id: u32,
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

// parsed structs

/// Story represents a Hacker News story
#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub author: String,
    pub points: u32,
    pub num_comments: u32,
    pub time: u64,
    pub children: Vec<Comment>,
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
            .map(|comment| comment.into())
            .collect();
        Comment {
            id: c.id,
            story_id: c.story_id,
            parent_id: c.parent_id,
            text: c.text.unwrap_or("[deleted]".to_string()),
            author: c.author.unwrap_or("[deleted]".to_string()),
            time: c.time,
            children,
        }
    }
}

impl From<StoryResponse> for Story {
    fn from(s: StoryResponse) -> Self {
        // need to make sure that highlight_result is not none,
        // and its title field is not none,
        let highlight_result = s.highlight_result.unwrap();
        let title = highlight_result.title.unwrap().value;
        let url = match highlight_result.url {
            None => String::new(),
            Some(url) => url.value,
        };
        let author = match highlight_result.author {
            None => String::new(),
            Some(author) => author.value,
        };
        let children = s
            .children
            .into_par_iter()
            .map(|comment| comment.into())
            .collect();
        Story {
            id: s.id,
            points: s.points,
            num_comments: s.num_comments,
            time: s.time,
            title,
            url,
            author,
            children,
        }
    }
}

// HN client get Story,Comment data by calling HN_ALGOLIA APIs
// and parsing the result into a corresponding struct

/// HNClient is a http client to communicate with Hacker News APIs.
#[derive(Clone)]
pub struct HNClient {
    client: ureq::Agent,
}

impl HNClient {
    /// Create new Hacker News Client
    pub fn new() -> Result<HNClient> {
        Ok(HNClient {
            client: ureq::AgentBuilder::new().timeout(CLIENT_TIMEOUT).build(),
        })
    }

    /// Get data from an item's id and parse it to the corresponding struct
    /// representing that item
    pub fn get_item_from_id<T>(&self, id: u32) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request_url = format!("{}/items/{}", HN_ALGOLIA_PREFIX, id);
        Ok(self.client.get(&request_url).call()?.into_json::<T>()?)
    }

    /// Get all comments from a story with a given id
    pub fn get_comments_from_story_id(&self, id: u32) -> Result<Vec<Comment>> {
        let time = SystemTime::now();
        let response = self.get_item_from_id::<StoryResponse>(id)?;
        if let Ok(elapsed) = time.elapsed() {
            debug!(
                "get comments from story {} took {}ms",
                id,
                elapsed.as_millis()
            );
        }

        Ok(response
            .children
            .into_par_iter()
            .map(|comment| comment.into())
            .collect())
    }

    /// Get a list of stories matching certain conditions
    pub fn get_matched_stories(&self, query: &str) -> Result<Vec<Story>> {
        let request_url = format!("{}/search?{}", HN_ALGOLIA_PREFIX, HN_SEARCH_QUERY_STRING);
        let time = SystemTime::now();
        let response = self
            .client
            .get(&request_url)
            .query("query", query)
            .call()?
            .into_json::<StoriesResponse>()?;
        if let Ok(elapsed) = time.elapsed() {
            debug!(
                "get matched stories with query {} took {}ms",
                query,
                elapsed.as_millis()
            );
        }

        Ok(response
            .hits
            .into_par_iter()
            .filter(|story| story.highlight_result.is_some() && story.title.is_some())
            .map(|story| story.into())
            .collect())
    }

    /// Get a list of stories on HN front page
    pub fn get_front_page_stories(&self) -> Result<Vec<Story>> {
        let request_url = format!("{}/search?tags=front_page", HN_ALGOLIA_PREFIX);
        let time = SystemTime::now();
        let response = self
            .client
            .get(&request_url)
            .call()?
            .into_json::<StoriesResponse>()?;
        if let Ok(elapsed) = time.elapsed() {
            debug!("get top stories took {}ms", elapsed.as_millis());
        }

        Ok(response
            .hits
            .into_par_iter()
            .filter(|story| story.highlight_result.is_some() && story.title.is_some())
            .map(|story| story.into())
            .collect())
    }
}
