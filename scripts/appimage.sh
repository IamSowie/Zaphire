#!/usr/bin/env bash
# Turn dist/zaphire-linux (from package.sh) into dist/Zaphire-<version>-x86_64.AppImage.
set -euo pipefail
cd "$(dirname "$0")/.."
SRC=dist/zaphire-linux
[ -x "$SRC/zaphire" ] || { echo "run scripts/package.sh first" >&2; exit 1; }
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
APPDIR=dist/Zaphire.AppDir
rm -rf "$APPDIR" && mkdir -p "$APPDIR/usr/lib" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/icons/hicolor/256x256/apps"
cp -r "$SRC" "$APPDIR/usr/lib/zaphire"
rm -f "$APPDIR/usr/lib/zaphire/zaphire.sh"
cp packaging/linux/zaphire.desktop "$APPDIR/zaphire.desktop"
cp packaging/linux/zaphire.desktop "$APPDIR/usr/share/applications/"
cp assets/zaphire-256.png "$APPDIR/zaphire.png"
cp assets/zaphire-256.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/zaphire.png"
cat > "$APPDIR/AppRun" <<'RUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")/usr/lib/zaphire"
export LD_LIBRARY_PATH="$HERE${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$HERE/zaphire" "$@"
RUN
chmod +x "$APPDIR/AppRun"

TOOL="${APPIMAGETOOL:-dist/appimagetool}"
if [ ! -x "$TOOL" ]; then
  curl -fsSL -o "$TOOL" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
  chmod +x "$TOOL"
fi
OUT="dist/Zaphire-$VERSION-x86_64.AppImage"
# Extract-and-run: works without FUSE (CI runners, containers).
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$TOOL" --no-appstream "$APPDIR" "$OUT"
rm -rf "$APPDIR"
echo "Built $OUT ($(du -h "$OUT" | cut -f1))"
