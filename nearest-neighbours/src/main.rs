use anyhow::anyhow;
use bitpart::metric::{euclidean::Euclidean, Metric};
use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;
use sisap_data::cartesian_parser::parse;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

const PBAR_TEMPLATE: &str = "[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta_precise})";

/// Program to calculate the Nth nearest-neighbours for each point in the dataset.
#[derive(Parser, Debug)]
struct Args {
    /// Input file location
    #[arg(short, long)]
    input: PathBuf,

    /// Number of nearest neighbours to record
    #[arg(short, long)]
    n: usize,

    /// Number of points to find nearest neighbour for
    #[arg(short, long)]
    points: Option<usize>,

    /// Output file location
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let points = parse(&fs::read_to_string(args.input)?)
        .map_err(|e| anyhow!(e.to_string()))?
        .1
         .1
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>();

    let bar = ProgressStyle::default_bar().template(PBAR_TEMPLATE)?;

    let closest = points
        .par_iter()
        .take(args.points.unwrap_or(points.len()))
        .progress_with_style(bar)
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

    let file = File::create(args.output)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &closest)?;
    writer.flush()?;

    Ok(())
}
