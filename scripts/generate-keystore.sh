#!/usr/bin/env bash
# Generate signing keystore for RalphHub Mobile APK
set -euo pipefail

KEYSTORE_PATH="ralphhub.keystore"
KEY_ALIAS="ralphhub"
STORE_PASS="${KEYSTORE_PASS:-ralphhub-mobile-2024}"
KEY_PASS="${KEY_PASS:-ralphhub-mobile-2024}"

if [ -f "$KEYSTORE_PATH" ]; then
  echo "Keystore already exists at $KEYSTORE_PATH"
  exit 0
fi

keytool -genkey -v \
  -keystore "$KEYSTORE_PATH" \
  -alias "$KEY_ALIAS" \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000 \
  -storepass "$STORE_PASS" \
  -keypass "$KEY_PASS" \
  -dname "CN=RalphHub Mobile, OU=Mobile, O=RalphHub, L=Remote, S=Remote, C=US"

echo "✓ Keystore generated: $KEYSTORE_PATH"
echo "  Store password: $STORE_PASS"
echo "  Key alias: $KEY_ALIAS"
echo ""
echo "IMPORTANT: Add KEYSTORE_PASS and KEY_PASS to your environment/CI secrets."
