.PHONY: all build test run-sequencer run-full-node clean

all: build

build:
	cargo build --release

test:
	cargo test

run-sequencer: build
	./target/release/community-coin

run-full-node: build
	./target/release/full-node

clean:
	cargo clean
