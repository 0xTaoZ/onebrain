#!/bin/bash
# Install the latest OneBrain release into /Applications.
# Usage: curl -fsSL https://raw.githubusercontent.com/0xTaoZ/onebrain/main/install.sh | bash
set -euo pipefail

REPO="0xTaoZ/onebrain"
APP="OneBrain.app"

if [[ "$(uname -s)" != "Darwin" || "$(uname -m)" != "arm64" ]]; then
  echo "OneBrain currently runs on Macs with Apple Silicon (M1 or later)." >&2
  exit 1
fi

echo "Finding the latest release..."
DMG_URL=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep -o '"browser_download_url": *"[^"]*\.dmg"' | head -1 | sed 's/.*"\(https[^"]*\)"/\1/')
if [[ -z "$DMG_URL" ]]; then
  echo "Could not find a macOS installer in the latest release." >&2
  exit 1
fi

WORK=$(mktemp -d)
MOUNT="$WORK/mount"
trap 'hdiutil detach "$MOUNT" -quiet >/dev/null 2>&1 || true; rm -rf "$WORK"' EXIT

echo "Downloading $(basename "$DMG_URL")..."
curl -fL --progress-bar "$DMG_URL" -o "$WORK/OneBrain.dmg"

echo "Installing to /Applications..."
mkdir -p "$MOUNT"
hdiutil attach "$WORK/OneBrain.dmg" -mountpoint "$MOUNT" -nobrowse -quiet
if pgrep -f "/Applications/$APP/Contents/MacOS" >/dev/null; then
  osascript -e 'tell application "OneBrain" to quit' >/dev/null 2>&1 || true
  sleep 2
fi
rm -rf "/Applications/$APP"
ditto "$MOUNT/$APP" "/Applications/$APP"

# Clear the download flag and sign locally so macOS opens it without a warning.
xattr -dr com.apple.quarantine "/Applications/$APP" 2>/dev/null || true
codesign --force --deep --sign - "/Applications/$APP" >/dev/null 2>&1 || true

echo "Done. Opening OneBrain..."
open "/Applications/$APP"
