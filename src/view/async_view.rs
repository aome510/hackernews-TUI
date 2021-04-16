use super::error_view::{self, ErrorViewEnum, ErrorViewWrapper};
use super::utils::get_story_view_desc_by_tag;

use crate::prelude::*;
use cursive_async_view::AsyncView;

/// Return an async view wraps CommentView of a HN story
/// with a loading screen when loading data
pub fn get_comment_view_async(
    siv: &mut Cursive,
    client: &hn_client::HNClient,
    story: &hn_client::Story,
    focus_id: u32,
) -> impl View {
    let id = story.id;

    AsyncView::new_with_bg_creator(
        siv,
        {
            let client = client.clone();
            let story = story.clone();
            move || match client.get_comments_from_story(&story, focus_id > 0) {
                Ok(stories) => Ok(Ok(stories)),
                Err(err) => {
                    warn!(
                        "failed to get comments from story (id={}): {:#?}\nRetrying...",
                        id, err
                    );
                    Ok(client.get_comments_from_story(&story, focus_id > 0))
                }
            }
        },
        {
            let client = client.clone();
            let story = story.clone();
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(comments) => ErrorViewEnum::Ok(comment_view::get_comment_view(
                        &story, &comments, &client, focus_id,
                    )),
                    Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                        &format!("failed to get comments from story (id={})", id),
                        err,
                        &client,
                    )),
                })
            }
        },
    )
}

/// Return an async view wraps StoryView with a loading screen when loading data
pub fn get_story_view_async(
    siv: &mut Cursive,
    client: &hn_client::HNClient,
    tag: &'static str,
    by_date: bool,
) -> impl View {
    AsyncView::new_with_bg_creator(
        siv,
        {
            let client = client.clone();
            move || match client.get_stories_by_tag(tag, by_date) {
                Ok(stories) => Ok(Ok(stories)),
                Err(err) => {
                    warn!(
                        "failed to get stories (tag={}, by_date={}): {:#?}\nRetrying...",
                        err, tag, by_date
                    );
                    Ok(client.get_stories_by_tag(tag, by_date))
                }
            }
        },
        {
            let client = client.clone();
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(stories) => ErrorViewEnum::Ok(story_view::get_story_view(
                        &get_story_view_desc_by_tag(tag),
                        stories,
                        &client,
                    )),
                    Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                        &format!("failed to get stories (tag={}, by_date={})", tag, by_date),
                        err,
                        &client,
                    )),
                })
            }
        },
    )
}
