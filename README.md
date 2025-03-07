rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-unknown-linux-gnu

ldd target/x86_64-pc-windows-gnu/release/notes_app_rust
ldd target/x86_64-unknown-linux-gnu/release/notes_app_rust
