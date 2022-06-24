mod error;
mod license;

use anyhow::Context;
use anyhow::{anyhow, Result};
use clap::Parser;
use colored::Colorize;
use ignore::WalkBuilder;
use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) use self::error::Error;

enum Report {
    Conform(PathBuf),
    Conflict(PathBuf),
}

fn check_file<P: AsRef<Path>>(path: P, template: &Regex) -> Result<Report> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file `{}`", path.as_ref().display()))?;

    Ok(match template.is_match(&content) {
        true => Report::Conform(path.as_ref().to_path_buf()),
        false => Report::Conflict(path.as_ref().to_path_buf()),
    })
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(
    bin_name = "cargo license-template",
    about = "A cargo subcommand to check each Rust file against a license template."
)]
pub struct CliArgs {
    /// The location of the license template.
    #[clap(long, value_parser)]
    pub template: String,
    /// Path to Cargo.toml.
    #[clap(long, value_parser)]
    manifest_path: Option<PathBuf>,
    /// Use verbose output
    #[clap(short, long, value_parser)]
    verbose: bool,
    /// Use colored output (if supported)
    #[clap(short, long, default_value_t = true, value_parser)]
    color: bool,
    /// The location of a file that contains ignored files and directories.
    #[clap(short, long, value_parser)]
    ignore: Option<String>,
}

fn is_rust_code(entry: impl AsRef<Path>) -> bool {
    entry.as_ref().is_file()
        && entry.as_ref().extension().and_then(|ext| ext.to_str()) == Some("rs")
}

fn main() -> Result<()> {
    // When called by `cargo`, we need to drop the extra `license-template` argument.
    let args = env::args().enumerate().filter_map(|(i, x)| {
        if (i, x.as_str()) == (1, "license-template") {
            None
        } else {
            Some(x)
        }
    });

    let cli_args = CliArgs::parse_from(args);

    let license_template = license::load_and_compile_template(&cli_args.template)?;

    let mut cmd = cargo_metadata::MetadataCommand::new();

    if let Some(path) = &cli_args.manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd.exec()?;

    let walker = if let Some(file_name) = cli_args.ignore {
        WalkBuilder::new(&metadata.workspace_root)
            .add_custom_ignore_filename(file_name)
            .to_owned()
    } else {
        WalkBuilder::new(&metadata.workspace_root)
    };

    let reports = walker.build().into_iter().filter_map(|e| {
        e.ok().and_then(|e| {
            if !e.path().starts_with(&metadata.target_directory) && is_rust_code(e.path()) {
                Some(check_file(e.path(), &license_template))
            } else {
                None
            }
        })
    });

    let mut no_errors = true;
    for report in reports {
        match report? {
            Report::Conform(path) => {
                if cli_args.verbose {
                    println!(
                        "{} ... {}",
                        path.strip_prefix(&metadata.workspace_root)?.display(),
                        "ok".green()
                    );
                }
            }
            Report::Conflict(path) => {
                println!(
                    "{} ... {}",
                    path.strip_prefix(&metadata.workspace_root)?.display(),
                    "failed".red()
                );
                no_errors = false;
            }
        }
    }

    match no_errors {
        true => Ok(()),
        false => Err(anyhow!("At least one non-conformed file has been found.")),
    }
}
