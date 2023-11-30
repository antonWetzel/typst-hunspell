use std::path::Path;

use crate::output;

pub struct Checker<'a> {
    zspell: &'a zspell::Dictionary,
    text: String,
    cursor: usize,
    path: &'a Path,

    pretty: Option<usize>,
    line: usize,
    column: usize,
}

impl<'a> Checker<'a> {
    pub fn new(
        zspell: &'a zspell::Dictionary,
        text: String,
        path: &'a Path,
        pretty: Option<usize>,
    ) -> Self {
        Self {
            zspell,
            text,
            cursor: 0,
            path,

            pretty,
            line: 1,
            column: 1,
        }
    }

    pub fn skip(&mut self, length: usize) {
        self.advance(length);
    }

    pub fn valid_word(&self, length: usize) -> bool {
        self.zspell
            .check_word(&self.text[self.cursor..(self.cursor + length)])
    }

    pub fn check(&mut self, length: usize) {
        let (line_start, column_start) = (self.line, self.column);
        let text_range = self.cursor..(self.cursor + length);
        self.advance(length);
        let entry = self.zspell.entry(&self.text[text_range.clone()]);
        let suggestions = match entry.suggest() {
            Some(suggestions) => suggestions,
            None => return,
        };
        let (line_end, column_end) = (self.line, self.column);

        if let Some(size) = self.pretty {
            output::output_pretty(
                &self.path.display().to_string(),
                line_start,
                &self.text,
                text_range,
                size,
                suggestions,
            );
        } else {
            output::output_plain(
                &self.path.display().to_string(),
                line_start,
                column_start,
                line_end,
                column_end,
                suggestions,
            );
        }
    }

    fn advance(&mut self, length: usize) {
        let text = &self.text[self.cursor..self.cursor + length];
        for char in text.chars() {
            match char {
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                }
                _ => {
                    self.column += 1;
                }
            }
        }
        self.cursor += length;
    }
}
