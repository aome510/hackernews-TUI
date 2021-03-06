use super::error_view::{self, ErrorViewEnum, ErrorViewWrapper};
use crate::prelude::*;

/// Wrap comment_view::get_comment_view into an async_view with a loading screen
pub fn get_comment_view_async(
    siv: &mut Cursive,
    client: &hn_client::HNClient,
    story: &hn_client::Story,
) -> impl View {
    let id = story.id;
    let url = story.url.clone();
    AsyncView::new_with_bg_creator(
        siv,
        {
            let client = client.clone();
            move || Ok(hn_client::get_comments_from_story_id(id, &client))
        },
        {
            let client = client.clone();
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(comments) => ErrorViewEnum::Ok(comment_view::get_comment_view(
                        url.clone(),
                        &client,
                        &comments,
                    )),
                    Err(err) => ErrorViewEnum::Err(error_view::get_error_view(format!(
                        "failed to get comments from story {}: {:#?}",
                        id, err
                    ))),
                })
            }
        },
    )
}

/// Wrap story_view::get_story_view into an async_view with a loading screen
pub fn get_story_view_async(siv: &mut Cursive, client: &hn_client::HNClient) -> impl View {
    AsyncView::new_with_bg_creator(
        siv,
        {
            let client = client.clone();
            move || Ok(client.get_top_stories())
        },
        {
            let client = client.clone();
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(stories) => ErrorViewEnum::Ok(story_view::get_story_view(stories, &client)),
                    Err(err) => ErrorViewEnum::Err(error_view::get_error_view(format!(
                        "failed to get top stories: {:#?}",
                        err
                    ))),
                })
            }
        },
    )
}
