use super::event_view;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;

/// StoryView is a View displaying a list stories corresponding
/// to a particular category (top stories, newest stories, most popular stories, etc).
pub struct StoryView {
    view: LinearLayout,
    stories: Vec<hn_client::Story>,
}

pub fn get_story_text(story: &hn_client::Story) -> StyledString {
    let mut story_text = StyledString::plain(format!(
        "{}",
        story.title.clone().unwrap_or("[deleted]".to_string())
    ));
    if story.url.is_some() {
        story_text.append_styled(
            format!("\n({})", shorten_url(story.url.clone().unwrap())),
            ColorStyle::from(LINK_COLOR),
        );
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
        StoryView { view, stories }
    }

    inner_getters!(self.view: LinearLayout);
}

impl ViewWrapper for StoryView {
    wrap_impl!(self.view: LinearLayout);
}

/// Return a cursive's View representing a StoryView of HN stories
pub fn get_story_view(stories: Vec<hn_client::Story>, client: &hn_client::HNClient) -> impl View {
    let client = client.clone();
    event_view::construct_event_view(StoryView::new(stories))
        .on_pre_event_inner(Key::Enter, move |s, _| {
            let client = client.clone();
            let id = s.get_inner().get_focus_index();
            let story = s.stories[id].clone();
            Some(EventResult::with_cb(move |s| {
                s.pop_layer();
                let async_view = async_view::get_comment_view_async(s, &client, &story);
                s.add_layer(async_view);
            }))
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
        .on_event('q', |s| s.quit())
        .scrollable()
}
