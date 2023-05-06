#!/bin/bash

set -e

VERSION=$(grep -m1 "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Building version $VERSION"

#x86_64-apple-darwin aarch64-apple-darwin

mkdir -p "./release/$VERSION"
for TARGET in x86_64-pc-windows-gnu x86_64-unknown-linux-gnu; do
    echo "Building target $TARGET"
    cargo build --release --target "$TARGET"
    if [[ "$TARGET" == "x86_64-pc-windows-gnu" ]]; then
        cp target/$TARGET/release/krunch.exe "./release/$VERSION/krunch-windows-amd64.exe"
    else
        cp target/$TARGET/release/krunch "./release/$VERSION/krunch-linux-amd64"
    fi
done

echo "Creating git tag v$VERSION"
git tag -a "v$VERSION" -m "Release version $VERSION"
git push origin "v$VERSION"