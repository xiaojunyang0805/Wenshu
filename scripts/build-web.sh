#!/usr/bin/env bash
# Build Wenshu's self-hostable browser app into dist/web (or $OUT): the app (trunk), Wenshu's open
# fonts in fonts/ with fonts/fonts.json (the order the app fetches them in), and the licences.
# Host the folder on any static web server, an intranet one too; nothing is sent anywhere else.
#
#   scripts/sync-upstream.sh    # first: the patched WordCraft (patch 0072 makes the web app a library)
#   scripts/build-web.sh
#
# Needs trunk (`cargo install trunk --locked`), the wasm32-unknown-unknown target, and what
# packaging/fonts.sh needs. With Python's fontTools and brotli the fonts are served as WOFF2
# (about half the size); without them as the original TTF/OTF.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${OUT:-$ROOT/dist/web}"
TARGET="${CARGO_TARGET_DIR:-$ROOT/eval/target/wordcraft}"
STAGE="$TARGET/wenshu-web-fonts"

(cd "$ROOT/apps/wenshu-web" && trunk build --release --dist "$OUT")
rm -f "$OUT/.trunk-lock"

. "$ROOT/packaging/fonts.sh"
rm -rf "$STAGE/fonts" "$STAGE/notices"
stage_wenshu_fonts "$TARGET/wenshu-fonts" "$STAGE/fonts" "$STAGE/notices"

# The order fonts are fetched in: a document's Latin defaults and the 公文 body first, the
# 26 MB Kai design last.
ORDER="Carlito-Regular.ttf Carlito-Bold.ttf LiberationSerif-Regular.ttf LiberationSerif-Bold.ttf
LiberationSans-Regular.ttf LiberationSans-Bold.ttf ZhuqueFangsong-Regular.ttf NotoSerifSC-Regular.otf
NotoSerifSC-Bold.otf Carlito-Italic.ttf Carlito-BoldItalic.ttf LiberationSerif-Italic.ttf
LiberationSerif-BoldItalic.ttf LiberationSans-Italic.ttf LiberationSans-BoldItalic.ttf
LiberationMono-Regular.ttf LiberationMono-Bold.ttf LiberationMono-Italic.ttf LiberationMono-BoldItalic.ttf
Caladea-Regular.ttf Caladea-Bold.ttf Caladea-Italic.ttf Caladea-BoldItalic.ttf LXGWWenKaiGB-Regular.ttf"

PY=""
for p in python3 python; do
  if command -v "$p" >/dev/null && "$p" -c "import fontTools.ttLib, brotli" 2>/dev/null; then PY="$p"; break; fi
done
rm -rf "$OUT/fonts" "$OUT/licenses"
mkdir -p "$OUT/fonts" "$OUT/licenses"
files=()
for f in $ORDER; do
  src="$STAGE/fonts/$f"
  [[ -f "$src" ]] || { echo "build-web: missing $f" >&2; exit 1; }
  if [[ -n "$PY" ]]; then
    out="${f%.*}.woff2"
    "$PY" -c "import sys; from fontTools.ttLib import TTFont; t = TTFont(sys.argv[1]); t.flavor = 'woff2'; t.save(sys.argv[2])" "$src" "$OUT/fonts/$out"
  else
    out="$f"
    cp "$src" "$OUT/fonts/$out"
  fi
  files+=("\"$out\"")
done
(IFS=,; printf '{"files": [%s]}\n' "${files[*]}") > "$OUT/fonts/fonts.json"

cp "$STAGE"/notices/* "$OUT/licenses/"
UP="$ROOT/upstream/wordcraft"
cp "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$OUT/licenses/"
cp "$UP/NOTICE" "$OUT/licenses/WordCraft-NOTICE"
cp "$UP/ATTRIBUTION.md" "$OUT/licenses/WordCraft-ATTRIBUTION.md"
cp "$UP/LICENSE-MIT" "$OUT/licenses/WordCraft-LICENSE-MIT"
cp "$UP/LICENSE-APACHE" "$OUT/licenses/WordCraft-LICENSE-APACHE"
cp "$UP"/assets/fonts/OFL-*.txt "$OUT/licenses/"
cp "$ROOT/apps/wenshu-web/HOSTING.md" "$OUT/README.md"
du -sh "$OUT" "$OUT/fonts" | sed 's/^/build-web: /'
