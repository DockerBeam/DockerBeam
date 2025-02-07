# Build for x86_64
cargo build --release --target x86_64-apple-darwin

# Build for aarch64
cargo build --release --target aarch64-apple-darwin

lipo -create -output target/release/universal_binary \
    target/x86_64-apple-darwin/release/your_app_name \
    target/aarch64-apple-darwin/release/your_app_name


#cargo install dmgwiz


dmgwiz --input ../ --output dockerbeam_MacOS.dmg


