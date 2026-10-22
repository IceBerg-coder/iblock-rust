.PHONY: all build test wasm extension clean fmt lint

all: wasm extension

build:
	cargo build --workspace --release

test:
	cargo test --workspace
	node tests/test-wasm.mjs

wasm:
	cargo build -p iblock-wasm --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/iblock_wasm.wasm \
		--out-dir dist/wasm \
		--target web
	mkdir -p extension/wasm
	cp dist/wasm/* extension/wasm/

extension: wasm
	mkdir -p dist
	cd extension && rm -f ../dist/iblock-extension.zip && zip -qr ../dist/iblock-extension.zip . && cd ..
	@echo "Package ready: dist/iblock-extension.zip"

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

clean:
	cargo clean
	rm -rf dist/iblock-extension.zip
