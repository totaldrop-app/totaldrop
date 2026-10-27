#!/usr/bin/env bash
# Creates the Google Play upload key for TotalDrop and android/key.properties.
# Run it yourself, once. The password is typed here and never leaves this
# computer; keep it and the .jks file in your password manager and backups.
# If the upload key is ever lost, Play support can reset it.
set -euo pipefail

dir="$HOME/.android-keys"
jks="$dir/totaldrop-upload.jks"
props="$(cd "$(dirname "$0")" && pwd)/key.properties"

if [ -e "$jks" ]; then
  echo "An upload key already exists at $jks; not overwriting it." >&2
  exit 1
fi

read -r -s -p "New upload key password (at least 12 characters): " pw; echo
read -r -s -p "Repeat it: " pw2; echo
[ "$pw" = "$pw2" ] || { echo "The passwords differ." >&2; exit 1; }
[ "${#pw}" -ge 12 ] || { echo "Use at least 12 characters." >&2; exit 1; }

mkdir -p "$dir" && chmod 700 "$dir"
TD_PW="$pw" keytool -genkeypair -keystore "$jks" -storetype PKCS12 -alias upload \
  -keyalg RSA -keysize 4096 -validity 10000 \
  -dname "CN=TotalDrop, O=TotalDrop, C=TR" \
  -storepass:env TD_PW -keypass:env TD_PW
chmod 600 "$jks"

umask 077
cat > "$props" <<PROPS
storeFile=$jks
storePassword=$pw
keyAlias=upload
keyPassword=$pw
PROPS
unset pw pw2
echo "Upload key: $jks"
echo "Signing settings: $props (never commit this file)"
