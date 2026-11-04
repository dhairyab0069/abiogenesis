#!/usr/bin/env bash
# Usage: build-dmg.sh <rust-target-triple> <version> <output.dmg>
# Assumes `cargo build --release --locked --target <triple>` has already run.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <rust-target-triple> <version> <output.dmg>" >&2
  exit 2
fi
target="$1"
version="$2"
out="$3"

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
bin="$repo/target/$target/release/abiogenesis"
[ -f "$bin" ] || { echo "missing binary: $bin" >&2; exit 1; }

mkdir -p "$(dirname "$out")"
out="$(cd "$(dirname "$out")" && pwd)/$(basename "$out")"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
stage="$work/stage"
app="$stage/Abiogenesis.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

cp "$bin" "$app/Contents/MacOS/abiogenesis"
chmod 755 "$app/Contents/MacOS/abiogenesis"
cp "$here/AppIcon.icns" "$app/Contents/Resources/AppIcon.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Abiogenesis</string>
  <key>CFBundleDisplayName</key><string>Abiogenesis</string>
  <key>CFBundleIdentifier</key><string>io.github.dhairyab0069.abiogenesis</string>
  <key>CFBundleExecutable</key><string>abiogenesis</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSApplicationCategoryType</key><string>public.app-category.education</string>
</dict>
</plist>
PLIST

# Ad-hoc signature so Apple Silicon will run it after the Gatekeeper prompt.
codesign --force --deep --sign - "$app"

ln -s /Applications "$stage/Applications"
rm -f "$out"
hdiutil create -volname Abiogenesis -srcfolder "$stage" -ov -format UDZO "$out"
echo "Wrote $out"
