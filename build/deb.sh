#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Define the target architectures
targets=(
    "i686-unknown-linux-gnu"        # 32-bit x86
    "x86_64-unknown-linux-gnu"      # 64-bit x86_64
    "armv7-unknown-linux-gnueabihf" # 32-bit ARM
    "aarch64-unknown-linux-gnu"     # 64-bit ARM
)

# Update Rust to the latest stable version
rustup update stable

# Install cargo-deb if it's not already installed
if ! command -v cargo-deb &> /dev/null; then
    cargo install cargo-deb
fi

# Add the necessary target architectures
for target in "${targets[@]}"; do
    rustup target add "$target"
done


# Build and package the project for each target
for target in "${targets[@]}"; do
    echo "Building and packaging for target: $target"
    cargo build --release --target "$target"
    cargo deb --no-build --target "$target"
done

