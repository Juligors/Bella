TARGET_WASM := CARGO_TARGET_DIR=target-wasm
OUT_WASM := out-wasm

wasm:
	$(TARGET_WASM) cargo build --profile wasm-release --target wasm32-unknown-unknown
	wasm-bindgen --no-typescript --out-name bella --out-dir $(OUT_WASM) --target web target-wasm/wasm32-unknown-unknown/wasm-release/bella.wasm

wasm-opt: wasm
	wasm-opt -Oz --output $(OUT_WASM)/bella_bg-optimized.wasm $(OUT_WASM)/bella_bg.wasm


# .PHONY: wasm, wasm-opt
