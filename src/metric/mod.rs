//! Metric space definitions

mod euclidean;
pub use euclidean::*;

/// Trait for types in metric space.
pub trait Metric: Clone {
    /// Distance between two points.
    /// # Axioms
    /// For a distance function to be valid, the following axioms must be met:
    /// 1. The distance from a point to itself is zero.
    /// 2. The distance between two distinct points is always positive.
    /// 3. The distance from `x` to `y` is always the same as the distance from `y` to `x`.
    /// 4. The triangle inequality:
    /// ```text
    /// x.distance(y) <= x.distance(y) + y.distance(z)
    /// ```
    ///
    /// **It is the responsibility of the implementer to ensure that the axiom are met.**
    fn distance(&self, rhs: &Self) -> f64;
}
