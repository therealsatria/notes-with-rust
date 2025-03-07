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

## Key Generation
openssl rand -base64 32

## sekaligus copy value kedalam file .env
openssl rand -base64 32 > .env

## contoh isi file .env
ENCRYPTION_KEY=Z2VuZXJhdGVkLWtleS1mb3Itc2VjdXJpdHktZXhhbXBsZQ==

## untuk mengamankan 
chmod 600 .env

## build standard:
cargo build --release

cp target/release/notes_app_rust /path/to/target/
cp .env /path/to/target/
cd /path/to/target/
./notes_app_rust



