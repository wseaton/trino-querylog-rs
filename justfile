set dotenv-load := true

build:
    cargo build --release && cp $CARGO_TARGET_DIR/release/libtrino_querylog_rs.so libtrino_querylog_rs.so
    podman build -t trino-querylog-rs .