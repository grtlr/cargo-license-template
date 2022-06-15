use clap::Parser;
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

mod license;

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
}

#[derive(Debug, Error)]
enum Error {
    #[error("Invalid license in file `{0}`")]
    InvalidLicense(PathBuf),
}

fn is_rust_code(entry: &DirEntry) -> bool {
    entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs")
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
                if !e.path().starts_with(&metadata.target_directory) {
                    Some(e)
                } else {
                    None
                }
            })
        })
    {
        if is_rust_code(&entry) {
            let file_contents = fs::read_to_string(entry.path())?;
            if !license_template.is_match(&file_contents) {
                return Err(Box::new(Error::InvalidLicense(entry.path().to_owned())));
            }
        }
    }

    Ok(())
}
