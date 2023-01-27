use builder::BitPartBuilder;
use exclusions::BallExclusion;
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
        Self::set_ball_exclusions(&builder, ref_points);
        todo!()
    }

    fn set_ball_exclusions(builder: &BitPartBuilder<T>, ref_points: &[T]) -> Vec<BallExclusion<T>> {
        let radii = [
            builder.mean_distance - 2.0 * builder.radius_increment,
            builder.mean_distance - builder.radius_increment,
            builder.mean_distance,
            builder.mean_distance + builder.radius_increment,
            builder.mean_distance + 2.0 * builder.radius_increment,
        ];

        let mut exclusions = vec![];

        for point in ref_points {
            let exclusions_subset = radii
                .into_iter()
                .map(|r| BallExclusion::new(point.clone(), r))
                .collect::<Vec<_>>();

            exclusions.extend(exclusions_subset);
        }
        exclusions
    }
}
