use crate::prelude::*;

use super::{article_view, comment_view, search_view, story_view};

type CommandGroupsView = ScrollView<LinearLayout>;

/// A help item used to describe a command and its keybindings
pub struct Command {
    keys_desc: String,
    desc: String,
}

/// A group of command help items grouped by certain categories
pub struct CommandGroup {
    desc: String,
    commands: Vec<Command>,
}

impl Command {
    pub fn new<X, Y>(keys_desc: X, desc: Y) -> Self
    where
        X: Into<String>,
        Y: Into<String>,
    {
        Self {
            keys_desc: keys_desc.into(),
            desc: desc.into(),
        }
    }

    /// converts into a command View which consists of
    /// - a keybindings text View
    /// - a command description text View
    pub fn to_command_view(self, width: Option<usize>) -> impl View {
        let key = StyledString::styled(
            self.keys_desc,
            config::get_config_theme().component_style.single_code_block,
        );

        let mut view = LinearLayout::horizontal();

        // command's keybindings
        match width {
            Some(width) => view.add_child(TextView::new(key).fixed_width(width)),
            None => view.add_child(TextView::new(key)),
        }

        // command's description
        view.add_child(TextView::new(format!(" {}", self.desc)));
        view
    }
}

impl CommandGroup {
    pub fn new<T>(desc: T, commands: Vec<Command>) -> Self
    where
        T: Into<String>,
    {
        Self {
            desc: desc.into(),
            commands,
        }
    }

    /// converts into a command group View which consists of multiple command View(s)
    pub fn to_group_view(self) -> impl View {
        let max_keys_len = match self
            .commands
            .iter()
            .max_by_key(|command| command.keys_desc.len())
        {
            None => 0,
            Some(command) => command.keys_desc.len(),
        };

        LinearLayout::vertical()
            // group description
            .child(TextView::new(StyledString::styled(
                self.desc,
                config::get_config_theme().component_style.bold,
            )))
            // a list of command View(s) in the group
            .with(|s| {
                for command in self.commands {
                    s.add_child(command.to_command_view(Some(max_keys_len)));
                }
            })
            // a bottom padding
            .child(TextView::new("\n"))
    }
}

/// HelpView is a help dialog displaying a list of commands with
/// corresponding keybindings and command descriptions.
///
/// Commands are grouped by certain categories.
pub struct HelpView {
    view: Dialog,
}

impl HelpView {
    pub fn new() -> Self {
        HelpView {
            view: Dialog::new()
                .title("Help Dialog")
                .content(LinearLayout::vertical().scrollable()),
        }
    }

    /// construct a new help view from the current one by appending new key groups
    pub fn command_groups(mut self, groups: Vec<CommandGroup>) -> Self {
        let content = self
            .view
            .get_content_mut()
            .downcast_mut::<CommandGroupsView>()
            .expect("the dialog's content should be a `CommandGroupsView`");

        for group in groups {
            content.get_inner_mut().add_child(group.to_group_view());
        }
        self
    }
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewWrapper for HelpView {
    wrap_impl!(self.view: Dialog);
}

// #[macro_export]
// macro_rules! other_key_shortcuts {
//     ($(($k:expr,$d:expr)),*) => {
//         (
//             "Others",
//             vec![
//                 $(
//                     ($k, $d),
//                 )*
//                 (config::get_global_keymap().open_help_dialog.to_string(), "Open the help dialog"),
//                 (config::get_global_keymap().quit.to_string(), "Quit the application"),
//                 (config::get_global_keymap().close_dialog.to_string(), "Close a dialog"),
//             ],
//         )
//     };
// }

// #[macro_export]
// macro_rules! view_navigation_key_shortcuts {
//     ($(($k:expr,$d:expr)),*) => {
//         (
//             "View Navigation",
//             vec![
//                 $(
//                     ($k, $d),
//                 )*
//                     (config::get_global_keymap().goto_previous_view.to_string(), "Go to the previous view"),
//                     (config::get_global_keymap().goto_search_view.to_string(), "Go to search view"),
//                     (config::get_global_keymap().goto_front_page_view.to_string(), "Go to front page view"),
//                     (config::get_global_keymap().goto_all_stories_view.to_string(), "Go to all stories view"),
//                     (config::get_global_keymap().goto_ask_hn_view.to_string(), "Go to ask HN view"),
//                     (config::get_global_keymap().goto_show_hn_view.to_string(), "Go to show HN view"),
//                     (config::get_global_keymap().goto_jobs_view.to_string(), "Go to jobs view"),
//             ],

//         )
//     };
// }

pub trait HasHelpView {
    fn construct_help_view() -> HelpView;
}

/// An empty struct representing a default HelpView
pub struct DefaultHelpView {}

impl HasHelpView for DefaultHelpView {
    fn construct_help_view() -> HelpView {
        HelpView::new()
    }
}

impl HasHelpView for story_view::StoryView {
    fn construct_help_view() -> HelpView {
        let story_view_keymap = config::get_story_view_keymap();
        let custom_keymaps = config::get_config()
            .keymap
            .custom_keymaps
            .iter()
            .map(|keymap| {
                Command::new(
                    keymap.key.to_string(),
                    format!(
                        "Go to {} view (by_date: {}, {})",
                        match keymap.tag.as_str() {
                            "front_page" => "front page",
                            "story" => "all stories",
                            "job" => "jobs",
                            "ask_hn" => "ask HN",
                            "show_hn" => "show HN",
                            _ => panic!("unknown view: {}", keymap.tag),
                        },
                        keymap.by_date,
                        keymap.numeric_filters.desc()
                    ),
                )
            })
            .collect::<Vec<_>>();

        let mut help_view = HelpView::new().command_groups(vec![
            CommandGroup::new(
                "Navigation",
                vec![
                    Command::new(
                        story_view_keymap.next_story.to_string(),
                        "Focus the next story",
                    ),
                    Command::new(
                        story_view_keymap.prev_story.to_string(),
                        "Focus the previous story",
                    ),
                    Command::new(
                        format!("{{story_id}} {}", story_view_keymap.goto_story),
                        "Focus the {story_id}-th story",
                    ),
                ],
            ),
            CommandGroup::new(
                "Paging/Filtering",
                vec![
                    Command::new(
                        story_view_keymap.next_page.to_string(),
                        "Go to the next page",
                    ),
                    Command::new(
                        story_view_keymap.prev_page.to_string(),
                        "Go the previous page",
                    ),
                    Command::new(
                        story_view_keymap.toggle_sort_by_date.to_string(),
                        "Toggle sort by date (only for non `Front Page` views)",
                    ),
                ],
            ),
            CommandGroup::new(
                "Open external links",
                vec![
                    Command::new(
                        story_view_keymap.open_article_in_browser.to_string(),
                        "Open in browser the article associated with the focused story",
                    ),
                    Command::new(
                        story_view_keymap.open_article_in_article_view.to_string(),
                        "Open in article view the article associated with the focused story",
                    ),
                    Command::new(
                        story_view_keymap.open_story_in_browser.to_string(),
                        "Open in browser the focused story",
                    ),
                ],
            ),
        ]);
        if !custom_keymaps.is_empty() {
            help_view =
                help_view.command_groups(vec![CommandGroup::new("Custom keymaps", custom_keymaps)]);
        }
        help_view
        // help_view.command_groups(vec![
        //     view_navigation_key_shortcuts!(
        //         (
        //             story_view_keymap.goto_story_comment_view.to_string(),
        //             "Go to the comment view associated with the focused story"
        //         ),
        //         (
        //             story_view_keymap.next_story_tag.to_string(),
        //             "Go to the next story tag"
        //         ),
        //         (
        //             story_view_keymap.prev_story_tag.to_string(),
        //             "Go to the previous story tag"
        //         )
        //     ),
        //     other_key_shortcuts!(),
        // ])
    }
}

impl HasHelpView for comment_view::CommentView {
    fn construct_help_view() -> HelpView {
        HelpView::new()
        // let comment_view_keymap = config::get_comment_view_keymap();
        // let story_view_keymap = config::get_story_view_keymap();

        // HelpView::new().command_groups(vec![
        //     (
        //         "Navigation",
        //         vec![
        //             (
        //                 comment_view_keymap.next_comment.to_string(),
        //                 "Focus the next comment",
        //             ),
        //             (
        //                 comment_view_keymap.prev_comment.to_string(),
        //                 "Focus the previous comment",
        //             ),
        //             (
        //                 comment_view_keymap.next_top_level_comment.to_string(),
        //                 "Focus the next top level comment",
        //             ),
        //             (
        //                 comment_view_keymap.prev_top_level_comment.to_string(),
        //                 "Focus the previous top level comment",
        //             ),
        //             (
        //                 comment_view_keymap.next_leq_level_comment.to_string(),
        //                 "Focus the next comment at smaller or equal level",
        //             ),
        //             (
        //                 comment_view_keymap.prev_leq_level_comment.to_string(),
        //                 "Focus the previous comment at smaller or equal level",
        //             ),
        //             (
        //                 comment_view_keymap.parent_comment.to_string(),
        //                 "Focus the parent comment (if exists)",
        //             ),
        //         ],
        //     ),
        //     (
        //         "Open external links",
        //         vec![
        //             (
        //                 story_view_keymap.open_article_in_browser.to_string(),
        //                 "Open in browser the article associated with the discussed story",
        //             ),
        //             (
        //                 story_view_keymap.open_article_in_article_view.to_string(),
        //                 "Open in article view the article associated with the discussed story",
        //             ),
        //             (
        //                 story_view_keymap.open_story_in_browser.to_string(),
        //                 "Open in browser the discussed story",
        //             ),
        //             (
        //                 comment_view_keymap.open_comment_in_browser.to_string(),
        //                 "Open in browser the focused comment",
        //             ),
        //             (
        //                 format!("{{link_id}} {}", comment_view_keymap.open_link_in_browser),
        //                 "Open in browser the {link_id}-th link in the focused comment",
        //             ),
        //             (
        //                 format!(
        //                     "{{link_id}} {}",
        //                     comment_view_keymap.open_link_in_article_view
        //                 ),
        //                 "Open in article view the {link_id}-th link in the focused comment",
        //             ),
        //         ],
        //     ),
        //     view_navigation_key_shortcuts!(),
        //     other_key_shortcuts!((
        //         comment_view_keymap.toggle_collapse_comment.to_string(),
        //         "Toggle collapsing the focused comment"
        //     )),
        // ])
    }
}

impl HasHelpView for search_view::SearchView {
    fn construct_help_view() -> HelpView {
        HelpView::new()
        // let search_view_keymap = config::get_search_view_keymap();
        // let story_view_keymap = config::get_story_view_keymap();

        // HelpView::new().command_groups(vec![
        //     (
        //         "Switch Mode",
        //         vec![
        //             (
        //                 search_view_keymap.to_navigation_mode.to_string(),
        //                 "Switch to navigation mode",
        //             ),
        //             (
        //                 search_view_keymap.to_search_mode.to_string(),
        //                 "Switch to search mode",
        //             ),
        //         ],
        //     ),
        //     (
        //         "Navigation Mode - Navigation",
        //         vec![
        //             (
        //                 story_view_keymap.next_story.to_string(),
        //                 "Focus the next story",
        //             ),
        //             (
        //                 story_view_keymap.prev_story.to_string(),
        //                 "Focus the previous story",
        //             ),
        //             (
        //                 format!("{{story_id}} {}", story_view_keymap.goto_story),
        //                 "Focus the {story_id}-th story",
        //             ),
        //         ],
        //     ),
        //     (
        //         "Navigation Mode - Paging/Filtering",
        //         vec![
        //             (
        //                 story_view_keymap.next_page.to_string(),
        //                 "Go to the next page",
        //             ),
        //             (
        //                 story_view_keymap.prev_page.to_string(),
        //                 "Go the previous page",
        //             ),
        //             (
        //                 story_view_keymap.toggle_sort_by_date.to_string(),
        //                 "Toggle sort by date",
        //             ),
        //         ],
        //     ),
        //     (
        //         "Navigation Mode - Open external links",
        //         vec![
        //             (
        //                 story_view_keymap.open_article_in_browser.to_string(),
        //                 "Open in browser the link associated with the focused story",
        //             ),
        //             (
        //                 story_view_keymap.open_article_in_article_view.to_string(),
        //                 "Open in article view the link associated with the focused story",
        //             ),
        //             (
        //                 story_view_keymap.open_story_in_browser.to_string(),
        //                 "Open in browser the focused story",
        //             ),
        //         ],
        //     ),
        //     view_navigation_key_shortcuts!((
        //         story_view_keymap.goto_story_comment_view.to_string(),
        //         "Go to the comment view associated with the focused story"
        //     )),
        //     other_key_shortcuts!(),
        // ])
    }
}

impl HasHelpView for article_view::ArticleView {
    fn construct_help_view() -> HelpView {
        HelpView::new()
        // let article_view_keymap = config::get_article_view_keymap().clone();
        // HelpView::new().command_groups(vec![
        //     (
        //         "Open external links",
        //         vec![
        //             (
        //                 article_view_keymap.open_article_in_browser.to_string(),
        //                 "Open article in browser",
        //             ),
        //             (
        //                 format!("{{link_id}} {}", article_view_keymap.open_link_in_browser),
        //                 "Open in browser {link_id}-th link",
        //             ),
        //             (
        //                 format!(
        //                     "{{link_id}} {}",
        //                     article_view_keymap.open_link_in_article_view
        //                 ),
        //                 "Open in article view {link_id}-th link",
        //             ),
        //         ],
        //     ),
        //     (
        //         "Link dialog",
        //         vec![
        //             (
        //                 article_view_keymap.open_link_dialog.to_string(),
        //                 "Open link dialog",
        //             ),
        //             (
        //                 article_view_keymap.link_dialog_focus_next.to_string(),
        //                 "Focus next link",
        //             ),
        //             (
        //                 article_view_keymap.link_dialog_focus_prev.to_string(),
        //                 "Focus previous link",
        //             ),
        //             (
        //                 article_view_keymap.open_link_in_browser.to_string(),
        //                 "Open in browser the focused link",
        //             ),
        //             (
        //                 article_view_keymap.open_link_in_article_view.to_string(),
        //                 "Open in article view the focused link",
        //             ),
        //         ],
        //     ),
        //     view_navigation_key_shortcuts!(),
        // ])
    }
}
