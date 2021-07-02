use crate::prelude::*;

/// TextView is a View displaying a text
pub struct TextView {
    content: utils::markup::StyledString,
    rows: Vec<lines::spans::Row>,
    width: usize,
    size_cache: Option<XY<SizeCache>>,
}

/// EditableTextView is a View displaying an editable text with cursor
pub struct EditableTextView {
    view: TextView,
    text: String,
    cursor: usize,
}

impl TextView {
    pub fn new<S>(content: S) -> Self
    where
        S: Into<utils::markup::StyledString>,
    {
        let content = content.into();
        TextView {
            content,
            rows: Vec::new(),
            width: 0,
            size_cache: None,
        }
    }

    pub fn set_content<S>(&mut self, content: S)
    where
        S: Into<utils::markup::StyledString>,
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
        self.rows = lines::spans::LinesIterator::new(&self.content, size.x).collect();
    }
}

impl View for TextView {
    fn draw(&self, printer: &Printer) {
        printer.with_selection(printer.focused, |printer| {
            self.rows.iter().enumerate().for_each(|(y, row)| {
                let mut x: usize = 0;
                row.resolve(&self.content).iter().for_each(|span| {
                    printer.with_style(*span.attr, |printer| {
                        let l = span.content.chars().count();
                        printer.print((x, y), span.content);
                        x += l;
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

    fn take_focus(&mut self, _: direction::Direction) -> bool {
        true
    }

    fn required_size(&mut self, size: Vec2) -> Vec2 {
        self.compute_rows(size);
        Vec2::new(self.width, self.rows.len())
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

    pub fn get_content(&self) -> String {
        self.text.clone()
    }

    pub fn add_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
        self.view.set_content(format!("{} ", self.text));
    }

    pub fn del_char(&mut self) {
        if !self.text.is_empty() && self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
            self.view.set_content(format!("{} ", self.text));
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
                            if pos == self.cursor {
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
