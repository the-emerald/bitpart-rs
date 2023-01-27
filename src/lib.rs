use builder::BitPartBuilder;
use exclusions::BallExclusion;
use itertools::Itertools;
use metric::Metric;

pub mod builder;
mod exclusions;
pub mod metric;

pub struct BitPart<T> {
    // todo: gets rid of unused type error
    pub tea: T,
}

impl<T> BitPart<T>
where
    T: Metric,
{
    fn setup(builder: BitPartBuilder<T>) -> Self {
        // TODO: actually randomise this
        let ref_points = &builder.dataset[0..(builder.ref_points as usize)];
        Self::ball_exclusions(&builder, ref_points);
        todo!()
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
}
