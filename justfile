install-tools:
    cargo install wasm-pack
    
build-wasm:
    wasm-pack build --target web -- --no-default-features --features "wasm"
    cp html/*.* pkg/