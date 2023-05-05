#!/bin/bash

set -e

VERSION=$(grep -m1 "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Building version $VERSION"

#for TARGET in x86_64-pc-windows-gnu x86_64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin; do
#    echo "Building target $TARGET"
#    cargo build --release --target "$TARGET"
#done

echo "Creating git tag v$VERSION"
git tag -a "v$VERSION" -m "Release version $VERSION"
git push origin "v$VERSION"