use super::error_view::{self, ErrorViewEnum, ErrorViewWrapper};
use super::{comment_view, story_view};
use crate::prelude::*;

/// Return an async view wraps CommentView of a HN story
/// with a loading screen when loading data
pub fn get_comment_view_async(
    siv: &mut Cursive,
    client: &hn_client::HNClient,
    story: &hn_client::Story,
) -> impl View {
    let id = story.id;
    let url = story.url.clone();
    let title = story.title.clone();

    AsyncView::new_with_bg_creator(
        siv,
        {
            let client = client.clone();
            move || Ok(client.get_comments_from_story_id(id))
        },
        {
            let client = client.clone();
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(comments) => ErrorViewEnum::Ok(comment_view::get_comment_view(
                        &title, &url, &client, &comments,
                    )),
                    Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                        &format!("failed to get comments from story {}", id),
                        err,
                        &client,
                    )),
                })
            }
        },
    )
}

/// Return an async view wraps StoryView (front page)
/// with a loading screen when loading data
pub fn get_front_page_story_view(siv: &mut Cursive, client: &hn_client::HNClient) -> impl View {
    AsyncView::new_with_bg_creator(
        siv,
        {
            let client = client.clone();
            move || Ok(client.get_front_page_stories())
        },
        {
            let client = client.clone();
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(stories) => ErrorViewEnum::Ok(story_view::get_story_view(
                        "Story View - Front Page",
                        stories,
                        &client,
                    )),
                    Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                        "failed to get top stories",
                        err,
                        &client,
                    )),
                })
            }
        },
    )
}
