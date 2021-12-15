use super::error_view::{self, ErrorViewEnum, ErrorViewWrapper};
use super::{article_view, comment_view, story_view};
use crate::prelude::*;
use cursive_aligned_view::Alignable;
use cursive_async_view::AsyncView;

/// Return an async view wrapping CommentView of a HN story
/// with a loading screen when loading data
pub fn get_comment_view_async(
    siv: &mut Cursive,
    client: &'static client::HNClient,
    story: &client::Story,
) -> impl View {
    let id = story.id;

    AsyncView::new_with_bg_creator(siv, move || Ok(client.lazy_load_story_comments(id)), {
        let story = story.clone();
        move |result: Result<client::CommentReceiver>| {
            ErrorViewWrapper::new(match result {
                Ok(receiver) => ErrorViewEnum::Ok(comment_view::get_comment_view(&story, receiver)),
                Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                    &format!("failed to load comments from story (id={}):", id),
                    &err.to_string(),
                )),
            })
        }
    })
    .align_center()
    .full_screen()
}

/// Return an async view wrapping StoryView with a loading screen when loading data
pub fn get_story_view_async(
    siv: &mut Cursive,
    client: &'static client::HNClient,
    tag: &'static str,
    by_date: bool,
    page: usize,
    numeric_filters: client::StoryNumericFilters,
) -> impl View {
    AsyncView::new_with_bg_creator(
        siv,
        move || Ok(client.get_stories_by_tag(tag, by_date, page, numeric_filters)),
        move |result| {
            ErrorViewWrapper::new(match result {
                Ok(stories) => ErrorViewEnum::Ok(story_view::get_story_view(
                    stories,
                    client,
                    tag,
                    by_date,
                    page,
                    numeric_filters,
                )),
                Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                    &format!(
                        "failed to get stories (tag={}, by_date={}, page={}):",
                        tag, by_date, page
                    ),
                    &err.to_string(),
                )),
            })
        },
    )
    .align_center()
    .full_screen()
}

/// Return an async_view wrapping ArticleView with a loading screen when
/// parsing the Article data
pub fn get_article_view_async(siv: &mut Cursive, article_url: &str) -> impl View {
    let article_parse_command = config::get_config().article_parse_command.clone();
    let err_desc = format!(
        "failed to execute command `{} {} {}`:\n\
         Please make sure you have configured `article_parse_command` config option as described in (https://github.com/aome510/hackernews-TUI#article-parse-command)",
        article_parse_command.command,
        article_parse_command.options.join(" "),
        article_url
    );
    AsyncView::new_with_bg_creator(
        siv,
        {
            let article_url = article_url.to_owned();
            move || {
                Ok(std::process::Command::new(article_parse_command.command)
                    .args(&article_parse_command.options)
                    .arg(&article_url)
                    .output())
            }
        },
        {
            let article_url = article_url.to_owned();
            move |output| {
                ErrorViewWrapper::new(match output {
                    Ok(output) => {
                        if output.status.success() {
                            match serde_json::from_slice::<article_view::Article>(&output.stdout) {
                                Ok(mut article) => {
                                    article.update_url(&article_url);
                                    ErrorViewEnum::Ok(article_view::get_article_view(
                                        article, false,
                                    ))
                                }
                                Err(_) => {
                                    let stdout = std::str::from_utf8(&output.stdout).unwrap();
                                    warn!(
                                        "failed to deserialize {} into the `Article` struct:",
                                        stdout
                                    );
                                    ErrorViewEnum::Err(error_view::get_error_view(
                                        &err_desc, stdout,
                                    ))
                                }
                            }
                        } else {
                            ErrorViewEnum::Err(error_view::get_error_view(
                                &err_desc,
                                std::str::from_utf8(&output.stderr).unwrap(),
                            ))
                        }
                    }
                    Err(err) => {
                        ErrorViewEnum::Err(error_view::get_error_view(&err_desc, &err.to_string()))
                    }
                })
            }
        },
    )
    .align_center()
    .full_screen()
}
