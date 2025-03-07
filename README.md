# Building and Running the Notes App (Rust)

This document outlines the steps to build and run the Notes App written in Rust, including cross-compilation for Windows and Linux, key generation, and deployment.

## Cross-Compilation Targets

First, you need to add the necessary targets for cross-compilation using `rustup`:

```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-unknown-linux-gnu

ldd target/x86_64-pc-windows-gnu/release/notes_app_rust
ldd target/x86_64-unknown-linux-gnu/release/notes_app_rust
