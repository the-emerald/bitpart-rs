use crate::builder::Builder;
use crate::exclusions::{BallExclusion, Exclusion, SheetExclusion};
use crate::metric::Metric;
use crate::BitPart;

use bitvec_simd::BitVec;
use itertools::Itertools;

/// Sequential BitPart.
///
/// This is essentially a Rust port of the [reference library](https://github.com/aldearle/BitPart) written in Java.
///
/// # Construction
///
/// # Query
pub struct Sequential<'a, T> {
    dataset: Vec<T>,
    exclusions: Vec<Box<dyn Exclusion<T> + 'a>>,
    bitset: Vec<BitVec>,
}

impl<T> BitPart<T> for Sequential<'_, T>
where
    T: Metric,
{
    fn range_search(&self, point: T, threshold: f64) -> Vec<(T, f64)> {
        let mut ins = vec![];
        let mut outs = vec![];

        for (idx, ez) in self.exclusions.iter().enumerate() {
            if ez.must_be_in(&point, threshold) {
                ins.push(idx);
            } else if ez.must_be_out(&point, threshold) {
                outs.push(idx);
            }
        }

        let ands: BitVec = ins
            .iter()
            .map(|&i| self.bitset.get(i).unwrap())
            .fold(BitVec::ones(self.dataset.len()), |acc, v| acc & v);

        let nots: BitVec = !outs
            .iter()
            .map(|&i| self.bitset.get(i).unwrap())
            .fold(BitVec::zeros(self.dataset.len()), |acc, v| acc | v);

        let candidates = ands & nots;

        candidates
            .into_usizes()
            .into_iter()
            .map(|i| self.dataset.get(i).unwrap())
            .map(|pt| (pt.clone(), pt.distance(&point)))
            .filter(|(_, dist)| *dist <= threshold)
            .collect()
    }
}

impl<'a, T> Sequential<'a, T>
where
    T: Metric,
    dyn Exclusion<T>: 'a,
{
    pub(crate) fn setup(builder: Builder<T>) -> Self {
        // TODO: actually randomise this
        let ref_points = &builder.dataset[0..(builder.ref_points as usize)];
        let mut exclusions = Self::ball_exclusions(&builder, ref_points);
        exclusions.extend(Self::sheet_exclusions(&builder, ref_points));
        let bitset = Self::make_bitset(&builder, &exclusions);
        Self {
            dataset: builder.dataset,
            bitset,
            exclusions,
        }
    }

    fn ball_exclusions(builder: &Builder<T>, ref_points: &[T]) -> Vec<Box<dyn Exclusion<T> + 'a>> {
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
                Box::new(BallExclusion::new(point.clone(), radius)) as Box<dyn Exclusion<T>>
            })
            .collect()
    }

    fn sheet_exclusions(
        _builder: &Builder<T>,
        ref_points: &[T],
    ) -> Vec<Box<dyn Exclusion<T> + 'a>> {
        ref_points
            .iter()
            .combinations(2)
            .map(|x| {
                Box::new(SheetExclusion::new(x[0].clone(), x[1].clone(), 0.0))
                    as Box<dyn Exclusion<T>>
            })
            .collect()
    }

    fn make_bitset(builder: &Builder<T>, exclusions: &[Box<dyn Exclusion<T> + 'a>]) -> Vec<BitVec> {
        exclusions
            .iter()
            .map(|ex| BitVec::from_bool_iterator(builder.dataset.iter().map(|pt| ex.is_in(pt))))
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::Euclidean;
    use sisap_data::{colors::parse_colors, nasa::parse_nasa};

    use super::*;

    pub(crate) const NASA: &str = include_str!("../sisap-data/src/nasa.ascii");
    pub(crate) const COLORS: &str = include_str!("../sisap-data/src/colors.ascii");

    fn test<T>(dataset: Vec<T>, bitpart: Sequential<T>, query: T, threshold: f64)
    where
        for<'a> T: Metric + 'a,
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
    fn sisap_nasa() {
        let nasa = parse_nasa(NASA)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let bitpart = Builder::new(nasa.clone(), 40).build();
        let query = nasa[317].clone();
        let threshold = 1.0;

        test(nasa, bitpart, query, threshold);
    }

    #[test]
    fn sisap_colors() {
        let colors = parse_colors(COLORS)
            .unwrap()
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let bitpart = Builder::new(colors.clone(), 40).build();
        let query = colors[70446].clone();
        let threshold = 0.5;

        test(colors, bitpart, query, threshold);
    }
}
