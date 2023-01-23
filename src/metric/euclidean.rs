use num::traits::real::Real;
use std::{marker::PhantomData, ops::Deref};

use super::Metric;

/// Wrapper struct to apply Euclidean distance to an object set.
/// # Example
/// ```
/// # use crate::bitpart::metric::{Metric, euclidean::Euclidean};
///
/// let point1: Euclidean<[f64; 2], f64> = Euclidean::new([0.0, 0.0]);
/// let point2: Euclidean<[f64; 2], f64> = Euclidean::new([1.0, 1.0]);
///
/// assert_eq!(point1.distance(&point2), 2.0_f64.sqrt());
/// ```
pub struct Euclidean<T, O>(T, PhantomData<O>);

impl<T, O> Euclidean<T, O>
where
    O: Real,
{
    /// Creates a new `Euclidean`.
    pub fn new(t: T) -> Self {
        Self(t, PhantomData)
    }
}

impl<T, O> Deref for Euclidean<T, O> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// pub fn euclidean_nasa(a: Euclidean<[f64; 20], f64>, b: Euclidean<[f64; 20], f64>) -> f64 {
//     // assert_eq!(a.len(), b.len());
//     a.distance(&b)
// }

impl<T, O> Metric for Euclidean<T, O>
where
    for<'a> &'a T: IntoIterator<Item = &'a O>,
    O: Real + std::iter::Sum,
{
    type Output = O;

    fn distance(&self, rhs: &Euclidean<T, O>) -> Self::Output {
        self.0
            .into_iter()
            .zip(rhs.0.into_iter())
            .map(|(x, y)| (x.sub(*y)).powi(2))
            .sum::<O>()
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclidean_2d() {
        let point1: Euclidean<[f64; 2], f64> = Euclidean::new([0.0, 0.0]);
        let point2: Euclidean<[f64; 2], f64> = Euclidean::new([1.0, 1.0]);

        assert_eq!(point1.distance(&point2), 2.0.sqrt());
    }
}
