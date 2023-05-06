//! Parser for the `.ascii` data format from SISAP.

#![deny(missing_docs)]

/// Colors test dataset
pub mod colors;

/// NASA test dataset
pub mod nasa;

/// Parser for `.ascii` files.
pub mod parser;

/// Errors that can be encountered while parsing a dataset.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Parser failure
    #[error("parser failure")]
    ParserError,
}

impl From<nom::error::Error<&str>> for Error {
    fn from(_: nom::error::Error<&str>) -> Self {
        Self::ParserError
    }
}
