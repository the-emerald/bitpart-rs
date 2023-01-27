use builder::BitPartBuilder;
use exclusions::{BallExclusion, SheetExclusion};
use itertools::Itertools;
use metric::Metric;

pub mod builder;
mod exclusions;
pub mod metric;

pub struct BitPart<T> {
    dataset: Vec<T>,
    ball_exclusions: Vec<BallExclusion<T>>,
    sheet_exclusions: Vec<SheetExclusion<T>>,
}

impl<T> BitPart<T>
where
    T: Metric,
{
    fn setup(builder: BitPartBuilder<T>) -> Self {
        // TODO: actually randomise this
        let ref_points = &builder.dataset[0..(builder.ref_points as usize)];
        let ball_exclusions = Self::ball_exclusions(&builder, ref_points);
        let sheet_exclusions = Self::sheet_exclusions(&builder, ref_points);
        Self {
            ball_exclusions,
            sheet_exclusions,
            dataset: builder.dataset,
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
            .map(|x| SheetExclusion::new(x[0].clone(), x[1].clone()))
            .collect()
    }
}
