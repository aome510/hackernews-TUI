mod async_view;
mod fn_view_wrapper;
mod link_dialog;
mod result_view;
mod text_view;
mod traits;
mod utils;

pub mod article_view;
pub mod comment_view;
pub mod help_view;
pub mod search_view;
pub mod story_view;

use crate::view::help_view::HasHelpView;

use super::prelude::*;

fn set_up_switch_story_view_shortcut(
    keys: config::Keys,
    tag: &'static str,
    s: &mut Cursive,
    client: &'static client::HNClient,
    numeric_filters: Option<client::StoryNumericFilters>,
) {
    s.set_on_post_event(keys, move |s| {
        story_view::add_story_view_layer(
            s,
            client,
            tag,
            true,
            0,
            numeric_filters.unwrap_or_default(),
            false,
        );
    });
}

fn set_up_global_callbacks(s: &mut Cursive, client: &'static client::HNClient) {
    s.clear_global_callbacks(Event::CtrlChar('c'));

    let global_keymap = config::get_global_keymap().clone();

    // .............................................................
    // global shortcuts for switching between different Story Views
    // .............................................................

    set_up_switch_story_view_shortcut(
        global_keymap.goto_front_page_view,
        "front_page",
        s,
        client,
        None,
    );
    set_up_switch_story_view_shortcut(
        global_keymap.goto_all_stories_view,
        "story",
        s,
        client,
        None,
    );
    set_up_switch_story_view_shortcut(global_keymap.goto_ask_hn_view, "ask_hn", s, client, None);
    set_up_switch_story_view_shortcut(global_keymap.goto_show_hn_view, "show_hn", s, client, None);
    set_up_switch_story_view_shortcut(global_keymap.goto_jobs_view, "job", s, client, None);

    // custom navigation shortcuts
    config::get_config()
        .keymap
        .custom_keymaps
        .iter()
        .for_each(|data| {
            s.set_on_post_event(data.key.clone(), move |s| {
                story_view::add_story_view_layer(
                    s,
                    client,
                    &data.tag,
                    data.by_date,
                    0,
                    data.numeric_filters,
                    false,
                );
            });
        });

    // ............................................
    // end of navigation shortcuts for Story Views
    // ............................................

    s.set_on_post_event(global_keymap.goto_previous_view, |s| {
        if s.screen_mut().len() > 1 {
            s.pop_layer();
        }
    });

    s.set_on_post_event(global_keymap.goto_search_view, move |s| {
        search_view::add_search_view_layer(s, client);
    });

    s.set_on_post_event(global_keymap.open_help_dialog, |s| {
        s.add_layer(help_view::DefaultHelpView::construct_on_event_help_view())
    });

    s.set_on_post_event(global_keymap.quit, |s| s.quit());
}

/// Initialize the application's UI
pub fn init_ui(client: &'static client::HNClient) -> cursive::CursiveRunnable {
    let mut s = cursive::default();

    // initialize `cursive` color palette which is determined by the application's theme
    let theme = config::get_config_theme();
    s.update_theme(|t| {
        t.palette.set_color("view", theme.palette.background.into());
        t.palette
            .set_color("primary", theme.palette.foreground.into());
        t.palette
            .set_color("title_primary", theme.palette.foreground.into());
        t.palette
            .set_color("highlight", theme.palette.selection_background.into());
        t.palette
            .set_color("highlight_text", theme.palette.selection_foreground.into());
    });

    set_up_global_callbacks(&mut s, client);

    // render `front_page` story view as the application's startup view
    story_view::add_story_view_layer(
        &mut s,
        client,
        "front_page",
        false,
        0,
        client::StoryNumericFilters::default(),
        false,
    );

    s
}
