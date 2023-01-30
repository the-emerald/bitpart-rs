use bitvec::prelude::*;
use builder::BitPartBuilder;
use exclusions::{BallExclusion, Exclusion, SheetExclusion};
use itertools::Itertools;
use metric::Metric;

pub mod builder;
pub mod exclusions;
pub mod metric;

pub struct BitPart<'a, T> {
    dataset: Vec<T>,
    reference_points: Vec<T>,
    exclusions: Vec<Box<dyn Exclusion<T> + 'a>>,
    bitset: Vec<BitVec>,
}

impl<'a, T> BitPart<'a, T>
where
    T: Metric,
    dyn Exclusion<T>: 'a,
{
    pub fn range_search(&self, point: T, threshold: f64) -> Vec<(T, f64)> {
        let mut in_zone = vec![];
        let mut out_zone = vec![];

        for ez in self.exclusions.iter() {
            if ez.must_be_in(&point, threshold) {
                in_zone.push(ez);
            } else if ez.must_be_out(&point, threshold) {
                out_zone.push(ez);
            }
        }

        todo!()
    }

    fn setup(builder: BitPartBuilder<T>) -> Self {
        // TODO: actually randomise this
        let ref_points = &builder.dataset[0..(builder.ref_points as usize)];
        let mut exclusions = Self::ball_exclusions(&builder, ref_points);
        exclusions.extend(Self::sheet_exclusions(&builder, ref_points));
        let bitset = Self::make_bitset(&builder, &exclusions);
        Self {
            reference_points: ref_points.to_vec(),
            dataset: builder.dataset,
            bitset,
            exclusions,
        }
    }

    fn ball_exclusions(
        builder: &BitPartBuilder<T>,
        ref_points: &[T],
    ) -> Vec<Box<dyn Exclusion<T> + 'a>> {
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
        _builder: &BitPartBuilder<T>,
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

    fn make_bitset(
        builder: &BitPartBuilder<T>,
        exclusions: &[Box<dyn Exclusion<T> + 'a>],
    ) -> Vec<BitVec> {
        exclusions
            .iter()
            .map(|ex| {
                builder
                    .dataset
                    .iter()
                    .map(|pt| ex.is_in(pt))
                    .collect::<BitVec>()
            })
            .collect::<Vec<_>>()
    }
}
