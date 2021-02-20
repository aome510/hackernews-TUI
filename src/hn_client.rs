use futures::future::join_all;
use serde::{de::DeserializeOwned, Deserialize};

use anyhow::Result;

const HN_URI_PREFIX: &str = "https://hacker-news.firebaseio.com/v0/";

#[derive(Clone, Debug, Deserialize)]
/// Story represents a story post in Hacker News.
pub struct Story {
    id: i32,
    kids: Vec<i32>,
    pub title: String,
    pub url: String,
    pub time: i64,
}

#[derive(Debug, Deserialize)]
/// Comment represents a comment in Hacker News.
pub struct Comment {
    id: i32,
    kids: Vec<i32>,
    pub subcomments: Vec<Box<Comment>>,
    pub text: String,
    pub by: String,
    pub time: i64,
}

/// HNClient is a http client to communicate with Hacker News APIs.
#[derive(Clone)]
pub struct HNClient {
    client: reqwest::Client,
}

impl Story {
    /// Retrieve all comments in a story post
    pub async fn get_all_comments(&self, hn_client: &HNClient) -> Result<Vec<Comment>> {
        hn_client.get_items_from_ids::<Comment>(&self.kids).await
    }
}

impl Comment {
    /// Retrieve all subcomments of a comment
    pub async fn get_all_subcomments(&mut self, hn_client: &HNClient) -> Result<()> {
        self.subcomments = hn_client
            .get_items_from_ids::<Comment>(&self.kids)
            .await?
            .into_iter()
            .map(|comment| Box::new(comment))
            .collect();
        Ok(())
    }
}

impl HNClient {
    /// Create new Hacker News Client
    pub fn new() -> HNClient {
        HNClient {
            client: reqwest::Client::new(),
        }
    }

    /// Retrieve data from an item id and parse it to the corresponding struct
    pub async fn get_item_from_id<T>(&self, id: i32) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request_url = format!("{}/item/{}.json", HN_URI_PREFIX, id);
        Ok(self
            .client
            .get(&request_url)
            .send()
            .await?
            .json::<T>()
            .await?)
    }

    /// Retrieve data of multiple items from their ids and parse to a Vector of
    /// the corresponding struct
    pub async fn get_items_from_ids<T: std::fmt::Debug>(&self, ids: &Vec<i32>) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let results = join_all(ids.iter().map(|id| self.get_item_from_id::<T>(*id))).await;
        Ok(results
            .into_iter()
            .filter(|result| match result {
                Ok(s) => {
                    eprintln!("{:#?}", s);
                    true
                },
                Err(_) => false,
            })
            .map(|result| result.unwrap())
            .collect::<Vec<T>>())
    }

    /// Retrieve a list of HN's top stories
    pub async fn get_top_stories(&self) -> Result<Vec<Story>> {
        let request_url = format!("{}/topstories.json", HN_URI_PREFIX);
        let mut story_ids = self
            .client
            .get(&request_url)
            .send()
            .await?
            .json::<Vec<i32>>()
            .await?;
        story_ids.truncate(10);
        self.get_items_from_ids::<Story>(&story_ids).await
    }
}
