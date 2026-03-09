#!/usr/bin/env bash
# Build AmitOS for Raspberry Pi (ARM64 + ARMv7)
# Requires: cross (cargo install cross), Docker

set -e

echo "======================================"
echo "  AmitOS — Raspberry Pi Build Script"
echo "======================================"

# Check prerequisites
if ! command -v cross &> /dev/null; then
  echo "Installing cross..."
  cargo install cross
fi

if ! command -v bun &> /dev/null; then
  echo "Error: bun is required. Install from https://bun.sh"
  exit 1
fi

# Build frontend
echo ""
echo "[1/4] Building SvelteKit frontend..."
bun install
bun run build

# Build ARM64 (RPi 4, RPi 5, RPi CM4)
echo ""
echo "[2/4] Building for ARM64 (aarch64-unknown-linux-gnu)..."
cross build \
  --manifest-path src-tauri/Cargo.toml \
  --target aarch64-unknown-linux-gnu \
  --release \
  --bin ralphhub

echo "ARM64 binary: src-tauri/target/aarch64-unknown-linux-gnu/release/ralphhub"

# Build ARMv7 (RPi 2, RPi 3, older models)
echo ""
echo "[3/4] Building for ARMv7 (armv7-unknown-linux-gnueabihf)..."
cross build \
  --manifest-path src-tauri/Cargo.toml \
  --target armv7-unknown-linux-gnueabihf \
  --release \
  --bin ralphhub

echo "ARMv7 binary: src-tauri/target/armv7-unknown-linux-gnueabihf/release/ralphhub"

# Package artifacts
echo ""
echo "[4/4] Packaging artifacts..."
mkdir -p dist/rpi
cp src-tauri/target/aarch64-unknown-linux-gnu/release/ralphhub dist/rpi/amitos-arm64 2>/dev/null || true
cp src-tauri/target/armv7-unknown-linux-gnueabihf/release/ralphhub dist/rpi/amitos-armv7 2>/dev/null || true

cat > dist/rpi/install-rpi.sh << 'INSTALL_SCRIPT'
#!/bin/bash
# One-click AmitOS installer for Raspberry Pi

set -e
ARCH=$(uname -m)
BIN="amitos-arm64"
if [ "$ARCH" = "armv7l" ] || [ "$ARCH" = "armhf" ]; then
  BIN="amitos-armv7"
fi

echo "Installing AmitOS for $ARCH..."
chmod +x "./$BIN"
sudo mkdir -p /opt/amitos/bin
sudo cp "./$BIN" /opt/amitos/bin/amitos
sudo chmod +x /opt/amitos/bin/amitos

# Create systemd service (optional)
if command -v systemctl &>/dev/null; then
  sudo tee /etc/systemd/system/amitos.service > /dev/null << 'SERVICE'
[Unit]
Description=AmitOS Universal AI OS
After=network.target

[Service]
Type=simple
User=pi
ExecStart=/opt/amitos/bin/amitos
Restart=on-failure
Environment=DISPLAY=:0

[Install]
WantedBy=multi-user.target
SERVICE
  sudo systemctl daemon-reload
  echo "Systemd service created. Enable with: sudo systemctl enable --now amitos"
fi

echo ""
echo "✅ AmitOS installed to /opt/amitos/bin/amitos"
echo "   Run: /opt/amitos/bin/amitos"
INSTALL_SCRIPT

chmod +x dist/rpi/install-rpi.sh

echo ""
echo "======================================"
echo "  Build complete!"
echo "======================================"
echo ""
echo "Artifacts:"
ls -lh dist/rpi/
echo ""
echo "To install on RPi:"
echo "  scp dist/rpi/* pi@<rpi-ip>:~/"
echo "  ssh pi@<rpi-ip> 'bash ~/install-rpi.sh'"
