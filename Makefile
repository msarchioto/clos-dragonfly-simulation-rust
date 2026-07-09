.PHONY: build test sweeps all-sweeps demo refs clean

SWITCH=6400
NIC=800
LINK=200
FACTOR=2.0

build:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
	rm -rf output_clos output_dragonfly output_dragonfly_high_bw

# Sweeps
sweep-clos:
	cargo run --release --bin clos-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK)

sweep-dragonfly:
	cargo run --release --bin dragonfly-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK)

sweep-high-bw:
	cargo run --release --bin dragonfly-high-bw-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --router-budget-factor $(FACTOR)

all-sweeps: sweep-clos sweep-dragonfly sweep-high-bw

demo:
	cargo run --release --bin clos-generate -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --num-hosts 128
	cargo run --release --bin dragonfly-generate -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --num-hosts 128
	cargo run --release --bin dragonfly-high-bw-generate -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --num-hosts 128 --router-budget-factor $(FACTOR)

refs:
	mkdir -p output_clos output_dragonfly output_dragonfly_high_bw
	cargo run --release --bin clos-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --force
	cargo run --release --bin dragonfly-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --force
	cargo run --release --bin dragonfly-high-bw-sweep -- --switch-throughput $(SWITCH) --nic-throughput $(NIC) --link-bandwidth $(LINK) --router-budget-factor $(FACTOR) --force
	@echo "References generated"