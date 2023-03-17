use crate::builder::BitPartBuilder;
use crate::exclusions::{BallExclusion, ExclusionSync, SheetExclusion};
use crate::metric::Metric;
use bitvec::prelude::*;
use itertools::{Either, Itertools};
use rayon::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct DiskBitPart<'a, T> {
    dataset: Vec<T>,
    exclusions: Vec<Box<dyn ExclusionSync<T> + Send + Sync + 'a>>,
    bitset: Vec<File>,
    block_size: usize,
}

impl<'a, T> DiskBitPart<'a, T>
where
    T: Metric + Send + Sync,
    dyn ExclusionSync<T>: Send + Sync + 'a,
{
    pub fn range_search(&self, point: T, threshold: f64) -> Vec<(T, f64)> {
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
            .map(|(block_idx, buf)| {
                (
                    block_idx,
                    bincode::deserialize_from::<&File, Vec<BitVec>>(buf).unwrap(),
                )
            })
            .flat_map(|(block_idx, bitvecs)| {
                assert!(bitvecs.iter().map(|x| x.len()).all_equal());

                let len = bitvecs[0].len();

                let ands = ins
                    .iter()
                    .map(|idx| bitvecs.get(*idx).unwrap())
                    .fold(BitVec::repeat(true, len), |acc, v| acc & v); // TODO: fold or reduce?

                let nots = !outs
                    .iter()
                    .map(|idx| bitvecs.get(*idx).unwrap())
                    .fold(BitVec::repeat(false, len), |acc, v| acc | v);

                let res = ands & nots;

                res.iter_ones()
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

    pub(crate) fn setup<P>(builder: BitPartBuilder<T>, path: P, block_size: Option<usize>) -> Self
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
        builder: &BitPartBuilder<T>,
        ref_points: &[T],
    ) -> Vec<Box<dyn ExclusionSync<T> + Send + Sync + 'a>> {
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
                Box::new(BallExclusion::new(point.clone(), radius))
                    as Box<dyn ExclusionSync<T> + Send + Sync>
            })
            .collect()
    }

    fn sheet_exclusions(
        _builder: &BitPartBuilder<T>,
        ref_points: &[T],
    ) -> Vec<Box<dyn ExclusionSync<T> + Send + Sync + 'a>> {
        ref_points
            .iter()
            .combinations(2)
            .map(|x| {
                Box::new(SheetExclusion::new(x[0].clone(), x[1].clone(), 0.0))
                    as Box<dyn ExclusionSync<T> + Send + Sync>
            })
            .collect()
    }

    fn make_bitset(
        block_size: usize,
        builder: &BitPartBuilder<T>,
        path: PathBuf,
        exclusions: &[Box<dyn ExclusionSync<T> + Send + Sync + 'a>],
    ) -> Vec<File> {
        builder
            .dataset
            .par_chunks(block_size)
            .enumerate()
            .map(|(idx, points)| {
                // Each block is mapped to a vector of bitvecs indexed by [ez_idx][point]
                (
                    idx,
                    exclusions
                        .iter()
                        .map(|ez| points.iter().map(|pt| ez.is_in(pt)).collect::<BitVec>())
                        .collect::<Vec<_>>(),
                )
            })
            .map(|(idx, v)| {
                let path = {
                    let mut p = path.clone();
                    p.push(format!("{}.bincode", idx));
                    p
                };

                let file = File::create(&path).unwrap();

                bincode::serialize_into(file, &v).unwrap();
                path
            })
            .map(|path| File::open(path).unwrap())
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::euclidean::Euclidean;
    use sisap_data::{colors::parse_colors, nasa::parse_nasa};

    use super::*;

    pub(crate) const NASA: &str = include_str!("../sisap-data/src/nasa.ascii");
    pub(crate) const COLORS: &str = include_str!("../sisap-data/src/colors.ascii");

    fn test<T>(dataset: Vec<T>, bitpart: DiskBitPart<T>, query: T, threshold: f64)
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
            .filter(|d| *d < threshold)
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
            BitPartBuilder::new(nasa.clone()).build_on_disk("/tmp/sisap_nasa_par/", Some(512));
        let query = nasa[317].clone();
        let threshold = 1.0;

        test(nasa, bitpart, query, threshold);
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
            BitPartBuilder::new(colors.clone()).build_on_disk("/tmp/sisap_colors_par/", Some(512));
        let query = colors[70446].clone();
        let threshold = 0.5;

        test(colors, bitpart, query, threshold);
        std::fs::remove_dir_all("/tmp/sisap_colors_par/").unwrap();
    }
}