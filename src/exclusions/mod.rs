use crate::metric::Metric;

#[cfg(feature = "rayon")]
/// Marker trait for exclusions that are also Send and Sync.
pub trait ExclusionSync<T>: Exclusion<T>
where
    T: Metric + Send + Sync,
{
}

pub trait Exclusion<T>
where
    T: Metric,
{
    fn is_in(&self, point: &T) -> bool;
    fn must_be_in(&self, point: &T, threshold: f64) -> bool;
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
