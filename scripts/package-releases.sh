#!/usr/bin/env bash
# Package Tervixa Wiggle release artifacts into releases/.
#
# Usage:
#   scripts/package-releases.sh [workspace-dir]
#
# Expects a completed `cargo build --release` for the host target.
# CI runs this per-OS and uploads releases/<os>/ as artifacts.
set -euo pipefail

TARGET="${1:-.}"
OS_NAME="$(uname -s)"
VERSION="$(grep -m1 '^version' "$TARGET/crates/wiggle-core/Cargo.toml" 2>/dev/null | cut -d'"' -f2 || echo "1.0.0")"
case "$OS_NAME" in
  MINGW*|MSYS*|CYGWIN*|Windows*) OS_DIR="windows" ;;
  Darwin) OS_DIR="macos" ;;
  *) OS_DIR="linux" ;;
esac

OUT="$TARGET/releases/$OS_DIR"
mkdir -p "$OUT"

echo "== packaging Tervixa Wiggle v$VERSION for $OS_DIR =="

cp "$TARGET/README.md" "$OUT/README.md"
cp "$TARGET/LICENSE" "$OUT/LICENSE"
cp "$TARGET/WIGGLE.md" "$OUT/WIGGLE.md"
cp -r "$TARGET/bundled-skills" "$OUT/bundled-skills"

BIN=""
for candidate in \
  "$TARGET/target/release/wiggle.exe" \
  "$TARGET/target/release/wiggle" \
  "$TARGET/target/release/wiggle-desktop.exe" \
  "$TARGET/target/release/wiggle-desktop"; do
  [ -f "$candidate" ] && BIN="$candidate" && break
done
if [ -z "$BIN" ]; then
  echo "no release binary found; run cargo build --release first" >&2
  exit 1
fi
cp "$BIN" "$OUT/"
echo "binary: $(basename "$BIN") -> $OUT"

# Bundled installers, when produced by cargo-bundle/tauri on this host.
for inst in \
  "$TARGET/target/release/bundle/nsis"/*.exe \
  "$TARGET/target/release/bundle/msi"/*.msi \
  "$TARGET/target/release/bundle/deb"/*.deb \
  "$TARGET/target/release/bundle/rpm"/*.rpm \
  "$TARGET/target/release/bundle/appimage"/*.AppImage \
  "$TARGET/target/release/bundle/dmg"/*.dmg; do
  [ -f "$inst" ] && cp "$inst" "$OUT/" && echo "installer: $(basename "$inst")"
done

# Refresh checksums across every OS directory.
(
  cd "$TARGET/releases"
  find windows linux macos -type f \( -name '*.exe' -o -name '*.msi' -o -name '*.deb' \
    -o -name '*.rpm' -o -name '*.AppImage' -o -name '*.dmg' \) 2>/dev/null \
    | sort | xargs -r sha256sum > SHA256SUMS.txt || true
)
echo "== SHA256SUMS.txt refreshed =="
