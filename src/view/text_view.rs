use crate::prelude::*;

/// TextView wraps a text string based on the view port
/// and the text alignment
pub struct TextView {
    content: utils::markup::StyledString,
    rows: Vec<lines::spans::Row>,
    width: usize,
    size_cache: Option<XY<SizeCache>>,
    focusable: bool,
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
            focusable: true,
        }
    }
    pub fn unfocusable(mut self) -> Self {
        self.focusable = false;
        self
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
        printer.with_selection(printer.focused && self.focusable, |printer| {
            self.rows.iter().enumerate().for_each(|(y, row)| {
                let mut x: usize = 0;
                row.resolve(&self.content).iter().for_each(|span| {
                    let l = span.content.chars().count();
                    printer.with_style(*span.attr, |printer| {
                        printer.print((x, y), span.content);
                    });
                    x += l;
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
        return self.focusable;
    }

    fn required_size(&mut self, size: Vec2) -> Vec2 {
        self.compute_rows(size);
        Vec2::new(self.width, self.rows.len())
    }
}
