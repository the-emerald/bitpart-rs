use std::ops::Deref;

use crate::cartesian_parser::parse;

/// Dimensionality of `colors.ascii`.
pub const COLORS_DIMENSION: usize = 112;

/// A data point in the NASA test dataset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors(pub [f64; COLORS_DIMENSION]);

impl Deref for Colors {
    type Target = [f64; COLORS_DIMENSION];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Colors {
    type Item = f64;
    type IntoIter = <[f64; COLORS_DIMENSION] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Colors {
    type Item = &'a f64;
    type IntoIter = <&'a [f64; COLORS_DIMENSION] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub fn parse_colors(input: &str) -> Result<Vec<Colors>, crate::Error> {
    let (_, (fc, v)) = parse(input)?;

    // Parser already ensures all vectors have the same dimension as file config, so
    // all we need to do is check against the config
    if fc.dimensions != COLORS_DIMENSION as u64 {
        return Err(crate::Error::IncorrectVectorSize(
            COLORS_DIMENSION as u64,
            fc.dimensions,
        ));
    }

    Ok(v.into_iter()
        .map(|x| Colors(x.try_into().unwrap()))
        .collect())
}
