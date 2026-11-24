#!/usr/bin/env bash
# Build a portable Zaphire folder in dist/zaphire-<os>/.
# Usage: scripts/package.sh [release|debug]
set -euo pipefail
cd "$(dirname "$0")/.."
PROFILE="${1:-release}"
FLAG=$([ "$PROFILE" = release ] && echo --release || true)

(cd ui && npm ci && npm run build)
cargo build -p zaphire-app $FLAG

case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) OS=windows; EXE=zaphire.exe ;;
  *) OS=linux; EXE=zaphire ;;
esac
SRC="target/$PROFILE"
OUT="dist/zaphire-$OS"
rm -rf "$OUT" && mkdir -p "$OUT"

cp "$SRC/$EXE" "$OUT/"
# CEF runtime files. The cef build script copies them next to the binary;
# fall back to the CEF_PATH export if it did not (e.g. cached build script).
for dir in "${CEF_PATH:-}" "$SRC"; do
  [ -n "$dir" ] && [ -d "$dir" ] || continue
  for f in "$dir"/*.pak "$dir"/*.dat "$dir"/*.bin "$dir"/*.json "$dir"/*.so* "$dir"/*.dll "$dir"/chrome-sandbox; do
    if [ -f "$f" ]; then cp "$f" "$OUT/"; fi
  done
  if [ -d "$dir/locales" ]; then cp -r "$dir/locales" "$OUT/"; fi
done
rm -f "$OUT/archive.json"
LIB=$([ "$OS" = windows ] && echo libcef.dll || echo libcef.so)
if [ ! -f "$OUT/$LIB" ]; then
  echo "error: $LIB not found. CEF_PATH=${CEF_PATH:-unset}" >&2
  ls "${CEF_PATH:-.}" "$SRC" >&2 || true
  exit 1
fi
cp -r ui/dist "$OUT/ui"
cp LICENSE README.md "$OUT/"
if [ "$OS" = linux ]; then
  # The CEF archive ships libcef.so with debug symbols (~1.4 GB → ~265 MB).
  command -v strip >/dev/null && strip --strip-unneeded "$OUT/libcef.so" "$OUT/$EXE"
  printf '#!/bin/sh\nHERE="$(dirname "$(readlink -f "$0")")"\nLD_LIBRARY_PATH="$HERE${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" exec "$HERE/zaphire" "$@"\n' > "$OUT/zaphire.sh"
  chmod +x "$OUT/zaphire.sh"
fi
echo "Packaged $OUT ($(du -sh "$OUT" | cut -f1))"
