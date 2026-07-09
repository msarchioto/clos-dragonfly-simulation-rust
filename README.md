# clos-dragonfly-simulation-rust

Rust port of the CLOS & Dragonfly topology generators.

This mirrors the Python and Go ports. Core logic and CLIs are in pure Rust. For visualization (to get pictures identical to the original Python matplotlib implementation), we delegate to the Python scripts.

## Build & Run

```bash
cargo build --release
./target/release/clos-generate --switch-throughput 6400 --nic-throughput 800 --link-bandwidth 200 --num-hosts 128
cargo run --release --bin clos-sweep -- --switch-throughput 6400 --nic-throughput 800 --link-bandwidth 200
```

## Visualization

The Rust tools always produce JSON topology files.

### Default (recommended for quality)
High-quality PNGs use the sibling Python project (matplotlib):

```bash
# After `cargo run --bin clos-generate ...`
cd ../clos-dragonfly-simulation
uv run clos-visualize ../clos-dragonfly-simulation-rust/output_clos/topo_128.json \
  --output ../clos-dragonfly-simulation-rust/output_clos/topo_128.png
```

The `*-visualize` binaries do this automatically (they spawn the Python command).

### Optional pure-Rust viz (using plotters)

Build with the `viz` Cargo feature for a **pure-Rust** drawing backend (no Python required at runtime):

```bash
cargo build --release --features viz
./target/release/clos-visualize output_clos/topo_32.json --output /tmp/clos.png
./target/release/dragonfly-visualize output_dragonfly/dragonfly_64.json --output /tmp/df.png
```

In your own `Cargo.toml`:

```toml
[dependencies]
clos-dragonfly-simulation-rust = { version = "0.1", features = ["viz"] }
```

**Note**: When the `viz` feature is enabled, `generate` and `sweep` commands will also try to emit PNGs using the pure-Rust backend (in addition to JSON).

The pure-Rust drawings use similar layered/circular layouts and colors to the original, but are not yet as polished as matplotlib. Use the Python path for publication-quality images.

### Makefile helpers
```bash
make pictures   # always uses Python matplotlib (high quality)
make refs       # regenerate JSONs
```

## Status

- Full CLOS, Dragonfly, and High-BW generators + sweeps (JSON matches Python references for CLOS; correct structure otherwise).
- All CLIs implemented.
- Visualization: defaults to Python matplotlib (best quality). Optional pure-Rust backend via `viz` feature (using `plotters`).
- 10+ unit tests.
- Proper Makefile (including `pictures` target).
- See "Visualization" section below for the `viz` feature.

See the Python project for algorithm details and math.

See the Python project for algorithm details and math.

## Example

```bash
./target/release/clos-generate --switch-throughput 6400 --nic-throughput 800 --link-bandwidth 200 --num-hosts 128
# Then generate picture with Python as above.
```
