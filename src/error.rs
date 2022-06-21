use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error("parsing failed, {0}")]
    Parse(String),
}
