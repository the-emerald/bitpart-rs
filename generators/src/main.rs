use itertools::Itertools;
use rand::{rngs::StdRng, SeedableRng};
use rand_distr::{Distribution, Normal};
use std::{
    fs::File,
    io::{BufWriter, Write},
};

const DIMENSIONS: usize = 20;
const POINTS: usize = 1_000_000;

const MEAN: f64 = 0.0;
const STD_DEV: f64 = 1.0;

const OUTPUT: &str = "output.ascii";
const SEED: u64 = 0xBAB5_5EED;

fn main() {
    let mut rng = StdRng::seed_from_u64(SEED);
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
