use regex::Regex;

use super::async_view;
use super::help_view::*;
use super::list_view::*;
use super::text_view;

use crate::prelude::*;

/// StoryView is a View displaying a list stories corresponding
/// to a particular category (top stories, newest stories, most popular stories, etc).
pub struct StoryView {
    view: ScrollListView,
    pub stories: Vec<client::Story>,

    raw_command: String,
}

impl ViewWrapper for StoryView {
    wrap_impl!(self.view: ScrollListView);
}

impl StoryView {
    pub fn new(stories: Vec<client::Story>, starting_id: usize) -> Self {
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

                    if !prefix.is_empty() {
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
        if !s.is_empty() {
            styled_s.append_styled(s, default_style);
        }
        styled_s
    }

    /// Get the description text summarizing basic information about a story
    fn get_story_text(story: &client::Story) -> StyledString {
        let mut story_text =
            Self::get_matched_text(story.highlight_result.title.clone(), ColorStyle::default());
        if !story.url.is_empty() {
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
    stories: Vec<client::Story>,
    client: &'static client::HNClient,
    starting_id: usize,
) -> OnEventView<StoryView> {
    let story_view_keymap = get_story_view_keymap().clone();

    let is_suffix_key =
        |c: &Event| -> bool { *c == get_story_view_keymap().goto_story.clone().into() };

    OnEventView::new(StoryView::new(stories, starting_id))
        // number parsing
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            match *e {
                Event::Char(c) if ('0'..='9').contains(&c) => {
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
        // story navigation shortcuts
        .on_pre_event_inner(story_view_keymap.prev_story, |s, _| {
            let id = s.get_focus_index();
            if id == 0 {
                None
            } else {
                s.set_focus_index(id - 1)
            }
        })
        .on_pre_event_inner(story_view_keymap.next_story, |s, _| {
            let id = s.get_focus_index();
            s.set_focus_index(id + 1)
        })
        .on_pre_event_inner(story_view_keymap.goto_story_comment_view, move |s, _| {
            let id = s.get_focus_index();
            // the story struct hasn't had any comments inside yet,
            // so it can be cloned without greatly affecting performance
            let story = s.stories[id].clone();
            Some(EventResult::with_cb({
                move |s| comment_view::add_comment_view_layer(s, client, &story, (0, 0), false)
            }))
        })
        // open external link shortcuts
        .on_pre_event_inner(story_view_keymap.open_article_in_browser, move |s, _| {
            let id = s.get_focus_index();
            open_url_in_browser(&s.stories[id].url);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(
            story_view_keymap.open_article_in_article_view,
            move |s, _| {
                let id = s.get_focus_index();
                let url = s.stories[id].url.clone();
                if !url.is_empty() {
                    Some(EventResult::with_cb({
                        move |s| article_view::add_article_view_layer(s, &url)
                    }))
                } else {
                    Some(EventResult::Consumed(None))
                }
            },
        )
        .on_pre_event_inner(story_view_keymap.open_story_in_browser, move |s, _| {
            let id = s.stories[s.get_focus_index()].id;
            let url = format!("{}/item?id={}", client::HN_HOST_URL, id);
            open_url_in_browser(&url);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(story_view_keymap.goto_story, move |s, _| {
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
    stories: Vec<client::Story>,
    client: &'static client::HNClient,
    tag: &'static str,
    by_date: bool,
    page: usize,
    numeric_filters: client::StoryNumericFilters,
) -> impl View {
    let starting_id = get_config().client.story_limit.get_story_limit_by_tag(tag) * page;
    let main_view = get_story_main_view(stories, client, starting_id).full_height();

    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(desc))
        .child(main_view)
        .child(construct_footer_view::<StoryView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let story_view_keymap = get_story_view_keymap().clone();

    OnEventView::new(view)
        .on_pre_event(get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(StoryView::construct_help_view())
        })
        // toggle sort_by
        .on_event(story_view_keymap.toggle_sort_by, move |s| {
            // disable "search_by_date" for front_page stories
            if tag == "front_page" {
                return;
            }
            add_story_view_layer(s, client, tag, !by_date, page, numeric_filters, true);
        })
        // paging
        .on_event(story_view_keymap.prev_page, move |s| {
            if page > 0 {
                add_story_view_layer(s, client, tag, by_date, page - 1, numeric_filters, true);
            }
        })
        .on_event(story_view_keymap.next_page, move |s| {
            add_story_view_layer(s, client, tag, by_date, page + 1, numeric_filters, true);
        })
}

/// Add a StoryView as a new layer to the main Cursive View
pub fn add_story_view_layer(
    s: &mut Cursive,
    client: &'static client::HNClient,
    tag: &'static str,
    by_date: bool,
    page: usize,
    numeric_filters: client::StoryNumericFilters,
    pop_layer: bool,
) {
    let async_view =
        async_view::get_story_view_async(s, client, tag, by_date, page, numeric_filters);
    if pop_layer {
        s.pop_layer();
    }
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
