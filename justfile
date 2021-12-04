set shell := ["powershell.exe", "-c"]

default: serve

# serves potential
serve: build
    miniserve --index index.html ./docs/

# build potential
build:
    cargo build --release --target wasm32-unknown-unknown --features web
    wasm-bindgen --target web --no-typescript --out-dir ./docs/ ./target/wasm32-unknown-unknown/release/potential.wasm