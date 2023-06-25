#!/bin/sh

miniflare \
    --kv STORAGE \
    --binding ADMIN_TOKEN="abcd" \
    --modules \
    --modules-rule CompiledWasm=*.wasm \
    --wasm index.wasm=./build/worker/index.wasm \
    ./build/worker/shim.mjs
