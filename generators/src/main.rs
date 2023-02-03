use std::{
    fs::File,
    io::{BufWriter, Write},
};

use itertools::Itertools;
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

const DIMENSIONS: usize = 2;
const POINTS: usize = 100_000_000;

const MEAN: f64 = 0.0;
const STD_DEV: f64 = 1.0;

const OUTPUT: &str = "output.ascii";

fn main() {
    let mut rng = thread_rng();
    let dist = Normal::new(MEAN, STD_DEV).unwrap();
    println!(
        "Generating {} {}-dimensional points with N({}, {})",
        POINTS, DIMENSIONS, MEAN, STD_DEV
    );

    let points = (0..POINTS)
        .map(|_| {
            (0..DIMENSIONS)
                .map(|_| dist.sample(&mut rng))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    println!("Writing to {}", OUTPUT);
    let mut writer = BufWriter::new(File::create(OUTPUT).unwrap());

    let first_line = format!("{} {} {}", DIMENSIONS, POINTS, 2);
    writeln!(writer, "{}", first_line).unwrap();
    for line in points {
        let line = line.into_iter().join(" ");
        writeln!(writer, "{}", line).unwrap();
    }
}
