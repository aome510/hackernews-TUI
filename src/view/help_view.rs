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
