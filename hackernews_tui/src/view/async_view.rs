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
    .with_animation_fn(animation)
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
    .with_animation_fn(animation)
    .align_center()
    .full_screen()
}

/// Return an async_view wrapping ArticleView with a loading screen when
/// parsing the Article data
pub fn get_article_view_async(siv: &mut Cursive, article_url: &str) -> impl View {
    let err_desc = format!(
        "Failed to execute the command: `{} {}`.\n\
            Please make sure you have configured the `article_parse_command` option as described in the below link:\n\
            \"https://github.com/aome510/hackernews-TUI/blob/main/config.md#article-parse-command\"",
        config::get_config().article_parse_command,
        article_url
    );

    AsyncView::new_with_bg_creator(
        siv,
        {
            let article_url = article_url.to_owned();
            move || Ok(client::HNClient::get_article(&article_url))
        },
        {
            move |result| {
                ErrorViewWrapper::new(match result {
                    Ok(article) => ErrorViewEnum::Ok(article_view::get_article_view(article)),
                    Err(err) => {
                        ErrorViewEnum::Err(error_view::get_error_view(&err_desc, &err.to_string()))
                    }
                })
            }
        },
    )
    .with_animation_fn(animation)
    .align_center()
    .full_screen()
}

fn animation(width: usize, _height: usize, frame_idx: usize) -> cursive_async_view::AnimationFrame {
    let n_frames = 120; // number of frames to complete an animation
    let style = ColorStyle::from(config::get_config_theme().component_style.loading_bar);

    if config::get_config().use_pacman_loading {
        let factor = (frame_idx as f64) / (n_frames as f64);
        let x = (factor * width as f64) as usize;

        let content = utils::combine_styled_strings(vec![
            StyledString::styled(repeat_str("- ", x / 2), style),
            StyledString::styled('ᗧ', style),
            StyledString::styled(repeat_str(" o", width.saturating_sub(x + 1) / 2), style),
        ]);

        cursive_async_view::AnimationFrame {
            content,
            next_frame_idx: (frame_idx + 1) % n_frames,
        }
    } else {
        let symbol = "━";
        let factor = (frame_idx as f64) / (n_frames as f64);
        let x = (factor * width as f64) as usize;

        let content = utils::combine_styled_strings(vec![
            StyledString::styled(repeat_str(symbol, x), style.back),
            StyledString::styled(repeat_str(symbol, width - x), style.front),
        ]);

        cursive_async_view::AnimationFrame {
            content,
            next_frame_idx: (frame_idx + 1) % n_frames,
        }
    }
}

fn repeat_str<S: Into<String>>(s: S, n: usize) -> String {
    s.into().repeat(n)
}
