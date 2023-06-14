prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true

test: build-contract
	mkdir -p tests/wasm
	cp target/wasm32-unknown-unknown/release/contract.wasm tests/wasm
	cd tests && cargo test -- --nocapture

clippy:
	cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cargo fmt -- --check

lint: clippy
	cargo fmt

clean:
	cargo clean
	rm -rf tests/wasm
