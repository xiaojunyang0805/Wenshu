#!/usr/bin/env bash
# Wenshu's open fonts, pinned by URL and SHA-256: the Linux packages (packaging/linux/package.sh)
# and the browser build (scripts/build-web.sh) both stage them from here.
#
#   . packaging/fonts.sh; stage_wenshu_fonts <download cache> <fonts dir> <licence notices dir>
#
# Needs curl, sha256sum, unzip, tar and install.

# Linux systems rarely have a FangSong or Kai design, so 仿宋_GB2312 and 楷体_GB2312 text would
# fall back to Song (wenshu-cjk's substitute lists). Ship two OFL-1.1 families, unmodified, in
# /usr/share/wenshu/fonts: wenshu-core catalogs that folder and loads a font only when a document
# uses it. Windows has FangSong and KaiTi, so its packages don't carry them.
stage_wenshu_fonts() {
  local FONT_CACHE="$1" FONTS="$2" NOTICES="$3"
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
  # Metric-compatible Latin fonts for Times New Roman / Arial / Courier New (Liberation 2), Calibri
  # (Carlito) and Cambria (Caladea): WordCraft draws those names in them when installed, keeping
  # line breaks; distributions often don't install them (Linux corpus run: 150 -> 167 / 200).
  fetch_font_file https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz   7191c669bf38899f73a2094ed00f7b800553364f90e2637010a69c0e268f25d0 liberation-fonts-ttf-2.1.5.tar.gz
  GF=https://raw.githubusercontent.com/google/fonts/f60b457bc90062c17f9bf80a882da25a86f56200/ofl
  fetch_font_file "$GF/carlito/Carlito-Regular.ttf" f6418f708baede9789daef5d458c0f53d2a888af9820e8062934e504fedc6595 Carlito-Regular.ttf
  fetch_font_file "$GF/carlito/Carlito-Bold.ttf" bb5d20f79b82599ec72983597437373a80f2d2085fa91fc144fd74e876a594db Carlito-Bold.ttf
  fetch_font_file "$GF/carlito/Carlito-Italic.ttf" 0b019225e58d702bfedcbd35c21696769f8ee115cb6343f84c2f240312450d1c Carlito-Italic.ttf
  fetch_font_file "$GF/carlito/Carlito-BoldItalic.ttf" b32928186c119599e03ca6a1ffc680fdcb7fac95772f4b95d989cf6cd3861517 Carlito-BoldItalic.ttf
  fetch_font_file "$GF/carlito/OFL.txt" 58402f82a7c332a700294988fe7554fbb0a63a8d27ccc1ee3bbc640311990a00 OFL-Carlito.txt
  fetch_font_file "$GF/caladea/Caladea-Regular.ttf" f1e899278b7b4491aba5b6a8253c4b04c050cc59b21865be5c37559a775153cd Caladea-Regular.ttf
  fetch_font_file "$GF/caladea/Caladea-Bold.ttf" ae3cb2dcbc925809dd29d2a44e9802211cab66be541bacbfc9c08c74b27c3742 Caladea-Bold.ttf
  fetch_font_file "$GF/caladea/Caladea-Italic.ttf" 4359a8e24f748b6447b1ff6d7a174febe70961d29f8bb8634b56dacd740a3deb Caladea-Italic.ttf
  fetch_font_file "$GF/caladea/Caladea-BoldItalic.ttf" ccabaa7b7e2fdf253d2b1a5fa699dd8a3df8d835a9eb285ad82631a677eb76c0 Caladea-BoldItalic.ttf
  fetch_font_file "$GF/caladea/OFL.txt" ccdab61d371d8c8683a128a92cd7d498dbdb1d37689f7cb21f1bf6b16658d213 OFL-Caladea.txt
  # A Song (宋体) design for systems without Chinese fonts of their own (the tar.gz can't pull in
  # fonts-noto-cjk): Noto Serif SC, Regular and Bold (OFL), from Noto CJK Serif 2.003. Without it
  # 宋体 text fell back to 霞鹜文楷 (a LoongArch root with only DejaVu: 28 -> 30 pages).
  NCJK=https://raw.githubusercontent.com/notofonts/noto-cjk/Serif2.003/Serif
  fetch_font_file "$NCJK/SubsetOTF/SC/NotoSerifSC-Regular.otf" e8f396decc1f0963a016a989c3d8852e863d1350996f573860a80767c83a1cd3 NotoSerifSC-Regular.otf
  fetch_font_file "$NCJK/SubsetOTF/SC/NotoSerifSC-Bold.otf" 24693d48bdb9152f0a06b02af625638a1097abd6de4010ebba027f6e82710527 NotoSerifSC-Bold.otf
  fetch_font_file "$NCJK/LICENSE" 6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2 OFL-NotoSerifCJK.txt
  mkdir -p "$FONTS" "$NOTICES"
  unzip -q -o -j "$FONT_CACHE/ZhuqueFangsong-v0.212.zip" 'ZhuqueFangsong-Regular.ttf' -d "$FONTS"
  install -m644 "$FONT_CACHE/LXGWWenKaiGB-Regular.ttf" "$FONTS/"
  install -m644 "$FONT_CACHE/OFL-ZhuqueFangsong.txt" "$FONT_CACHE/OFL-LXGWWenKaiGB.txt" "$NOTICES/"
  tar -xzf "$FONT_CACHE/liberation-fonts-ttf-2.1.5.tar.gz" -C "$FONTS" --strip-components=1 --wildcards 'liberation-fonts-ttf-2.1.5/*.ttf'
  tar -xzf "$FONT_CACHE/liberation-fonts-ttf-2.1.5.tar.gz" -O liberation-fonts-ttf-2.1.5/LICENSE > "$NOTICES/OFL-Liberation.txt"
  install -m644 "$FONT_CACHE"/Carlito-*.ttf "$FONT_CACHE"/Caladea-*.ttf "$FONTS/"
  install -m644 "$FONT_CACHE/OFL-Carlito.txt" "$FONT_CACHE/OFL-Caladea.txt" "$NOTICES/"
  install -m644 "$FONT_CACHE/NotoSerifSC-Regular.otf" "$FONT_CACHE/NotoSerifSC-Bold.otf" "$FONTS/"
  install -m644 "$FONT_CACHE/OFL-NotoSerifCJK.txt" "$NOTICES/"
}
