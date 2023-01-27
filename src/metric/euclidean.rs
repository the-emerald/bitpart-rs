use std::ops::{Deref, Sub};

use super::Metric;

/// Wrapper struct to apply Euclidean distance to an object set.
/// # Example
/// ```
/// # use crate::bitpart::metric::{Metric, euclidean::Euclidean};
///
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

// pub fn euclidean_nasa(a: Euclidean<[f64; 20], f64>, b: Euclidean<[f64; 20], f64>) -> f64 {
//     // assert_eq!(a.len(), b.len());
//     a.distance(&b)
// }

impl<T> Metric for Euclidean<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a f64>,
    T: Clone,
{
    fn distance(&self, rhs: &Euclidean<T>) -> f64 {
        self.0
            .into_iter()
            .zip(rhs.0.into_iter())
            .map(|(x, y)| (x.sub(*y)).powi(2))
            .sum::<f64>()
            .sqrt()
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
