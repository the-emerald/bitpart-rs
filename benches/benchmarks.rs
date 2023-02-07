use bitpart::{
    builder::BitPartBuilder,
    metric::{euclidean::Euclidean, Metric},
};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use sisap_data::{
    cartesian_parser::parse,
    colors::{parse_colors, Colors},
    nasa::{parse_nasa, Nasa},
};
use std::{fs, time::Duration};

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

pub fn synthetic_query(c: &mut Criterion) {
    let points = parse(&fs::read_to_string("generators/output.ascii").unwrap())
        .unwrap()
        .1
         .1
        .into_iter()
        .map(|v| v.try_into().unwrap())
        .map(Euclidean::new)
        .collect::<Vec<Euclidean<[f64; 2]>>>();
    let query = Euclidean::new([1.0, 1.0]);
    let threshold = 1.0;

    let mut group = c.benchmark_group("synthetic_query");

    // Benchmark a brute force search
    group.bench_function("bruteforce", |bn| {
        bn.iter_batched(
            || points.clone(),
            |data| {
                data.into_iter()
                    .map(|pt| (pt.clone(), pt.distance(&query)))
                    .filter(|d| d.1 <= threshold)
                    .collect::<Vec<_>>()
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark query (parallel), but disable parallel queries... this lets us measure any performance lost by library overhead
    let bitpart_parallel = BitPartBuilder::new(points.clone()).build_parallel(None);
    group.bench_function("par_seq", |bn| {
        bn.iter(|| bitpart_parallel.range_search(query.clone(), threshold));
    });

    // Now benchmark query (parallel) using job sizes from 1 to 2^10.
    for sz in (0..=10).map(|x| 2_u64.pow(x)) {
        let bitpart_parallel = BitPartBuilder::new(points.clone()).build_parallel(Some(sz));
        group.bench_with_input(BenchmarkId::new("par", sz), &bitpart_parallel, |bn, x| {
            bn.iter(|| x.range_search(query.clone(), threshold));
        });
    }
}

pub fn sisap_colors_setup(c: &mut Criterion) {
    let colors = get_colors();
    let mut group = c.benchmark_group("sisap_colors_setup");

    // Benchmark setup time (sequential)
    group.bench_function("seq", |bn| {
        bn.iter_batched(
            || BitPartBuilder::new(colors.clone()),
            |data| data.build(),
            BatchSize::SmallInput,
        )
    });

    // Benchmark setup time (with parallelism)
    group.bench_function("par", |bn| {
        bn.iter_batched(
            || BitPartBuilder::new(colors.clone()),
            |data| data.build_parallel(None),
            BatchSize::SmallInput,
        )
    });
}

pub fn sisap_colors_query(c: &mut Criterion) {
    let colors = get_colors();
    let query = Euclidean::new(Colors(COLORS_QUERY));

    let mut group = c.benchmark_group("sisap_colors_query");

    // Benchmark a brute force search
    group.bench_function("bruteforce", |bn| {
        bn.iter_batched(
            || colors.clone(),
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
    let bitpart = BitPartBuilder::new(colors.clone()).build();
    group.bench_function("seq", |bn| {
        bn.iter(|| bitpart.range_search(query.clone(), COLORS_THRESHOLD));
    });

    // Benchmark query (parallel), but disable parallel queries... this lets us measure any performance lost by library overhead
    let bitpart_parallel = BitPartBuilder::new(colors.clone()).build_parallel(None);
    group.bench_function("par_seq", |bn| {
        bn.iter(|| bitpart_parallel.range_search(query.clone(), COLORS_THRESHOLD));
    });

    // Now benchmark query (parallel) using job sizes from 1 to 2^10.
    for sz in (0..=10).map(|x| 2_u64.pow(x)) {
        let bitpart_parallel = BitPartBuilder::new(colors.clone()).build_parallel(Some(sz));
        group.bench_with_input(BenchmarkId::new("par", sz), &bitpart_parallel, |bn, x| {
            bn.iter(|| x.range_search(query.clone(), COLORS_THRESHOLD));
        });
    }
}

pub fn sisap_nasa_setup(c: &mut Criterion) {
    let nasa = get_nasa();
    let mut group = c.benchmark_group("sisap_nasa_setup");

    // Benchmark setup time (sequential)
    group.bench_function("seq", |bn| {
        bn.iter_batched(
            || BitPartBuilder::new(nasa.clone()),
            |data| data.build(),
            BatchSize::SmallInput,
        )
    });

    // Benchmark setup time (with parallelism)
    group.bench_function("par", |bn| {
        bn.iter_batched(
            || BitPartBuilder::new(nasa.clone()),
            |data| data.build_parallel(None),
            BatchSize::SmallInput,
        )
    });
}

pub fn sisap_nasa_query(c: &mut Criterion) {
    let nasa = get_nasa();
    let query = Euclidean::new(Nasa(NASA_QUERY));

    let mut group = c.benchmark_group("sisap_nasa_query");

    // Benchmark a brute force search
    group.bench_function("bruteforce", |bn| {
        bn.iter_batched(
            || nasa.clone(),
            |data| {
                data.into_iter()
                    .map(|pt| (pt.clone(), pt.distance(&query)))
                    .filter(|d| d.1 <= NASA_THRESHOLD)
                    .collect::<Vec<_>>()
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark query (sequential)
    let bitpart = BitPartBuilder::new(nasa.clone()).build();
    group.bench_function("seq", |bn| {
        bn.iter(|| bitpart.range_search(query.clone(), NASA_THRESHOLD));
    });

    // Benchmark query (parallel), but disable parallel queries... this lets us measure any performance lost by library overhead
    let bitpart_parallel = BitPartBuilder::new(nasa.clone()).build_parallel(None);
    group.bench_function("par_seq", |bn| {
        bn.iter(|| bitpart_parallel.range_search(query.clone(), NASA_THRESHOLD));
    });

    // Now benchmark query (parallel) using job sizes from 1 to 2^10.
    for sz in (0..=10).map(|x| 2_u64.pow(x)) {
        let bitpart_parallel = BitPartBuilder::new(nasa.clone()).build_parallel(Some(sz));
        group.bench_with_input(BenchmarkId::new("par", sz), &bitpart_parallel, |bn, x| {
            bn.iter(|| x.range_search(query.clone(), COLORS_THRESHOLD));
        });
    }
}

// criterion_group!(benches, sisap_nasa, sisap_colors);
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(15, 0));
    targets = sisap_nasa_query, sisap_colors_query, synthetic_query
}
criterion_main!(benches);

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
