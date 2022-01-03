set shell := ["powershell.exe", "-c"]

default: serve

# serves potential
serve: build
    miniserve --index index.html ./docs/

# build potential
build:
    cargo run --release -p builder
    cargo build --release -p potential
    cargo build --release --target wasm32-unknown-unknown -p potential --features web
    wasm-bindgen --target web --no-typescript --out-dir ./docs/ ./target/wasm32-unknown-unknown/release/potential.wasm