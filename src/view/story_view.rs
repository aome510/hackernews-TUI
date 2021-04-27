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
    pub fn new(stories: Vec<hn_client::Story>, starting_id: usize) -> Self {
        let view = LinearLayout::vertical()
            .with(|s| {
                stories.iter().enumerate().for_each(|(i, story)| {
                    let mut story_text = StyledString::plain(format!("{}. ", starting_id + i + 1));
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
    starting_id: usize,
) -> OnEventView<StoryView> {
    let story_view_keymap = &get_config_keymap().story_view_keymap;

    let is_suffix_key = |c: &Event| -> bool {
        *c == get_config_keymap()
            .story_view_keymap
            .goto_story_comment_view
            .clone()
            .into()
    };

    construct_scroll_list_event_view(StoryView::new(stories, starting_id))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            match *e {
                Event::Char(c) if '0' <= c && c <= '9' => {
                    s.raw_command.push(c);
                }
                _ => {
                    if !is_suffix_key(e) {
                        s.raw_command.clear();
                    }
                }
            };
            None
        })
        .on_pre_event_inner(story_view_keymap.prev_story.clone(), |s, _| {
            let id = s.get_focus_index();
            if id == 0 {
                None
            } else {
                s.set_focus_index(id - 1)
            }
        })
        .on_pre_event_inner(story_view_keymap.next_story.clone(), |s, _| {
            let id = s.get_focus_index();
            s.set_focus_index(id + 1)
        })
        .on_pre_event_inner(story_view_keymap.goto_story_comment_view.clone(), {
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
                        s.screen_mut().add_transparent_layer(Layer::new(async_view))
                    }
                }))
            }
        })
        .on_pre_event_inner(
            story_view_keymap.open_article_in_browser.clone(),
            move |s, _| {
                let id = s.get_focus_index();
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
            },
        )
        .on_pre_event_inner(
            story_view_keymap.open_story_in_browser.clone(),
            move |s, _| {
                let id = s.stories[s.get_focus_index()].id;
                thread::spawn(move || {
                    let url = format!("{}/item?id={}", hn_client::HN_HOST_URL, id);
                    if let Err(err) = webbrowser::open(&url) {
                        warn!("failed to open link {}: {}", url, err);
                    }
                });
                Some(EventResult::Consumed(None))
            },
        )
        .on_pre_event_inner(story_view_keymap.goto_story.clone(), move |s, _| {
            match s.raw_command.parse::<usize>() {
                Ok(number) => {
                    s.raw_command.clear();
                    if number < starting_id + 1 {
                        return None;
                    }
                    let number = number - 1 - starting_id;
                    if number < s.len() {
                        s.set_focus_index(number).unwrap();
                        Some(EventResult::Consumed(None))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
}

/// Return a StoryView given a story list and the view description
pub fn get_story_view(
    desc: &str,
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
    tag: &'static str,
    by_date: bool,
    page: usize,
    time_offset_in_secs: Option<u64>,
) -> impl View {
    let starting_id = CONFIG
        .get()
        .unwrap()
        .client
        .story_limit
        .get_story_limit_by_tag(tag)
        * page;
    let main_view = get_story_main_view(stories.clone(), client, starting_id).full_height();

    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(desc))
        .child(main_view)
        .child(construct_footer_view::<StoryView>());
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
                            warn!(
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

    let day_in_secs = 24 * 60 * 60;
    let story_view_keymap = &get_config_keymap().story_view_keymap;

    OnEventView::new(view)
        .on_pre_event(
            get_config_keymap().global_keymap.open_help_dialog.clone(),
            |s| s.add_layer(StoryView::construct_help_view()),
        )
        // time_offset filter options
        .on_event(story_view_keymap.filter_past_day.clone(), {
            let client = client.clone();
            move |s| {
                add_story_view_layer(s, &client, tag, by_date, page, Some(day_in_secs * 1), true);
            }
        })
        .on_event(story_view_keymap.filter_past_week.clone(), {
            let client = client.clone();
            move |s| {
                add_story_view_layer(s, &client, tag, by_date, page, Some(day_in_secs * 7), true);
            }
        })
        .on_event(story_view_keymap.filter_past_month.clone(), {
            let client = client.clone();
            move |s| {
                add_story_view_layer(s, &client, tag, by_date, page, Some(day_in_secs * 30), true);
            }
        })
        .on_event(story_view_keymap.filter_past_year.clone(), {
            let client = client.clone();
            move |s| {
                add_story_view_layer(
                    s,
                    &client,
                    tag,
                    by_date,
                    page,
                    Some(day_in_secs * 365),
                    true,
                );
            }
        })
        // toggle sort_by
        .on_event(story_view_keymap.toggle_sort_by.clone(), {
            let client = client.clone();
            move |s| {
                add_story_view_layer(s, &client, tag, !by_date, page, time_offset_in_secs, true);
            }
        })
        // paging
        .on_event(story_view_keymap.prev_page.clone(), {
            let client = client.clone();
            move |s| {
                if page > 0 {
                    add_story_view_layer(
                        s,
                        &client,
                        tag,
                        by_date,
                        page - 1,
                        time_offset_in_secs,
                        true,
                    );
                }
            }
        })
        .on_event(story_view_keymap.next_page.clone(), {
            let client = client.clone();
            move |s| {
                add_story_view_layer(
                    s,
                    &client,
                    tag,
                    by_date,
                    page + 1,
                    time_offset_in_secs,
                    true,
                );
            }
        })
}

/// Add StoryView as a new layer to the main Cursive View
pub fn add_story_view_layer(
    s: &mut Cursive,
    client: &hn_client::HNClient,
    tag: &'static str,
    by_date: bool,
    page: usize,
    time_offset_in_secs: Option<u64>,
    pop_layer: bool,
) {
    let async_view =
        async_view::get_story_view_async(s, client, tag, by_date, page, time_offset_in_secs);
    if pop_layer {
        s.pop_layer();
    }
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
