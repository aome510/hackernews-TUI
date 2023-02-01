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
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u32,
    pub level: usize,
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

//// parse story's text
// let text = {
//     let metadata = utils::combine_styled_strings(vec![
//         StyledString::plain(parsed_title.source()),
//         StyledString::plain("\n"),
//         StyledString::styled(
//             format!(
//                 "{} points | by {} | {} ago | {} comments\n",
//                 s.points,
//                 author,
//                 utils::get_elapsed_time_as_text(s.time),
//                 s.num_comments,
//             ),
//             config::get_config_theme().component_style.metadata,
//         ),
//     ]);

//     // the HTML story text returned by HN Algolia API doesn't wrap a
//     // paragraph inside a `<p><\p>` tag pair.
//     // Instead, it seems to use `<p>` to represent a paragraph break.
//     let mut story_text = decode_html(&s.text.unwrap_or_default()).replace("<p>", "\n\n");

//     let minimized_text = if story_text.is_empty() {
//         metadata.clone()
//     } else {
//         story_text = format!("\n{story_text}");

//         utils::combine_styled_strings(vec![
//             metadata.clone(),
//             StyledString::plain("... (more)"),
//         ])
//     };

//     let mut text = metadata;
//     let result = parse_hn_html_text(story_text, Style::default(), 0);
//     text.append(result.s);

//     HnItem {
//         id: s.id,
//         level: 0,
//         state: DisplayState::Normal,
//         minimized_text,
//         text,
//         links: result.links,
//     }
// };

//// parse story title
// let mut parsed_title = StyledString::new();

// // parse the story title and decorate it based on the story category
// let title = {
//     if let Some(title) = title.strip_prefix("Ask HN") {
//         parsed_title
//             .append_styled("Ask HN", config::get_config_theme().component_style.ask_hn);
//         title
//     } else if let Some(title) = title.strip_prefix("Tell HN") {
//         parsed_title.append_styled(
//             "Tell HN",
//             config::get_config_theme().component_style.tell_hn,
//         );
//         title
//     } else if let Some(title) = title.strip_prefix("Show HN") {
//         parsed_title.append_styled(
//             "Show HN",
//             config::get_config_theme().component_style.show_hn,
//         );
//         title
//     } else if let Some(title) = title.strip_prefix("Launch HN") {
//         parsed_title.append_styled(
//             "Launch HN",
//             config::get_config_theme().component_style.launch_hn,
//         );
//         title
//     } else {
//         &title
//     }
// };

// // parse the story title that may contain search matches wrapped inside `<em>` tags
// // The matches are decorated with a corresponding style.
// {
//     // an index such that `title[curr_pos..]` represents the part of the
//     // text that hasn't been parsed.
//     let mut curr_pos = 0;
//     for caps in MATCH_RE.captures_iter(title) {
//         let whole_match = caps.get(0).unwrap();
//         // the part that doesn't match any patterns should be rendered in the default style
//         if curr_pos < whole_match.start() {
//             parsed_title.append_plain(&title[curr_pos..whole_match.start()]);
//         }
//         curr_pos = whole_match.end();

//         parsed_title.append_styled(
//             caps.name("match").unwrap().as_str(),
//             config::get_config_theme().component_style.matched_highlight,
//         );
//     }
//     if curr_pos < title.len() {
//         parsed_title.append_plain(&title[curr_pos..title.len()]);
//     }
// }

// // parse comment
// let metadata = utils::combine_styled_strings(vec![
//     StyledString::styled(
//         c.author.unwrap_or_default(),
//         config::get_config_theme().component_style.username,
//     ),
//     StyledString::styled(
//         format!(" {} ago ", utils::get_elapsed_time_as_text(c.time)),
//         config::get_config_theme().component_style.metadata,
//     ),
// ]);

// let mut text =
//     utils::combine_styled_strings(vec![metadata.clone(), StyledString::plain("\n")]);

// let result = parse_hn_html_text(
//     decode_html(&c.text.unwrap_or_default()),
//     Style::default(),
//     0,
// );
// text.append(result.s);
