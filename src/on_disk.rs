use crate::builder::Builder;
use crate::exclusions::{BallExclusion, ExclusionSync, SheetExclusion};
use crate::metric::Metric;

use bitvec::prelude::*;
use itertools::{Either, Itertools};
use rayon::prelude::*;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

/// On-disk BitPart.
///
/// BitPart variant which stores partitioning data on disk. Instead of holding a vector of bitsets in memory, this struct
/// holds a vector of [`Mmap`](memmap2::Mmap)s and only deserializes the relevant columns into memory at query-time.
///
/// The BitPart data structure consists of three components: the dataset itself, information about exclusion zones, and a
/// vector of bitsets which represent the partitioning data for each point and exclusion zone. Note that for a given query,
/// not every partition matters - exclusion zones are only considered if the
/// query point [must be in](crate::exclusions::Exclusion::must_be_in) or [must be out](crate::exclusions::Exclusion::must_be_out).
/// Therefore, we can reduce the amount of memory used by BitPart by only loading columns useful to a particular query.
///
/// As a benchmark figure, on the default setting of 40 reference points, `40 * 5 = 200` balls and `40c2 = 780` plane exclusions are made.
/// At 20 dimensions, each point requires `20 * 64 = 1280` bits of storage plus `200 + 780 = 980` bits of partitioning data.
///
/// Because the bitsets are memory-mapped to files, the kernel may decide to cache reads into memory. This has the consequence that
/// if `Disk` is used when volatile memory can fully hold the partitioning data, queries are almost native speed (with a ~10% performance penalty).
///
/// `Disk` is parallelised. See [`Parallel`](crate::Parallel) for an explanation of the mechanics.
pub struct Disk<'a, T> {
    dataset: Vec<T>,
    exclusions: Vec<Box<dyn ExclusionSync<T> + 'a>>,
    bitset: Vec<memmap2::Mmap>,
    block_size: usize,
}

impl<T> crate::BitPart<T> for Disk<'_, T>
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

        let ins = ins
            .into_par_iter()
            .map(|idx| bincode::deserialize::<BitVec>(self.bitset.get(idx).unwrap()).unwrap())
            .collect::<Vec<_>>();

        let outs = outs
            .into_par_iter()
            .map(|idx| bincode::deserialize::<BitVec>(self.bitset.get(idx).unwrap()).unwrap())
            .collect::<Vec<_>>();

        self.dataset
            .par_chunks(self.block_size)
            .enumerate()
            .flat_map(|(blk_idx, points)| {
                let from = blk_idx * self.block_size;
                let to = (blk_idx * self.block_size) + points.len();

                let blk_ins = ins.iter().map(|bv| &bv[from..to]).collect::<Vec<_>>();
                let blk_outs = outs.iter().map(|bv| &bv[from..to]).collect::<Vec<_>>();

                let len = points.len();

                let ands = blk_ins
                    .into_iter()
                    .fold(BitVec::repeat(true, len), |acc: BitVec, v| acc & v);

                let nots = !blk_outs
                    .into_iter()
                    .fold(BitVec::repeat(false, len), |acc: BitVec, v| acc | v);

                let res = ands & nots;

                res.iter_ones().map(|idx| &points[idx]).collect::<Vec<_>>()
            })
            .map(|pt| (pt.clone(), point.distance(pt)))
            .filter(|(_, d)| *d <= threshold)
            .collect::<Vec<_>>()
    }
}

impl<'a, T> Disk<'a, T>
where
    T: Metric + Send + Sync,
    dyn ExclusionSync<T>: 'a,
{
    pub(crate) fn setup<P>(builder: Builder<T>, path: P, block_size: Option<usize>) -> Self
    where
        P: AsRef<Path> + 'a,
    {
        let block_size = block_size.unwrap_or(builder.dataset.len());
        let path = path.as_ref().to_owned();
        // TODO: actually randomise this
        let ref_points = &builder.dataset[0..(builder.ref_points as usize)];
        let mut exclusions = Self::ball_exclusions(&builder, ref_points);
        exclusions.extend(Self::sheet_exclusions(&builder, ref_points));
        let bitset = Self::make_bitset(block_size, &builder, path, &exclusions);
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
        path: PathBuf,
        exclusions: &[Box<dyn ExclusionSync<T> + 'a>],
    ) -> Vec<memmap2::Mmap> {
        exclusions
            .par_iter()
            .enumerate()
            .map(|(idx, ez)| Self::make_mmap(&builder.dataset, path.clone(), idx, ez.as_ref()))
            .collect::<Vec<_>>()
    }

    fn make_mmap(
        dataset: &[T],
        mut path: PathBuf,
        index: usize,
        ez: &(dyn ExclusionSync<T> + 'a),
    ) -> memmap2::Mmap {
        let bv = dataset.iter().map(|pt| ez.is_in(pt)).collect::<BitVec>();

        path.push(format!("{}.bincode", index));
        let file = File::create(&path).unwrap();
        bincode::serialize_into(file, &bv).unwrap();

        unsafe { memmap2::Mmap::map(&File::open(path).unwrap()).unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use crate::{metric::Euclidean, BitPart};
    use sisap_data::{colors::parse_colors, nasa::parse_nasa, parser::parse};
    use std::fs;

    use super::*;

    pub(crate) const NASA: &str = include_str!("../sisap-data/src/nasa.ascii");
    pub(crate) const COLORS: &str = include_str!("../sisap-data/src/colors.ascii");

    fn test<T>(dataset: &Vec<T>, bitpart: &Disk<T>, query: T, threshold: f64)
    where
        for<'a> T: Metric + Send + Sync + 'a,
    {
        let res = bitpart.range_search(query.clone(), threshold);
        bitpart.range_search(query.clone(), threshold);

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
        std::fs::remove_dir_all("/tmp/sisap_nasa_par/").ok();
        let nasa = parse_nasa(NASA)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let bitpart =
            Builder::new(nasa.clone(), 40).build_on_disk("/tmp/sisap_nasa_par/", Some(8192));
        let query = nasa[317].clone();
        let threshold = 1.0;

        test(&nasa, &bitpart, query, threshold);
        std::fs::remove_dir_all("/tmp/sisap_nasa_par/").unwrap();
    }

    #[test]
    fn sisap_colors_par() {
        std::fs::remove_dir_all("/tmp/sisap_colors_par/").ok();
        let colors = parse_colors(COLORS)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let bitpart =
            Builder::new(colors.clone(), 40).build_on_disk("/tmp/sisap_colors_par/", Some(8192));
        let query = colors[70446].clone();
        let threshold = 0.5;

        test(&colors, &bitpart, query, threshold);
        std::fs::remove_dir_all("/tmp/sisap_colors_par/").unwrap();
    }

    #[test]
    fn nearest_neighbour() {
        std::fs::remove_dir_all("/tmp/nn/").ok();

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

        let bitpart = Builder::new(points.clone(), 40).build_on_disk("/tmp/nn/", Some(8192));

        for (query, threshold) in queries {
            test(&points, &bitpart, query, threshold);
        }

        std::fs::remove_dir_all("/tmp/nn/").unwrap();
    }
}

impl<T> Builder<T>
where
    for<'a> T: Metric + Send + Sync + 'a,
{
    /// Construct a [`Disk`](crate::Disk).
    ///
    /// `path` should be a path for the directory in which partitioning data will be stored.
    /// This function uses [`create_dir`](std::fs::create_dir) to create the directory, *not* [`create_dir_all`](std::fs::create_dir_all).
    ///
    /// # Panics
    /// This function will panic if the `create_dir` call is unsuccessful.
    pub fn build_on_disk<'a, P>(self, path: P, block_size: Option<usize>) -> Disk<'a, T>
    where
        P: AsRef<std::path::Path> + 'a,
    {
        std::fs::create_dir(&path).unwrap();
        Disk::setup(self, path, block_size)
    }
}
