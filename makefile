MAKEFLAGS +=  -B

TARGET_WASM := CARGO_TARGET_DIR=target-wasm
OUT_WASM := out-wasm
WEBELLA_DIR := ../webella/public

# helper
build-wasm:
	$(TARGET_WASM) cargo build --profile wasm-release --target wasm32-unknown-unknown -F bella_web
	wasm-bindgen --no-typescript --out-name bella --out-dir $(OUT_WASM) --target web target-wasm/wasm32-unknown-unknown/wasm-release/bella.wasm

# basic build for web
wasm: build-wasm
	yes | cp $(OUT_WASM)/bella_bg.wasm $(WEBELLA_DIR)/bella_bg.wasm
	yes | cp $(OUT_WASM)/bella.js $(WEBELLA_DIR)/bella.js

# optimized build for web
wasm-opt: build-wasm
	wasm-opt -Oz --output $(OUT_WASM)/bella_bg-optimized.wasm $(OUT_WASM)/bella_bg.wasm
	yes | cp $(OUT_WASM)/bella_bg-optimized.wasm $(WEBELLA_DIR)/bella_bg.wasm
	yes | cp $(OUT_WASM)/bella.js $(WEBELLA_DIR)/bella.js
