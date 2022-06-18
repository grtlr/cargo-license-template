use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error("parsing failed, {0}")]
    Parse(String),
    #[error("file `{0}` does not match the license template")]
    InvalidLicense(PathBuf),
}
