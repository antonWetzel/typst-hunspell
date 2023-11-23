mod check;
mod checker;
mod output;

use clap::{Parser, ValueEnum};
use hunspell_rs::Hunspell;
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
    /// Multiple Languages can be specified.
    #[clap(short, long, default_value = "en", num_args = 1..)]
    language: Vec<String>,

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
        &args.language[0],
    ));

    let aff_path = path.display().to_string();
    path.set_extension("dic");
    let dic_path = path.display().to_string();

    let mut hunspell = Hunspell::new(&aff_path, &dic_path);

    for language in args.language.iter().skip(1) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(format!("dictionaries/dictionaries/{}/index.dic", language));
        hunspell.add_dictionary(&path.display().to_string());
    }

    if let Some(words) = &args.words {
        let mut file = File::open(words).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        for line in data.lines() {
            hunspell.add(line);
        }
    }

    match args.task {
        Task::Check => check(args, hunspell)?,
        Task::Watch => watch(args, hunspell)?,
    }
    Ok(())
}

fn check(args: Args, hunspell: Hunspell) -> Result<(), Error> {
    handle_file(&hunspell, &args, &args.path)?;
    Ok(())
}

fn watch(args: Args, mut hunspell: Hunspell) -> Result<(), Error> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = new_debouncer(Duration::from_secs_f64(args.delay), None, tx)?;
    watcher
        .watcher()
        .watch(&args.path, RecursiveMode::Recursive)?;

    for events in rx {
        for event in events.unwrap() {
            if let Some(words) = &args.words {
                if event.path == Path::new(words) {
                    let mut file = File::open(words).unwrap();
                    let mut data = String::new();
                    file.read_to_string(&mut data).unwrap();
                    for line in data.lines() {
                        hunspell.add(line);
                    }
                    continue;
                }
            }

            match event.path.extension() {
                Some(ext) if ext == "typ" => {}
                _ => continue,
            }
            handle_file(&hunspell, &args, &event.path)?;
        }
    }

    Ok(())
}

fn handle_file(hunspell: &Hunspell, args: &Args, file: &Path) -> Result<(), Error> {
    let text = fs::read_to_string(&file)?;

    let root = typst_syntax::parse(&text);
    let mut checker = Checker::new(
        hunspell,
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
