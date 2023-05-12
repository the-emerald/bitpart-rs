use nom::Finish;
use std::ops::Deref;

use crate::parser::parse_array;

/// Dimensionality of `nasa.ascii`.
pub const NASA_DIMENSION: usize = 20;

/// A data point in the NASA test dataset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nasa(pub [f64; NASA_DIMENSION]);

impl Deref for Nasa {
    type Target = [f64; NASA_DIMENSION];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Nasa {
    type Item = f64;
    type IntoIter = <[f64; NASA_DIMENSION] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Nasa {
    type Item = &'a f64;
    type IntoIter = <&'a [f64; NASA_DIMENSION] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Convenience function to parse `nasa.ascii` and wrap points in [`Nasa`].
pub fn parse_nasa(input: &str) -> Result<Vec<Nasa>, nom::error::Error<&str>> {
    let (_, (_, v)) = parse_array(input).finish()?;

    Ok(v.into_iter().map(Nasa).collect())
}
