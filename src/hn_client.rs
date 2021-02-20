use anyhow::Result;
use rayon::prelude::*;
use serde::{de::DeserializeOwned, Deserialize};

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
    #[serde(skip_deserializing)]
    pub subcomments: Vec<Box<Comment>>,
    pub text: String,
    pub by: String,
    pub time: i64,
}

/// HNClient is a http client to communicate with Hacker News APIs.
#[derive(Clone)]
pub struct HNClient {
    client: reqwest::blocking::Client,
}

impl Story {
    /// Retrieve all comments in a story post
    pub fn get_all_comments(&self, hn_client: &HNClient) -> Vec<Comment> {
        hn_client.get_items_from_ids::<Comment>(&self.kids)
    }
}

impl Comment {
    /// Retrieve all subcomments of a comment
    pub fn get_all_subcomments(&mut self, hn_client: &HNClient) -> Result<()> {
        self.subcomments = hn_client
            .get_items_from_ids::<Comment>(&self.kids)
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
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Retrieve data from an item id and parse it to the corresponding struct
    pub fn get_item_from_id<T>(&self, id: i32) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request_url = format!("{}/item/{}.json", HN_URI_PREFIX, id);
        Ok(self.client.get(&request_url).send()?.json::<T>()?)
    }

    /// Retrieve data of multiple items from their ids and parse to a Vector of
    /// the corresponding struct
    pub fn get_items_from_ids<T: Send>(&self, ids: &Vec<i32>) -> Vec<T>
    where
        T: DeserializeOwned,
    {
        let (results, errors): (Vec<_>, Vec<_>) = ids
            .par_iter()
            .map(|id| self.get_item_from_id::<T>(*id))
            .partition(Result::is_ok);
        errors.into_iter().for_each(|e| {
            if let Err(err) = e {
                eprintln!("failed to retrieve item: {:#?}", err);
            }
        });
        results
            .into_iter()
            .map(|result| result.unwrap())
            .collect::<Vec<T>>()
    }

    /// Retrieve a list of HN's top stories
    pub fn get_top_stories(&self) -> Result<Vec<Story>> {
        let request_url = format!("{}/topstories.json", HN_URI_PREFIX);
        let mut story_ids = self.client.get(&request_url).send()?.json::<Vec<i32>>()?;
        Ok(self.get_items_from_ids::<Story>(&story_ids))
    }
}
