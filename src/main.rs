mod check;
mod checker;
mod output;

use clap::{Parser, ValueEnum};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::{
    fmt::Display,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;

use crate::checker::Checker;

#[derive(ValueEnum, Clone, Debug)]
enum Task {
    Check,
    Watch,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("No dictionary specified")]
    NoDictionarySpecified,
    #[error("{0}")]
    NofifyError(#[from] notify::Error),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    JSONError(#[from] serde_json::Error),
}

#[derive(ValueEnum, Clone, Debug)]
enum Style {
    Pretty,
    Plain,
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pretty => write!(f, "pretty"),
            Self::Plain => write!(f, "plain"),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    task: Task,

    /// File to check, may be a folder with `watch`.
    path: PathBuf,

    /// Document Language. Use language codes from <https://github.com/wooorm/dictionaries/tree/main/dictionaries>.
    #[clap(short, long, default_value = "en")]
    language: String,

    /// Delay in seconds for file watcher.
    #[clap(short, long, default_value_t = 0.1)]
    delay: f64,

    /// Print results with annotations. Disable for easy regex evaluation.
    #[clap(short, long, default_value_t = Style::Pretty)]
    style: Style,

    /// Chars before and after the word on the same line with pretty printing.
    #[clap(long, default_value_t = 80)]
    context_length: usize,

    /// Path to file with additional words.
    #[clap(short, long, default_value = None)]
    words: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    if args.language.is_empty() {
        return Err(Error::NoDictionarySpecified);
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(format!(
        "dictionaries/dictionaries/{}/index.aff",
        &args.language
    ));

    let aff_content = std::fs::read_to_string(&path).unwrap();
    path.set_extension("dic");
    let dic_content = std::fs::read_to_string(&path).unwrap();

    let words = if let Some(words) = &args.words {
        std::fs::read_to_string(&words).unwrap()
    } else {
        String::new()
    };
    let zspell = zspell::builder()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .personal_str(&words)
        .build()
        .unwrap();

    match args.task {
        Task::Check => check(args, zspell)?,
        Task::Watch => watch(args, zspell)?,
    }
    Ok(())
}

fn check(args: Args, zspell: zspell::Dictionary) -> Result<(), Error> {
    handle_file(&zspell, &args, &args.path)?;
    Ok(())
}

fn watch(args: Args, zspell: zspell::Dictionary) -> Result<(), Error> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = new_debouncer(Duration::from_secs_f64(args.delay), None, tx)?;
    watcher
        .watcher()
        .watch(&args.path, RecursiveMode::Recursive)?;

    for events in rx {
        for event in events.unwrap() {
            match event.path.extension() {
                Some(ext) if ext == "typ" => {}
                _ => continue,
            }
            handle_file(&zspell, &args, &event.path)?;
        }
    }

    Ok(())
}

fn handle_file(zspell: &zspell::Dictionary, args: &Args, file: &Path) -> Result<(), Error> {
    let text = fs::read_to_string(&file)?;

    let root = typst_syntax::parse(&text);
    let mut checker = Checker::new(
        zspell,
        text,
        file,
        if let Style::Pretty = args.style {
            Some(args.context_length)
        } else {
            None
        },
    );

    if let Style::Plain = args.style {
        println!("START");
    }
    check::check(&root, &mut checker);

    if let Style::Plain = args.style {
        println!("END");
    }
    Ok(())
}
