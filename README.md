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

The Rust tools produce the JSON topology files. To generate the high-quality PNG diagrams (layered for CLOS, grouped for Dragonfly, matching the original exactly):

```bash
# After generating a JSON with Rust:
cd ../clos-dragonfly-simulation
uv run clos-visualize ../clos-dragonfly-simulation-rust/output_clos/topo_128.json --output ../clos-dragonfly-simulation-rust/output_clos/topo_128.png

uv run dragonfly-visualize ../clos-dragonfly-simulation-rust/output_dragonfly/dragonfly_64.json --output ../clos-dragonfly-simulation-rust/output_dragonfly/dragonfly_64.png
```

The `*-visualize` Rust binaries also delegate to the Python matplotlib scripts automatically (requires the sibling Python project with `uv`).

## Status

- Full CLOS, Dragonfly, and High-BW generators + sweeps (JSON matches Python references for CLOS; correct structure otherwise).
- All CLIs implemented.
- Visualization reverts to Python's matplotlib for best quality (identical pictures).
- Outputs include full set of JSON + PNG for 4/8/16/32/64 (+128 where applicable).

See the Python project for algorithm details and math.

## Example

```bash
./target/release/clos-generate --switch-throughput 6400 --nic-throughput 800 --link-bandwidth 200 --num-hosts 128
# Then generate picture with Python as above.
```
