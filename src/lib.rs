use bitvec::prelude::*;
use builder::BitPartBuilder;
use exclusions::{BallExclusion, Exclusion, SheetExclusion};
use itertools::Itertools;
use metric::Metric;

pub mod builder;
mod exclusions;
pub mod metric;

pub struct BitPart<T> {
    dataset: Vec<T>,
    reference_points: Vec<T>,
    ball_exclusions: Vec<BallExclusion<T>>,
    sheet_exclusions: Vec<SheetExclusion<T>>,
    bitset: Vec<BitVec>,
}

impl<T> BitPart<T>
where
    T: Metric,
{
    pub fn range_search(&self, point: T, threshold: f64) -> Vec<(T, f64)> {
        // let distances = self
        //     .reference_points
        //     .iter()
        //     .map(|rp| rp.distance(&point))
        //     .collect::<Vec<_>>();

        let mut in_ball = vec![];
        let mut out_ball = vec![];
        
        for ez in self.ball_exclusions.iter() {
            if ez.must_be_in(&point, threshold) {
                in_ball.push(ez);
            }
            else if ez.must_be_out(&point, threshold) {
                out_ball.push(ez);
            }
        }

        let mut in_sheet = vec![];
        let mut out_sheet = vec![];
        
        for ez in self.sheet_exclusions.iter() {
            if ez.must_be_in(&point, threshold) {
                in_sheet.push(ez);
            }
            else if ez.must_be_out(&point, threshold) {
                out_sheet.push(ez);
            }
        }

        todo!()
    }

    fn setup(builder: BitPartBuilder<T>) -> Self {
        // TODO: actually randomise this
        let ref_points = &builder.dataset[0..(builder.ref_points as usize)];
        let ball_exclusions = Self::ball_exclusions(&builder, ref_points);
        let sheet_exclusions = Self::sheet_exclusions(&builder, ref_points);
        let bitset = Self::make_bitset(&builder, &ball_exclusions, &sheet_exclusions);
        Self {
            reference_points: ref_points.to_vec(),
            dataset: builder.dataset,
            ball_exclusions,
            sheet_exclusions,
            bitset,
        }
    }

    fn ball_exclusions(builder: &BitPartBuilder<T>, ref_points: &[T]) -> Vec<BallExclusion<T>> {
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
            .map(|(point, radius)| BallExclusion::new(point.clone(), radius))
            .collect()
    }

    fn sheet_exclusions(_builder: &BitPartBuilder<T>, ref_points: &[T]) -> Vec<SheetExclusion<T>> {
        ref_points
            .iter()
            .combinations(2)
            .map(|x| SheetExclusion::new(x[0].clone(), x[1].clone(), 0.0))
            .collect()
    }

    fn make_bitset(
        builder: &BitPartBuilder<T>,
        ball_exclusions: &[BallExclusion<T>],
        sheet_exclusions: &[SheetExclusion<T>],
    ) -> Vec<BitVec> {
        let mut ball_bitvecs = ball_exclusions
            .iter()
            .map(|ex| {
                builder
                    .dataset
                    .iter()
                    .map(|pt| ex.is_in(pt))
                    .collect::<BitVec>()
            })
            .collect::<Vec<_>>();

        let sheet_bitvecs = sheet_exclusions.iter().map(|ex| {
            builder
                .dataset
                .iter()
                .map(|pt| ex.is_in(pt))
                .collect::<BitVec>()
        });

        ball_bitvecs.extend(sheet_bitvecs);
        ball_bitvecs
    }
}
