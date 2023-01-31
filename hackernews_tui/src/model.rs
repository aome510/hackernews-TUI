use serde::Deserialize;

use crate::prelude::*;

use std::{borrow::Cow, collections::HashMap};

pub type CommentSender = crossbeam_channel::Sender<Vec<HnItem>>;
pub type CommentReceiver = crossbeam_channel::Receiver<Vec<HnItem>>;

/// A HackerNews story data
pub struct StoryData {
    /// story's page content in raw HTML
    pub raw_html: String,
    /// a channel for lazily loading the story's comments,
    /// which is used to reduce the loading latency of a large story.
    /// See `client::lazy_load_story_comments` for more details.
    pub receiver: CommentReceiver,
    /// vote_state: id -> (auth, upvoted)
    /// See `Client::parse_story_vote_data` for more details
    /// on the data representation of the `vote_state` field.
    pub vote_state: HashMap<String, (String, bool)>,
}

/// A parsed Hacker News story
#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub title: StyledString,
    pub url: String,
    pub author: String,
    pub text: HnItem,
    pub points: u32,
    pub num_comments: usize,
    pub time: u64,
}

/// A parsed Hacker News item
#[derive(Debug, Clone)]
pub struct HnItem {
    pub id: u32,
    pub level: usize,
    pub state: CollapseState,
    pub text: StyledString,
    /// The minimized version of the text used to display the text component when it's partially collapsed.
    pub minimized_text: StyledString,
    pub links: Vec<String>,
}

#[derive(Debug, Clone)]
/// The collapse state of a HN item
pub enum CollapseState {
    Collapsed,
    PartiallyCollapsed,
    Normal,
}

#[derive(Debug, Clone, Deserialize)]
/// A web article in a reader mode
pub struct Article {
    pub title: String,
    pub url: String,
    pub content: String,
    pub author: Option<String>,
    pub date_published: Option<String>,
}

impl Story {
    /// get the story's article URL.
    /// If the article URL is empty (in case of "AskHN" stories), fallback to the HN story's URL
    pub fn get_url(&self) -> Cow<str> {
        if self.url.is_empty() {
            Cow::from(self.story_url())
        } else {
            Cow::from(&self.url)
        }
    }

    pub fn story_url(&self) -> String {
        format!("{}/item?id={}", client::HN_HOST_URL, self.id)
    }
}
