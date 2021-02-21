use anyhow::Result;
use log::warn;
use rayon::prelude::*;
use serde::{de::DeserializeOwned, Deserialize};

const HN_URI_PREFIX: &str = "https://hacker-news.firebaseio.com/v0/";

#[derive(Clone, Debug, Deserialize)]
/// Story represents a story post in Hacker News.
pub struct Story {
    id: i32,
    #[serde(default)]
    kids: Vec<i32>,
    pub title: String,
    #[serde(default)]
    pub url: String,
    pub by: String,
    pub time: i64,
}

#[derive(Debug, Deserialize)]
/// Comment represents a comment in Hacker News.
pub struct Comment {
    id: i32,
    parent: i32,
    #[serde(default)]
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
        let mut comments = hn_client.get_items_from_ids::<Comment>(&self.kids);
        comments.par_iter_mut().for_each(|comment| {
            comment.update_subcomments(hn_client);
        });
        comments
    }
}

impl Comment {
    /// Update the subcomment list of a comment and its subcomments
    pub fn update_subcomments(&mut self, hn_client: &HNClient) {
        self.subcomments = hn_client
            .get_items_from_ids::<Comment>(&self.kids)
            .into_iter()
            .map(|comment| Box::new(comment))
            .collect();

        // recursively update subcomment list for each subcomment of
        // the current comment
        self.subcomments.par_iter_mut().for_each(|comment| {
            comment.update_subcomments(hn_client);
        });
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
        ids.par_iter()
            .flat_map(|id| match self.get_item_from_id::<T>(*id) {
                Ok(item) => vec![item],
                Err(err) => {
                    warn!("failed to get item {}: {:#?}", id, err);
                    vec![]
                }
            })
            .collect()
    }

    /// Retrieve a list of HN's top stories
    pub fn get_top_stories(&self) -> Result<Vec<Story>> {
        let request_url = format!("{}/topstories.json", HN_URI_PREFIX);
        let mut story_ids = self.client.get(&request_url).send()?.json::<Vec<i32>>()?;
        // only get top 50 stories
        story_ids.truncate(50);
        Ok(self.get_items_from_ids::<Story>(&story_ids))
    }
}
