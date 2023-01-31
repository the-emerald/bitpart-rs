use std::fs;

use bitpart::{
    builder::BitPartBuilder,
    metric::{euclidean::Euclidean, Metric},
};
use sisap_data::colors::{parse_colors, Colors};

fn main() {
    let colors = parse_colors(&fs::read_to_string("../sisap-data/src/colors.ascii").unwrap())
        .unwrap()
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>();

    let bitpart = BitPartBuilder::new(colors.clone()).build();

    // Line 319 in nasa.ascii
    let query = Euclidean::new(Colors(QUERY));
    let threshold = 0.5;

    let res = bitpart.range_search(query.clone(), threshold);
    println!("{} points returned", res.len());

    print!("CHECK: all returned points within threshold... ");
    if res.iter().all(|(pt, _)| pt.distance(&query) <= threshold) {
        println!("ok");
    } else {
        println!("fail");
    }

    print!("CHECK: compare against linear search... ");
    let brute_force = colors
        .into_iter()
        .map(|pt| pt.distance(&query))
        .filter(|d| *d < threshold)
        .count();
    if brute_force != res.len() {
        println!(
            "fail. brute force search returned {} results, but bitpart returned {}",
            brute_force,
            res.len()
        );
    } else {
        println!("ok")
    }
}

const QUERY: [f64; 112] = [
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
