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

    let query = Euclidean::new(Nasa([0.0; 20]));
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
