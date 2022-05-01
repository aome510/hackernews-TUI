use crate::prelude::*;

use super::{article_view, comment_view, search_view, story_view};

type CommandGroupsView = ScrollView<LinearLayout>;

/// A help item used to describe a command and its keybindings
#[derive(Clone)]
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

    /// constructs a new help view from the current one by appending new key groups
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

pub trait HasHelpView {
    fn construct_help_view() -> HelpView;
}

/// An empty struct representing a default HelpView
pub struct DefaultHelpView {}

impl HasHelpView for DefaultHelpView {
    fn construct_help_view() -> HelpView {
        HelpView::new().command_groups(vec![
            CommandGroup::new("View navigation", default_view_navigation_commands()),
            CommandGroup::new("Other", default_other_commands()),
        ])
    }
}

fn default_other_commands() -> Vec<Command> {
    vec![
        Command::new(
            config::get_global_keymap().open_help_dialog.to_string(),
            "Open the help dialog",
        ),
        Command::new(
            config::get_global_keymap().quit.to_string(),
            "Quit the application",
        ),
        Command::new(
            config::get_global_keymap().close_dialog.to_string(),
            "Close a dialog",
        ),
    ]
}

fn default_view_navigation_commands() -> Vec<Command> {
    vec![
        Command::new(
            config::get_global_keymap().goto_previous_view.to_string(),
            "Go to the previous view",
        ),
        Command::new(
            config::get_global_keymap().goto_search_view.to_string(),
            "Go to search view",
        ),
        Command::new(
            config::get_global_keymap().goto_front_page_view.to_string(),
            "Go to front page view",
        ),
        Command::new(
            config::get_global_keymap()
                .goto_all_stories_view
                .to_string(),
            "Go to all stories view",
        ),
        Command::new(
            config::get_global_keymap().goto_ask_hn_view.to_string(),
            "Go to ask HN view",
        ),
        Command::new(
            config::get_global_keymap().goto_show_hn_view.to_string(),
            "Go to show HN view",
        ),
        Command::new(
            config::get_global_keymap().goto_jobs_view.to_string(),
            "Go to jobs view",
        ),
    ]
}

impl HasHelpView for story_view::StoryView {
    fn construct_help_view() -> HelpView {
        let story_view_keymap = config::get_story_view_keymap();

        let mut help_view = HelpView::new().command_groups(vec![
            CommandGroup::new(
                "Story navigation",
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

        let custom_commands = config::get_config()
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

        if !custom_commands.is_empty() {
            help_view = help_view
                .command_groups(vec![CommandGroup::new("Custom keymaps", custom_commands)]);
        }

        help_view.command_groups(vec![
            CommandGroup::new(
                "View navigation",
                [
                    vec![
                        Command::new(
                            story_view_keymap.goto_story_comment_view.to_string(),
                            "Go to the comment view associated with the focused story",
                        ),
                        Command::new(
                            story_view_keymap.next_story_tag.to_string(),
                            "Go to the next story tag",
                        ),
                        Command::new(
                            story_view_keymap.prev_story_tag.to_string(),
                            "Go to the previous story tag",
                        ),
                    ],
                    default_view_navigation_commands(),
                ]
                .concat(),
            ),
            CommandGroup::new("Others", default_other_commands()),
        ])
    }
}

impl HasHelpView for comment_view::CommentView {
    fn construct_help_view() -> HelpView {
        let comment_view_keymap = config::get_comment_view_keymap();
        let story_view_keymap = config::get_story_view_keymap();

        HelpView::new().command_groups(vec![
            CommandGroup::new(
                "Comment navigation",
                vec![
                    Command::new(
                        comment_view_keymap.next_comment.to_string(),
                        "Focus the next comment",
                    ),
                    Command::new(
                        comment_view_keymap.prev_comment.to_string(),
                        "Focus the previous comment",
                    ),
                    Command::new(
                        comment_view_keymap.next_top_level_comment.to_string(),
                        "Focus the next top level comment",
                    ),
                    Command::new(
                        comment_view_keymap.prev_top_level_comment.to_string(),
                        "Focus the previous top level comment",
                    ),
                    Command::new(
                        comment_view_keymap.next_leq_level_comment.to_string(),
                        "Focus the next comment at smaller or equal level",
                    ),
                    Command::new(
                        comment_view_keymap.prev_leq_level_comment.to_string(),
                        "Focus the previous comment at smaller or equal level",
                    ),
                    Command::new(
                        comment_view_keymap.parent_comment.to_string(),
                        "Focus the parent comment (if exists)",
                    ),
                ],
            ),
            CommandGroup::new(
                "Open external links",
                vec![
                    Command::new(
                        story_view_keymap.open_article_in_browser.to_string(),
                        "Open in browser the article associated with the discussed story",
                    ),
                    Command::new(
                        story_view_keymap.open_article_in_article_view.to_string(),
                        "Open in article view the article associated with the discussed story",
                    ),
                    Command::new(
                        story_view_keymap.open_story_in_browser.to_string(),
                        "Open in browser the discussed story",
                    ),
                    Command::new(
                        comment_view_keymap.open_comment_in_browser.to_string(),
                        "Open in browser the focused comment",
                    ),
                    Command::new(
                        format!("{{link_id}} {}", comment_view_keymap.open_link_in_browser),
                        "Open in browser the {link_id}-th link in the focused comment",
                    ),
                    Command::new(
                        format!(
                            "{{link_id}} {}",
                            comment_view_keymap.open_link_in_article_view
                        ),
                        "Open in article view the {link_id}-th link in the focused comment",
                    ),
                ],
            ),
            CommandGroup::new("View navigation", default_view_navigation_commands()),
            CommandGroup::new(
                "Others",
                [
                    vec![Command::new(
                        comment_view_keymap.toggle_collapse_comment.to_string(),
                        "Toggle collapsing the focused comment",
                    )],
                    default_other_commands(),
                ]
                .concat(),
            ),
        ])
    }
}

impl HasHelpView for search_view::SearchView {
    fn construct_help_view() -> HelpView {
        let search_view_keymap = config::get_search_view_keymap();
        let story_view_keymap = config::get_story_view_keymap();

        HelpView::new().command_groups(vec![
            CommandGroup::new(
                "Switch Mode",
                vec![
                    Command::new(
                        search_view_keymap.to_navigation_mode.to_string(),
                        "Switch to navigation mode",
                    ),
                    Command::new(
                        search_view_keymap.to_search_mode.to_string(),
                        "Switch to search mode",
                    ),
                ],
            ),
            CommandGroup::new(
                "Navigation Mode - Navigation",
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
                "Navigation Mode - Paging/Filtering",
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
                        "Toggle sort by date",
                    ),
                ],
            ),
            CommandGroup::new(
                "Navigation Mode - Open external links",
                vec![
                    Command::new(
                        story_view_keymap.open_article_in_browser.to_string(),
                        "Open in browser the link associated with the focused story",
                    ),
                    Command::new(
                        story_view_keymap.open_article_in_article_view.to_string(),
                        "Open in article view the link associated with the focused story",
                    ),
                    Command::new(
                        story_view_keymap.open_story_in_browser.to_string(),
                        "Open in browser the focused story",
                    ),
                ],
            ),
            CommandGroup::new(
                "View navigation",
                [
                    vec![Command::new(
                        story_view_keymap.goto_story_comment_view.to_string(),
                        "Go to the comment view associated with the focused story",
                    )],
                    default_view_navigation_commands(),
                ]
                .concat(),
            ),
            CommandGroup::new("Others", default_other_commands()),
        ])
    }
}

impl HasHelpView for article_view::ArticleView {
    fn construct_help_view() -> HelpView {
        let article_view_keymap = config::get_article_view_keymap().clone();
        HelpView::new().command_groups(vec![
            CommandGroup::new(
                "Open external links",
                vec![
                    Command::new(
                        article_view_keymap.open_article_in_browser.to_string(),
                        "Open article in browser",
                    ),
                    Command::new(
                        format!("{{link_id}} {}", article_view_keymap.open_link_in_browser),
                        "Open in browser {link_id}-th link",
                    ),
                    Command::new(
                        format!(
                            "{{link_id}} {}",
                            article_view_keymap.open_link_in_article_view
                        ),
                        "Open in article view {link_id}-th link",
                    ),
                ],
            ),
            CommandGroup::new(
                "Link dialog",
                vec![
                    Command::new(
                        article_view_keymap.open_link_dialog.to_string(),
                        "Open link dialog",
                    ),
                    Command::new(
                        article_view_keymap.link_dialog_focus_next.to_string(),
                        "Focus next link",
                    ),
                    Command::new(
                        article_view_keymap.link_dialog_focus_prev.to_string(),
                        "Focus previous link",
                    ),
                    Command::new(
                        article_view_keymap.open_link_in_browser.to_string(),
                        "Open in browser the focused link",
                    ),
                    Command::new(
                        article_view_keymap.open_link_in_article_view.to_string(),
                        "Open in article view the focused link",
                    ),
                ],
            ),
            CommandGroup::new("View navigation", default_view_navigation_commands()),
            CommandGroup::new("Others", default_other_commands()),
        ])
    }
}
