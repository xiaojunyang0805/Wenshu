#!/usr/bin/env bash
# Build and package Wenshu (文枢) for Linux. <arch> is x86_64 or aarch64.
#
#   $DIST/wenshu-<version>-linux-<arch>.tar.gz    plain FHS-style tree (bin/, share/)
#   $DIST/wenshu-<version>-linux-<arch>.deb       Debian, Ubuntu, Deepin, UOS, Kylin, ...
#   $DIST/wenshu-<version>-linux-<arch>.AppImage  any distro with glibc >= the build host's
#
# Usage: packaging/linux/package.sh [--skip-build] [--formats "tar deb appimage"]
#
# Needs: Rust (rustup; see rust-version in Cargo.toml), WordCraft's Linux build packages
# (pkg-config libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev
# libgtk-3-dev libfontconfig1-dev), dpkg-deb for the .deb, and `scripts/sync-upstream.sh` run
# first. The AppImage downloads appimagetool and its runtime at pinned versions, checked by SHA-256
# (the same pins as WordCraft's packaging/linux/package.sh). Build on an old distro (Ubuntu 22.04,
# glibc 2.35) so the binaries run on newer ones. Optional: desktop-file-validate, appstreamcli.
#
# Like WordCraft's script, adapted: binaries wenshu-desktop and wenshu, app id
# io.github.xiaojunyang0805.wenshu, and the .deb is built with dpkg-deb rather than nfpm.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERE="$ROOT/packaging/linux"
UP="$ROOT/upstream/wordcraft"
APP_ID=io.github.xiaojunyang0805.wenshu

SKIP_BUILD=0
FORMATS="tar deb appimage"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    --formats) FORMATS="$2"; shift 2 ;;
    -h | --help) sed -n '2,19p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
has() { case " $FORMATS " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }

VERSION="$(awk '
  /^\[/ { in_pkg = ($0 == "[workspace.package]"); next }
  in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
' "$ROOT/Cargo.toml")"
[[ -n "$VERSION" ]] || { echo "error: no [workspace.package] version in Cargo.toml" >&2; exit 1; }
BUILD_DATE="${BUILD_DATE:-$(date -u +%Y-%m-%d)}"
DIST="${DIST:-$ROOT/dist}"
mkdir -p "$DIST"
# .cargo/config.toml puts the target dir here.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/eval/target/wordcraft}"
[[ -d "$UP" ]] || { echo "error: $UP missing; run scripts/sync-upstream.sh first" >&2; exit 1; }

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) DEB_ARCH=amd64 ;;
  aarch64 | arm64) ARCH=aarch64; DEB_ARCH=arm64 ;;
  *) echo "unsupported architecture $ARCH" >&2; exit 2 ;;
esac
BASENAME="wenshu-$VERSION-linux-$ARCH"
echo "==> Wenshu $VERSION for Linux $ARCH ($FORMATS)"

if [[ "$SKIP_BUILD" = 0 ]]; then
  (cd "$ROOT" && cargo build --release --locked -p wenshu-desktop -p wenshu-cli)
fi
BIN="$CARGO_TARGET_DIR/release"
WORK="$CARGO_TARGET_DIR/wenshu-linux-package"
STAGE="$WORK/root"
rm -rf "$WORK"

# ---- stage an FHS tree (shared by every format) -------------------------------------------------
install -Dm755 "$BIN/wenshu-desktop" "$STAGE/usr/bin/wenshu-desktop"
install -Dm755 "$BIN/wenshu" "$STAGE/usr/bin/wenshu"
strip "$STAGE/usr/bin/wenshu-desktop" "$STAGE/usr/bin/wenshu" 2>/dev/null || true
install -Dm644 "$HERE/$APP_ID.desktop" "$STAGE/usr/share/applications/$APP_ID.desktop"
mkdir -p "$STAGE/usr/share/metainfo"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@DATE@/$BUILD_DATE/g" \
  "$HERE/$APP_ID.metainfo.xml.in" >"$STAGE/usr/share/metainfo/$APP_ID.metainfo.xml"
install -Dm644 "$ROOT/apps/wenshu-desktop/assets/wenshu-256.png" \
  "$STAGE/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"
DOC="$STAGE/usr/share/doc/wenshu"
mkdir -p "$DOC/notices"
cp "$ROOT/README.md" "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$DOC/"
cp "$UP/NOTICE" "$DOC/notices/WordCraft-NOTICE"
cp "$UP/ATTRIBUTION.md" "$DOC/notices/WordCraft-ATTRIBUTION.md"
cp "$UP/LICENSE-MIT" "$DOC/notices/WordCraft-LICENSE-MIT"
cp "$UP/LICENSE-APACHE" "$DOC/notices/WordCraft-LICENSE-APACHE"
cp "$UP"/assets/fonts/OFL-*.txt "$DOC/notices/"

if command -v desktop-file-validate >/dev/null; then
  desktop-file-validate "$STAGE/usr/share/applications/$APP_ID.desktop"
fi
if command -v appstreamcli >/dev/null; then
  appstreamcli validate --no-net --explain "$STAGE/usr/share/metainfo/$APP_ID.metainfo.xml"
fi

# ---- .tar.gz ------------------------------------------------------------------------------------
if has tar; then
  mkdir -p "$WORK/tar"
  cp -R "$STAGE/usr" "$WORK/tar/$BASENAME"
  tar -C "$WORK/tar" --owner=0 --group=0 -czf "$DIST/$BASENAME.tar.gz" "$BASENAME"
  echo "wrote $DIST/$BASENAME.tar.gz"
fi

# ---- .deb ---------------------------------------------------------------------------------------
if has deb; then
  command -v dpkg-deb >/dev/null || { echo "error: dpkg-deb not found" >&2; exit 1; }
  DEB="$WORK/deb"
  cp -R "$STAGE" "$DEB"
  mkdir -p "$DEB/DEBIAN"
  install -m755 "$HERE/postinst.sh" "$DEB/DEBIAN/postinst"
  install -m755 "$HERE/postinst.sh" "$DEB/DEBIAN/postrm"
  # semver pre-releases (0.2.0-rc.1) sort before the release in Debian as 0.2.0~rc.1.
  DEB_VERSION="${VERSION/-/\~}-1"
  # The binaries link only glibc and libgcc_s directly. Windowing (X11/Wayland, xkbcommon) and the
  # GPU (Vulkan, EGL/GL) are loaded at runtime, so they are dependencies here, as in WordCraft's.
  # On X11 (still the default on Deepin, UOS and Kylin, and WordCraft's fallback) the app panics
  # at start without libxkbcommon-x11 (seen under WSLg); WordCraft's list lacks it.
  cat >"$DEB/DEBIAN/control" <<EOF
Package: wenshu
Version: $DEB_VERSION
Architecture: $DEB_ARCH
Maintainer: ${WENSHU_MAINTAINER:-The Wenshu authors <https://github.com/xiaojunyang0805/Wenshu>}
Installed-Size: $(du -sk "$STAGE" | cut -f1)
Depends: libc6 (>= 2.35), libgcc-s1, libxkbcommon0, libxkbcommon-x11-0, libvulkan1 | libegl1
Recommends: libwayland-client0, libx11-6, libx11-xcb1, libxcursor1, libxi6, libxrandr2, mesa-vulkan-drivers, xdg-desktop-portal, fonts-noto-cjk
Section: editors
Priority: optional
Homepage: https://github.com/xiaojunyang0805/Wenshu
Description: Chinese-first word processor (文枢)
 Open-source word processor built on WordCraft, with Chinese layout, Chinese
 blank-document defaults, GB/T 9704 official-document (公文) templates and a
 format checker. Reads and writes .docx and exports PDF. Includes wenshu, the
 command-line 公文 generator, checker and converter.
EOF
  dpkg-deb --root-owner-group -Zxz --build "$DEB" "$DIST/$BASENAME.deb" >/dev/null
  echo "wrote $DIST/$BASENAME.deb"
fi

# ---- AppImage -----------------------------------------------------------------------------------
if has appimage; then
  APPIMAGETOOL_VERSION=1.9.1
  RUNTIME_VERSION=20251108
  case "$ARCH" in
    x86_64)
      APPIMAGETOOL_SHA256=ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
      RUNTIME_SHA256=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d
      ;;
    aarch64)
      APPIMAGETOOL_SHA256=f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158
      RUNTIME_SHA256=00cbdfcf917cc6c0ff6d3347d59e0ca1f7f45a6df1a428a0d6d8a78664d87444
      ;;
  esac
  # Downloads $1 to $3 once and checks it against SHA-256 $2, the cached copy again on every run.
  fetch_pinned() {
    local url="$1" expected_sha="$2" target="$3"
    if [[ ! -x "$target" ]]; then
      curl --proto '=https' -fsSL -o "$target.download" "$url"
      printf '%s  %s\n' "$expected_sha" "$target.download" | sha256sum -c - >/dev/null
      chmod 755 "$target.download"
      mv "$target.download" "$target"
    fi
    printf '%s  %s\n' "$expected_sha" "$target" | sha256sum -c - >/dev/null
  }
  APPDIR="$WORK/Wenshu.AppDir"
  cp -R "$STAGE" "$APPDIR"
  ln -s usr/bin/wenshu-desktop "$APPDIR/AppRun"
  cp "$HERE/$APP_ID.desktop" "$APPDIR/$APP_ID.desktop"
  cp "$ROOT/apps/wenshu-desktop/assets/wenshu-256.png" "$APPDIR/$APP_ID.png"
  ln -s "$APP_ID.png" "$APPDIR/.DirIcon"

  TOOL="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
  if [[ -z "$TOOL" ]]; then
    TOOL="$CARGO_TARGET_DIR/appimagetool-$APPIMAGETOOL_VERSION-$ARCH.AppImage"
    fetch_pinned "https://github.com/AppImage/appimagetool/releases/download/$APPIMAGETOOL_VERSION/appimagetool-$ARCH.AppImage" \
      "$APPIMAGETOOL_SHA256" "$TOOL"
  fi
  RUNTIME="$CARGO_TARGET_DIR/appimage-runtime-$RUNTIME_VERSION-$ARCH"
  fetch_pinned "https://github.com/AppImage/type2-runtime/releases/download/$RUNTIME_VERSION/runtime-$ARCH" \
    "$RUNTIME_SHA256" "$RUNTIME"
  OUT="$(cd "$DIST" && pwd)/$BASENAME.AppImage"
  # Extract-and-run: works without FUSE (containers, WSL, CI). The output embeds the static
  # runtime, so users don't need libfuse2. No update information: Wenshu has no releases yet.
  ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 "$TOOL" --no-appstream --runtime-file "$RUNTIME" \
    "$APPDIR" "$OUT" >/dev/null
  echo "wrote $OUT"
fi

# A smoke run: with no arguments the CLI prints its usage and exits 0.
"$STAGE/usr/bin/wenshu" >/dev/null && echo "ok wenshu runs"
echo "==> done"
ls -lh "$DIST" | grep -F "wenshu-$VERSION-linux" || true
