use std::fs;

use bitpart::{
    builder::BitPartBuilder,
    metric::{euclidean::Euclidean, Metric},
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
        .collect::<Vec<Euclidean<[f64; 20]>>>();
    println!("read ok");

    let bitpart = BitPartBuilder::new(points.clone()).build_parallel(Some(512));

    let query = Euclidean::new([
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
    ]);
    let threshold = 3.0;

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
