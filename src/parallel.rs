use crate::builder::Builder;
use crate::exclusions::{BallExclusion, ExclusionSync, SheetExclusion};
use crate::metric::Metric;
use crate::BitPart;

use bitvec_simd::BitVec;
use itertools::{Either, Itertools};
use rayon::prelude::*;
use std::collections::HashSet;

/// Parallel BitPart.
///
/// The BitPart algorithm (and its data structures) are designed to be highly parallelisable.
/// `Parallel` takes advantage of this by using [`rayon`](rayon) to parallelise both
/// the initial construction of the data structure and subsequent queries.
///
/// Jobs are distributed across threads by work-stealing. By default `rayon` will create as many threads
/// as there are logical cores.
///
/// In general, it is not possible to change the "size" of each job as `rayon`'s work-stealing strategy works well to eliminate
/// overhead no matter the job size. The one and only exception is when filtering candidate points based on partitioning data; points
/// are explicitly processed in chunks to enable instruction-level parallelism when comparing bitsets.
/// See [`build_parallel`](crate::builder::Builder::build_parallel) for configuration.
pub struct Parallel<'a, T> {
    dataset: Vec<T>,
    exclusions: Vec<Box<dyn ExclusionSync<T> + 'a>>,
    bitset: Vec<Vec<BitVec>>,
    block_size: usize,
}

impl<T> BitPart<T> for Parallel<'_, T>
where
    T: Metric + Send + Sync,
{
    fn range_search(&self, point: T, threshold: f64) -> Vec<(T, f64)> {
        let (ins, outs): (Vec<usize>, Vec<usize>) = self
            .exclusions
            .par_iter()
            .enumerate()
            .filter_map(|(idx, ez)| {
                if ez.must_be_in(&point, threshold) {
                    Some(Either::Left(idx))
                } else if ez.must_be_out(&point, threshold) {
                    Some(Either::Right(idx))
                } else {
                    None
                }
            })
            .partition_map(|x| x);

        self.bitset
            .par_iter()
            .enumerate()
            .flat_map(|(block_idx, bitvecs)| {
                let len = bitvecs[0].len();

                let ands = ins
                    .iter()
                    .map(|idx| bitvecs.get(*idx).unwrap())
                    .fold(BitVec::ones(len), |acc, v| acc & v); // TODO: fold or reduce?

                let nots = !outs
                    .iter()
                    .map(|idx| bitvecs.get(*idx).unwrap())
                    .fold(BitVec::zeros(len), |acc, v| acc | v);

                let res = ands & nots;

                res.into_usizes()
                    .into_iter()
                    .map(|internal_idx| {
                        self.dataset
                            .get((block_idx * self.block_size) + internal_idx)
                            .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .map(|pt| (pt.clone(), point.distance(pt)))
            .filter(|(_, d)| *d <= threshold)
            .collect::<Vec<_>>()
    }
}

impl<'a, T> Parallel<'a, T>
where
    T: Metric + Send + Sync,
    dyn ExclusionSync<T>: 'a,
{
    pub(crate) fn setup(builder: Builder<T>, block_size: Option<usize>) -> Self {
        let block_size = block_size.unwrap_or(builder.dataset.len());
        // TODO: actually randomise this
        let ref_points = &builder.dataset[0..(builder.ref_points as usize)];
        let mut exclusions = Self::ball_exclusions(&builder, ref_points);
        exclusions.extend(Self::sheet_exclusions(&builder, ref_points));
        let bitset = Self::make_bitset(block_size, &builder, &exclusions);
        Self {
            dataset: builder.dataset,
            bitset,
            exclusions,
            block_size,
        }
    }

    fn ball_exclusions(
        builder: &Builder<T>,
        ref_points: &[T],
    ) -> Vec<Box<dyn ExclusionSync<T> + 'a>> {
        let radii = [
            builder.mean_distance - 2.0 * builder.radius_increment,
            builder.mean_distance - builder.radius_increment,
            builder.mean_distance,
            builder.mean_distance + builder.radius_increment,
            builder.mean_distance + 2.0 * builder.radius_increment,
        ];

        ref_points
            .iter()
            .cartesian_product(radii.into_iter())
            .map(|(point, radius)| {
                Box::new(BallExclusion::new(point.clone(), radius)) as Box<dyn ExclusionSync<T>>
            })
            .collect()
    }

    fn sheet_exclusions(
        _builder: &Builder<T>,
        ref_points: &[T],
    ) -> Vec<Box<dyn ExclusionSync<T> + 'a>> {
        ref_points
            .iter()
            .combinations(2)
            .map(|x| {
                Box::new(SheetExclusion::new(x[0].clone(), x[1].clone(), 0.0))
                    as Box<dyn ExclusionSync<T>>
            })
            .collect()
    }

    fn make_bitset(
        _block_size: usize,
        builder: &Builder<T>,
        exclusions: &[Box<dyn ExclusionSync<T> + 'a>],
    ) -> Vec<Vec<BitVec>> {
        builder
            .dataset
            .par_chunks(_block_size)
            .map(|points| {
                exclusions
                    .par_iter()
                    .map(|ez| BitVec::from_bool_iterator(points.iter().map(|pt| ez.is_in(pt))))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn ratio(&self, ones: usize) -> f64 {
        ones as f64 / self.dataset.len() as f64
    }

    /// Cull exclusion zones with low exclusion power.
    /// This function will compare all zones in in the data structure with one another and calculate their [Hamming distance](https://en.wikipedia.org/wiki/Hamming_distance).
    /// If a zone's similarity ratio is above the given `threshold`, it is marked for removal.
    pub fn cull_by_similarity(&mut self, threshold: f64) {
        let mut to_cull = HashSet::new();
        for indices in (0..self.exclusions.len()).combinations(2) {
            let i = indices[0];
            let j = indices[1];
            let hamming = {
                self.bitset
                    .iter()
                    .map(|bvs| (bvs[i].xor_cloned(&bvs[j])).count_ones())
                    .sum::<usize>()
            };

            if 1.0 - self.ratio(hamming) > threshold {
                to_cull.insert(j);
            }
        }

        self.cull(to_cull)
    }

    /// Cull exclusion zones with low exclusion power.
    /// This function measures the exclusion power of a zone by counting the ratio of points that are in/out to the dataset.
    /// If either ratio is above the `threshold` given, it is marked for removal.
    pub fn cull_by_popcnt(&mut self, threshold: f64) {
        let len = self.exclusions.len();
        let mut to_cull = HashSet::new();

        // Count ones for each column, across all the blocks.
        let popcnt = self.bitset.iter().fold(vec![0_usize; len], |acc, x| {
            acc.into_iter()
                .zip(x.iter())
                .map(|(a, b)| a + b.count_ones())
                .collect()
        });

        for (idx, cnt) in popcnt.into_iter().enumerate() {
            if self.ratio(cnt) > threshold
                || self.ratio(self.dataset.len() - cnt) > threshold
            {
                to_cull.insert(idx);
            }
        }

        self.cull(to_cull)
    }

    fn cull(&mut self, to_cull: HashSet<usize>) {
        let keep = (0..self.exclusions.len())
            .map(|idx| !to_cull.contains(&idx))
            .collect::<Vec<_>>();

        for bvs in self.bitset.iter_mut() {
            let mut iter = keep.iter();

            bvs.retain(|_| *iter.next().unwrap());
        }

        let mut iter = keep.iter();
        self.exclusions.retain(|_| *iter.next().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::Euclidean;
    use sisap_data::{cartesian_parser::parse, colors::parse_colors, nasa::parse_nasa};
    use std::fs;

    use super::*;

    pub(crate) const NASA: &str = include_str!("../sisap-data/src/nasa.ascii");
    pub(crate) const COLORS: &str = include_str!("../sisap-data/src/colors.ascii");

    fn test<T>(dataset: &Vec<T>, bitpart: &Parallel<T>, query: T, threshold: f64)
    where
        for<'a> T: Metric + Send + Sync + 'a,
    {
        let res = bitpart.range_search(query.clone(), threshold);

        // Check all points within threshold
        assert!(res
            .iter()
            .all(|(point, _)| point.distance(&query) <= threshold));

        // Check results match up with linear search
        let brute_force = dataset
            .into_iter()
            .map(|pt| pt.distance(&query))
            .filter(|d| *d <= threshold)
            .count();

        assert_eq!(res.len(), brute_force);
    }

    #[test]
    fn sisap_nasa_par() {
        let nasa = parse_nasa(NASA)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let bitpart = Builder::new(nasa.clone(), 40).build_parallel(Some(512));
        let query = nasa[317].clone();
        let threshold = 1.0;

        test(&nasa, &bitpart, query.clone(), threshold);
    }

    #[test]
    fn sisap_nasa_par_cull_popcnt() {
        let nasa = parse_nasa(NASA)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let mut bitpart = Builder::new(nasa.clone(), 40).build_parallel(Some(512));
        let query = nasa[317].clone();
        let threshold = 1.0;

        bitpart.cull_by_popcnt(0.95);
        test(&nasa, &bitpart, query, threshold);
    }

    #[test]
    fn sisap_nasa_par_cull_similarity() {
        let nasa = parse_nasa(NASA)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let mut bitpart = Builder::new(nasa.clone(), 40).build_parallel(Some(512));
        let query = nasa[317].clone();
        let threshold = 1.0;

        bitpart.cull_by_similarity(0.95);
        test(&nasa, &bitpart, query, threshold);
    }

    #[test]
    fn sisap_colors_par() {
        let colors = parse_colors(COLORS)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let bitpart = Builder::new(colors.clone(), 40).build_parallel(Some(512));
        let query = colors[70446].clone();
        let threshold = 0.5;

        test(&colors, &bitpart, query.clone(), threshold);
    }

    #[test]
    fn sisap_colors_par_cull_popcnt() {
        let colors = parse_colors(COLORS)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let mut bitpart = Builder::new(colors.clone(), 40).build_parallel(Some(512));
        let query = colors[70446].clone();
        let threshold = 0.5;

        bitpart.cull_by_popcnt(0.95);
        test(&colors, &bitpart, query, threshold);
    }

    #[test]
    fn sisap_colors_par_cull_similarity() {
        let colors = parse_colors(COLORS)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let mut bitpart = Builder::new(colors.clone(), 40).build_parallel(Some(512));
        let query = colors[70446].clone();
        let threshold = 0.5;

        bitpart.cull_by_similarity(0.95);
        test(&colors, &bitpart, query, threshold);
    }

    #[test]
    fn nearest_neighbour() {
        let points = parse(&fs::read_to_string("data/100k_d20_flat.ascii").unwrap())
            .unwrap()
            .1
             .1
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let nns: Vec<Vec<(usize, f64)>> =
            serde_json::from_str(&fs::read_to_string("data/100k_d20_flat.json").unwrap()).unwrap();

        let queries = points
            .iter()
            .cloned()
            .zip(nns.into_iter())
            .map(|(pt, nn)| (pt, nn.last().unwrap().1))
            .take(1000)
            .collect::<Vec<_>>();

        let bitpart = Builder::new(points.clone(), 40).build_parallel(Some(8192));

        for (query, threshold) in queries {
            test(&points, &bitpart, query, threshold);
        }
    }

    #[test]
    fn nearest_neighbour_cull_popcnt() {
        let points = parse(&fs::read_to_string("data/100k_d20_flat.ascii").unwrap())
            .unwrap()
            .1
             .1
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let nns: Vec<Vec<(usize, f64)>> =
            serde_json::from_str(&fs::read_to_string("data/100k_d20_flat.json").unwrap()).unwrap();

        let queries = points
            .iter()
            .cloned()
            .zip(nns.into_iter())
            .map(|(pt, nn)| (pt, nn.last().unwrap().1))
            .take(1000)
            .collect::<Vec<_>>();

        let mut bitpart = Builder::new(points.clone(), 40).build_parallel(Some(8192));

        bitpart.cull_by_popcnt(0.95);
        for (query, threshold) in queries {
            test(&points, &bitpart, query, threshold);
        }
    }

    #[test]
    fn nearest_neighbour_cull_similarity() {
        let points = parse(&fs::read_to_string("data/100k_d20_flat.ascii").unwrap())
            .unwrap()
            .1
             .1
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let nns: Vec<Vec<(usize, f64)>> =
            serde_json::from_str(&fs::read_to_string("data/100k_d20_flat.json").unwrap()).unwrap();

        let queries = points
            .iter()
            .cloned()
            .zip(nns.into_iter())
            .map(|(pt, nn)| (pt, nn.last().unwrap().1))
            .take(1000)
            .collect::<Vec<_>>();

        let mut bitpart = Builder::new(points.clone(), 40).build_parallel(Some(8192));

        bitpart.cull_by_popcnt(0.95);
        for (query, threshold) in queries {
            test(&points, &bitpart, query, threshold);
        }
    }
}

impl<T> Builder<T>
where
    for<'a> T: Metric + Send + Sync + 'a,
{
    /// Construct a [`Parallel`](crate::Parallel).
    ///
    /// `block_size` sets how many points are processed sequentially in the partition search phase during a range search. For example, `Some(N)` means that
    /// each block will be of size `N` rows. `None` will disable parallelism during queries - this is useful for small datasets where you only wish
    /// to parallelise the bitset creation.
    ///
    /// In other words, `block_size` controls the granularity of parallelisation: the higher the size, the more coarse the parallelism is. It is
    /// recommended that you set a power-of-two value such as `Some(512)` to allow for instruction-level parallelism, while still letting `rayon`
    /// dispatch jobs efficiently to multiple threads.
    pub fn build_parallel<'a>(self, block_size: Option<usize>) -> Parallel<'a, T> {
        Parallel::setup(self, block_size)
    }
}
