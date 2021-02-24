use crate::prelude::*;

/// Construct a new Event view from a SelectView by adding
/// event handlers for a key pressed
fn construct_event_view<T: 'static>(view: SelectView<T>) -> OnEventView<SelectView<T>> {
    // add "j" and "k" for moving down and up the story list
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        })
}

/// Return a cursive's View from a story list
pub fn get_story_view(
    stories: Vec<hn_client::Story>,
    hn_client: &hn_client::HNClient,
) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();
    construct_event_view(
        SelectView::new()
            .with_all(stories.into_iter().enumerate().map(|(i, story)| {
                (
                    format!(
                        "{}. {} (author: {}, {} comments, {} points)",
                        i + 1,
                        story.title.clone().unwrap_or("unknown title".to_string()),
                        story.author.clone().unwrap_or("-unknown_user-".to_string()),
                        story.num_comments,
                        story.points
                    ),
                    story,
                )
            }))
            .on_submit(
                move |s, story| match comment_view::get_comment_view(story, &hn_client) {
                    Err(err) => {
                        error!("failed to construct comment view: {:#?}", err);
                    }
                    Ok(comment_view) => {
                        s.pop_layer();
                        s.add_layer(comment_view);
                    }
                },
            ),
    )
    .scrollable()
}
