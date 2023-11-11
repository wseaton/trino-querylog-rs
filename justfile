set dotenv-load := true

build:
    gradle shadow
    cargo build --release && cp $CARGO_TARGET_DIR/release/libtrino_querylog_rs.so libtrino_querylog_rs.so
    podman build -t trino-querylog-rs .

run: build
    podman run -e RUST_BACKTRACE=1 -p 8080:8080 -it localhost/trino-querylog-rs:latest