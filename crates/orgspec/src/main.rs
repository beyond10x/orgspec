//! `orgspec check` and `orgspec type check`.

use clap::{Args, Parser, Subcommand};
use orgspec::check::{DesignReport, TypeReport, check_design, check_type};
use orgspec::typeset::{directory, type_dirs};
use serde::Serialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "orgspec",
    version,
    about = "Checks organization designs and organization types."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check designs against the structural rules and the rules of their types.
    ///
    /// Exits 0 when every design is valid, 1 when a gate counts a violation or a type is
    /// refused, 2 when a document cannot be read.
    Check(Files),
    /// Work with organization types.
    Type {
        #[command(subcommand)]
        command: TypeCommand,
    },
}

#[derive(Subcommand)]
enum TypeCommand {
    /// Check types against the types they extend.
    ///
    /// Exits 0 when every type is accepted, 1 when one is refused, 2 when a document cannot be
    /// read.
    Check(Files),
}

#[derive(Args)]
struct Files {
    /// A directory of type documents. Read as well: each entry of ORGSPEC_TYPE_PATH, ./types,
    /// for a design the directory `types` beside the design's directory, and for a type the
    /// directory of its file.
    #[arg(long = "type-path", value_name = "DIR")]
    type_path: Vec<PathBuf>,
    /// Print one JSON array with one object per file.
    #[arg(long)]
    json: bool,
    #[arg(required = true, value_name = "FILE")]
    files: Vec<PathBuf>,
}

/// What one file came to: its exit code, its lines and its JSON object.
struct Outcome {
    code: u8,
    lines: Vec<String>,
    json: Value,
}

fn outcome<R: Serialize>(
    file: &Path,
    result: Result<R, String>,
    code: fn(&R) -> u8,
    lines: fn(&R) -> Vec<String>,
) -> Outcome {
    match result {
        Ok(report) => Outcome {
            code: code(&report),
            lines: lines(&report),
            json: serde_json::to_value(&report).unwrap_or_default(),
        },
        Err(error) => {
            let name = file.display().to_string();
            eprintln!("orgspec: {name}: {error}");
            Outcome {
                code: 2,
                lines: Vec::new(),
                json: json!({ "file": name, "valid": false, "error": error }),
            }
        }
    }
}

/// Checks every file with the type path of that file and prints the outcomes. `beside` names
/// the directory a file adds to the type path.
fn run(
    files: &Files,
    beside: impl Fn(&Path) -> PathBuf,
    check: impl Fn(&Path, &[PathBuf]) -> Outcome,
) -> ExitCode {
    let env = std::env::var("ORGSPEC_TYPE_PATH").ok();
    let outcomes: Vec<Outcome> = files
        .files
        .iter()
        .map(|file| {
            let beside = beside(file);
            let dirs = type_dirs(
                &files.type_path,
                env.as_deref(),
                Path::new(""),
                Some(&beside),
            );
            check(file, &dirs)
        })
        .collect();
    if files.json {
        let all: Vec<&Value> = outcomes.iter().map(|o| &o.json).collect();
        println!("{}", serde_json::to_string_pretty(&all).unwrap_or_default());
    } else {
        for line in outcomes.iter().flat_map(|o| &o.lines) {
            println!("{line}");
        }
    }
    ExitCode::from(outcomes.iter().map(|o| o.code).max().unwrap_or(0))
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Check(files) => run(
            &files,
            |file| directory(file).join("..").join("types"),
            |file, dirs| {
                let result = check_design(file, dirs);
                outcome(file, result, DesignReport::exit_code, DesignReport::lines)
            },
        ),
        Command::Type {
            command: TypeCommand::Check(files),
        } => run(&files, directory, |file, dirs| {
            let result = check_type(file, dirs);
            outcome(file, result, TypeReport::exit_code, TypeReport::lines)
        }),
    }
}
