use std::thread;

use super::async_view;
use super::event_view;
use super::help_view::*;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;

/// StoryView is a View displaying a list stories corresponding
/// to a particular category (top stories, newest stories, most popular stories, etc).
pub struct StoryView {
    view: LinearLayout,
    pub stories: Vec<hn_client::Story>,

    raw_command: String,
}

/// Return a StyledString representing a matched text in which
/// matches are highlighted
fn get_matched_text(mut s: String, default_style: ColorStyle) -> StyledString {
    let match_re = Regex::new(r"<em>(?P<match>.*?)</em>").unwrap();
    let mut styled_s = StyledString::new();
    loop {
        match match_re.captures(&s.clone()) {
            None => break,
            Some(c) => {
                let m = c.get(0).unwrap();
                let match_text = c.name("match").unwrap().as_str();

                let range = m.range();
                let mut prefix: String = s
                    .drain(std::ops::Range {
                        start: 0,
                        end: m.end(),
                    })
                    .collect();
                prefix.drain(range);

                if prefix.len() > 0 {
                    styled_s.append_styled(&prefix, default_style);
                }

                styled_s.append_styled(match_text, ColorStyle::back(HIGHLIGHT_COLOR));
                continue;
            }
        };
    }
    if s.len() > 0 {
        styled_s.append_styled(s, default_style);
    }
    styled_s
}

/// Get the description text summarizing basic information about a story
pub fn get_story_text(story: &hn_client::Story) -> StyledString {
    let mut story_text = get_matched_text(story.title.clone(), ColorStyle::default());
    if story.url.len() > 0 {
        let url = format!("\n{}", story.url);
        story_text.append(get_matched_text(url, ColorStyle::front(LINK_COLOR)));
    }
    story_text.append_styled(
        format!(
            "\n{} points | by {} | {} ago | {} comments",
            story.points,
            story.author,
            get_elapsed_time_as_text(story.time),
            story.num_comments,
        ),
        ColorStyle::from(DESC_COLOR),
    );
    story_text
}

impl StoryView {
    pub fn new(stories: Vec<hn_client::Story>) -> Self {
        let view = LinearLayout::vertical().with(|s| {
            stories.iter().enumerate().for_each(|(i, story)| {
                let mut story_text = StyledString::plain(format!("{}. ", i + 1));
                story_text.append(get_story_text(story));
                s.add_child(text_view::TextView::new(story_text));
            })
        });
        StoryView {
            view,
            stories,
            raw_command: String::new(),
        }
    }

    crate::raw_command!();

    inner_getters!(self.view: LinearLayout);
}

impl ViewWrapper for StoryView {
    wrap_impl!(self.view: LinearLayout);
}

/// Return a main view of a StoryView displaying the story list.
/// The main view of a StoryView is a View without status bar or footer.
pub fn get_story_main_view(
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
) -> impl View {
    event_view::construct_list_event_view(StoryView::new(stories))
        .on_pre_event_inner(Key::Enter, {
            let client = client.clone();
            move |s, _| {
                let id = s.get_inner().get_focus_index();
                // the story struct hasn't had any comments inside yet,
                // so it can be cloned without greatly affecting performance
                let story = s.stories[id].clone();
                Some(EventResult::with_cb({
                    let client = client.clone();
                    move |s| {
                        let async_view = async_view::get_comment_view_async(s, &client, &story);
                        s.pop_layer();
                        s.screen_mut().add_transparent_layer(Layer::new(async_view))
                    }
                }))
            }
        })
        .on_pre_event_inner('O', move |s, _| {
            let id = s.get_inner().get_focus_index();
            let url = s.stories[id].url.clone();
            if url.len() > 0 {
                thread::spawn(move || {
                    if let Err(err) = webbrowser::open(&url) {
                        warn!("failed to open link {}: {}", url, err);
                    }
                });
                Some(EventResult::Consumed(None))
            } else {
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner('g', move |s, _| match s.get_raw_command_as_number() {
            Ok(number) => {
                s.clear_raw_command();
                let s = s.get_inner_mut();
                if number == 0 {
                    return None;
                }
                let number = number - 1;
                if number < s.len() {
                    s.set_focus_index(number).unwrap();
                    Some(EventResult::Consumed(None))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .full_height()
        .scrollable()
}

/// Add StoryView as a new layer to the main Cursive View
pub fn add_story_view_layer(s: &mut Cursive, client: &hn_client::HNClient) {
    let async_view = async_view::get_front_page_story_view_async(s, client);
    s.pop_layer();
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}

/// Return a StoryView given a story list and the view description
pub fn get_story_view(
    desc: &str,
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
) -> impl View {
    let main_view = get_story_main_view(stories, client);
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(desc))
        .child(main_view)
        .child(construct_footer_view());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view).on_event(Event::AltChar('h'), |s| {
        s.add_layer(StoryView::construct_help_view())
    })
}
