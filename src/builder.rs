use crate::{metric::Metric, Sequential};

/// Builder for the BitPart data structure.
#[derive(Debug, Clone)]
pub struct Builder<T> {
    pub(crate) dataset: Vec<T>,

    pub(crate) radius_increment: f64,
    pub(crate) mean_distance: f64,

    pub(crate) four_point: bool,
    pub(crate) ref_points: u64,
}

impl<T> Builder<T>
where
    for<'a> T: Metric + 'a,
{
    /// Create a new `BitPartBuilder` from a dataset.
    ///
    /// # Panics
    /// This function will panic if `ref_points` is greater than the size of the dataset, or is zero.
    pub fn new(dataset: impl IntoIterator<Item = T>, ref_points: u64) -> Self {
        let dataset = dataset.into_iter().collect::<Vec<_>>();

        assert!(ref_points as usize <= dataset.len());
        assert!(ref_points > 0);

        Self {
            dataset,
            mean_distance: 1.81,
            radius_increment: 0.3,
            four_point: true,
            ref_points,
        }
    }

    /// Set the mean distance used when generating ball exclusion zones.
    /// For historical reasons, the default value is `1.81`.
    ///
    /// To generate ball exclusions, five radii are chosen per reference point. Their distances are:
    /// ```text
    /// mean_distance + 2 * radius_increment
    /// mean_distance + radius_increment
    /// mean_distance
    /// mean_distance - radius_increment
    /// mean_distance - 2 * radius_increment
    /// ```
    pub fn mean_distance(mut self, mean_distance: f64) -> Self {
        self.mean_distance = mean_distance;
        self
    }

    /// Set the radius incremented used when generating exclusion zones.
    /// For historical reasons, the default value is `0.3`.
    ///
    /// See [`mean_distance`](crate::Builder::mean_distance) for a detailed explanation of how this value is used.
    pub fn radius_increment(mut self, radius_increment: f64) -> Self {
        self.radius_increment = radius_increment;
        self
    }

    /// Set whether to use four-point or three-point method for sheet exclusions.
    #[deprecated(note = "This option does nothing.")]
    pub fn four_point(mut self, four_point: bool) -> Self {
        self.four_point = four_point;
        self
    }

    /// Set the number of ref points
    ///
    /// # Panics
    /// This function will panic if `ref_points` is greater than the size of the dataset, or is zero.
    pub fn ref_points(mut self, ref_points: u64) -> Self {
        assert!(ref_points as usize <= self.dataset.len());
        assert!(ref_points > 0);

        self.ref_points = ref_points;
        self
    }

    /// Build the BitPart.
    pub fn build<'a>(self) -> Sequential<'a, T> {
        Sequential::setup(self)
    }
}
