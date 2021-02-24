use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;

const HN_ALGOLIA_PREFIX: &'static str = "https://hn.algolia.com/api/v1";
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

fn parse_id<'de, D>(d: D) -> std::result::Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse::<i32>().map_err(serde::de::Error::custom)
}

fn parse_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
/// Story represents a story post in Hacker News.
pub struct Story {
    #[serde(default)]
    #[serde(rename(deserialize = "objectID"))]
    #[serde(deserialize_with = "parse_id")]
    id: i32,

    #[serde(default)]
    pub children: Vec<Box<Comment>>,

    #[serde(deserialize_with = "parse_null_default")]
    pub title: String,
    #[serde(deserialize_with = "parse_null_default")]
    pub author: String,
    #[serde(deserialize_with = "parse_null_default")]
    pub url: String,
    #[serde(rename(deserialize = "created_at_i"))]
    pub time: u64,
    #[serde(deserialize_with = "parse_null_default")]
    pub points: i32,
    #[serde(default)]
    #[serde(deserialize_with = "parse_null_default")]
    pub num_comments: i32,
}

#[derive(Debug, Deserialize)]
/// Comment represents a comment in Hacker News.
pub struct Comment {
    id: i32,
    parent_id: i32,
    story_id: i32,
    #[serde(default)]
    pub children: Vec<Box<Comment>>,

    #[serde(deserialize_with = "parse_null_default")]
    pub text: String,
    #[serde(deserialize_with = "parse_null_default")]
    pub author: String,
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
    client: reqwest::blocking::Client,
}

impl Story {
    pub fn get_comments(&self, client: &HNClient) -> Result<Vec<Box<Comment>>> {
        let story = client.get_item_from_id::<Story>(self.id)?;
        Ok(story.children)
    }
}

impl HNClient {
    /// Create new Hacker News Client
    pub fn new() -> Result<HNClient> {
        Ok(HNClient {
            client: reqwest::blocking::Client::builder()
                .timeout(CLIENT_TIMEOUT)
                .build()?,
        })
    }

    /// Retrieve data from an item id and parse it to the corresponding struct
    pub fn get_item_from_id<T>(&self, id: i32) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let request_url = format!("{}/items/{}", HN_ALGOLIA_PREFIX, id);
        Ok(self.client.get(&request_url).send()?.json::<T>()?)
    }

    /// Retrieve a list of stories on HN frontpage
    pub fn get_top_stories(&self) -> Result<Vec<Story>> {
        // get top 50 stories. However, angolia normally returns the top 32 stories at most
        let request_url = format!(
            "{}/search?tags=front_page&hitsPerPage=50",
            HN_ALGOLIA_PREFIX
        );
        let response = self
            .client
            .get(&request_url)
            .send()?
            .json::<StoriesResponse>()?;
        Ok(response.hits)
    }
}
