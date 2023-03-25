use bitpart::metric::Metric;
use itertools::Itertools;
use sisap_data::cartesian_parser::parse;
use std::fs;

fn main() {
    let points = parse(&fs::read_to_string("data/100k_flat.ascii").unwrap())
        .unwrap()
        .1
         .1
        .into_iter()
        .map(Point)
        .collect::<Vec<_>>();

    let query = &points[0];
    let threshold = 1.9188728695060282;

    for _ in 0..1000 {
        let _ = points
            .iter()
            .map(|pt| (pt.clone(), pt.distance(query)))
            .filter(|d| d.1 <= threshold)
            .collect::<Vec<_>>();
    }
}

#[derive(Clone, Debug)]
pub struct Point(pub Vec<f64>);

impl Metric for Point {
    #[inline(never)]
    fn distance(&self, rhs: &Self) -> f64 {
        self.0
            .iter()
            .zip(rhs.0.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}
