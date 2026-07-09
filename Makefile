.PHONY: help build release test check fmt clippy clean clean-outputs sweeps all-sweeps demo refs pictures install

# Default parameters (matching the Python project)
SWITCH=6400
NIC=800
LINK=200
FACTOR=2.0

CARGO = cargo
RELEASE = --release

help:
	@echo "Available targets:"
	@echo "  build          - Build debug binaries"
	@echo "  release        - Build release binaries"
	@echo "  test           - Run tests"
	@echo "  check          - cargo check"
	@echo "  fmt            - Format code"
	@echo "  clippy         - Run clippy lints"
	@echo "  clean          - Clean build artifacts and outputs"
	@echo "  clean-outputs  - Remove only generated output dirs"
	@echo "  sweeps         - Run all three sweeps (debug)"
	@echo "  all-sweeps     - Alias for sweeps"
	@echo "  demo           - Run demo generates for 128 hosts"
	@echo "  refs           - Regenerate reference outputs (force)"
	@echo "  pictures       - Regenerate PNGs using Python viz on current JSONs (requires sibling Python project)"
	@echo "  install        - cargo install the binaries"

build:
	$(CARGO) build

release:
	$(CARGO) build $(RELEASE)

test:
	$(CARGO) test

check:
	$(CARGO) check

fmt:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy -- -D warnings

clean:
	$(CARGO) clean
	rm -rf output_clos output_dragonfly output_dragonfly_high_bw

clean-outputs:
	rm -rf output_clos output_dragonfly output_dragonfly_high_bw

sweeps all-sweeps:
	$(CARGO) run $(RELEASE) --bin clos-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK)
	$(CARGO) run $(RELEASE) --bin dragonfly-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK)
	$(CARGO) run $(RELEASE) --bin dragonfly-high-bw-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --router-budget-factor $(FACTOR)

demo:
	$(CARGO) run $(RELEASE) --bin clos-generate -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --num-hosts 128
	$(CARGO) run $(RELEASE) --bin dragonfly-generate -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --num-hosts 128
	$(CARGO) run $(RELEASE) --bin dragonfly-high-bw-generate -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --num-hosts 128 --router-budget-factor $(FACTOR)

refs:
	mkdir -p output_clos output_dragonfly output_dragonfly_high_bw
	$(CARGO) run $(RELEASE) --bin clos-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --force
	$(CARGO) run $(RELEASE) --bin dragonfly-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --force
	$(CARGO) run $(RELEASE) --bin dragonfly-high-bw-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --router-budget-factor $(FACTOR) --force
	@echo "Reference outputs regenerated."

# Regenerate PNG diagrams using the Python matplotlib code (best quality).
# Use `cargo build --features viz` for pure-Rust plotters backend instead.
# Assumes you are in a shell with uv and the sibling Python project exists.
pictures:
	@echo "Regenerating pictures via Python matplotlib (best quality)."
	@echo "For pure-Rust (plotters) build with: cargo build --features viz"
	@cd ../clos-dragonfly-simulation && \
	for f in ../clos-dragonfly-simulation-rust/output_clos/*.json; do \
		uv run clos-visualize "$$f" --output "../clos-dragonfly-simulation-rust/output_clos/$$(basename $$f .json).png"; \
	done && \
	for f in ../clos-dragonfly-simulation-rust/output_dragonfly/*.json; do \
		uv run dragonfly-visualize "$$f" --output "../clos-dragonfly-simulation-rust/output_dragonfly/$$(basename $$f .json).png"; \
	done && \
	for f in ../clos-dragonfly-simulation-rust/output_dragonfly_high_bw/*.json; do \
		uv run dragonfly-visualize "$$f" --output "../clos-dragonfly-simulation-rust/output_dragonfly_high_bw/$$(basename $$f .json).png"; \
	done
	@echo "Pictures regenerated."

install:
	$(CARGO) install --path . --bins
