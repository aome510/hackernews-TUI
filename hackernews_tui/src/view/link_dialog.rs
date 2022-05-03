use super::{text_view, utils};
use crate::prelude::*;

type LinkDialogContent = ScrollView<LinearLayout>;

/// A link dialog displaying a list of links
pub struct LinkDialog {
    view: Dialog,
}

impl ViewWrapper for LinkDialog {
    wrap_impl!(self.view: Dialog);
}

impl LinkDialog {
    pub fn new(links: &[String]) -> Self {
        let view = Dialog::around(
            LinearLayout::vertical()
                .with(|v| {
                    links.iter().enumerate().for_each(|(id, link)| {
                        let mut link_styled_string = StyledString::plain(format!("{}. ", id + 1));
                        link_styled_string.append_styled(
                            crate::utils::shorten_url(link),
                            config::get_config_theme().component_style.link,
                        );
                        v.add_child(text_view::TextView::new(link_styled_string));
                    })
                })
                .scrollable(),
        );

        Self { view }
    }

    pub fn content(&self) -> &LinearLayout {
        self.view
            .get_content()
            .downcast_ref::<LinkDialogContent>()
            .expect("the help dialog's content should have `LinkDialogContent` type")
            .get_inner()
    }

    pub fn content_mut(&mut self) -> &mut LinearLayout {
        self.view
            .get_content_mut()
            .downcast_mut::<LinkDialogContent>()
            .expect("the help dialog's content should have `LinkDialogContent` type")
            .get_inner_mut()
    }
}

pub fn get_link_dialog(links: &[String]) -> impl View {
    let view = LinkDialog::new(links);
    let link_dialog_keymap = config::get_link_dialog_keymap().clone();

    OnEventView::new(view)
        .on_pre_event_inner(link_dialog_keymap.next, |s, _| {
            let focus_id = s.content().get_focus_index();
            s.content_mut()
                .set_focus_index(focus_id + 1)
                .unwrap_or(EventResult::Consumed(None));
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(link_dialog_keymap.prev, |s, _| {
            let focus_id = s.content().get_focus_index();
            if focus_id > 0 {
                s.content_mut()
                    .set_focus_index(focus_id - 1)
                    .unwrap_or(EventResult::Consumed(None));
            }
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(link_dialog_keymap.open_link_in_browser, {
            let links = links.to_owned();
            move |s, _| utils::open_ith_link_in_browser(&links, s.content().get_focus_index())
        })
        .on_pre_event_inner(link_dialog_keymap.open_link_in_article_view, {
            let links = links.to_owned();
            move |s, _| utils::open_ith_link_in_article_view(&links, s.content().get_focus_index())
        })
        .on_pre_event(config::get_global_keymap().close_dialog.clone(), |s| {
            s.pop_layer();
        })
        .max_height(32)
        .max_width(64)
}
