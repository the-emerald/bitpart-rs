use clap::{Parser, Subcommand};
use indicatif::{ProgressIterator, ProgressStyle};
use itertools::Itertools;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use rand_distr::{Distribution, Normal, Uniform};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

const PBAR_TEMPLATE: &str = "[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta_precise})";

/// Program to generate randomly sampled points and save them in .ascii format.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Number of dimensions
    #[arg(short, long)]
    dimensions: usize,

    /// Number of points to generate
    #[arg(short, long)]
    points: usize,

    #[command(subcommand)]
    distribution: Command,

    /// Location for output file
    #[arg(short, long)]
    output: PathBuf,

    /// Seed to use for the RNG
    #[arg(short, long)]
    seed: Option<u64>,
}

/// Distribution to use
#[derive(Subcommand, Debug)]
enum Command {
    Normal {
        /// Mean of the normal distribution
        #[arg(short, long)]
        mean: f64,

        /// Standard deviation of the normal distribution
        #[arg(short, long)]
        std_dev: f64,
    },
    /// TODO: Flat distribution
    Flat {
        /// Lower bound
        #[arg(short, long, allow_negative_numbers = true)]
        low: f64,

        /// Upper bound
        #[arg(short, long, allow_negative_numbers = true)]
        high: f64,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut rng = StdRng::seed_from_u64(args.seed.unwrap_or_else(rand::random::<u64>));
    let points = match args.distribution {
        Command::Normal { mean, std_dev } => {
            let dist = Normal::new(mean, std_dev).unwrap();
            generate_points(args.dimensions, args.points, dist, &mut rng)
        }
        Command::Flat { low, high } => {
            let dist = Uniform::new(low, high);
            generate_points(args.dimensions, args.points, dist, &mut rng)
        }
    }?;

    let mut writer = BufWriter::new(File::create(args.output)?);

    let first_line = format!("{} {} {}", args.dimensions, args.points, 2);
    writeln!(writer, "{}", first_line)?;
    for line in points {
        let line = line.into_iter().join(" ");
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

fn generate_points<D, R>(
    dimensions: usize,
    points: usize,
    distribution: D,
    rng: &mut R,
) -> anyhow::Result<Vec<Vec<f64>>>
where
    D: Distribution<f64>,
    R: RngCore,
{
    Ok((0..points)
        .progress_with_style(ProgressStyle::default_bar().template(PBAR_TEMPLATE)?)
        .map(|_| {
            (0..dimensions)
                .map(|_| distribution.sample(rng))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>())
}
