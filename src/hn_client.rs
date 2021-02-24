use anyhow::Result;
use serde::Deserialize;

const HN_ALGOLIA_PREFIX: &'static str = "https://hn.algolia.com/api/v1";

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
    #[serde(rename(deserialize = "objectID"))]
    #[serde(deserialize_with = "parse_id")]
    id: i32,

    #[serde(default)]
    pub children: Vec<Box<Comment>>,

    pub title: String,
    pub author: String,
    #[serde(deserialize_with = "parse_null_default")]
    pub url: String,
    #[serde(rename(deserialize = "created_at_i"))]
    pub time: u64,
    #[serde(deserialize_with = "parse_null_default")]
    pub points: i32,
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

    pub text: String,
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

impl HNClient {
    /// Create new Hacker News Client
    pub fn new() -> HNClient {
        HNClient {
            client: reqwest::blocking::Client::new(),
        }
    }

    // /// Retrieve data from an item id and parse it to the corresponding struct
    // pub fn get_item_from_id<T>(&self, id: i32) -> Result<T>
    // where
    //     T: DeserializeOwned,
    // {
    //     let request_url = format!("{}/items/{}", HN_ALGOLIA_PREFIX, id);
    //     Ok(self.client.get(&request_url).send()?.json::<T>()?)
    // }

    // /// Retrieve data of multiple items from their ids and parse to a Vector of
    // /// the corresponding struct
    // pub fn get_items_from_ids<T: Send>(&self, ids: &Vec<i32>) -> Vec<T>
    // where
    //     T: DeserializeOwned,
    // {
    //     ids.par_iter()
    //         .flat_map(|id| match self.get_item_from_id::<T>(*id) {
    //             Ok(item) => vec![item],
    //             Err(err) => {
    //                 warn!("failed to get item {}: {:#?}", id, err);
    //                 vec![]
    //             }
    //         })
    //         .collect()
    // }

    /// Retrieve a list of stories on HN frontpage
    pub fn get_top_stories(&self) -> Result<Vec<Story>> {
        // get top 50 stories. However, angolia normally returns the top 32 stories at most
        let request_url = format!(
            "{}/search?tags=front_page&hitsPerPage=32",
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
