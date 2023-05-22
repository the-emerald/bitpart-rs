# bitpart-rs
An implementation of the BitPart search algorithm for metric spaces in Rust.

## Benchmarking
Generate test data by running `make-data.sh` first.

This crate uses `criterion` for benchmarking. Install `cargo-criterion` first, and then:
```
cargo criterion --all-features
```

## Examples
Generate test data first, and then pick a binary to run:
```
cargo run --release -p example --bin [name_of_example]
```