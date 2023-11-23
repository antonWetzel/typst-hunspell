mod check;
mod checker;
mod output;

use clap::{Parser, ValueEnum};
use hunspell_rs::Hunspell;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::{
    fs::{self, File},
    io::Read,
    ops::Not,
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
    #[error("{0}")]
    NofifyError(#[from] notify::Error),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    JSONError(#[from] serde_json::Error),
}

#[derive(Parser, Debug)]
struct Args {
    task: Task,

    /// File to check, may be a folder with `watch`.
    path: PathBuf,

    /// Document Language. Defaults to auto-detect, but explicit codes ("de-DE", "en-US", ...) enable more checks.
    #[clap(short, long, default_value = None)]
    language: Option<String>,

    /// Delay in seconds.
    #[clap(short, long, default_value_t = 0.1)]
    delay: f64,

    /// Print results with annotations. Disable for easy regex evaluation.
    #[clap(short, long, default_value_t = true)]
    pretty: bool,

    /// Chars before and after the word on the same line with pretty printing.
    #[clap(long, default_value_t = 80)]
    context_length: usize,

    /// Path to file with additional words.
    #[clap(short, long, default_value = None)]
    words: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("dictionaries/dictionaries/de/index.aff");
    let aff_path = path.display().to_string();
    path.set_extension("dic");
    let dic_path = path.display().to_string();

    let mut hunspell = Hunspell::new(&aff_path, &dic_path);

    if let Some(words) = &args.words {
        let mut file = File::open(words).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        for line in data.lines() {
            hunspell.add(line);
        }
    }

    match args.task {
        Task::Check => check(args, &hunspell)?,
        Task::Watch => watch(args, &hunspell)?,
    }
    Ok(())
}

fn check(args: Args, hunspell: &Hunspell) -> Result<(), Error> {
    handle_file(hunspell, &args, &args.path)?;
    Ok(())
}

fn watch(args: Args, hunspell: &Hunspell) -> Result<(), Error> {
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
        if args.pretty {
            Some(args.context_length)
        } else {
            None
        },
    );

    if args.pretty.not() {
        println!("START");
    }
    check::check(&root, &mut checker);

    if args.pretty.not() {
        println!("END");
    }
    Ok(())
}
