use super::event_view;
use super::search_view;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;

/// StoryView is a View displaying a list stories corresponding
/// to a particular category (top stories, newest stories, most popular stories, etc).
pub struct StoryView {
    raw_command: String,
    view: LinearLayout,
    pub stories: Vec<hn_client::Story>,
}

pub fn get_story_text(story: &hn_client::Story) -> StyledString {
    let mut story_text = StyledString::plain(format!(
        "{}",
        story.title.clone().unwrap_or("[deleted]".to_string())
    ));
    if story.url.is_some() {
        let story_url = story.url.clone().unwrap();
        if story_url.len() > 0 {
            story_text.append_styled(
                format!("\n({})", shorten_url(story_url)),
                ColorStyle::from(LINK_COLOR),
            );
        }
    }
    story_text.append_styled(
        format!(
            "\n{} points | by {} | {} ago | {} comments",
            story.points,
            story.author.clone().unwrap_or("[deleted]".to_string()),
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
            raw_command: String::new(),
            view,
            stories,
        }
    }

    crate::raw_command!();

    inner_getters!(self.view: LinearLayout);
}

impl ViewWrapper for StoryView {
    wrap_impl!(self.view: LinearLayout);
}

pub fn get_story_main_view(
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
) -> impl View {
    let client = client.clone();
    let stories = stories
        .into_iter()
        .filter(|story| story.title.is_some())
        .collect();
    event_view::construct_list_event_view(StoryView::new(stories))
        .on_pre_event_inner(Key::Enter, {
            move |s, _| {
                let id = s.get_inner().get_focus_index();
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
            if s.stories[id].url.is_some() {
                let url = s.stories[id].url.clone().unwrap();
                match webbrowser::open(&url) {
                    Ok(_) => Some(EventResult::Consumed(None)),
                    Err(err) => {
                        warn!("failed to open link {}: {}", url, err);
                        None
                    }
                }
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

/// Return a cursive's View representing a StoryView of HN stories
pub fn get_story_view(stories: Vec<hn_client::Story>, client: &hn_client::HNClient) -> impl View {
    let main_view = get_story_main_view(stories, client);
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc("Story View - Front Page"))
        .child(main_view)
        .child(construct_footer_view());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view).on_event(Event::AltChar('s'), {
        let client = client.clone();
        move |s| {
            let cb_sink = s.cb_sink().clone();
            s.pop_layer();
            s.screen_mut()
                .add_transparent_layer(Layer::new(search_view::get_search_view(&client, cb_sink)))
        }
    })
}
