PINNED_TOOLCHAIN := $(shell cat rust-toolchain)
prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}
build-marketplace-gen0:
	cd marketplace-gen0 && cargo build --release --target wasm32-unknown-unknown
	wasm-strip marketplace-gen0/target/wasm32-unknown-unknown/release/marketplace-gen0.wasm 2>/dev/null | true
build-marketplace-gen1:
	cd marketplace-gen1 && cargo build --release --target wasm32-unknown-unknown
	wasm-strip marketplace-gen1/target/wasm32-unknown-unknown/release/marketplace-gen1.wasm 2>/dev/null | true
build-payment-contract:
	cd payment-contract && cargo build --release --target wasm32-unknown-unknown
	wasm-strip payment-contract/target/wasm32-unknown-unknown/release/payment_contract.wasm 2>/dev/null | true
build-contracts: build-marketplace-gen0 build-marketplace-gen1 build-payment-contract
	mkdir -p target
	cp marketplace-gen0/target/wasm32-unknown-unknown/release/marketplace-gen0.wasm target/
	cp marketplace-gen1/target/wasm32-unknown-unknown/release/marketplace-gen1.wasm target/
	cp payment-contract/target/wasm32-unknown-unknown/release/payment_contract.wasm target/

clippy:
	cd marketplace-gen0 && cargo clippy --all-targets --all-features -- -D warnings
	cd marketplace-gen1 && cargo clippy --all-targets --all-features -- -D warnings
	cd payment-contract && cargo clippy --all-targets --all-features -- -D warnings



check-lint: clippy
	cd marketplace-gen0 && cargo fmt -- --check
	cd marketplace-gen1 && cargo fmt -- --check
	cd payment-contract && cargo fmt -- --check


lint: clippy
	cd marketplace-gen0 && cargo fmt
	cd marketplace-gen1 && cargo fmt
	cd payment-contract && cargo fmt
clean:
	cd marketplace-gen0 && cargo clean
	cd marketplace-gen1 && cargo clean
	cd payment-contract && cargo clean
	rm -rf target/


