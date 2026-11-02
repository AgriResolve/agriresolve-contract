.PHONY: all build build-wasm test fmt fmt-check clippy clean

all: build test

build:
	cargo build

build-wasm:
	SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2=1 cargo build --target wasm32v1-none --release

test:
	cargo test

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --workspace -- -D warnings

clean:
	cargo clean
