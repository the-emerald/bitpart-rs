use crate::builder::BitPartBuilder;
use crate::exclusions::{BallExclusion, ExclusionSync, SheetExclusion};
use crate::metric::Metric;
use bitvec::prelude::*;
use itertools::{Either, Itertools};
use rayon::prelude::*;

pub struct ParallelBitPart<'a, T> {
    dataset: Vec<T>,
    exclusions: Vec<Box<dyn ExclusionSync<T> + Send + Sync + 'a>>,
    bitset: Vec<Vec<BitVec>>,
    block_size: usize,
}

impl<'a, T> ParallelBitPart<'a, T>
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

    pub(crate) fn setup(builder: BitPartBuilder<T>, block_size: Option<usize>) -> Self {
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
        exclusions: &[Box<dyn ExclusionSync<T> + Send + Sync + 'a>],
    ) -> Vec<Vec<BitVec>> {
        builder
            .dataset
            .par_chunks(block_size)
            .map(|points| {
                // Each block is mapped to a vector of bitvecs indexed by [ez_idx][point]
                exclusions
                    .iter()
                    .map(|ez| points.iter().map(|pt| ez.is_in(pt)).collect::<BitVec>())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<Vec<_>>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::euclidean::Euclidean;
    use sisap_data::{cartesian_parser::parse, colors::parse_colors, nasa::parse_nasa};
    use std::fs;

    use super::*;

    pub(crate) const NASA: &str = include_str!("../sisap-data/src/nasa.ascii");
    pub(crate) const COLORS: &str = include_str!("../sisap-data/src/colors.ascii");

    fn test<T>(dataset: &Vec<T>, bitpart: &ParallelBitPart<T>, query: T, threshold: f64)
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

        let bitpart = BitPartBuilder::new(nasa.clone()).build_parallel(Some(512));
        let query = nasa[317].clone();
        let threshold = 1.0;

        test(&nasa, &bitpart, query, threshold);
    }

    #[test]
    fn sisap_colors_par() {
        let colors = parse_colors(COLORS)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let bitpart = BitPartBuilder::new(colors.clone()).build_parallel(Some(512));
        let query = colors[70446].clone();
        let threshold = 0.5;

        test(&colors, &bitpart, query, threshold);
    }

    #[test]
    fn nearest_neighbour() {
        let points = parse(&fs::read_to_string("data/100k_flat.ascii").unwrap())
            .unwrap()
            .1
             .1
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let nns: Vec<Vec<(usize, f64)>> =
            serde_json::from_str(&fs::read_to_string("data/100k_flat.json").unwrap()).unwrap();

        let queries = points
            .iter()
            .cloned()
            .zip(nns.into_iter())
            .map(|(pt, nn)| (pt, nn.last().unwrap().1))
            .take(1000)
            .collect::<Vec<_>>();

        let bitpart = BitPartBuilder::new(points.clone()).build_parallel(Some(8192));

        for (query, threshold) in queries {
            test(&points, &bitpart, query, threshold);
        }
    }
}
