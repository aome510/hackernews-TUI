use crate::prelude::*;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// TextView is a View displaying a text
pub struct TextView {
    content: StyledString,
    rows: Vec<lines::spans::Row>,
    width: usize,
    size_cache: Option<XY<SizeCache>>,
    padding: TextPadding,
}

#[derive(Default)]
pub struct TextPadding {
    pub left: Option<StyledPaddingChar>,
    pub top: Option<StyledPaddingChar>,
}

/// EditableTextView is a View displaying an editable text with cursor
pub struct EditableTextView {
    view: TextView,
    text: String,
    cursor: usize,
}

pub struct StyledPaddingChar(char, Style);

impl TextView {
    pub fn new<S>(content: S) -> Self
    where
        S: Into<StyledString>,
    {
        let content = content.into();
        TextView {
            content,
            rows: Vec::new(),
            width: 0,
            size_cache: None,
            padding: TextPadding::default(),
        }
    }

    pub fn padding(self, padding: TextPadding) -> Self {
        Self {
            padding,
            size_cache: None,
            ..self
        }
    }

    pub fn set_content<S>(&mut self, content: S)
    where
        S: Into<StyledString>,
    {
        self.content = content.into();
        self.size_cache = None;
    }

    fn is_size_cache_valid(&self, size: Vec2) -> bool {
        match self.size_cache {
            None => false,
            Some(ref last) => last.x.accept(size.x) && last.y.accept(size.y),
        }
    }

    fn compute_rows(&mut self, size: Vec2) {
        if self.is_size_cache_valid(size) {
            return;
        }

        self.size_cache = None;
        self.width = size.x;

        // a row's width based on the given width from parent view minus the padding's width
        let row_width = self.width.saturating_sub(self.padding.width());
        self.rows = lines::spans::LinesIterator::new(&self.content, row_width).collect();
    }
}

impl View for TextView {
    fn draw(&self, printer: &Printer) {
        printer.with_selection(printer.focused, |printer| {
            // print the top padding
            if let Some(ref p) = self.padding.top {
                printer.with_style(p.1, |printer| {
                    printer.print_hline((0, 0), printer.size.x, &p.0.to_string());
                });
            }

            self.rows.iter().enumerate().for_each(|(y, row)| {
                let y = y + usize::from(self.padding.top.is_some());
                let mut x: usize = 0;

                // print the left padding
                if let Some(ref p) = self.padding.left {
                    printer.with_style(p.1, |printer| {
                        printer.print((x, y), &p.0.to_string());
                        x += p.0.width().unwrap_or_default();
                    });
                }

                // print the actual text content
                row.resolve(&self.content).iter().for_each(|span| {
                    printer.with_style(*span.attr, |printer| {
                        printer.print((x, y), span.content);
                        x += span.content.width();
                    });
                });
                if x < printer.size.x {
                    printer.print_hline((x, y), printer.size.x - x, " ");
                }
            });
        });
    }

    fn layout(&mut self, size: Vec2) {
        // Compute the text rows.
        self.compute_rows(size);

        // The entire "virtual" size (includes all rows)
        let my_size = Vec2::new(self.width, self.rows.len());

        // Build a fresh cache.
        self.size_cache = Some(SizeCache::build(my_size, size));
    }

    fn needs_relayout(&self) -> bool {
        self.size_cache.is_none()
    }

    fn take_focus(&mut self, _: direction::Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }

    fn required_size(&mut self, size: Vec2) -> Vec2 {
        self.compute_rows(size);
        Vec2::new(self.width, self.rows.len() + self.padding.height())
    }
}

impl EditableTextView {
    pub fn new() -> Self {
        EditableTextView {
            view: TextView::new(" "),
            text: String::new(),
            cursor: 0,
        }
    }

    fn get_content(&self) -> String {
        format!("{} ", self.text)
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn add_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
        self.view.set_content(self.get_content());
    }

    pub fn del_char(&mut self) {
        if !self.text.is_empty() && self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
            self.view.set_content(self.get_content());
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }
    pub fn move_cursor_to_begin(&mut self) {
        self.cursor = 0;
    }
    pub fn move_cursor_to_end(&mut self) {
        self.cursor = self.text.len();
    }
}

impl Default for EditableTextView {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewWrapper for EditableTextView {
    wrap_impl!(self.view: TextView);

    fn wrap_draw(&self, printer: &Printer) {
        printer.with_selection(printer.focused, |printer| {
            let mut pos: usize = 0;
            self.view.rows.iter().enumerate().for_each(|(y, row)| {
                let mut x: usize = 0;
                row.resolve(&self.view.content).iter().for_each(|span| {
                    printer.with_style(*span.attr, |printer| {
                        span.content.chars().for_each(|c| {
                            if pos == self.cursor && printer.focused {
                                printer.with_effect(Effect::Reverse, |printer| {
                                    printer.print((x, y), &c.to_string());
                                });
                            } else {
                                printer.print((x, y), &c.to_string())
                            }
                            x += 1;
                            pos += 1;
                        });
                    });
                });
                if x < printer.size.x {
                    printer.print_hline((x, y), printer.size.x - x, " ");
                }
            });
        });
    }
}

impl TextPadding {
    pub fn left(self, l: StyledPaddingChar) -> Self {
        Self {
            left: Some(l),
            ..self
        }
    }

    pub fn top(self, l: StyledPaddingChar) -> Self {
        Self {
            top: Some(l),
            ..self
        }
    }

    pub fn width(&self) -> usize {
        self.left.as_ref().map(|p| p.width()).unwrap_or_default()
    }

    pub fn height(&self) -> usize {
        usize::from(self.top.is_some())
    }
}

impl StyledPaddingChar {
    pub fn new(c: char, style: Style) -> Self {
        Self(c, style)
    }

    pub fn width(&self) -> usize {
        self.0.width().unwrap_or_default()
    }
}
