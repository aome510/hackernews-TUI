use super::comment_view::CommentView;
use super::search_view::SearchView;
use super::story_view::StoryView;
use super::theme::*;
use crate::prelude::*;

/// HelpView displays a dialog with a list of key shortcut/description
pub struct HelpView {
    view: OnEventView<Dialog>,
    // "section description" followed by a vector of ("key", "key description") pairs
    keys: Vec<(&'static str, Vec<(&'static str, &'static str)>)>,
}

impl HelpView {
    pub fn new() -> Self {
        HelpView {
            view: HelpView::construct_help_dialog_event_view(Dialog::new().title("Help Dialog")),
            keys: vec![],
        }
    }

    fn construct_key_view(key: (String, String), max_key_width: usize) -> impl View {
        let key_string =
            StyledString::styled(key.0, ColorStyle::new(PaletteColor::Primary, CODE_COLOR));
        let desc_string = StyledString::plain(key.1);
        LinearLayout::horizontal()
            .child(TextView::new(key_string).fixed_width(max_key_width))
            .child(TextView::new(desc_string))
    }

    fn construct_help_dialog_event_view(view: Dialog) -> OnEventView<Dialog> {
        OnEventView::new(view).on_event(Key::Esc, |s| {
            s.pop_layer();
        })
    }

    fn construct_keys_view(&self) -> impl View {
        LinearLayout::vertical().with(|s| {
            self.keys.iter().for_each(|(desc, keys)| {
                s.add_child(TextView::new(StyledString::styled(
                    desc.to_string(),
                    ColorStyle::from(BaseColor::Black),
                )));
                s.add_child({
                    let max_key_len = match keys.iter().max_by_key(|key| key.0.len()) {
                        None => 0,
                        Some(key) => key.0.len(),
                    };

                    PaddedView::lrtb(
                        0,
                        0,
                        0,
                        1,
                        LinearLayout::vertical()
                            .with(|s| {
                                keys.iter().for_each(|key| {
                                    s.add_child(HelpView::construct_key_view(
                                        (key.0.to_string(), key.1.to_string()),
                                        max_key_len + 1,
                                    ));
                                });
                            })
                            .fixed_width(64),
                    )
                });
            });
        })
    }

    pub fn keys(
        mut self,
        mut keys: Vec<(&'static str, Vec<(&'static str, &'static str)>)>,
    ) -> Self {
        self.keys.append(&mut keys);
        let key_view = self.construct_keys_view();
        self.view.get_inner_mut().set_content(key_view);
        self
    }
}

impl ViewWrapper for HelpView {
    wrap_impl!(self.view: OnEventView<Dialog>);
}

impl StoryView {
    pub fn construct_help_view() -> impl View {
        HelpView::new().keys(vec![
            (
                "Navigation",
                vec![
                    ("j", "Focus the next story"),
                    ("k", "Focus the previous story"),
                    ("t", "Focus the story at the top"),
                    ("b", "Focus the story at the bottom"),
                    ("`{story_id} g`", "Focus the {story_id}-th story"),
                    (
                        "<enter>",
                        "Go the comment view associated with the focused story",
                    ),
                ],
            ),
            (
                "Open external links",
                vec![(
                    "O",
                    "Open in browser the link associated with the focused story",
                )],
            ),
            (
                "Others",
                vec![
                    ("<alt-s>", "Go to the story search page"),
                    ("<alt-q>", "Quit the application"),
                    ("<esc>", "Close this help dialog"),
                ],
            ),
        ])
    }
}

impl CommentView {
    pub fn construct_help_view() -> impl View {
        HelpView::new().keys(vec![
            (
                "Navigation",
                vec![
                    ("j", "Focus the next comment"),
                    ("k", "Focus the previous comment"),
                    ("t", "Focus the comment at the top"),
                    ("b", "Focus the comment at the bottom"),
                    ("l", "Focus the next comment with smaller or equal level"),
                    (
                        "h",
                        "Focus the previous comment with smaller or equal level",
                    ),
                ],
            ),
            (
                "Open external links",
                vec![
                    (
                        "O",
                        "Open in browser the link associated with the discussed story",
                    ),
                    (
                        "{link_id} f",
                        "Open in browser the {link_id}-th link in the focused comment",
                    ),
                ],
            ),
            (
                "Others",
                vec![
                    ("<alt-f>", "Go to the front page"),
                    ("<alt-s>", "Go to the story search page"),
                    ("<alt-q>", "Quit the application"),
                    ("<esc>", "Close this help dialog"),
                ],
            ),
        ])
    }
}

impl SearchView {
    pub fn construct_help_view() -> impl View {
        HelpView::new().keys(vec![
            (
                "Search Mode - Keys",
                vec![("<esc>", "Switch to navigation mode")],
            ),
            (
                "Navigation Mode - Keys",
                vec![
                    ("i", "Switch to search mode"),
                    ("j", "Focus the next story"),
                    ("k", "Focus the previous story"),
                    ("t", "Focus the story at the top"),
                    ("b", "Focus the story at the bottom"),
                    ("`{story_id} g`", "Focus the {story_id}-th story"),
                    (
                        "<enter>",
                        "Go the comment view associated with the focused story",
                    ),
                    (
                        "O",
                        "Open in browser the link associated with the focused story",
                    ),
                ],
            ),
            (
                "Others",
                vec![
                    ("<alt-f>", "Go to the front page"),
                    ("<alt-q>", "Quit the application"),
                    ("<esc>", "Close this help dialog"),
                ],
            ),
        ])
    }
}
