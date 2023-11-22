use std::path::Path;

use hunspell_rs::{CheckResult, Hunspell};

use crate::output;

pub struct Checker<'a> {
    hunspell: &'a Hunspell,
    text: String,
    cursor: usize,
    path: &'a Path,

    line: usize,
    column: usize,
}

impl<'a> Checker<'a> {
    pub fn new(hunspell: &'a Hunspell, text: String, path: &'a Path) -> Self {
        Self {
            hunspell,
            text,
            cursor: 0,
            path,

            line: 1,
            column: 1,
        }
    }

    pub fn skip(&mut self, length: usize) {
        self.advance(length);
    }

    pub fn check(&mut self, length: usize) {
        let text_range = self.cursor..(self.cursor + length);
        match self.hunspell.check(&self.text[text_range.clone()]) {
            CheckResult::MissingInDictionary => {}
            CheckResult::FoundInDictionary => return self.advance(length),
        }
        let (line_start, column_start) = (self.line, self.column);
        self.advance(length);
        let (line_end, column_end) = (self.line, self.column);
        let suggestions = self.hunspell.suggest(&self.text[text_range.clone()]);
        output::output_plain(
            self.path,
            line_start,
            column_start,
            line_end,
            column_end,
            &self.text[text_range],
            suggestions,
        );
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
