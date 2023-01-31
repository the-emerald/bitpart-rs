pub mod cartesian_parser;
pub mod colors;
pub mod nasa;

/// Errors that can be encountered while parsing a dataset.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parser failure")]
    ParserError,
    #[error("vectors does not have dimension {0}, was {0}")]
    IncorrectVectorSize(u64, u64),
}

impl From<nom::Err<nom::error::Error<&str>>> for Error {
    fn from(_: nom::Err<nom::error::Error<&str>>) -> Self {
        Self::ParserError
    }
}
