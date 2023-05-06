use nom::Finish;
use std::ops::Deref;

use crate::parser::parse_array;

/// Dimensionality of `colors.ascii`.
pub const COLORS_DIMENSION: usize = 112;

/// A data point in the Colors test dataset.
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

/// Convenience function to parse `colors.ascii` and wrap points in [`Colors`].
pub fn parse_colors(input: &str) -> Result<Vec<Colors>, crate::Error> {
    let (_, (_, v)) = parse_array(input).finish()?;

    Ok(v.into_iter().map(Colors).collect())
}
