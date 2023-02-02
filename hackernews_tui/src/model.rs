use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;

use crate::parser::parse_hn_html_text;
use crate::prelude::*;
use crate::utils;

use std::{borrow::Cow, collections::HashMap};

pub type CommentSender = crossbeam_channel::Sender<Vec<Comment>>;
pub type CommentReceiver = crossbeam_channel::Receiver<Vec<Comment>>;

/// a regex that matches a search match in the response from HN Algolia search API
static MATCH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<em>(?P<match>.*?)</em>").unwrap());

#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub url: String,
    pub author: String,
    pub points: u32,
    pub num_comments: usize,
    pub time: u64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u32,
    pub level: usize,
    pub n_children: usize,
    pub author: String,
    pub time: u64,
    pub content: String,
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
///
/// This struct is a shared representation between a story and
/// a comment for rendering the item's content.
pub struct HnItem {
    pub id: u32,
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

impl From<Story> for HnItem {
    fn from(story: Story) -> Self {
        let component_style = &config::get_config_theme().component_style;

        let metadata = utils::combine_styled_strings(vec![
            story.styled_title(),
            StyledString::plain("\n"),
            StyledString::styled(
                format!(
                    "{} points | by {} | {} ago | {} comments\n",
                    story.points,
                    story.author,
                    utils::get_elapsed_time_as_text(story.time),
                    story.num_comments,
                ),
                component_style.metadata,
            ),
        ]);

        // The HTML story text returned by HN Algolia APIs doesn't wrap a paragraph inside a `<p><\p>` tag pair.
        // Instead, it seems to use `<p>` to represent a paragraph break.
        // Replace `<p>` with linebreaks to make the text easier to parse.
        let mut story_text = story.content.replace("<p>", "\n\n");

        let minimized_text = if story_text.is_empty() {
            metadata.clone()
        } else {
            story_text = format!("\n{story_text}");

            utils::combine_styled_strings(vec![metadata.clone(), StyledString::plain("... (more)")])
        };

        let mut text = metadata;
        let result = parse_hn_html_text(story_text, Style::default(), 0);
        text.append(result.s);

        HnItem {
            id: story.id,
            level: 0, // story is at level 0 by default
            display_state: DisplayState::Normal,
            links: result.links,
            text,
            minimized_text,
        }
    }
}

impl From<Comment> for HnItem {
    fn from(comment: Comment) -> Self {
        let component_style = &config::get_config_theme().component_style;

        let metadata = utils::combine_styled_strings(vec![
            StyledString::styled(comment.author, component_style.username),
            StyledString::styled(
                format!(" {} ago ", utils::get_elapsed_time_as_text(comment.time)),
                component_style.metadata,
            ),
        ]);

        let mut text =
            utils::combine_styled_strings(vec![metadata.clone(), StyledString::plain("\n")]);
        let minimized_text = utils::combine_styled_strings(vec![
            metadata,
            StyledString::styled(
                format!("({} more)", comment.n_children + 1),
                component_style.metadata,
            ),
        ]);

        let result = parse_hn_html_text(comment.content, Style::default(), 0);
        text.append(result.s);

        HnItem {
            id: comment.id,
            level: comment.level,
            display_state: DisplayState::Normal,
            links: result.links,
            text,
            minimized_text,
        }
    }
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

    /// Get the decorated story's title
    pub fn styled_title(&self) -> StyledString {
        let mut parsed_title = StyledString::new();
        let mut title = self.title.clone();

        let component_style = &config::get_config_theme().component_style;

        // decorate the story title based on the story category
        {
            let categories = ["Ask HN", "Tell HN", "Show HN", "Launch HN"];
            let styles = [
                component_style.ask_hn,
                component_style.tell_hn,
                component_style.show_hn,
                component_style.launch_hn,
            ];

            assert!(categories.len() == styles.len());

            for i in 0..categories.len() {
                if let Some(t) = title.strip_prefix(categories[i]) {
                    parsed_title.append_styled(categories[i], styles[i]);
                    title = t.to_string();
                }
            }
        }

        // The story title may contain search matches wrapped inside `<em>` tags.
        // The matches are decorated with a corresponding style.
        {
            // an index represents the part of the text that hasn't been parsed (e.g `title[curr_pos..]` )
            let mut curr_pos = 0;
            for caps in MATCH_RE.captures_iter(&title) {
                let whole_match = caps.get(0).unwrap();
                // the part that doesn't match any patterns should be rendered in the default style
                if curr_pos < whole_match.start() {
                    parsed_title.append_plain(&title[curr_pos..whole_match.start()]);
                }
                curr_pos = whole_match.end();

                parsed_title.append_styled(
                    caps.name("match").unwrap().as_str(),
                    component_style.matched_highlight,
                );
            }
            if curr_pos < title.len() {
                parsed_title.append_plain(&title[curr_pos..]);
            }
        }

        parsed_title
    }

    /// Get the story's plain title
    pub fn plain_title(&self) -> String {
        self.title.replace("<em>", "").replace("</em>", "") // story's title from the search view can have `<em>` inside it
    }
}
