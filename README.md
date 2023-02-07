# bitpart-rs
An implementation of the BitPart search algorithm for metric spaces in Rust.

## Benchmarking
Note: The `synthetic` benchmark is enabled by default, but the files are too large to be stored in version control.
Either disable it by commenting it out, or generate a fresh set by:
```
cd generators && cargo run --release
```

This crate uses `criterion` for benchmarking. Install `cargo-criterion` first, and then:
```
cargo criterion --all-features
```

## Examples
Examples are available in the `examples` directory. To run them:
```
cargo run --release --bin [name_of_example]
```