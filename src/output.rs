use std::ops::Range;

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

pub fn output_plain(
    file_name: &str,
    line_start: usize,
    column_start: usize,
    line_end: usize,
    column_end: usize,
    suggestions: Vec<&str>,
) {
    print!(
        "{} {}:{}-{}:{} info ",
        file_name, line_start, column_start, line_end, column_end,
    );
    for (index, suggestion) in suggestions.iter().enumerate() {
        if index < suggestions.len() - 1 {
            print!("{}, ", suggestion);
        } else {
            print!("{}", suggestion);
        }
    }
    println!()
}

pub fn output_pretty(
    file_name: &str,
    line_start: usize,
    text: &str,
    range: Range<usize>,
    context_length: usize,
    suggestions: Vec<&str>,
) {
    let mut start = 0;

    for (index, (x, char)) in text[0..range.start].char_indices().rev().enumerate() {
        if matches!(char, '\n' | '\t' | '\r') || index >= context_length {
            start = x + char.len_utf8();
            break;
        }
    }

    let mut end = text.len();
    for (index, (x, char)) in text[range.end..end].char_indices().enumerate() {
        if matches!(char, '\n' | '\t' | '\r') || index >= context_length {
            end = range.end + x;
            break;
        }
    }

    let context = &text[start..end];
    let (mut char_start, mut char_end) = (0, 0);
    for (index, (c_index, char)) in context.char_indices().enumerate() {
        char_end = index + 1;
        if c_index + start == range.start {
            char_start = index;
        } else if c_index + start + char.len_utf8() == range.end {
            break;
        }
    }

    let mut annotations = Vec::new();
    annotations.push(SourceAnnotation {
        label: "",
        annotation_type: AnnotationType::Info,
        range: (char_start, char_end),
    });

    for replacement in &suggestions {
        annotations.push(SourceAnnotation {
            label: replacement,
            annotation_type: AnnotationType::Help,
            range: (char_start, char_end),
        })
    }

    let snippet = Snippet {
        title: Some(Annotation {
            label: Some("Unknown word"),
            annotation_type: AnnotationType::Info,
            id: None,
        }),
        footer: Vec::new(),
        slices: vec![Slice {
            source: &context,
            line_start: line_start,
            origin: Some(file_name),
            fold: true,
            annotations,
        }],
        opt: FormatOptions {
            color: true,
            anonymized_line_numbers: false,
            margin: None,
        },
    };
    println!("{}", DisplayList::from(snippet));
}
