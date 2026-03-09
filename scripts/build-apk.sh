#!/usr/bin/env bash
# RalphHub Mobile APK Build Script
# Requires: Java 17+, Android SDK (ANDROID_HOME set), Node.js, Bun
set -euo pipefail

echo "=== RalphHub Mobile APK Builder ==="
echo ""

# Check prereqs
check_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "ERROR: $1 not found. $2"; exit 1; }; }
check_cmd java "Install JDK 17+ from https://adoptium.net"
check_cmd bun "curl -fsSL https://bun.sh/install | bash"
check_cmd npx "Install Node.js from https://nodejs.org"

if [ -z "${ANDROID_HOME:-}" ]; then
  echo "ERROR: ANDROID_HOME not set. Install Android SDK and set ANDROID_HOME."
  exit 1
fi

echo "✓ Java: $(java -version 2>&1 | head -1)"
echo "✓ Bun: $(bun --version)"
echo "✓ Android SDK: $ANDROID_HOME"
echo ""

# Step 1: Build SvelteKit
echo "→ Building SvelteKit frontend..."
bun run build
echo "✓ Frontend built to ./build/"
echo ""

# Step 2: Sync to Capacitor Android
echo "→ Syncing to Capacitor Android project..."
npx cap sync android
echo "✓ Synced"
echo ""

# Step 3: Build APK
echo "→ Building Android APK (this may take 2-5 minutes)..."
cd android

# Use release build if keystore available, else debug
if [ -f "../ralphhub.keystore" ]; then
  ./gradlew assembleRelease \
    -Pandroid.injected.signing.store.file="$(pwd)/../ralphhub.keystore" \
    -Pandroid.injected.signing.store.password="${KEYSTORE_PASS:-ralphhub}" \
    -Pandroid.injected.signing.key.alias=ralphhub \
    -Pandroid.injected.signing.key.password="${KEY_PASS:-ralphhub}"
  APK_PATH="app/build/outputs/apk/release/app-release.apk"
  echo "✓ Release APK built"
else
  echo "⚠ No keystore found — building debug APK"
  ./gradlew assembleDebug
  APK_PATH="app/build/outputs/apk/debug/app-debug.apk"
  echo "✓ Debug APK built"
fi

cd ..

# Step 4: Copy to static/downloads for QR download
mkdir -p static/downloads
cp "android/${APK_PATH}" static/downloads/ralphhub-mobile.apk
echo ""
echo "=== BUILD COMPLETE ==="
echo "APK: static/downloads/ralphhub-mobile.apk"
echo ""
echo "Size: $(du -sh static/downloads/ralphhub-mobile.apk | cut -f1)"
echo ""
echo "→ Serve via Tauri desktop or nginx on VPS"
echo "→ QR code generated automatically in Settings → Mobile ↗"
echo ""
echo "MOBILE + ARCHITECTURE MEGA COMPLETE"
