SEED=3132448493

# Make 1M points (synthetic)
cargo run --release -p generators -- -d 20 -p 1000000 -s ${SEED} -o "data/output.ascii" normal --mean 0 --std-dev 1.0

# Make 100k flat
cargo run --release -p generators -- -d 20 -p 100000 -s ${SEED} -o "data/100k_flat.ascii" flat --low "-0.5" --high "0.5"
cargo run --release -p nearest-neighbours -- -i "data/100k_flat.ascii" -n 10 -o "data/100k_flat.json" -p 1000

# # Make 150M flat (on-disk)
# cargo run --release -p generators -- -d 20 -p 15000000 -s ${SEED} -o "data/150M_flat.ascii" flat --low "-0.5" --high "0.5"
# cargo run --release -p nearest-neighbours -- -i "data/150M_flat.ascii" -n 10 -o "data/150M_flat.json" -p 1000