use crate::prelude::*;

/// HelpView is a View displaying a help dialog with a list of key shortcuts and descriptions
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
        let key_string = StyledString::styled(
            key.0,
            ColorStyle::new(
                PaletteColor::TitlePrimary,
                get_config_theme().code_block_bg.color,
            ),
        );
        let desc_string = StyledString::plain(key.1);
        LinearLayout::horizontal()
            .child(TextView::new(key_string).fixed_width(max_key_width))
            .child(TextView::new(desc_string))
    }

    fn construct_help_dialog_event_view(view: Dialog) -> OnEventView<Dialog> {
        OnEventView::new(view)
            .on_event(Key::Esc, |s| {
                s.pop_layer();
            })
            .on_event(Event::CtrlChar('q'), |s| s.quit())
            .on_event(Event::AltChar('q'), |s| s.quit())
            .on_event(EventTrigger::from_fn(|_| true), |_| {})
    }

    fn construct_keys_view(&self) -> impl View {
        LinearLayout::vertical()
            .with(|s| {
                self.keys.iter().for_each(|(desc, keys)| {
                    s.add_child(TextView::new(StyledString::styled(
                        desc.to_string(),
                        ColorStyle::from(PaletteColor::TitlePrimary),
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
            .scrollable()
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

#[macro_export]
macro_rules! other_key_shortcuts {
    ($(($k:expr,$d:expr)),*) => {
        (
            "Others",
            vec![
                $(
                    ($k, $d),
                )*
                ("?/<ctrl-h>/<alt-h>", "Open the help dialog"),
                ("<ctrl-q>/<alt-q>", "Quit the application"),
                ("<esc>", "Close this help dialog"),
            ],
        )
    };
}

#[macro_export]
macro_rules! view_navigation_key_shortcuts {
    ($(($k:expr,$d:expr)),*) => {
        (
            "View Navigation",
            vec![
                $(
                    ($k, $d),
                )*
                    ("<ctrl-p>/<alt-p>", "Go to the previous view"),
                    ("<ctrl-f>/<alt-f>", "Go to front page view"),
                    ("<ctrl-s>/<alt-s>", "Go to search view"),
                    ("<ctrl-z>/<alt-z>", "Go to all stories view"),
                    ("<ctrl-x>/<alt-x>", "Go to ask HN view"),
                    ("<ctrl-c>/<alt-c>", "Go to show HN view"),
                    ("<ctrl-v>/<alt-v>", "Go to jobs view"),
            ],

        )
    };
}

pub trait HasHelpView {
    fn construct_help_view() -> HelpView {
        HelpView::new().keys(vec![
            view_navigation_key_shortcuts!(),
            other_key_shortcuts!(),
        ])
    }
}

/// An empty struct used to construct the default HelpView
pub struct DefaultHelpView {}

impl HasHelpView for DefaultHelpView {}

impl HasHelpView for StoryView {
    fn construct_help_view() -> HelpView {
        HelpView::new().keys(vec![
            (
                "Navigation",
                vec![
                    ("j", "Focus the next story"),
                    ("k", "Focus the previous story"),
                    ("t", "Focus the story at the top"),
                    ("b", "Focus the story at the bottom"),
                    ("`{story_id} g`", "Focus the {story_id}-th story"),
                ],
            ),
            (
                "Paging/Filtering",
                vec![
                    ("n", "Go to the next page"),
                    ("p", "Go the previous page"),
                    ("d", "Toggle sort by date/popularity"),
                    ("q", "Filter stories past 24 hours"),
                    ("w", "Filter stories past week"),
                    ("e", "Filter stories past month"),
                    ("r", "Filter stories past year"),
                ],
            ),
            (
                "Open external links",
                vec![
                    (
                        "O",
                        "Open in browser the link associated with the focused story",
                    ),
                    ("S", "Open in browser the focused story"),
                ],
            ),
            view_navigation_key_shortcuts!((
                "<enter>",
                "Go to the comment view associated with the focused story"
            )),
            other_key_shortcuts!(),
        ])
    }
}

impl HasHelpView for CommentView {
    fn construct_help_view() -> HelpView {
        HelpView::new().keys(vec![
            (
                "Navigation",
                vec![
                    ("j", "Focus the next comment"),
                    ("k", "Focus the previous comment"),
                    ("n", "Focus the next top level comment"),
                    ("p", "Focus the previous top level comment"),
                    ("l", "Focus the next comment at smaller or equal level"),
                    ("h", "Focus the previous comment at smaller or equal level"),
                    ("t", "Focus the comment at the top"),
                    ("b", "Focus the comment at the bottom"),
                ],
            ),
            (
                "Open external links",
                vec![
                    (
                        "O",
                        "Open in browser the link associated with the discussed story",
                    ),
                    ("S", "Open in browser the discussed story"),
                    ("C", "Open in browser the focused comment"),
                    (
                        "`{link_id} f`",
                        "Open in browser the {link_id}-th link in the focused comment",
                    ),
                ],
            ),
            view_navigation_key_shortcuts!(),
            other_key_shortcuts!(("r", "Reload the comment view")),
        ])
    }
}

impl HasHelpView for SearchView {
    fn construct_help_view() -> HelpView {
        HelpView::new().keys(vec![
            (
                "Switch Mode",
                vec![
                    ("<esc>", "Switch to navigation mode"),
                    ("i", "Switch to search mode"),
                ],
            ),
            (
                "Navigation Mode - Navigation",
                vec![
                    ("j", "Focus the next story"),
                    ("k", "Focus the previous story"),
                    ("t", "Focus the story at the top"),
                    ("b", "Focus the story at the bottom"),
                    ("`{story_id} g`", "Focus the {story_id}-th story"),
                ],
            ),
            (
                "Navigation Mode - Paging/Filtering",
                vec![
                    ("n", "Go to the next page"),
                    ("p", "Go the previous page"),
                    ("d", "Toggle sort by date/popularity"),
                ],
            ),
            (
                "Navigation Mode - Open external links",
                vec![
                    (
                        "O",
                        "Open in browser the link associated with the focused story",
                    ),
                    ("S", "Open in browser the focused story"),
                ],
            ),
            view_navigation_key_shortcuts!(),
            other_key_shortcuts!(),
        ])
    }
}
