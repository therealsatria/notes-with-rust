cargo clean
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
ldd target/x86_64-unknown-linux-gnu/release/notes_app_rust