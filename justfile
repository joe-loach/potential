set shell := ["powershell.exe", "-c"]

serve:
    cargo build-web potential --release --features web
    miniserve --index index.html ./docs/

debug:
    cargo run -p potential

run:
    cargo run --release -p potential
