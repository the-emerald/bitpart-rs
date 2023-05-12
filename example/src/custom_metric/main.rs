use anyhow::{anyhow, Context, Result};
use bitpart::metric::Metric;
use sisap_data::parser::parse;
use std::fs;

fn main() -> Result<()> {
    let data = fs::read_to_string("data/100k_d10_flat.ascii")
        .context("dataset not found. perhaps you forgot to generate them?")?;
    let points = parse(&data)
        .map_err(|e| anyhow!(e.to_string()))?
        .1
         .1
        .into_iter()
        .map(Point)
        .collect::<Vec<_>>();

    let query = &points[0];
    let threshold = 1.9188728695060282;

    println!("linear search, 1000 iterations");
    for _ in 0..1000 {
        let _ = points
            .iter()
            .map(|pt| (pt.clone(), pt.distance(query)))
            .filter(|d| d.1 <= threshold)
            .collect::<Vec<_>>();
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Point(pub Vec<f64>);

impl Metric for Point {
    // This will help you inspect the assembly generated if necessary.
    // Re-enable inlining if attempting to measure performance.
    #[inline(never)]
    fn distance(&self, rhs: &Self) -> f64 {
        assert_eq!(self.0.len(), rhs.0.len());

        let mut acc = 0.0;
        for (x, y) in self.0.iter().zip(rhs.0.iter()) {
            acc += (x - y).powi(2);
        }

        acc.sqrt()
    }
}
