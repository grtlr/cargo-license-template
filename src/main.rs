mod error;
mod license;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(crate) use self::error::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(
    bin_name = "cargo license-template",
    about = "A cargo subcommand to check each Rust file against a license template."
)]
pub struct CliArgs {
    /// The location of the license template.
    #[clap(value_parser, long)]
    pub template: String,
    /// Path to Cargo.toml.
    #[clap(value_parser, long)]
    manifest_path: Option<PathBuf>,
    /// Use verbose output
    #[clap(value_parser, short, long)]
    verbose: bool,
}

fn is_rust_code(entry: impl AsRef<Path>) -> bool {
    entry.as_ref().is_file()
        && entry.as_ref().extension().and_then(|ext| ext.to_str()) == Some("rs")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    for entry in WalkDir::new(metadata.workspace_root)
        .into_iter()
        .filter_map(|e| {
            e.ok().and_then(|e| {
                if !e.path().starts_with(&metadata.target_directory) && is_rust_code(e.path()) {
                    Some(e)
                } else {
                    None
                }
            })
        })
    {
        let path = entry.path();
        if cli_args.verbose {
            print!("Checking file `{}`", path.display());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file `{}`", path.display()))?;
        if !license_template.is_match(&content) {
            return Err(Box::new(Error::InvalidLicense(entry.path().to_owned())));
        }
        if cli_args.verbose {
            println!(" ... ok");
        }
    }

    Ok(())
}
