use super::error_view::{self, ErrorViewEnum, ErrorViewWrapper};
use super::utils::get_story_view_desc_by_tag;

use crate::prelude::*;
use cursive_aligned_view::Alignable;
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
                    )),
                })
            }
        },
    )
    .align_center()
    .full_screen()
}

/// Return an async view wraps StoryView with a loading screen when loading data
pub fn get_story_view_async(
    siv: &mut Cursive,
    client: &hn_client::HNClient,
    tag: &'static str,
    by_date: bool,
    page: usize,
    time_offset_in_secs: Option<u64>,
) -> impl View {
    AsyncView::new_with_bg_creator(
        siv,
        {
            let client = client.clone();
            move || match client.get_stories_by_tag(tag, by_date, page, time_offset_in_secs) {
                Ok(stories) => Ok(Ok(stories)),
                Err(err) => {
                    warn!(
                        "failed to get stories (tag={}, by_date={}, page={}): {:#?}\nRetrying...",
                        err, tag, by_date, page
                    );
                    Ok(client.get_stories_by_tag(tag, by_date, page, time_offset_in_secs))
                }
            }
        },
        {
            let client = client.clone();
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(stories) => ErrorViewEnum::Ok(story_view::get_story_view(
                        &get_story_view_desc_by_tag(tag, by_date, page, time_offset_in_secs),
                        stories,
                        &client,
                        tag,
                        by_date,
                        page,
                        time_offset_in_secs,
                    )),
                    Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                        &format!(
                            "failed to get stories (tag={}, by_date={}, page={})",
                            tag, by_date, page
                        ),
                        err,
                    )),
                })
            }
        },
    )
    .align_center()
    .full_screen()
}

pub fn get_article_view_async(siv: &mut Cursive, article_url: String) -> impl View {
    AsyncView::new_with_bg_creator(
        siv,
        move || {
            let stdout_output = std::process::Command::new("mercury-parser")
                .arg("--format")
                .arg("markdown")
                .arg(&article_url)
                .output()
                .unwrap()
                .stdout;
            Ok(std::str::from_utf8(&stdout_output).unwrap().to_string())
        },
        |result| ArticleView::new(result),
    )
    .align_center()
    .full_screen()
}
