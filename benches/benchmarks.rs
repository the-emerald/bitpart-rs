use bitpart::{
    metric::{Euclidean, Metric},
    BitPart, Builder,
};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rayon::prelude::*;
use sisap_data::{
    cartesian_parser::parse,
    colors::{parse_colors, Colors},
    nasa::{parse_nasa, Nasa},
};
use std::{fs, time::Duration};

/// Benchmark setup times for a particular dataset.
fn setup_with<T>(c: &mut Criterion, group_name: String, builder: Builder<T>)
where
    for<'a> T: Metric + Send + Sync + 'a,
{
    let mut group = c.benchmark_group(group_name);

    // Benchmark setup time (sequential)
    group.bench_function("seq", |bn| {
        bn.iter_batched(
            || builder.clone(),
            |data| data.build(),
            BatchSize::SmallInput,
        )
    });

    // Benchmark setup time (with parallelism)
    group.bench_function("par", |bn| {
        bn.iter_batched(
            || builder.clone(),
            |data| data.build_parallel(Some(8192)),
            BatchSize::SmallInput,
        )
    });
}

/// Benchmark query times for a particular dataset, with query and threshold.
fn query_with<T>(
    c: &mut Criterion,
    group_name: String,
    dataset: Vec<T>,
    builder: Builder<T>,
    query: T,
    threshold: f64,
) where
    for<'a> T: Metric + Send + Sync + 'a,
{
    let mut group = c.benchmark_group(group_name);

    // Benchmark a brute force search
    group.bench_function("bruteforce", |bn| {
        bn.iter_batched(
            || dataset.clone(),
            |data| {
                data.into_iter()
                    .map(|pt| (pt.clone(), pt.distance(&query)))
                    .filter(|d| d.1 <= COLORS_THRESHOLD)
                    .collect::<Vec<_>>()
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark query (sequential)
    let bitpart = builder.clone().build();
    group.bench_function("seq", |bn| {
        bn.iter(|| bitpart.range_search(query.clone(), threshold));
    });

    // Benchmark query (parallel)
    let bitpart_parallel = builder.clone().build_parallel(Some(8192));
    group.bench_function("par", |bn| {
        bn.iter(|| bitpart_parallel.range_search(query.clone(), threshold));
    });
}

fn get_colors() -> Vec<Euclidean<Colors>> {
    parse_colors(&fs::read_to_string("sisap-data/src/colors.ascii").unwrap())
        .unwrap()
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>()
}

fn get_nasa() -> Vec<Euclidean<Nasa>> {
    parse_nasa(&fs::read_to_string("sisap-data/src/nasa.ascii").unwrap())
        .unwrap()
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>()
}

fn get_synthetic() -> Vec<Euclidean<[f64; 20]>> {
    parse(&fs::read_to_string("data/output.ascii").unwrap())
        .unwrap()
        .1
         .1
        .into_iter()
        .map(|v| v.try_into().unwrap())
        .map(Euclidean::new)
        .collect::<Vec<Euclidean<[f64; 20]>>>()
}

pub fn synthetic_setup(c: &mut Criterion) {
    let points = get_synthetic();
    let builder = Builder::new(points, 40);

    setup_with(c, "synthetic_setup".to_owned(), builder);
}

pub fn synthetic_query(c: &mut Criterion) {
    let points = get_synthetic();

    let query = Euclidean::new(SYNTHETIC_QUERY);

    let builder = Builder::new(points.clone(), 40);

    query_with(
        c,
        "synthetic_query".to_owned(),
        points.clone(),
        builder,
        query.clone(),
        SYNTHETIC_THRESHOLD,
    );

    let builder = Builder::new(points.clone(), 20);
    query_with(
        c,
        "synthetic_query_20".to_owned(),
        points.clone(),
        builder,
        query,
        SYNTHETIC_THRESHOLD,
    );
}

pub fn sisap_colors_setup(c: &mut Criterion) {
    let colors = get_colors();
    let builder = Builder::new(colors, 40);

    setup_with(c, "sisap_colors_setup".to_owned(), builder);
}

pub fn sisap_colors_query(c: &mut Criterion) {
    let colors = get_colors();
    let query = Euclidean::new(Colors(COLORS_QUERY));

    let builder = Builder::new(colors.clone(), 40);

    query_with(
        c,
        "sisap_colors_query".to_owned(),
        colors.clone(),
        builder,
        query.clone(),
        COLORS_THRESHOLD,
    );

    let builder = Builder::new(colors.clone(), 20);
    query_with(
        c,
        "sisap_colors_query_20".to_owned(),
        colors,
        builder,
        query,
        COLORS_THRESHOLD,
    );
}

pub fn sisap_nasa_setup(c: &mut Criterion) {
    let nasa = get_nasa();
    let builder = Builder::new(nasa, 40);

    setup_with(c, "sisap_colors_setup".to_owned(), builder);
}

pub fn sisap_nasa_query(c: &mut Criterion) {
    let nasa = get_nasa();
    let query = Euclidean::new(Nasa(NASA_QUERY));

    let builder = Builder::new(nasa.clone(), 40);

    query_with(
        c,
        "sisap_nasa_query".to_owned(),
        nasa.clone(),
        builder,
        query.clone(),
        NASA_THRESHOLD,
    );

    let builder = Builder::new(nasa.clone(), 20);
    query_with(
        c,
        "sisap_nasa_query_20".to_owned(),
        nasa,
        builder,
        query,
        NASA_THRESHOLD,
    );
}

pub fn nn_query_inner<T>(
    c: &mut Criterion,
    group_name: String,
    n: usize,
    builder: Builder<T>,
    points: Vec<T>,
    thresholds: Vec<f64>,
) where
    for<'a> T: Metric + Send + Sync + 'a,
{
    let mut group = c.benchmark_group(group_name);

    // Benchmark a brute force search
    group.bench_function("lin", |bn| {
        bn.iter_batched(
            || (points.clone(), thresholds.clone()),
            |(points_inner, thresholds_inner)| {
                for (query, threshold) in points_inner
                    .into_iter()
                    .zip(thresholds_inner.into_iter())
                    .take(n)
                {
                    let _ = points
                        .iter()
                        .map(|pt| (pt.clone(), pt.distance(&query)))
                        .filter(|d| d.1 <= threshold)
                        .collect::<Vec<_>>();
                }
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark a parallel brute force search
    group.bench_function("lin_par", |bn| {
        bn.iter_batched(
            || (points.clone(), thresholds.clone()),
            |(points_inner, thresholds_inner)| {
                for (query, threshold) in points_inner
                    .into_iter()
                    .zip(thresholds_inner.into_iter())
                    .take(n)
                {
                    let _ = points
                        .par_iter()
                        .map(|pt| (pt.clone(), pt.distance(&query)))
                        .filter(|d| d.1 <= threshold)
                        .collect::<Vec<_>>();
                }
            },
            BatchSize::SmallInput,
        )
    });

    let bitpart_seq = builder.clone().build();
    group.bench_function("seq", |bn| {
        bn.iter(|| {
            for (query, threshold) in points.iter().zip(thresholds.iter()).take(n) {
                bitpart_seq.range_search(query.clone(), *threshold);
            }
        });
    });

    // Benchmark query (parallel)
    let bitpart_parallel = builder.clone().build_parallel(Some(8192));
    group.bench_function("par", |bn| {
        bn.iter(|| {
            for (query, threshold) in points.iter().zip(thresholds.iter()).take(n) {
                bitpart_parallel.range_search(query.clone(), *threshold);
            }
        });
    });

    for cull_threshold in [0.95, 0.9, 0.85, 0.8] {
        // Cull by popcnt
        let mut bitpart_cull = builder.clone().build_parallel(Some(8192));
        bitpart_cull.cull_by_popcnt(cull_threshold);
        group.bench_function(BenchmarkId::new("cull_pop", cull_threshold), |bn| {
            bn.iter(|| {
                for (query, threshold) in points.iter().zip(thresholds.iter()).take(n) {
                    bitpart_cull.range_search(query.clone(), *threshold);
                }
            });
        });

        // Cull by similarity
        let mut bitpart_cull = builder.clone().build_parallel(Some(8192));
        bitpart_cull.cull_by_similarity(cull_threshold);
        group.bench_function(BenchmarkId::new("cull_sim", cull_threshold), |bn| {
            bn.iter(|| {
                for (query, threshold) in points.iter().zip(thresholds.iter()).take(n) {
                    bitpart_cull.range_search(query.clone(), *threshold);
                }
            });
        });

        // Cull both
        let mut bitpart_cull = builder.clone().build_parallel(Some(8192));
        bitpart_cull.cull_by_popcnt(cull_threshold);
        bitpart_cull.cull_by_similarity(cull_threshold);
        group.bench_function(BenchmarkId::new("cull_all", cull_threshold), |bn| {
            bn.iter(|| {
                for (query, threshold) in points.iter().zip(thresholds.iter()).take(n) {
                    bitpart_cull.range_search(query.clone(), *threshold);
                }
            });
        });
    }

    std::fs::remove_dir_all("/tmp/benchmark/").ok();
    // Benchmark on-disk
    let bitpart_parallel = builder.clone().build_on_disk("/tmp/benchmark/", Some(8192));
    group.bench_function("disk", |bn| {
        bn.iter(|| {
            for (query, threshold) in points.iter().zip(thresholds.iter()).take(n) {
                bitpart_parallel.range_search(query.clone(), *threshold);
            }
        });
    });
    std::fs::remove_dir_all("/tmp/benchmark/").ok();
}

pub fn nn_query(c: &mut Criterion) {
    for dims in (10..=30).step_by(2) {
        let points = parse(&fs::read_to_string(format!("data/100k_d{dims}_flat.ascii")).unwrap())
            .unwrap()
            .1
             .1
            .into_iter()
            .map(Euclidean::new)
            .collect::<Vec<_>>();

        let thresholds = serde_json::from_str::<Vec<Vec<(usize, f64)>>>(
            &fs::read_to_string(format!("data/100k_d{dims}_flat.json")).unwrap(),
        )
        .unwrap()
        .into_iter()
        .map(|nn| nn.last().unwrap().1)
        .collect::<Vec<_>>();

        let builder = Builder::new(points.clone(), 40);

        nn_query_inner(
            c,
            format!("100k_d{dims}_flat"),
            500,
            builder,
            points,
            thresholds,
        );
    }
}

// criterion_group!(benches, sisap_nasa, sisap_colors);
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(60, 0));
    targets = sisap_nasa_query, sisap_colors_query, synthetic_query
}

criterion_group! {
    name = nn_benches;
    config = Criterion::default().measurement_time(Duration::new(120, 0));
    targets = nn_query
}

// criterion_main!(benches, nn_benches);
criterion_main!(nn_benches);

const NASA_THRESHOLD: f64 = 1.0;

const NASA_QUERY: [f64; 20] = [
    0.00722561, 0.0599118, 0.0165916, 0.121793, 0.0404137, 0.297534, 0.979138, -0.792623, 0.242515,
    0.162952, -0.209939, 0.0275739, -0.16217, -0.0176906, -0.0309458, 0.0530525, -0.437606,
    0.00760368, -0.153654, 0.0296254,
];

const COLORS_THRESHOLD: f64 = 0.5;

const COLORS_QUERY: [f64; 112] = [
    0.057581,
    0.0228588,
    0.0280671,
    0.0461878,
    0.0,
    0.000253183,
    0.00423177,
    0.000506366,
    0.00155527,
    0.00238715,
    0.0,
    0.0212312,
    0.00947627,
    0.00495515,
    0.00712529,
    0.00802951,
    0.0,
    0.0937862,
    0.0186994,
    0.039388,
    0.0152633,
    0.0,
    0.000289352,
    0.0633319,
    0.0265842,
    0.0712167,
    0.0341435,
    0.0198929,
    0.0,
    0.000217014,
    0.000325521,
    0.000868056,
    0.0016276,
    0.0,
    0.0,
    0.0,
    0.000470197,
    0.0,
    0.00173611,
    0.00072338,
    0.0,
    0.0,
    0.00365307,
    0.0,
    0.00227865,
    0.0181207,
    0.0,
    0.0,
    0.0,
    0.0,
    0.000542535,
    0.00169994,
    0.0,
    0.0304181,
    0.00166377,
    0.0,
    0.0,
    0.0,
    0.0031467,
    0.0,
    0.0,
    0.049443,
    0.0242332,
    0.013057,
    0.0,
    0.0,
    0.0,
    0.0164931,
    0.0269459,
    0.0,
    0.0,
    0.0,
    0.0300203,
    0.0461516,
    0.0659722,
    0.0,
    3.6169e-05,
    0.0,
    0.0,
    0.000253183,
    0.0163845,
    0.03852,
    0.0,
    0.0,
    0.0,
    0.0,
    0.000108507,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.000651042,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.00470197,
    0.000108507,
    0.0,
    0.0,
    7.2338e-05,
    0.0,
    0.000144676,
    0.0,
    0.0,
    0.000180845,
    0.00133825,
    0.0,
    0.000651042,
    0.0,
];

const SYNTHETIC_THRESHOLD: f64 = 3.0;

const SYNTHETIC_QUERY: [f64; 20] = [
    -1.087991147654979,
    0.4045582471357857,
    -0.9259290219334685,
    1.5862709369979888,
    1.6644108467594723,
    -0.7515492023423321,
    -1.31650770460433,
    1.222645925453442,
    -0.2379306470307699,
    1.380453153401442,
    -0.6375512992790882,
    -0.0625774616217966,
    -0.34047167632557473,
    -0.23828855469139995,
    -1.1329267432810688,
    0.015545842628269484,
    -0.39737937291629055,
    0.3352322337712804,
    -0.6905092989551525,
    1.6185724453054442,
];
