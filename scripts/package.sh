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
# CEF runtime files (copied next to the binary by the cef build script).
for f in "$SRC"/*.pak "$SRC"/*.dat "$SRC"/*.bin "$SRC"/*.json "$SRC"/*.so* "$SRC"/*.dll; do
  [ -e "$f" ] && cp "$f" "$OUT/"
done
[ -d "$SRC/locales" ] && cp -r "$SRC/locales" "$OUT/"
[ -e "$SRC/chrome-sandbox" ] && cp "$SRC/chrome-sandbox" "$OUT/"
cp -r ui/dist "$OUT/ui"
cp LICENSE README.md "$OUT/"
if [ "$OS" = linux ]; then
  # The CEF archive ships libcef.so with debug symbols (~1.4 GB → ~265 MB).
  command -v strip >/dev/null && strip --strip-unneeded "$OUT/libcef.so" "$OUT/$EXE"
  printf '#!/bin/sh\nHERE="$(dirname "$(readlink -f "$0")")"\nLD_LIBRARY_PATH="$HERE${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" exec "$HERE/zaphire" "$@"\n' > "$OUT/zaphire.sh"
  chmod +x "$OUT/zaphire.sh"
fi
echo "Packaged $OUT ($(du -sh "$OUT" | cut -f1))"
