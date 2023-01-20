use num::traits::real::Real;

pub mod euclidean;

pub trait Metric<T> {
    type Output: Real;

    fn distance(self, rhs: T) -> Self::Output;
}
