use super::{
    article_view, async_view, comment_view, help_view::HasHelpView, text_view, traits::*, utils,
};
use crate::client::StoryNumericFilters;
use crate::prelude::*;

static STORY_TAGS: [&str; 5] = ["front_page", "story", "ask_hn", "show_hn", "job"];

/// StoryView is a View displaying a list stories corresponding
/// to a particular category (top stories, newest stories, most popular stories, etc).
pub struct StoryView {
    pub stories: Vec<client::Story>,

    view: ScrollView<LinearLayout>,
    raw_command: String,
}

impl ViewWrapper for StoryView {
    wrap_impl!(self.view: ScrollView<LinearLayout>);
}

impl StoryView {
    pub fn new(stories: Vec<client::Story>, starting_id: usize) -> Self {
        StoryView {
            view: Self::construct_story_view(&stories, starting_id),
            stories,
            raw_command: String::new(),
        }
    }

    fn construct_story_view(
        stories: &[client::Story],
        starting_id: usize,
    ) -> ScrollView<LinearLayout> {
        // Determine the maximum length of a story's ID string.
        // This maximum length is used to align the display of the story IDs.
        let max_id_len = {
            let max_id = starting_id + stories.len() + 1;
            let mut width = 0;
            let mut pw = 1;
            while pw <= max_id {
                pw *= 10;
                width += 1;
            }

            width
        };

        LinearLayout::vertical()
            .with(|s| {
                stories.iter().enumerate().for_each(|(i, story)| {
                    let mut story_text = StyledString::styled(
                        format!("{1:>0$}. ", max_id_len, starting_id + i + 1),
                        config::get_config_theme().component_style.metadata,
                    );
                    story_text.append(Self::get_story_text(max_id_len, story));

                    s.add_child(text_view::TextView::new(story_text));
                })
            })
            .scrollable()
    }

    /// Get the text summarizing basic information about a story
    fn get_story_text(max_id_len: usize, story: &client::Story) -> StyledString {
        let mut story_text = story.title.clone();

        if let Ok(url) = url::Url::parse(&story.url) {
            if let Some(domain) = url.domain() {
                story_text.append_styled(
                    format!(" ({domain})"),
                    config::get_config_theme().component_style.link,
                );
            }
        }

        story_text.append_plain("\n");

        story_text.append_styled(
            // left-align the story's metadata by `max_id_len+2`
            // which is the width of the string "{max_story_id}. "
            format!(
                "{:width$}{} points | by {} | {} ago | {} comments",
                " ",
                story.points,
                story.author,
                crate::utils::get_elapsed_time_as_text(story.time),
                story.num_comments,
                width = max_id_len + 2,
            ),
            config::get_config_theme().component_style.metadata,
        );
        story_text
    }

    inner_getters!(self.view: ScrollView<LinearLayout>);
}

impl ListViewContainer for StoryView {
    fn get_inner_list(&self) -> &LinearLayout {
        self.get_inner().get_inner()
    }

    fn get_inner_list_mut(&mut self) -> &mut LinearLayout {
        self.get_inner_mut().get_inner_mut()
    }

    fn on_set_focus_index(&mut self, old_id: usize, new_id: usize) {
        let direction = old_id <= new_id;

        // enable auto-scrolling when changing the focused index of the view
        self.scroll(direction);
    }
}

impl ScrollViewContainer for StoryView {
    type ScrollInner = LinearLayout;

    fn get_inner_scroll_view(&self) -> &ScrollView<LinearLayout> {
        self.get_inner()
    }

    fn get_inner_scroll_view_mut(&mut self) -> &mut ScrollView<LinearLayout> {
        self.get_inner_mut()
    }
}

pub fn construct_story_main_view(
    stories: Vec<client::Story>,
    client: &'static client::HNClient,
    starting_id: usize,
) -> OnEventView<StoryView> {
    let is_suffix_key =
        |c: &Event| -> bool { config::get_story_view_keymap().goto_story.has_event(c) };

    let story_view_keymap = config::get_story_view_keymap().clone();

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

            // don't allow the inner `LinearLayout` child view to handle the event
            // because of its pre-defined `on_event` function
            Some(EventResult::Ignored)
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
                move |s| comment_view::construct_and_add_new_comment_view(s, client, &story, false)
            }))
        })
        // open external link shortcuts
        .on_pre_event_inner(story_view_keymap.open_article_in_browser, move |s, _| {
            let id = s.get_focus_index();
            utils::open_url_in_browser(s.stories[id].get_url().as_ref());
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(
            story_view_keymap.open_article_in_article_view,
            move |s, _| {
                let id = s.get_focus_index();
                let url = s.stories[id].url.clone();
                if !url.is_empty() {
                    Some(EventResult::with_cb({
                        move |s| article_view::construct_and_add_new_article_view(s, &url)
                    }))
                } else {
                    Some(EventResult::Consumed(None))
                }
            },
        )
        .on_pre_event_inner(story_view_keymap.open_story_in_browser, move |s, _| {
            let url = s.stories[s.get_focus_index()].story_url();
            utils::open_url_in_browser(&url);
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
        .on_scroll_events()
}

fn get_story_view_title_bar(tag: &'static str, sort_mode: client::StorySortMode) -> impl View {
    let style = config::get_config_theme().component_style.title_bar;
    let mut title = StyledString::styled(
        "[Y]",
        Style::from(style).combine(ColorStyle::front(
            config::get_config_theme().palette.light_white,
        )),
    );
    title.append_styled(" Hacker News", style);

    for (i, item) in STORY_TAGS.iter().enumerate() {
        title.append_styled(" | ", style);
        if *item == tag {
            let sort_mode_desc = match sort_mode {
                client::StorySortMode::None => "",
                client::StorySortMode::Date => " (by_date)",
                client::StorySortMode::Points => " (by_point)",
            };
            title.append_styled(
                format!("{}.{}{}", i + 1, item, sort_mode_desc),
                Style::from(style)
                    .combine(config::get_config_theme().component_style.current_story_tag),
            );
        } else {
            title.append_styled(format!("{}.{}", i + 1, item), style);
        }
    }
    title.append_styled(" | ", style);

    PaddedView::lrtb(
        0,
        0,
        0,
        1,
        Layer::with_color(TextView::new(title), style.into()),
    )
}

/// Construct a story view given a list of stories.
pub fn construct_story_view(
    stories: Vec<client::Story>,
    client: &'static client::HNClient,
    tag: &'static str,
    sort_mode: client::StorySortMode,
    page: usize,
    numeric_filters: client::StoryNumericFilters,
) -> impl View {
    let starting_id = client::STORY_LIMIT * page;
    let main_view = construct_story_main_view(stories, client, starting_id).full_height();

    let mut view = LinearLayout::vertical()
        .child(get_story_view_title_bar(tag, sort_mode))
        .child(main_view)
        .child(utils::construct_footer_view::<StoryView>());
    view.set_focus_index(1)
        .unwrap_or(EventResult::Consumed(None));

    let current_tag_pos = STORY_TAGS
        .iter()
        .position(|t| *t == tag)
        .unwrap_or_else(|| panic!("unkwnown tag {tag}"));

    let story_view_keymap = config::get_story_view_keymap().clone();

    // Because we re-use the story main view to construct a search view,
    // some of the story keymaps need to be handled here instead of by the main view like
    // for comment views or article views.

    OnEventView::new(view)
        .on_pre_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(StoryView::construct_on_event_help_view())
        })
        .on_pre_event(story_view_keymap.cycle_sort_mode, move |s| {
            // disable "search_by_date" for front_page stories
            if tag == "front_page" {
                return;
            }
            construct_and_add_new_story_view(
                s,
                client,
                tag,
                sort_mode.next(tag),
                0,
                numeric_filters,
                true,
            );
        })
        // story tag navigation
        .on_pre_event(story_view_keymap.next_story_tag, move |s| {
            let next_tag = STORY_TAGS[(current_tag_pos + 1) % STORY_TAGS.len()];
            construct_and_add_new_story_view(
                s,
                client,
                next_tag,
                if next_tag == "story" || next_tag == "job" {
                    client::StorySortMode::Date
                } else {
                    client::StorySortMode::None
                },
                0,
                StoryNumericFilters::default(),
                false,
            );
        })
        .on_pre_event(story_view_keymap.prev_story_tag, move |s| {
            let prev_tag = STORY_TAGS[(current_tag_pos + STORY_TAGS.len() - 1) % STORY_TAGS.len()];
            construct_and_add_new_story_view(
                s,
                client,
                prev_tag,
                if prev_tag == "story" || prev_tag == "job" {
                    client::StorySortMode::Date
                } else {
                    client::StorySortMode::None
                },
                0,
                StoryNumericFilters::default(),
                false,
            );
        })
        // paging
        .on_pre_event(story_view_keymap.prev_page, move |s| {
            if page > 0 {
                construct_and_add_new_story_view(
                    s,
                    client,
                    tag,
                    sort_mode,
                    page - 1,
                    numeric_filters,
                    true,
                );
            }
        })
        .on_pre_event(story_view_keymap.next_page, move |s| {
            construct_and_add_new_story_view(
                s,
                client,
                tag,
                sort_mode,
                page + 1,
                numeric_filters,
                true,
            );
        })
}

/// Retrieve a list of stories satisfying some conditions and construct a story view displaying them.
pub fn construct_and_add_new_story_view(
    s: &mut Cursive,
    client: &'static client::HNClient,
    tag: &'static str,
    sort_mode: client::StorySortMode,
    page: usize,
    numeric_filters: client::StoryNumericFilters,
    pop_layer: bool,
) {
    let async_view =
        async_view::construct_story_view_async(s, client, tag, sort_mode, page, numeric_filters);
    if pop_layer {
        s.pop_layer();
    }
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
