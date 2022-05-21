// modules
mod parser;
mod query;

// re-export
pub use parser::{Article, CollapseState, HnText, Story};
pub use query::StoryNumericFilters;

use crate::prelude::*;
use parser::*;
use rayon::prelude::*;

const HN_ALGOLIA_PREFIX: &str = "https://hn.algolia.com/api/v1";
const HN_OFFICIAL_PREFIX: &str = "https://hacker-news.firebaseio.com/v0";
const HN_SEARCH_QUERY_STRING: &str =
    "tags=story&restrictSearchableAttributes=title,url&typoTolerance=false";
pub const HN_HOST_URL: &str = "https://news.ycombinator.com";
pub const STORY_LIMIT: usize = 20;
pub const SEARCH_LIMIT: usize = 15;

static CLIENT: once_cell::sync::OnceCell<HNClient> = once_cell::sync::OnceCell::new();

pub type CommentSender = crossbeam_channel::Sender<Vec<HnText>>;
pub type CommentReceiver = crossbeam_channel::Receiver<Vec<HnText>>;

/// HNClient is a HTTP client to communicate with Hacker News APIs.
#[derive(Clone)]
pub struct HNClient {
    client: ureq::Agent,
}

/// A macro to log the runtime of an expression
macro_rules! log {
    ($e:expr, $desc:expr) => {{
        let time = std::time::SystemTime::now();
        let result = $e;
        if let Ok(elapsed) = time.elapsed() {
            info!("{} took {}ms", $desc, elapsed.as_millis());
        }
        result
    }};
}

impl HNClient {
    /// Create a new Hacker News Client
    pub fn new() -> Result<HNClient> {
        let timeout = config::get_config().client_timeout;
        Ok(HNClient {
            client: ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(timeout))
                .build(),
        })
    }

    /// Get data of a HN item based on its id then parse the data
    /// to a corresponding struct representing that item
    pub fn get_item_from_id<T>(&self, id: u32) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let request_url = format!("{}/items/{}", HN_ALGOLIA_PREFIX, id);
        let item = log!(
            self.client.get(&request_url).call()?.into_json::<T>()?,
            format!("get HN item (id={}) using {}", id, request_url)
        );
        Ok(item)
    }

    /// Lazily load a story's comments
    pub fn lazy_load_story_comments(&self, story_id: u32) -> Result<CommentReceiver> {
        let request_url = format!("{}/item/{}.json", HN_OFFICIAL_PREFIX, story_id);
        let mut ids = log!(
            self.client
                .get(&request_url)
                .call()?
                .into_json::<HNStoryResponse>()?
                .kids,
            format!("get story (id={}) using {}", story_id, request_url)
        );

        let (sender, receiver) = crossbeam_channel::bounded(32);

        // loads first 5 comments to ensure the corresponding `CommentView` has data to render
        self.load_comments(&sender, &mut ids, 5)?;
        std::thread::spawn({
            let client = self.clone();
            let sleep_dur = std::time::Duration::from_millis(1000);
            move || {
                while !ids.is_empty() {
                    if let Err(err) = client.load_comments(&sender, &mut ids, 5) {
                        warn!("encountered an error when loading comments: {}", err);
                        break;
                    }
                    std::thread::sleep(sleep_dur);
                }
            }
        });
        Ok(receiver)
    }

    /// Load the first `size` comments from a list of comment IDs.
    fn load_comments(&self, sender: &CommentSender, ids: &mut Vec<u32>, size: usize) -> Result<()> {
        let size = std::cmp::min(ids.len(), size);
        if size == 0 {
            return Ok(());
        }

        let responses = ids
            .drain(0..size)
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|id| match self.get_item_from_id::<CommentResponse>(id) {
                Ok(response) => Some(response),
                Err(err) => {
                    warn!("failed to get comment (id={}): {}", id, err);
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        for response in responses {
            sender.send(response.into())?;
        }

        Ok(())
    }

    /// Get a story based on its id
    pub fn get_story_from_story_id(&self, id: u32) -> Result<Story> {
        let request_url = format!("{}/search?tags=story,story_{}", HN_ALGOLIA_PREFIX, id);
        let response = log!(
            self.client
                .get(&request_url)
                .call()?
                .into_json::<StoriesResponse>()?,
            format!("get story (id={}) using {}", id, request_url)
        );

        match <Vec<Story>>::from(response).pop() {
            Some(story) => Ok(story),
            None => Err(anyhow::anyhow!("failed to get story with id {}", id)),
        }
    }

    /// Get a list of stories matching certain conditions
    pub fn get_matched_stories(
        &self,
        query: &str,
        by_date: bool,
        page: usize,
    ) -> Result<Vec<Story>> {
        let request_url = format!(
            "{}/{}?{}&hitsPerPage={}&page={}",
            HN_ALGOLIA_PREFIX,
            if by_date { "search_by_date" } else { "search" },
            HN_SEARCH_QUERY_STRING,
            SEARCH_LIMIT,
            page
        );
        let response = log!(
            self.client
                .get(&request_url)
                .query("query", query)
                .call()?
                .into_json::<StoriesResponse>()?,
            format!(
                "get matched stories with query {} (by_date={}, page={}) using {}",
                query, by_date, page, request_url
            )
        );

        Ok(response.into())
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
        page: usize,
        numeric_filters: query::StoryNumericFilters,
    ) -> Result<Vec<Story>> {
        let request_url = format!("{}/topstories.json", HN_OFFICIAL_PREFIX);
        let stories = log!(
            self.client
                .get(&request_url)
                .call()?
                .into_json::<Vec<u32>>()?,
            format!("get front page stories using {}", request_url)
        );

        let start_id = STORY_LIMIT * page;
        if start_id >= stories.len() {
            return Ok(vec![]);
        }

        let end_id = std::cmp::min(start_id + STORY_LIMIT, stories.len());
        let ids = &stories[start_id..end_id];

        let request_url = format!(
            "{}/search?tags=story,({}){}&hitsPerPage={}",
            HN_ALGOLIA_PREFIX,
            ids.iter().fold("".to_owned(), |tags, story_id| format!(
                "{}story_{},",
                tags, story_id
            )),
            numeric_filters.query(),
            STORY_LIMIT,
        );

        let response = log!(
            self.client
                .get(&request_url)
                .call()?
                .into_json::<StoriesResponse>()?,
            format!(
                "get stories (tag=front_page, by_date=false, page={}) using {}",
                page, request_url
            )
        );

        Ok(self.reoder_front_page_stories(response.into(), ids))
    }

    /// Get a list of stories filtering on a specific tag
    pub fn get_stories_by_tag(
        &self,
        tag: &str,
        by_date: bool,
        page: usize,
        numeric_filters: query::StoryNumericFilters,
    ) -> Result<Vec<Story>> {
        if tag == "front_page" {
            return self.get_front_page_stories(page, numeric_filters);
        }
        let request_url = format!(
            "{}/{}?tags={}&hitsPerPage={}&page={}{}",
            HN_ALGOLIA_PREFIX,
            if by_date { "search_by_date" } else { "search" },
            tag,
            STORY_LIMIT,
            page,
            numeric_filters.query(),
        );

        let response = log!(
            self.client
                .get(&request_url)
                .call()?
                .into_json::<StoriesResponse>()?,
            format!(
                "get stories (tag={}, by_date={}, page={}, numeric_filters={}) using {}",
                tag,
                by_date,
                page,
                numeric_filters.query(),
                request_url
            )
        );

        Ok(response.into())
    }

    /// gets a web article from a URL
    pub fn get_article(url: &str) -> Result<Article> {
        let article_parse_command = &config::get_config().article_parse_command;
        let output = std::process::Command::new(&article_parse_command.command)
            .args(&article_parse_command.options)
            .arg(url)
            .output()?;

        if output.status.success() {
            match serde_json::from_slice::<Article>(&output.stdout) {
                Ok(mut article) => {
                    // Replace a tab character by 4 spaces as it's possible
                    // that the terminal cannot render the tab character.
                    article.content = article.content.replace('\t', "    ");

                    article.url = url.to_string();
                    Ok(article)
                }
                Err(err) => {
                    let stdout = std::str::from_utf8(&output.stdout)?;
                    warn!("failed to deserialize {} into an `Article` struct:", stdout);
                    Err(anyhow::anyhow!(err))
                }
            }
        } else {
            let stderr = std::str::from_utf8(&output.stderr)?.to_string();
            Err(anyhow::anyhow!(stderr))
        }
    }
}

pub fn init_client() -> &'static HNClient {
    let client = HNClient::new().unwrap();
    CLIENT.set(client).unwrap_or_else(|_| {
        panic!("failed to set up the application's HackerNews Client");
    });
    CLIENT.get().unwrap()
}
