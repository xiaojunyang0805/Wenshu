#!/usr/bin/env bash
# Build and package Wenshu (文枢) for Linux. <arch> is x86_64, aarch64 or loongarch64.
#
#   $DIST/wenshu-<version>-linux-<arch>.tar.gz    plain FHS-style tree (bin/, share/)
#   $DIST/wenshu-<version>-linux-<arch>.deb       Debian, Ubuntu, Deepin, UOS, Kylin, ...
#   $DIST/wenshu-<version>-linux-<arch>.AppImage  any distro with glibc >= the build host's
#
# Usage: packaging/linux/package.sh [--skip-build] [--formats "tar deb appimage"] [--arch loongarch64]
#
# --arch loongarch64 cross-builds on an x86_64 host with cargo-zigbuild (Zig as the C compiler and
# linker), against glibc 2.36, the first with LoongArch support. That is the "new world" ABI
# (deepin 23+, openKylin 2, Loongnix 25, Debian loong64, AOSC, openEuler, Anolis 23); old-world
# systems (UOS 20, Kylin V10, Loongnix 20) need libLoL. Needs `rustup target add
# loongarch64-unknown-linux-gnu`, Zig 0.13 and cargo-zigbuild. No AppImage: AppImage publishes no
# LoongArch runtime. Set CROSS_RUNNER (e.g. "qemu-loongarch64 -L <loong64 sysroot>") to run the
# smoke test; without it the smoke test is skipped.
#
# Needs: Rust (rustup; see rust-version in Cargo.toml), WordCraft's Linux build packages
# (pkg-config libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev
# libgtk-3-dev libfontconfig1-dev), curl, unzip, dpkg-deb for the .deb, and
# `scripts/sync-upstream.sh` run first. Every package carries two open 公文 fonts (朱雀仿宋,
# 霞鹜文楷 GB), downloaded once at pinned versions and checked by SHA-256. The AppImage downloads appimagetool and its runtime at pinned versions, checked by SHA-256
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
    --arch) ARCH="$2"; shift 2 ;;
    -h | --help) sed -n '2,27p' "$0"; exit 0 ;;
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

HOST_ARCH="$(uname -m)"
ARCH="${ARCH:-$HOST_ARCH}"
GLIBC_MIN=2.35 # the build host's (Ubuntu 22.04)
case "$ARCH" in
  x86_64) DEB_ARCH=amd64 ;;
  aarch64 | arm64) ARCH=aarch64; DEB_ARCH=arm64 ;;
  loongarch64 | loong64)
    ARCH=loongarch64; DEB_ARCH=loong64; GLIBC_MIN=2.36
    [[ "$HOST_ARCH" = loongarch64 ]] || CROSS_TARGET=loongarch64-unknown-linux-gnu
    FORMATS="${FORMATS/appimage/}"
    ;;
  *) echo "unsupported architecture $ARCH" >&2; exit 2 ;;
esac
CROSS_TARGET="${CROSS_TARGET:-}"
BASENAME="wenshu-$VERSION-linux-$ARCH"
echo "==> Wenshu $VERSION for Linux $ARCH ($FORMATS)"

if [[ -n "$CROSS_TARGET" ]]; then
  # The glibc suffix makes Zig link against that glibc's symbol versions.
  if [[ "$SKIP_BUILD" = 0 ]]; then
    (cd "$ROOT" && CARGO_PROFILE_RELEASE_STRIP=symbols cargo zigbuild --release --locked \
      -p wenshu-desktop -p wenshu-cli --target "$CROSS_TARGET.$GLIBC_MIN")
  fi
  BIN="$CARGO_TARGET_DIR/$CROSS_TARGET/release"
else
  if [[ "$SKIP_BUILD" = 0 ]]; then
    (cd "$ROOT" && cargo build --release --locked -p wenshu-desktop -p wenshu-cli)
  fi
  BIN="$CARGO_TARGET_DIR/release"
fi
# Refuse binaries for another machine (a stale or wrong --skip-build).
case "$ARCH" in x86_64) MACHINE=x86-64 ;; aarch64) MACHINE=aarch64 ;; loongarch64) MACHINE=LoongArch ;; esac
for b in wenshu-desktop wenshu; do
  file -b "$BIN/$b" | grep -q "$MACHINE" || { echo "error: $BIN/$b is not a $ARCH binary" >&2; exit 1; }
done
# Staging (WENSHU_STAGE_DIR): must be a Unix filesystem, or dpkg-deb rejects the permissions.
WORK="${WENSHU_STAGE_DIR:-$CARGO_TARGET_DIR/wenshu-linux-package}"
STAGE="$WORK/root"
rm -rf "$WORK"

# ---- stage an FHS tree (shared by every format) -------------------------------------------------
install -Dm755 "$BIN/wenshu-desktop" "$STAGE/usr/bin/wenshu-desktop"
install -Dm755 "$BIN/wenshu" "$STAGE/usr/bin/wenshu"
[[ -n "$CROSS_TARGET" ]] || strip "$STAGE/usr/bin/wenshu-desktop" "$STAGE/usr/bin/wenshu" 2>/dev/null || true
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

# ---- open 公文 fonts ------------------------------------------------------------------------------
# Linux systems rarely have a FangSong or Kai design, so 仿宋_GB2312 and 楷体_GB2312 text would
# fall back to Song (wenshu-cjk's substitute lists). Ship two OFL-1.1 families, unmodified, in
# /usr/share/wenshu/fonts: wenshu-core catalogs that folder and loads a font only when a document
# uses it. Windows has FangSong and KaiTi, so its packages don't carry them.
FONT_CACHE="$CARGO_TARGET_DIR/wenshu-fonts"
mkdir -p "$FONT_CACHE"
# Downloads $1 to $FONT_CACHE/$3 once and checks it against SHA-256 $2 on every run.
fetch_font_file() {
  local url="$1" sha="$2" target="$FONT_CACHE/$3"
  if [[ ! -f "$target" ]]; then
    curl --proto '=https' -fsSL -o "$target.download" "$url"
    printf '%s  %s\n' "$sha" "$target.download" | sha256sum -c - >/dev/null
    mv "$target.download" "$target"
  fi
  printf '%s  %s\n' "$sha" "$target" | sha256sum -c - >/dev/null
}
# 朱雀仿宋 Zhuque Fangsong v0.212 (TrionesType)
fetch_font_file https://github.com/TrionesType/zhuque/releases/download/v0.212/ZhuqueFangsong-v0.212.zip \
  bb8b661a7643d2296a72d9d10530a00949419c4e527fb61783f73c2ba1a8c062 ZhuqueFangsong-v0.212.zip
fetch_font_file https://raw.githubusercontent.com/TrionesType/zhuque/v0.212/LICENSE.txt \
  66cc01ad4df62fca6936431a426104cb32a52ce8da49de9a7d643e02d7659043 OFL-ZhuqueFangsong.txt
# 霞鹜文楷 GB LXGW WenKai GB v1.530 (GB 18030 glyph forms)
fetch_font_file https://github.com/lxgw/LxgwWenkaiGB/releases/download/v1.530/LXGWWenKaiGB-Regular.ttf \
  5561d48097d92e5ab99fe7b1459d381bbcdff533bfc479207b089c6860c3990e LXGWWenKaiGB-Regular.ttf
fetch_font_file https://raw.githubusercontent.com/lxgw/LxgwWenkaiGB/v1.530/OFL.txt \
  a8aa32c38a3402ba4e40fb6d39c913300dd4a0336290dce5c14e3a2f9308d092 OFL-LXGWWenKaiGB.txt
FONTS="$STAGE/usr/share/wenshu/fonts"
mkdir -p "$FONTS"
unzip -q -o -j "$FONT_CACHE/ZhuqueFangsong-v0.212.zip" 'ZhuqueFangsong-Regular.ttf' -d "$FONTS"
install -m644 "$FONT_CACHE/LXGWWenKaiGB-Regular.ttf" "$FONTS/"
install -m644 "$FONT_CACHE/OFL-ZhuqueFangsong.txt" "$FONT_CACHE/OFL-LXGWWenKaiGB.txt" "$DOC/notices/"
chmod 644 "$FONTS"/*

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
Depends: libc6 (>= $GLIBC_MIN), libgcc-s1, libxkbcommon0, libxkbcommon-x11-0, libvulkan1 | libegl1
Recommends: libwayland-client0, libx11-6, libx11-xcb1, libxcursor1, libxi6, libxrandr2, mesa-vulkan-drivers, xdg-desktop-portal, fonts-noto-cjk, fonts-liberation2, fonts-crosextra-carlito, fonts-crosextra-caladea
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
if [[ -z "$CROSS_TARGET" ]]; then
  "$STAGE/usr/bin/wenshu" >/dev/null && echo "ok wenshu runs"
elif [[ -n "${CROSS_RUNNER:-}" ]]; then
  $CROSS_RUNNER "$STAGE/usr/bin/wenshu" >/dev/null && echo "ok wenshu runs ($CROSS_RUNNER)"
else
  echo "skipped the smoke run (cross build; set CROSS_RUNNER)"
fi
echo "==> done"
ls -lh "$DIST" | grep -F "wenshu-$VERSION-linux" || true
