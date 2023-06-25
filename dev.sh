#!/bin/sh

miniflare \
    --modules \
    --modules-rule CompiledWasm=*.wasm \
    --wasm index.wasm=./build/worker/index.wasm \
    ./build/worker/shim.mjs
