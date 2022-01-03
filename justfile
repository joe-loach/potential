set shell := ["powershell.exe", "-c"]

default: run

# serves potential
serve: build
    miniserve --index index.html ./docs/

run: build
    cargo run --release -p potential

debug: build
    cargo run -p potential

# build potential
build:
    cargo run --release -p builder
    cargo build --release -p potential
    cargo build --release --target wasm32-unknown-unknown -p potential --features web
    wasm-bindgen --target web --no-typescript --out-dir ./docs/ ./target/wasm32-unknown-unknown/release/potential.wasm