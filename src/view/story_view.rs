use super::event_view;
use super::text_view;
use crate::prelude::*;

/// Return a cursive's View from a story list
pub fn get_story_view(
    stories: Vec<hn_client::Story>,
    hn_client: &hn_client::HNClient,
) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();
    let ids = stories.iter().map(|story| story.id).collect::<Vec<i32>>();
    event_view::construct_event_view(LinearLayout::vertical().with(|s| {
        stories.into_iter().enumerate().for_each(|(i, story)| {
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
    }))
    .on_pre_event_inner(Key::Enter, move |s, _| {
        let hn_client = hn_client.clone();
        let id = s.get_focus_index();
        match hn_client::get_comments_from_story_id(ids[id], &hn_client) {
            Ok(comments) => Some(EventResult::with_cb(move |s| {
                s.pop_layer();
                let comment_view = comment_view::CommentView::new(&comments);
                s.add_layer(comment_view.get_comment_view(&hn_client));
            })),
            Err(err) => {
                error!("failed to get comments from story {}: {:#?}", ids[id], err);
                None
            }
        }
    })
    .on_event('q', |s| s.quit())
    .scrollable()
}
