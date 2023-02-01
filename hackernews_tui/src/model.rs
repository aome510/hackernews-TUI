use serde::Deserialize;

use crate::prelude::*;

use std::{borrow::Cow, collections::HashMap};

pub type CommentSender = crossbeam_channel::Sender<Vec<Comment>>;
pub type CommentReceiver = crossbeam_channel::Receiver<Vec<Comment>>;

#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub url: String,
    pub author: String,
    pub points: u32,
    pub num_comments: usize,
    pub time: u64,
    pub title: StyledString, // story's title can be represented as a styled string in SearchView
    pub content: StyledString,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u32,
    pub level: usize,
    pub author: String,
    pub time: u64,
    pub content: StyledString,
}

pub struct StoryHiddenData {
    pub comment_receiver: CommentReceiver,
    pub vote_state: HashMap<String, VoteData>,
}

#[derive(Debug, Clone)]
pub struct VoteData {
    pub auth: String,
    pub upvoted: bool,
}

#[derive(Debug, Clone)]
/// A HackerNews item which can be either a story or a comment.
pub struct HnItem {
    pub level: usize,
    pub display_state: DisplayState,
    pub text: StyledString,
    pub minimized_text: StyledString,
    pub links: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum DisplayState {
    Hidden,
    Minimized,
    Normal,
}

#[derive(Debug, Clone, Deserialize)]
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
