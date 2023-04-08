SEED=3132448493

cargo build --release -p generators
cargo build --release -p nearest-neighbours

# Make 1M points (synthetic)
echo "synthetic"
cargo run -q --release -p generators -- -d 20 -p 1000000 -s ${SEED} -o "data/output.ascii" normal --mean 0 --std-dev 1.0

# 100k, differing dimensions
for ((DIMS=10; DIMS<=30; DIMS+=2));
do
    echo "100k, d${DIMS}"
    # Make 100k flat
    cargo run -q --release -p generators -- -d ${DIMS} -p 100000 -s ${SEED} -o "data/100k_d${DIMS}_flat.ascii" flat --low "-0.5" --high "0.5"
    cargo run -q --release -p nearest-neighbours -- -i "data/100k_d${DIMS}_flat.ascii" -n 10 -o "data/100k_d${DIMS}_flat.json" -p 1000
done

# # Make 150M flat (on-disk)
# cargo run --release -p generators -- -d 20 -p 15000000 -s ${SEED} -o "data/150M_flat.ascii" flat --low "-0.5" --high "0.5"
# cargo run --release -p nearest-neighbours -- -i "data/150M_flat.ascii" -n 10 -o "data/150M_flat.json" -p 1000