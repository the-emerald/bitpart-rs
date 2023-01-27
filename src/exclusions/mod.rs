use crate::metric::Metric;

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
