use crate::{metric::Metric, BitPart};

#[cfg(feature = "rayon")]
use crate::exclusions::ExclusionSync;

#[cfg(feature = "rayon")]
use crate::parallel::ParallelBitPart;

#[cfg(feature = "on_disk")]
use crate::on_disk::DiskBitPart;

/// Builder for the BitPart data structure.
#[derive(Debug, Clone)]
pub struct BitPartBuilder<T> {
    pub(crate) dataset: Vec<T>,

    pub(crate) radius_increment: f64,
    pub(crate) mean_distance: f64,

    pub(crate) four_point: bool,
    pub(crate) ref_points: u64,
}

impl<T> BitPartBuilder<T>
where
    for<'a> T: Metric + 'a,
{
    /// Create a new `BitPartBuilder` from a dataset.
    /// # Panics
    /// This function will panic if `ref_points` is greater than the size of the dataset.
    pub fn new(dataset: impl IntoIterator<Item = T>, ref_points: u64) -> Self {
        let dataset = dataset.into_iter().collect::<Vec<_>>();
        assert!(ref_points as usize <= dataset.len());

        Self {
            dataset,
            mean_distance: 1.81,
            radius_increment: 0.3,
            four_point: true,
            ref_points,
        }
    }

    /// Set the mean distance.
    ///
    /// For historical reasons, the default value is `1.81`.
    pub fn mean_distance(mut self, mean_distance: f64) -> Self {
        self.mean_distance = mean_distance;
        self
    }

    /// Set the radius increment.
    ///
    /// For historical reasons, the default value is `0.3`.
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
    /// # Panics
    /// This function will panic if `ref_points` is greater than the size of the dataset.
    pub fn ref_points(mut self, ref_points: u64) -> Self {
        assert!(ref_points as usize <= self.dataset.len());

        self.ref_points = ref_points;
        self
    }

    /// Build the BitPart.
    pub fn build<'a>(self) -> BitPart<'a, T> {
        BitPart::setup(self)
    }
}

#[cfg(feature = "rayon")]
impl<T> BitPartBuilder<T>
where
    for<'a> T: Metric + Send + Sync + 'a,
    dyn ExclusionSync<T>: Send + Sync,
{
    /// Construct a [`ParallelBitPart`](crate::ParallelBitPart).
    ///
    /// `block_size` sets how many points are processed sequentially in the partition search phase during a range search. For example, `Some(N)` means that
    /// each block will be of size `N` rows. `None` will disable parallelism during queries - this is useful for small datasets where you only wish
    /// to parallelise the bitset creation.
    ///
    /// In other words, `block_size` controls the granularity of parallelisation: the higher the size, the more coarse the parallelism is. It is
    /// recommended that you set a power-of-two value such as `Some(512)` to allow for instruction-level parallelism, while still letting `rayon`
    /// dispatch jobs efficiently to multiple threads.
    pub fn build_parallel<'a>(self, block_size: Option<usize>) -> ParallelBitPart<'a, T> {
        ParallelBitPart::setup(self, block_size)
    }
}

#[cfg(feature = "on_disk")]
impl<T> BitPartBuilder<T>
where
    for<'a> T: Metric + Send + Sync + 'a,
    dyn ExclusionSync<T>: Send + Sync,
{
    /// Construct a [`DiskBitPart`](crate::DiskBitPart).
    ///
    /// `path` should be a path for the directory in which partitioning data will be stored.
    /// This function uses [`create_dir`](std::fs::create_dir) to create the directory, *not* [`create_dir_all`](std::fs::create_dir_all).
    ///
    /// # Panics
    ///
    /// This function will panic if the `create_dir` call is unsuccessful.
    pub fn build_on_disk<'a, P>(self, path: P, block_size: Option<usize>) -> DiskBitPart<'a, T>
    where
        P: AsRef<std::path::Path> + 'a,
    {
        std::fs::create_dir(&path).unwrap();
        DiskBitPart::setup(self, path, block_size)
    }
}
