#!/usr/bin/env bash
# Downloads a static FFmpeg build for macOS (Apple Silicon) and places it as the Tauri sidecar.
# Source: https://ffmpeg.martin-riedl.de/ (static arm64 release build)
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$root/src-tauri/binaries/ffmpeg-aarch64-apple-darwin"
if [ -f "$dest" ]; then echo "Already present: $dest"; exit 0; fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "Downloading FFmpeg..."
curl -fL "https://ffmpeg.martin-riedl.de/redirect/latest/macos/arm64/release/ffmpeg.zip" -o "$tmp/ffmpeg.zip"
unzip -q "$tmp/ffmpeg.zip" -d "$tmp"
cp "$tmp/ffmpeg" "$dest"
chmod +x "$dest"
xattr -d com.apple.quarantine "$dest" 2>/dev/null || true
echo "Installed: $dest"
