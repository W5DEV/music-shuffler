#!/bin/bash

# Windows Cross-Compilation Script for macOS/Linux
# Run this to build Windows executable locally

set -e

echo "🎵 Building Music Shuffler for Windows..."

# Check if Windows target is installed
if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "📦 Installing Windows target..."
    rustup target add x86_64-pc-windows-gnu
fi

# Install mingw-w64 if on macOS and not already installed
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo "📦 Installing mingw-w64 via Homebrew..."
        brew install mingw-w64
    fi
fi

# Build for Windows
echo "🔨 Building release binary..."
cargo build --release --target x86_64-pc-windows-gnu

# Create release directory
echo "📁 Creating release package..."
rm -rf release-windows
mkdir -p release-windows

# Copy executable
cp target/x86_64-pc-windows-gnu/release/music-shuffler.exe release-windows/

# Create README
cat > release-windows/README.txt << 'EOF'
# Music Shuffler for Windows

A fast, lightweight music player that creates random playlists from your music library.

## Quick Start:
1. Run music-shuffler.exe
2. Click "Select Directory" and choose your music folder
3. Click "Generate Playlist" 
4. Enjoy your music!

## Features:
- Supports MP3, FLAC, OGG, WAV, M4A, AAC
- Fast parallel scanning for large libraries
- Intelligent metadata caching
- Album art display
- Simple, clean interface

## Troubleshooting:

### No Audio:
- Install Visual C++ Redistributables from Microsoft
- Check Windows audio drivers
- Try different audio formats

### Slow Performance:
- First scan takes time (builds cache for future)
- Subsequent launches are instant
- Close other audio applications

### File Errors:
- Some MP3 files may be corrupted
- Try re-encoding problematic files with modern tools
- Use VLC or Audacity to convert

## System Requirements:
- Windows 10 or later
- 100MB free disk space
- Audio output device

Built with Rust for maximum performance and reliability.
EOF

# Create ZIP file
echo "📦 Creating ZIP package..."
cd release-windows
zip -r ../music-shuffler-windows.zip .
cd ..

echo ""
echo "✅ Windows build complete!"
echo "📁 Files created:"
echo "   - release-windows/music-shuffler.exe"
echo "   - release-windows/README.txt" 
echo "   - music-shuffler-windows.zip"
echo ""
echo "🚀 Ready for Windows deployment!"

# Show file sizes
echo "📊 Package size:"
ls -lh music-shuffler-windows.zip 