use crate::prelude::*;

const HN_ALGOLIA_PREFIX: &'static str = "https://hn.algolia.com/api/v1";
const HN_SEARCH_QUERY_STRING: &'static str = "tags=story&restrictSearchableAttributes=title&typoTolerance=false&hitsPerPage=16&minProximity=8&queryType=prefixLast";
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

fn parse_id<'de, D>(d: D) -> std::result::Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse::<i32>().map_err(de::Error::custom)
}

fn parse_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MatchResult {
    value: Option<String>,
    #[serde(default)]
    matched_words: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HighlightResult {
    title: Option<MatchResult>,
    url: Option<MatchResult>,
    author: Option<MatchResult>,
}

#[derive(Clone, Debug, Deserialize)]
/// Story represents a story post in Hacker News.
pub struct Story {
    #[serde(default)]
    #[serde(rename(deserialize = "objectID"))]
    #[serde(deserialize_with = "parse_id")]
    pub id: i32,

    #[serde(default)]
    pub children: Vec<Box<Comment>>,

    pub title: Option<String>,
    pub author: Option<String>,
    pub url: Option<String>,
    #[serde(rename(deserialize = "created_at_i"))]
    pub time: u64,
    #[serde(deserialize_with = "parse_null_default")]
    pub points: i32,
    #[serde(default)]
    #[serde(deserialize_with = "parse_null_default")]
    pub num_comments: i32,

    // search result
    #[serde(rename(deserialize = "_highlightResult"))]
    pub highlight_result: Option<HighlightResult>,
}

#[derive(Clone, Debug, Deserialize)]
/// Comment represents a comment in Hacker News.
pub struct Comment {
    pub id: i32,
    #[serde(deserialize_with = "parse_null_default")]
    pub parent_id: i32,
    #[serde(deserialize_with = "parse_null_default")]
    pub story_id: i32,
    #[serde(default)]
    pub children: Vec<Box<Comment>>,

    pub text: Option<String>,
    pub author: Option<String>,
    #[serde(rename(deserialize = "created_at_i"))]
    pub time: u64,
}

#[derive(Debug, Deserialize)]
struct StoriesResponse {
    pub hits: Vec<Story>,
}

/// HNClient is a http client to communicate with Hacker News APIs.
#[derive(Clone)]
pub struct HNClient {
    client: ureq::Agent,
}

/// Retrieves all comments from a story with a given id
pub fn get_comments_from_story_id(id: i32, client: &HNClient) -> Result<Vec<Box<Comment>>> {
    let time = SystemTime::now();
    let story = client.get_item_from_id::<Story>(id)?;
    if let Ok(elapsed) = time.elapsed() {
        debug!(
            "get comments from story {} took {}ms",
            id,
            elapsed.as_millis()
        );
    }
    Ok(story.children)
}

impl HNClient {
    /// Create new Hacker News Client
    pub fn new() -> Result<HNClient> {
        Ok(HNClient {
            client: ureq::AgentBuilder::new().timeout(CLIENT_TIMEOUT).build(),
        })
    }

    /// Retrieve data from an item id and parse it to the corresponding struct
    pub fn get_item_from_id<T>(&self, id: i32) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request_url = format!("{}/items/{}", HN_ALGOLIA_PREFIX, id);
        Ok(self.client.get(&request_url).call()?.into_json::<T>()?)
    }

    /// Return a list of stories whose titles match a query
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
        Ok(response.hits)
    }

    /// Retrieve a list of stories on HN frontpage
    pub fn get_top_stories(&self) -> Result<Vec<Story>> {
        // get top 50 stories. However, angolia front-page API normally returns at most top 33 stories
        let request_url = format!(
            "{}/search?tags=front_page&hitsPerPage=50",
            HN_ALGOLIA_PREFIX
        );
        let time = SystemTime::now();
        let response = self
            .client
            .get(&request_url)
            .call()?
            .into_json::<StoriesResponse>()?;
        if let Ok(elapsed) = time.elapsed() {
            debug!("get top stories took {}ms", elapsed.as_millis());
        }
        Ok(response.hits)
    }
}
