use super::event_view;
use super::text_view;
use crate::prelude::*;

pub struct StoryView {
    view: LinearLayout,
    stories: Vec<hn_client::Story>,
}

impl StoryView {
    pub fn new(stories: Vec<hn_client::Story>) -> Self {
        let view = LinearLayout::vertical().with(|s| {
            stories.iter().enumerate().for_each(|(i, story)| {
                s.add_child(text_view::TextView::new(format!(
                    "{}. {}\n{} points | by {} | {} ago | {} comments",
                    i + 1,
                    story.title.clone().unwrap_or("[deleted]".to_string()),
                    story.points,
                    story.author.clone().unwrap_or("[deleted]".to_string()),
                    super::get_elapsed_time_as_text(story.time),
                    story.num_comments,
                )));
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
pub fn get_story_view(
    stories: Vec<hn_client::Story>,
    hn_client: &hn_client::HNClient,
) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();
    event_view::construct_event_view(StoryView::new(stories))
        .on_pre_event_inner(Key::Enter, move |s, _| {
            let hn_client = hn_client.clone();
            let id = s.get_inner().get_focus_index();

            match hn_client::get_comments_from_story_id(s.stories[id].id, &hn_client) {
                Ok(comments) => {
                    let story_url = s.stories[id].url.clone();
                    Some(EventResult::with_cb(move |s| {
                        s.pop_layer();
                        s.add_layer(comment_view::get_comment_view(
                            story_url.clone(),
                            &hn_client,
                            &comments,
                        ));
                    }))
                }
                Err(err) => {
                    error!(
                        "failed to get comments from story {}: {:#?}",
                        s.stories[id].id, err
                    );
                    None
                }
            }
        })
        .on_pre_event_inner('O', move |s, _| {
            let id = s.get_inner().get_focus_index();
            if s.stories[id].url.is_some() {
                let url = s.stories[id].url.clone().unwrap();
                match webbrowser::open(&url) {
                    Ok(_) => Some(EventResult::Consumed(None)),
                    Err(err) => {
                        error!("failed to open link {}: {}", url, err);
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
