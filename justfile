install-tools:
    cargo install wasm-pack --locked

build-wasm:
    wasm-pack build --target web -- --no-default-features --features "wasm"
    cp html/*.* pkg/