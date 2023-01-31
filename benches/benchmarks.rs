use bitpart::{builder::BitPartBuilder, metric::euclidean::Euclidean};
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use sisap_data::{
    colors::{parse_colors, Colors},
    nasa::{parse_nasa, Nasa},
};
use std::fs;

pub fn sisap_colors(c: &mut Criterion) {
    let colors = parse_colors(&fs::read_to_string("sisap-data/src/colors.ascii").unwrap())
        .unwrap()
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("SISAP Colors");

    group.bench_function("Setup", |bn| {
        bn.iter_batched(
            || BitPartBuilder::new(colors.clone()),
            |data| data.build(),
            BatchSize::SmallInput,
        )
    });

    let bitpart = BitPartBuilder::new(colors).build();
    let query = Euclidean::new(Colors(COLORS_QUERY));

    group.bench_function("Query", |bn| {
        bn.iter(|| bitpart.range_search(query.clone(), COLORS_THRESHOLD));
    });
}

pub fn sisap_nasa(c: &mut Criterion) {
    let nasa = parse_nasa(&fs::read_to_string("sisap-data/src/nasa.ascii").unwrap())
        .unwrap()
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("SISAP NASA");

    group.bench_function("Setup", |bn| {
        bn.iter_batched(
            || BitPartBuilder::new(nasa.clone()),
            |data| data.build(),
            BatchSize::SmallInput,
        )
    });

    let bitpart = BitPartBuilder::new(nasa).build();
    let query = Euclidean::new(Nasa(NASA_QUERY));

    group.bench_function("Query", |bn| {
        bn.iter(|| bitpart.range_search(query.clone(), NASA_THRESHOLD));
    });
}

criterion_group!(benches, sisap_nasa, sisap_colors);
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
