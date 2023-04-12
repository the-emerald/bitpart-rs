//! Exclusion zone definitions.
//!
//! The BitPart algorithm defines exclusion zones which partition the entire metric space in two (referred to as "in" or "out").
//!
//! # Default implementation
//! The reference implementation uses both ball and sheet implementations, as described in the paper.
//!
//! While the Exclusion traits are not sealed, it is currently not possible to use custom exclusion zones in BitPart.

use crate::metric::Metric;

#[cfg(feature = "rayon")]
/// Marker trait for exclusions that are also `Send` and `Sync`.
pub trait ExclusionSync<T>: Exclusion<T> + Send + Sync
where
    T: Metric + Send + Sync,
{
}

/// An exclusion zone.
pub trait Exclusion<T>
where
    T: Metric,
{
    /// Tests whether a point is in the exclusion zone.
    fn is_in(&self, point: &T) -> bool;
    /// Tests whether a point must be inside the exclusion zone.
    fn must_be_in(&self, point: &T, threshold: f64) -> bool;
    /// Tests whether a point must be outside the exclusion zone.
    fn must_be_out(&self, point: &T, threshold: f64) -> bool;
}

pub(crate) struct BallExclusion<T> {
    pub(crate) point: T,
    pub(crate) radius: f64,
}

impl<T> BallExclusion<T>
where
    T: Metric,
{
    pub(crate) fn new(point: T, radius: f64) -> Self {
        Self { point, radius }
    }
}

impl<T> Exclusion<T> for BallExclusion<T>
where
    T: Metric,
{
    fn is_in(&self, point: &T) -> bool {
        self.point.distance(point) < self.radius
    }

    fn must_be_in(&self, point: &T, threshold: f64) -> bool {
        self.point.distance(point) < (self.radius - threshold)
    }

    fn must_be_out(&self, point: &T, threshold: f64) -> bool {
        self.point.distance(point) >= (self.radius + threshold)
    }
}

#[cfg(feature = "rayon")]
impl<T> ExclusionSync<T> for BallExclusion<T> where T: Metric + Send + Sync {}

// todo: this is only 3p
pub(crate) struct SheetExclusion<T> {
    a: T,
    b: T,
    offset: f64,
}

impl<T> SheetExclusion<T>
where
    T: Metric,
{
    pub(crate) fn new(a: T, b: T, offset: f64) -> Self {
        Self { a, b, offset }
    }
}

impl<T> Exclusion<T> for SheetExclusion<T>
where
    T: Metric,
{
    fn is_in(&self, point: &T) -> bool {
        self.a.distance(point) - self.b.distance(point) - self.offset < 0.0
    }

    fn must_be_in(&self, point: &T, threshold: f64) -> bool {
        point.distance(&self.a) - point.distance(&self.b) - self.offset < (-2.0 * threshold)
    }

    fn must_be_out(&self, point: &T, threshold: f64) -> bool {
        point.distance(&self.a) - point.distance(&self.b) - self.offset >= (2.0 * threshold)
    }
}

#[cfg(feature = "rayon")]
impl<T> ExclusionSync<T> for SheetExclusion<T> where T: Metric + Send + Sync {}
