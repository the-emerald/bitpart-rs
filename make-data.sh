SEED=3132448493

# Make 1M points (synthetic)
cargo run --release -p generators -- -d 20 -p 1000000 -s ${SEED} -o "data/output.ascii" normal --mean 0 --std-dev 1.0

# Make 100k flat
cargo run --release -p generators -- -d 20 -p 100000 -s ${SEED} -o "data/100k_flat.ascii" flat --low "-1.0" --high "1.0"
cargo run --release -p nearest-neighbours -- -i "data/100k_flat.ascii" -n 10 -o "data/100k_flat.json"