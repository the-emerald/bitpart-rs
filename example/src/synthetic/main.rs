use std::fs;

use bitpart::{
    builder::BitPartBuilder, metric::{euclidean::Euclidean, Metric},
};
use sisap_data::cartesian_parser::parse;

fn main() {
    let points = parse(&fs::read_to_string("../generators/output.ascii").unwrap())
        .unwrap()
        .1
        .1
        .into_iter()
        .map(|v| v.try_into().unwrap())
        .map(Euclidean::new)
        .collect::<Vec<Euclidean<[f64; 2]>>>();
    println!("read ok");

    let bitpart = BitPartBuilder::new(points.clone()).build_parallel(Some(1));

    let query = Euclidean::new([1.0, 1.0]);
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
    let brute_force = points
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
