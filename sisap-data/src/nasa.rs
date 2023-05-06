use std::ops::Deref;

use crate::parser::parse;

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

pub fn parse_nasa(input: &str) -> Result<Vec<Nasa>, crate::Error> {
    let (_, (fc, v)) = parse(input)?;

    // Parser already ensures all vectors have the same dimension as file config, so
    // all we need to do is check against the config
    if fc.dimensions != NASA_DIMENSION as u64 {
        return Err(crate::Error::IncorrectVectorSize(
            NASA_DIMENSION as u64,
            fc.dimensions,
        ));
    }

    Ok(v.into_iter().map(|x| Nasa(x.try_into().unwrap())).collect())
}
