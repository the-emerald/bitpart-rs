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
    exclusions: Vec<Box<dyn Exclusion<T> + 'a>>,
    bitset: Vec<BitVec>,
}

impl<'a, T> BitPart<'a, T>
where
    T: Metric,
    dyn Exclusion<T>: 'a,
{
    pub fn range_search(&self, point: T, threshold: f64) -> Vec<(T, f64)> {
        let mut ins = vec![];
        let mut outs = vec![];

        for (idx, ez) in self.exclusions.iter().enumerate() {
            if ez.must_be_in(&point, threshold) {
                ins.push((idx, ez));
            } else if ez.must_be_out(&point, threshold) {
                outs.push((idx, ez));
            }
        }

        match (ins.len(), outs.len()) {
            // No exclusions at all, linear search
            (0, 0) => self
                .dataset
                .iter()
                .cloned()
                .map(|pt| (pt.clone(), pt.distance(&point)))
                .filter(|(_, dist)| *dist < threshold)
                .collect(),
            // nots, flip, filter
            (0, _) => {
                let nots = self.get_nots(outs.iter().map(|(i, _)| *i));
                let nots = !nots; // TODO: is nots always of length self.dataset.len()?
                self.filter_contenders(threshold, point, nots)
            }
            // filter
            (_, 0) => {
                let ands = self.get_ands(ins.iter().map(|(i, _)| *i));
                self.filter_contenders(threshold, point, ands)
            }
            // nots, flip, and, filter
            (_, _) => {
                let ands = self.get_ands(ins.iter().map(|(i, _)| *i));
                let nots = self.get_nots(outs.iter().map(|(i, _)| *i));
                let nots = !nots;
                let ands = ands & nots;
                self.filter_contenders(threshold, point, ands)
            }
        }

        // if ins not empty:
        //      ands = get_and()
        //      if outs not empty:
        //          nots = get_nots()
        //          nots flip 0..sz
        //          and &= nots
        //          filter_contenders()
        //      else:
        //          filter_contenders()
        // else:
        //      if outs not empty:
        //          nots = get_nots()
        //          nots flip 0..sz
        //          filter_contenders()
        //      else:
        //          for d in data:
        //              dist = query.distance(d)
        //              if dist < threshold:
        //                  res.push(d)
    }

    /// Performs a bitwise-or on all exclusion zone columns that do not contain the query point.
    fn get_nots(&self, outs: impl IntoIterator<Item = usize>) -> BitVec {
        outs.into_iter()
            .map(|i| self.bitset.get(i).unwrap())
            .cloned()
            .reduce(|acc, bv| acc | bv)
            .unwrap()
    }

    /// Performs a bitwise-and on all exclusion zone columns that contain the query point.
    fn get_ands(&self, ins: impl IntoIterator<Item = usize>) -> BitVec {
        ins.into_iter()
            .map(|i| self.bitset.get(i).unwrap())
            .cloned()
            .reduce(|acc, bv| acc & bv)
            .unwrap()
    }

    fn filter_contenders(&self, threshold: f64, point: T, res: BitVec) -> Vec<(T, f64)> {
        res.iter_ones()
            .map(|i| self.dataset.get(i).unwrap())
            .map(|pt| (pt.clone(), pt.distance(&point)))
            .filter(|(_, dist)| *dist <= threshold)
            .collect()
    }

    fn setup(builder: BitPartBuilder<T>) -> Self {
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
