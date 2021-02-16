use serde::{Deserialize, Serialize};
use futures::future::join_all;

const HN_URI_PREFIX: &str = "https://hacker-news.firebaseio.com/v0/";

#[derive(Debug, Deserialize, Serialize)]
pub struct Story {
    pub title: String,
    pub url: String,
    pub time: i64
}

pub struct HNClient {
    client: reqwest::Client
}

impl HNClient {
    pub fn new() -> HNClient {
        HNClient {
            client: reqwest::Client::new()
        }
    }

    async fn get_story_from_id(&self, id: i32) -> Result<Story, Box<dyn std::error::Error>> {
        let request_url = format!("{}/item/{}.json", HN_URI_PREFIX, id);
        Ok(self.client.get(&request_url)
            .send()
            .await?
            .json::<Story>()
            .await?)
    }

    pub async fn get_top_stories(&self) -> Result<Vec<Story>, Box<dyn std::error::Error>> {
        let request_url = format!("{}/topstories.json", HN_URI_PREFIX);
        let story_ids = self.client.get(&request_url)
            .send()
            .await?
            .json::<Vec<i32>>()
            .await?;
        let results = join_all(story_ids.into_iter().map(|id| self.get_story_from_id(id))).await;
        Ok(results.into_iter()
            .filter(|result| {
                match result {
                    Ok(_) => true,
                    Err(_) => false
                }
            })
            .map(|result| result.unwrap())
            .collect())
    }
}
