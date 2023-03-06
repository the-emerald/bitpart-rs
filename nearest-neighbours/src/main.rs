use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use bitpart::metric::{euclidean::Euclidean, Metric};
use clap::Parser;
use rayon::prelude::*;
use sisap_data::cartesian_parser::parse;

/// Program to calculate the Nth nearest-neighbours for each point in the dataset.
#[derive(Parser, Debug)]
struct Args {
    /// Input file location
    #[arg(short, long)]
    input: PathBuf,

    /// Number of nearest neighbours to record
    #[arg(short, long)]
    n: usize,

    /// Output file location
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let points = parse(&fs::read_to_string(args.input).unwrap())
        .unwrap()
        .1
         .1
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>();

    let closest = points
        .par_iter()
        .map(|pt| {
            let mut points = points
                .par_iter()
                .cloned()
                .enumerate()
                .map(|(idx, p)| (idx, p.distance(pt)))
                .collect::<Vec<_>>();
            points.par_sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
            // Always skip first point, because it's ourselves
            points[1..=args.n].to_vec()
        })
        .collect::<Vec<_>>();

    let file = File::create(args.output).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &closest).unwrap();
    writer.flush().unwrap();
}
