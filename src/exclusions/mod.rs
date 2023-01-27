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

    pub(crate) fn is_in(&self, point: &T) -> bool {
        self.point.distance(point) < self.radius
    }
}

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

    pub(crate) fn is_in(&self, point: &T) -> bool {
        self.a.distance(point) - self.b.distance(point) - self.offset < 0.0
    }
}
