use std::ops::{Deref, Sub};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Metric;

/// Wrapper struct to apply Euclidean distance to an object set.
/// # Example
/// ```
/// # use bitpart::metric::{Euclidean, Metric};
/// #
/// let point1: Euclidean<[f64; 2]> = Euclidean::new([0.0, 0.0]);
/// let point2: Euclidean<[f64; 2]> = Euclidean::new([1.0, 1.0]);
///
/// assert_eq!(point1.distance(&point2), 2.0_f64.sqrt());
/// ```
#[derive(Debug, Clone)]
pub struct Euclidean<T>(T);

impl<T> Euclidean<T> {
    /// Creates a new `Euclidean`.
    pub fn new(t: T) -> Self {
        Self(t)
    }
}

impl<T> Deref for Euclidean<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> IntoIterator for Euclidean<T>
where
    T: IntoIterator,
{
    type Item = <T as IntoIterator>::Item;
    type IntoIter = <T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Euclidean<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Metric for Euclidean<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a f64>,
    T: Clone,
{
    fn distance(&self, rhs: &Euclidean<T>) -> f64 {
        // Euclidean distance is the sqrt of the sum of (point1 - point2)^2 for each dimension.
        self.0
            .into_iter()
            .zip(rhs.0.into_iter())
            .map(|(x, y)| (x.sub(*y)).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for Euclidean<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Euclidean<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Euclidean::new(T::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclidean_2d() {
        let point1: Euclidean<[f64; 2]> = Euclidean::new([0.0, 0.0]);
        let point2: Euclidean<[f64; 2]> = Euclidean::new([1.0, 1.0]);

        assert_eq!(point1.distance(&point2), 2.0_f64.sqrt());
    }
}
