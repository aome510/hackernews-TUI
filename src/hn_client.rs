use::serde::{Deserialize, Serialize}
const HN_URI_PREFIX: &str = "https://hacker-news.firebaseio.com/v0/";

#[derive(Deserialize, Serialize)]
struct Story {
    title: String,
    url: String,
    time: i64
}

struct HNClient {
    client: reqwest::Client
}

impl HNClient {
    // async fn get_story_from_story_id(id: i32) -> Result<Story, Box<dyn std::error::Error>> {
    //     let story = reqwest::get(concat!(HN_URI_PREFIX, "item/
    pub fn new() -> HNClient {
        HNClient {
            client: reqwest::Client::new()
        }
    }

    pub async fn get_story(&self, id: i32) -> Result<Story, Box<dyn std::error::Error>> {
        let request_url = format!("{}/item/{}.json", HN_URI_PREFIX, id);
        self.client.get(&request_url)
            .send()
            .await?
            .json::<Story>()
            .await?
    }

    pub async fn get_top_stories(&self) -> Result<Vec<Story>, Box<dyn std::error::Error>> {
        let request_url = format!("{}/topstories", HN_URI_PREFIX);
        let story_ids = self.client.get(&request_url)
            .send()
            .await?
            .json::<Vec<i32>>()
            .await?;
        let a = story_ids.into_iter().map(|id| self.get_story(id)).collect();
    }
}
