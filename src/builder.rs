use crate::{exclusions::Exclusion, metric::Metric, BitPart};

/// Builder for a BitPart query.
#[derive(Debug)]
pub struct BitPartBuilder<T> {
    pub(crate) dataset: Vec<T>,

    pub(crate) radius_increment: f64,
    pub(crate) mean_distance: f64,

    pub(crate) four_point: bool,
    pub(crate) ref_points: u64,
}

impl<T> BitPartBuilder<T>
where
    T: Metric + Send + Sync + 'static,
    dyn Exclusion<T>: Send + Sync,
{
    /// Create a new `BitPartBuilder` from a dataset.
    pub fn new(dataset: impl IntoIterator<Item = T>) -> Self {
        let dataset = dataset.into_iter().collect::<Vec<_>>();

        Self {
            dataset,
            mean_distance: 1.81,   // TODO: Number was copied from ref impl, why?
            radius_increment: 0.3, // TODO: Number was copied from ref impl, why?
            four_point: true,      // TODO: What does this do?
            ref_points: 40,        // TODO: What if there are fewer than 40 points?
        }
    }

    /// Set the mean distance.
    pub fn mean_distance(mut self, mean_distance: f64) -> Self {
        self.mean_distance = mean_distance;
        self
    }

    /// Set the radius increment.
    pub fn radius_increment(mut self, radius_increment: f64) -> Self {
        self.radius_increment = radius_increment;
        self
    }

    /// Set whether to use four-point or three-point method for sheet exclusions.
    pub fn four_point(mut self, four_point: bool) -> Self {
        self.four_point = four_point;
        self
    }

    /// Set the number of ref points
    /// # Panics
    /// This function will panic if `ref_points` is greater than the size of the dataset.
    pub fn ref_points(mut self, ref_points: u64) -> Self {
        if ref_points as usize > self.dataset.len() {
            panic!("ref_points greater than size of dataset");
        }

        self.ref_points = ref_points;
        self
    }

    /// Build the BitPart.
    pub fn build<'a>(self) -> BitPart<'a, T> {
        BitPart::setup(self)
    }
}
