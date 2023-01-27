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

    pub(crate) fn set_witnesses(&mut self, witnesses: impl IntoIterator<Item = T>) {
        let mut distances = witnesses
            .into_iter()
            .map(|p| self.point.distance(&p))
            .collect::<Vec<_>>();

        let half = distances.len() / 2;
        distances.select_nth_unstable_by(half, |a, b| a.partial_cmp(b).unwrap());
        self.radius = distances.get(half).unwrap().clone();
    }
}
