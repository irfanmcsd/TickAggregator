#!/bin/bash
set -e

VERSION="2.0.0"
COMMIT=$(git rev-parse --short HEAD)
PLATFORM="windows"
OUT_DIR="bin/$PLATFORM"

echo "🚀 Building TickAggregator (Rust) for Windows version $VERSION ($COMMIT)..."

# Clean and prepare output folder
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

# Detect windres binary
if command -v windres >/dev/null 2>&1; then
    WINDRES="windres"
elif command -v x86_64-w64-mingw32-windres >/dev/null 2>&1; then
    WINDRES="x86_64-w64-mingw32-windres"
else
    WINDRES=""
    echo "⚠️  windres not found — will use Rust winres crate to embed icon"
fi

# Compile icon if windres is available
if [ -n "$WINDRES" ]; then
    if [ ! -f "resources/icon.rc" ]; then
        echo "❌ Missing resources/icon.rc — please add it before building."
        exit 1
    fi
    echo "🖼  Compiling Windows icon via $WINDRES..."
    $WINDRES resources/icon.rc -O coff -o resources/icon.res
fi

# Ensure Windows target is installed
echo "🔧 Checking Rust target..."
rustup target add x86_64-pc-windows-gnu >/dev/null 2>&1

# Build the Rust binary for Windows
echo "🪟 Building for Windows..."
RUSTFLAGS="-C link-args=-mwindows" \
cargo build --release --target x86_64-pc-windows-gnu

# Determine the crate name dynamically
BINARY_NAME=$(cargo metadata --format-version 1 --no-deps \
    | jq -r '.packages[0].targets[0].name')

# Move the compiled exe to output folder
cp "target/x86_64-pc-windows-gnu/release/${BINARY_NAME}.exe" "$OUT_DIR/${BINARY_NAME}-win.exe"

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

echo "✅ Windows build complete."
echo "📁 Output in: $OUT_DIR"
echo "📦 Archive: $TAR_NAME"
