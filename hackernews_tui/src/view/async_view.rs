use super::{article_view, comment_view, result_view::ResultView, story_view};
use crate::prelude::*;
use anyhow::Context;
use cursive_aligned_view::Alignable;
use cursive_async_view::AsyncView;

pub fn construct_comment_view_async(
    siv: &mut Cursive,
    client: &'static client::HNClient,
    story: &client::Story,
) -> impl View {
    let id = story.id;

    // try to test upvote function
    if let Err(err) = client.upvote(id) {
        log::info!("Got an error {err}");
    }

    AsyncView::new_with_bg_creator(siv, move || Ok(client.lazy_load_story_comments(id)), {
        let story = story.clone();
        move |result: Result<_>| {
            ResultView::new(
                result.with_context(|| format!("failed to load comments from story (id={})", id)),
                |receiver| comment_view::construct_comment_view(&story, receiver),
            )
        }
    })
    .with_animation_fn(animation)
    .align_center()
    .full_screen()
}

pub fn construct_story_view_async(
    siv: &mut Cursive,
    client: &'static client::HNClient,
    tag: &'static str,
    sort_mode: client::StorySortMode,
    page: usize,
    numeric_filters: client::StoryNumericFilters,
) -> impl View {
    AsyncView::new_with_bg_creator(
        siv,
        move || Ok(client.get_stories_by_tag(tag, sort_mode, page, numeric_filters)),
        move |result| {
            ResultView::new(
                result.with_context(|| {
                    format!(
                        "failed to get stories (tag={}, sort_mode={:?}, page={}, numeric_filters={{{}}})",
                        tag, sort_mode, page, numeric_filters,
                    )
                }),
                |stories| {
                    story_view::construct_story_view(stories, client, tag, sort_mode, page, numeric_filters)
                },
            )
        },
    )
    .with_animation_fn(animation)
    .align_center()
    .full_screen()
}

pub fn construct_article_view_async(siv: &mut Cursive, article_url: &str) -> impl View {
    let err_context = format!(
        "Failed to execute the command:\n\
         `{} {}`.\n\n\
         Please make sure you have configured the `article_parse_command` option as described in the below link:\n\
         \"https://github.com/aome510/hackernews-TUI/blob/main/doc/config.md#article-parse-command\"",
        config::get_config().article_parse_command,
        article_url);

    AsyncView::new_with_bg_creator(
        siv,
        {
            let article_url = article_url.to_owned();
            move || Ok(client::HNClient::get_article(&article_url))
        },
        move |result| {
            let err_context = err_context.clone();
            ResultView::new(result.with_context(|| err_context), |article| {
                article_view::construct_article_view(article)
            })
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

        let content = crate::utils::combine_styled_strings(vec![
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

        let content = crate::utils::combine_styled_strings(vec![
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
