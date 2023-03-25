use std::fs;
use bitpart::metric::Metric;
use sisap_data::cartesian_parser::parse;

fn main() {
    let points = parse(&fs::read_to_string("data/100k_flat.ascii").unwrap())
        .unwrap()
        .1
        .1
        .into_iter()
        .map(|v| v.try_into().unwrap())
        .map(Point)
        .collect::<Vec<_>>();
    
    let query = &points[0];
    let threshold = 1.9188728695060282;

    let _ = points
        .iter()
        .map(|pt| (pt.clone(), pt.distance(query)))
        .filter(|d| d.1 <= threshold)
        .collect::<Vec<_>>();
}

#[derive(Clone, Debug)]
pub struct Point(pub [f64; 20]);

impl Metric for Point {
    fn distance(&self, rhs: &Self) -> f64 {
        assert!(self.0.len() == rhs.0.len());
        let mut acc = 0.0;
        for (l, r) in self.0.iter().zip(rhs.0.iter()) {
            acc += (l - r).powi(2);
        }
        acc.sqrt()
        // self.0
        //     .iter()
        //     .zip(rhs.0.iter())
        //     .map(|(x, y)| (x.sub(y)).powi(2))
        //     .sum::<f64>()
        //     .sqrt()
    }
}
