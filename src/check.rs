use std::ops::Not;

use typst_syntax::{SyntaxKind, SyntaxNode};

use crate::checker::Checker;

pub fn check(node: &SyntaxNode, checker: &mut Checker) {
    let state = State {
        mode: Mode::Markdown,
    };
    state.convert(node, checker);
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Markdown,
    Code,
}

#[derive(Clone, Copy)]
struct State {
    mode: Mode,
}

impl State {
    fn convert(mut self, node: &SyntaxNode, checker: &mut Checker) {
        match node.kind() {
            SyntaxKind::Text if self.mode == Mode::Markdown => {
                let text = node.text();

                #[derive(Clone, Copy)]
                enum Last {
                    Start,
                    Whitespace,
                    Text,
                }

                let mut start = 0;
                let mut last = Last::Start;

                for (index, char) in text.char_indices() {
                    match (
                        last,
                        char.is_ascii_whitespace() || char.is_ascii_punctuation(),
                    ) {
                        (Last::Start, true) => last = Last::Whitespace,
                        (Last::Start, false) => last = Last::Text,
                        (Last::Text, true)
                            if char != '.' || checker.valid_word(index - start + 1).not() =>
                        {
                            checker.check(index - start);
                            start = index;
                            last = Last::Whitespace;
                        }
                        (Last::Whitespace, false) => {
                            checker.skip(index - start);
                            start = index;
                            last = Last::Text;
                        }
                        _ => {}
                    }
                }
                match last {
                    Last::Start => {}
                    Last::Text => checker.check(text.len() - start),
                    Last::Whitespace => checker.skip(text.len() - start),
                }
            }
            SyntaxKind::Equation => {
                checker.skip(node.text().len());
                self.skip(node, checker);
            }
            SyntaxKind::FuncCall => {
                self.mode = Mode::Code;
                for child in node.children() {
                    self.convert(child, checker);
                }
            }
            SyntaxKind::Code
            | SyntaxKind::ModuleImport
            | SyntaxKind::ModuleInclude
            | SyntaxKind::LetBinding
            | SyntaxKind::ShowRule
            | SyntaxKind::SetRule => {
                self.mode = Mode::Code;
                for child in node.children() {
                    self.convert(child, checker);
                }
            }
            SyntaxKind::Heading => {
                // output.add_encoded(String::new(), String::from("\n\n"));
                for child in node.children() {
                    self.convert(child, checker);
                }
                // output.add_encoded(String::new(), String::from("\n\n"));
            }
            SyntaxKind::Ref => {
                // output.add_encoded(String::new(), String::from("X"));
                self.skip(node, checker);
            }
            SyntaxKind::LeftBracket | SyntaxKind::RightBracket => {
                checker.skip(node.text().len());
                // output.add_encoded(node.text().into(), String::from("\n\n"));
            }
            SyntaxKind::Markup => {
                self.mode = Mode::Markdown;
                for child in node.children() {
                    self.convert(child, checker);
                }
            }
            SyntaxKind::Shorthand if node.text() == "~" => {
                // output.add_encoded(node.text().into(), String::from(" "));
                checker.skip(node.text().len());
            }
            SyntaxKind::Space if self.mode == Mode::Markdown => checker.skip(node.text().len()),
            SyntaxKind::Parbreak => checker.skip(node.text().len()),
            SyntaxKind::SmartQuote if self.mode == Mode::Markdown => {
                checker.check(node.text().len());
            }
            _ => {
                checker.skip(node.text().len());
                for child in node.children() {
                    self.convert(child, checker);
                }
            }
        }
    }

    fn skip(self, node: &SyntaxNode, checker: &mut Checker) {
        checker.skip(node.text().len());
        for child in node.children() {
            self.skip(child, checker);
        }
    }
}
