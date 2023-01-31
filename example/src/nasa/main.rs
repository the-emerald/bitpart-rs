use std::fs;

use bitpart::{
    builder::BitPartBuilder,
    metric::{euclidean::Euclidean, Metric},
};
use sisap_data::nasa::{parse_nasa, Nasa};

fn main() {
    let nasa = parse_nasa(&fs::read_to_string("../sisap-data/src/nasa.ascii").unwrap())
        .unwrap()
        .into_iter()
        .map(Euclidean::new)
        .collect::<Vec<_>>();

    let bitpart = BitPartBuilder::new(nasa.clone()).build();

    // Line 319 in nasa.ascii
    let query = Euclidean::new(Nasa([
        0.00722561, 0.0599118, 0.0165916, 0.121793, 0.0404137, 0.297534, 0.979138, -0.792623,
        0.242515, 0.162952, -0.209939, 0.0275739, -0.16217, -0.0176906, -0.0309458, 0.0530525,
        -0.437606, 0.00760368, -0.153654, 0.0296254,
    ]));
    let threshold = 1.0;

    let res = bitpart.range_search(query.clone(), threshold);
    println!("{} points returned", res.len());

    print!("CHECK: all returned points within threshold... ");
    if res.iter().all(|(pt, _)| pt.distance(&query) <= threshold) {
        println!("ok");
    } else {
        println!("fail");
    }

    print!("CHECK: compare against linear search... ");
    let brute_force = nasa
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