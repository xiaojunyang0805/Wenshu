# Working with upstream engines

## Pinned engines (`upstream.lock`)

| Engine | Repo | Revision | Licence |
|---|---|---|---|
| WordCraft | storytold/wordcraft | `3b7f7ed` (2026-10-30, v0.12.0) | MIT OR Apache-2.0 |

## Workflow

```sh
scripts/sync-upstream.sh              # clone/reset upstream/<engine> at the pin, apply patches/<engine>/*.patch
# edit upstream/wordcraft/… , build and test through Wenshu
scripts/sync-upstream.sh --export     # write the diff to patches/<engine>/0001-wenshu.patch.new, then split by topic
```

Bumping a pin means updating `upstream.lock`, re-running the sync, re-basing any patch that no
longer applies, and running the fidelity corpus before and after.

## Patch rules

1. **One topic per patch**, named `NNNN-crate-topic.patch`, with a header line saying why it's
   needed and the upstream PR link once opened.
2. **Upstream-ready:** follows the engine's AGENTS.md: no panics, no `unsafe`, a test, docs comment,
   clean room, and no Chinese data tables (data stays in `wenshu-cjk`).
3. **Generic:** a hook other languages could use too. A Chinese-only rule goes in Wenshu.
4. **Sent upstream within a week**, and dropped here once merged. More than 10 open patches on one
   engine is a signal to stop and talk to the maintainers.

## Patch queue

| Patch | Crate | What | Upstream |
|---|---|---|---|
| 0001-fonts-substitute-provider-hook | wordcraft-fonts | `SubstituteProvider` + `set_substitute_provider()`: named substitutes from the embedding app, tried before the built-in lists | not yet opened |
| 0002-layout-grid-cells-and-char-indents | wordcraft-layout | Character grid: text larger than the default size takes `ceil` cells (was `round`: 22 pt glyphs squeezed into one 15.8 pt cell and overlapped); `w:firstLineChars`/`leftChars`/`rightChars` count grid cells on a character grid (a 公文 first line held 25 + indent instead of 26 + indent). 2 tests | not yet opened |
| 0003-east-asian-hint-and-named-ea-substitutes | doc, docx, layout | Reads and writes `w:rFonts/@w:hint`; with `hint="eastAsia"`, ambiguous characters (“ ” ‘ ’ — … · ° …) use the East Asian font (ECMA-376 §17.3.2.26). An East Asian font substituted *by name* (仿宋_GB2312 → FangSong) is kept instead of dropped to generic fallback. Round-trip + layout tests | not yet opened |
| 0004-layout-kinsoku-curly-quotes | wordcraft-layout | Kinsoku sets gain ” ’ 〃 〆 ‰ ℃ (no line start) and “ ‘ (no line end), needed once break opportunities beside quotes exist (Unicode 15.1 LB19a, kinsoku-off paths). Test | not yet opened |
| 0005-layout-punctuation-compression | doc, docx, layout | `Settings::char_spacing` (`w:characterSpacingControl`) is read and written; it was always saved as `doNotCompress`, so a WPS document's `compressPunctuation` was lost on save. With compression on, adjacent full-width marks share one blank half (`。”` `）、` `”（` `《“`; JIS X 4051, clreq §3.1.6), only where the mark's advance is ≥ 0.9 em. Text on a character grid keeps its cells, as with auto-spacing. Auto-spacing and compression share one neighbour walk. 4 layout + 1 docx test, round-trip extended | not yet opened |

All five apply in order on `3b7f7ed`; the `wordcraft-layout` (571), `wordcraft-doc` (242) and
`wordcraft-docx` (500) suites pass with them, and clippy is clean on doc, docx, layout and engine.

## Issues found (status)

- ✅ Character grid: 27 instead of 28 characters per line; large text overlapping. Fixed by 0002.
  The cell rule for larger text (`ceil(size / default size)`) follows Word's observed spreading of
  large text on a grid; confirm against a Word/WPS reference render once `corpus/ref/` exists.
- ✅ Curly quotes drawn half-width in hinted East Asian runs. Fixed by 0003. Unhinted runs keep the
  Latin font, as the spec says.
- ✅ Kinsoku for curly quotes. Fixed by 0004. Dashes and ellipses may start a line (GB/T 15834:
  they only must not be split; UAX #14 already keeps —— and …… together).
- ✅ Punctuation compression for adjacent marks. Done by 0005. Still open: compression at the
  line start (`w:topLinePunct`) and line end, and whether Word or WPS compress on a
  `linesAndChars` grid (0005 says no, so 公文 bodies are unchanged). Both need reference renders.
