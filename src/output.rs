use std::{io::stdout, io::Write, path::Path, str::Chars};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

pub fn output_plain(
    file: &Path,
    line_start: usize,
    column_start: usize,
    line_end: usize,
    column_end: usize,
    _text: &str,
    suggestions: Vec<String>,
) {
    print!(
        "{} {}:{}-{}:{} info ",
        file.display(),
        line_start,
        column_start,
        line_end,
        column_end,
    );
    for suggestion in suggestions {
        print!("{}, ", suggestion);
    }
    println!()
}

// const PRETTY_RANGE: usize = 20;

// pub fn output_pretty(
//     file: &Path,
//     start: usize,
//     length: usize,
//     text: &str,
//     _suggestions: Vec<String>,
// ) {
//     let mut last = 0;
//     let file_name = format!("{}", file.display());
//     for info in &response.matches {
//         if info.offset > PRETTY_RANGE {
//             start.advance(info.offset - PRETTY_RANGE - last);
//             last = info.offset - PRETTY_RANGE;
//         }
//         print_pretty(&file_name, start, info);
//     }
//     start.advance(total - last);
// }

// fn print_pretty(file_name: &str, start: &Position, info: &Match) {
//     let start_buffer = info.offset.min(PRETTY_RANGE);

//     let context = start
//         .clone()
//         .content
//         .take(start_buffer + info.length + PRETTY_RANGE)
//         .collect::<String>();

//     let mut annotations = Vec::new();
//     annotations.push(SourceAnnotation {
//         label: &info.message,
//         annotation_type: AnnotationType::Info,
//         range: (start_buffer, start_buffer + info.length),
//     });
//     for replacement in &info.replacements {
//         let pos = start_buffer + info.length + 2;
//         annotations.push(SourceAnnotation {
//             label: &replacement.value,
//             annotation_type: AnnotationType::Help,
//             range: (pos, pos),
//         })
//     }

//     if let Some(urls) = &info.rule.urls {
//         for url in urls {
//             annotations.push(SourceAnnotation {
//                 label: &url.value,
//                 annotation_type: AnnotationType::Note,
//                 range: (2, 2),
//             })
//         }
//     }

//     let snippet = Snippet {
//         title: Some(Annotation {
//             label: Some(&info.rule.description),
//             annotation_type: AnnotationType::Info,
//             id: Some(&info.rule.id),
//         }),
//         footer: Vec::new(),
//         slices: vec![Slice {
//             source: &context,
//             line_start: start.line,
//             origin: Some(file_name),
//             fold: true,
//             annotations,
//         }],
//         opt: FormatOptions {
//             color: true,
//             anonymized_line_numbers: false,
//             margin: None,
//         },
//     };
//     println!("{}", DisplayList::from(snippet));
// }
