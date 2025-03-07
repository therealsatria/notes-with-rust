rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-unknown-linux-gnu

ldd target/x86_64-pc-windows-gnu/release/notes_app_rust
ldd target/x86_64-unknown-linux-gnu/release/notes_app_rust

untuk generate key
openssl rand -base64 32

lalu copy kedalam .env 
misalnya 
ENCRYPTION_KEY=Z2VuZXJhdGVkLWtleS1mb3Itc2VjdXJpdHktZXhhbXBsZQ==

untuk mengamankan 
chmod 600 .env

build standard:
cargo build --release

cp target/release/notes_app_rust /path/to/target/
cp .env /path/to/target/
cd /path/to/target/
./notes_app_rust



