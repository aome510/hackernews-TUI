use regex::Regex;
use std::{
    thread::{self, sleep},
    time::Duration,
};

use super::async_view;
use super::help_view::*;
use super::list_view::*;
use super::text_view;
use super::utils::*;

use crate::prelude::*;

/// StoryView is a View displaying a list stories corresponding
/// to a particular category (top stories, newest stories, most popular stories, etc).
pub struct StoryView {
    view: ScrollListView,
    pub stories: Vec<hn_client::Story>,

    raw_command: String,
}

impl ViewWrapper for StoryView {
    wrap_impl!(self.view: ScrollListView);
}

impl StoryView {
    pub fn new(stories: Vec<hn_client::Story>) -> Self {
        let view = LinearLayout::vertical()
            .with(|s| {
                stories.iter().enumerate().for_each(|(i, story)| {
                    let mut story_text = StyledString::plain(format!("{}. ", i + 1));
                    story_text.append(Self::get_story_text(story));
                    s.add_child(text_view::TextView::new(story_text));
                })
            })
            .scrollable();
        StoryView {
            view,
            stories,
            raw_command: String::new(),
        }
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
                    let matched_text = c.name("match").unwrap().as_str();

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

                    styled_s.append_styled(
                        matched_text,
                        ColorStyle::new(
                            PaletteColor::TitlePrimary,
                            get_config_theme().search_highlight_bg.color,
                        ),
                    );
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
    fn get_story_text(story: &hn_client::Story) -> StyledString {
        let mut story_text =
            Self::get_matched_text(story.highlight_result.title.clone(), ColorStyle::default());
        if story.url.len() > 0 {
            let url = format!("\n{}", story.highlight_result.url);
            story_text.append(Self::get_matched_text(
                url,
                ColorStyle::front(get_config_theme().link_text.color),
            ));
        }
        story_text.append_styled(
            format!(
                "\n{} points | by {} | {} ago | {} comments",
                story.points,
                story.author,
                get_elapsed_time_as_text(story.time),
                story.num_comments,
            ),
            ColorStyle::from(PaletteColor::Secondary),
        );
        story_text
    }

    inner_getters!(self.view: ScrollListView);
}

/// Return a main view of a StoryView displaying the story list.
/// The main view of a StoryView is a View without status bar or footer.
pub fn get_story_main_view(
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
) -> OnEventView<StoryView> {
    construct_scroll_list_event_view(StoryView::new(stories))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| {
            match *e {
                Event::Char(c) if '0' <= c && c <= '9' => {
                    s.raw_command.push(c);
                }
                Event::Char('g') => {}
                _ => {
                    s.raw_command.clear();
                }
            };
            None
        })
        .on_pre_event_inner(Key::Enter, {
            let client = client.clone();
            move |s, _| {
                let id = s.get_focus_index();
                // the story struct hasn't had any comments inside yet,
                // so it can be cloned without greatly affecting performance
                let story = s.stories[id].clone();
                Some(EventResult::with_cb({
                    let client = client.clone();
                    move |s| {
                        let async_view = async_view::get_comment_view_async(s, &client, &story, 0);
                        s.pop_layer();
                        s.screen_mut().add_transparent_layer(Layer::new(async_view))
                    }
                }))
            }
        })
        .on_pre_event_inner('O', move |s, _| {
            let id = s.get_focus_index();
            let url = s.stories[id].url.clone();
            if url.len() > 0 {
                thread::spawn(move || {
                    if let Err(err) = webbrowser::open(&url) {
                        error!("failed to open link {}: {}", url, err);
                    }
                });
                Some(EventResult::Consumed(None))
            } else {
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner('S', move |s, _| {
            let id = s.stories[s.get_focus_index()].id;
            thread::spawn(move || {
                let url = format!("{}/item?id={}", hn_client::HN_HOST_URL, id);
                if let Err(err) = webbrowser::open(&url) {
                    error!("failed to open link {}: {}", url, err);
                }
            });
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('g', move |s, _| match s.raw_command.parse::<usize>() {
            Ok(number) => {
                s.raw_command.clear();
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
}

/// Return a StoryView given a story list and the view description
pub fn get_story_view(
    desc: &str,
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
    tag: &'static str,
    by_date: bool,
) -> impl View {
    let main_view = get_story_main_view(stories.clone(), client)
        .on_event(
            EventTrigger::from_fn(|e| match e {
                Event::CtrlChar('d') | Event::AltChar('d') => true,
                _ => false,
            }),
            {
                let client = client.clone();
                move |s| {
                    add_story_view_layer(s, &client, tag, !by_date);
                }
            },
        )
        .full_height();

    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(desc))
        .child(main_view)
        .child(construct_footer_view::<StoryView>(client));
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let story_pooling = &CONFIG.get().unwrap().story_pooling;

    // pooling stories in background
    if story_pooling.enable {
        if story_pooling
            .allows
            .iter()
            .any(|allowed_tag| allowed_tag == tag)
        {
            let client = client.clone();
            thread::spawn(move || {
                stories.iter().for_each(|story| {
                    match client.get_comments_from_story(story, false) {
                        Err(err) => {
                            error!(
                                "failed to get comments from story (id={}): {:#?}",
                                story.id, err
                            );
                        }
                        _ => {}
                    };

                    sleep(Duration::from_secs(story_pooling.delay));
                });
            });
        }
    }

    OnEventView::new(view).on_pre_event(
        EventTrigger::from_fn(|e| match e {
            Event::CtrlChar('h') | Event::AltChar('h') => true,
            _ => false,
        }),
        |s| s.add_layer(StoryView::construct_help_view()),
    )
}

/// Add StoryView as a new layer to the main Cursive View
pub fn add_story_view_layer(
    s: &mut Cursive,
    client: &hn_client::HNClient,
    tag: &'static str,
    by_date: bool,
) {
    let async_view = async_view::get_story_view_async(s, client, tag, by_date);
    s.pop_layer();
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
