#!/bin/bash
set -e

VERSION="2.0.0"
COMMIT=$(git rev-parse --short HEAD)
PLATFORM="mac"
OUT_DIR="bin/$PLATFORM"

echo "🚀 Building TickAggregator (Rust) for macOS version $VERSION ($COMMIT)..."

# Clean and prepare output folder
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

# Build the Rust binary for macOS
echo "🍎 Building for macOS..."
cargo build --release

# Determine the crate name dynamically
BINARY_NAME="TickAggregator"  # or use jq if desired

# Move the compiled exe to output folder
cp "target/release/${BINARY_NAME}" "$OUT_DIR/${BINARY_NAME}-mac"

# Copy config if it exists
if [ -f "appsettings.yaml" ]; then
    cp appsettings.yaml "$OUT_DIR/"
fi

# Write version info
echo "Version: $VERSION" > "$OUT_DIR/version.txt"
echo "Commit: $COMMIT" >> "$OUT_DIR/version.txt"

# Create archive
TAR_NAME="${BINARY_NAME}-${PLATFORM}-v${VERSION}.tar.gz"
tar -czf "$TAR_NAME" -C bin "$PLATFORM"

echo "✅ macOS build complete."
echo "📁 Output in: $OUT_DIR"
echo "📦 Archive: $TAR_NAME"
