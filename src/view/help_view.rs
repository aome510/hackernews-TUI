use super::text_view;
use super::theme::*;
use crate::prelude::*;

/// HelpView displays a dialog with a list of key shortcut/description
pub struct HelpView {
    view: OnEventView<Dialog>,
    // ("key", "description") pair
    keys: Vec<(String, String)>,
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
            .child(
                text_view::TextView::new(key_string)
                    .unfocusable()
                    .fixed_width(max_key_width),
            )
            .child(text_view::TextView::new(desc_string).unfocusable())
    }

    fn construct_help_dialog_event_view(view: Dialog) -> OnEventView<Dialog> {
        OnEventView::new(view).on_event(Key::Esc, |s| {
            s.pop_layer();
        })
    }

    fn construct_keys_view(keys: Vec<(String, String)>) -> impl View {
        let max_key_len = match keys.iter().max_by_key(|key| key.0.len()) {
            None => 0,
            Some(key) => key.0.len(),
        };

        LinearLayout::vertical()
            .with(|s| {
                keys.into_iter().for_each(|key| {
                    s.add_child(HelpView::construct_key_view(key, max_key_len + 1));
                });
            })
            .fixed_width(64)
    }

    pub fn keys(mut self, keys: Vec<(&str, &str)>) -> Self {
        self.keys.append(
            &mut keys
                .into_iter()
                .map(|key| (key.0.to_string(), key.1.to_string()))
                .collect(),
        );
        let keys = self.keys.clone();
        self.view
            .get_inner_mut()
            .set_content(HelpView::construct_keys_view(keys));
        self
    }
}

impl ViewWrapper for HelpView {
    wrap_impl!(self.view: OnEventView<Dialog>);
}
