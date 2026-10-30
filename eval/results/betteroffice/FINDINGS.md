# BetterOffice — evaluation (2026-10-30)

Evaluated at commit `1b20b2e` (v0.5.1). Artifacts (logs, PNGs, check files, eval host source) are in this folder.

**Recommendation: reuse crates and contribute selectively; don't fork.** The parse, round-trip, xlsx-calc and text crates are strong. But about 700k lines (Rust + TS), one maintainer and agent-speed change make a fork impossible to own, and the CJK gaps sit in the layout core.

## Architecture
- Each format follows the same pipeline: `<fmt>-parse` (lossless) → `<fmt>-edit` (Yrs CRDT) → layout/display list → raster (tiny-skia) → crates.io facade.
- Formats: DOCX, XLSX, PPTX, VSDX across 42 crates.
- Shared crates: `ooxml-opc`, `ooxml-drawingml` (incl. charts), `ooxml-text` (rustybuzz/skrifa, UAX-14 + kinsoku, a 24k-line Office font-metric table), `ooxml-metafile`, `ooxml-diff`, `ooxml-redact`.
- `docx-layout` is about 37k lines: keep-with-next, widow control, column balancing, footnotes, docGrid line-pitch snapping, incremental relayout.
- Packaging: WASM npm, N-API, PyO3 wheels, MCP server. UI: React editors plus macOS-only desktop and viewer apps; **no Windows or Linux desktop**.
- License Apache-2.0 **with a CLA**. Dependencies are permissive only. Fonts are OFL.
- The README's SSIM fidelity numbers measure the **browser** renderer, not the native one; the project's own docs say so.

## Size
- 918,654 `.rs` lines in total. Tests are 384k of that, and 41k is a vendored copy of yrs.
- About **400k lines of real crate code**, plus about 296k lines of TypeScript (not counting tests).
- Largest files: `docx-edit/src/engine.rs` at 32.7k lines and `display_list.rs` at 17.5k.
- 9,832 tests and 99 `unsafe` blocks in source.
- 1,529 of 1,763 commits are by one author, so the bus factor is 1. The code is likely agent-written; that is an inference.

## Build and tests
- **Fails on Rust 1.94.1**: yrs uses if-let guards, and `xlsx-render` uses an API unstable before 1.96. No minimum Rust version is declared.
- Built with a side-by-side **Rust 1.99.1** toolchain that the evaluation agent installed: 6m16s, 0 warnings.
- Tests: 1,814 passed, 5 failed, 2 ignored. The 5 failures are golden-file comparisons broken by CRLF line endings from Windows `core.autocrlf`, not engine bugs.

## Corpus
| File | Result |
|---|---|
| 01_gongwen.docx | 1 page, good quality. 仿宋 maps to SimFang; 方正小标宋 is missing, so SimSun is used. 27 chars/line, indent and signature correct |
| 02_mixed_cjk.docx | 4 pages. Mixed scripts, lists, merged cell, page break and kinsoku correct. **Header never painted** on the native path |
| 03_sales.xlsx | **34/34 formulas correct** after recalculation. The CLI raster draws **CJK as tofu** (no font API), and the **chart is empty** (it reads cached values only) |
| 04_deck.pptx | 3 slides render well; saving keeps all text |
| DOCX edit/round-trip | The edit API needs `w14:paraId`, so python-docx files can't be edited until IDs are injected; then edits round-trip cleanly. An unedited save drops `rsid` attributes and re-serializes the header, so the README's "byte-identical" claim doesn't hold |

## CJK
- **Supported:** eastAsia font slots and hints, CJK–Latin auto-spacing, docGrid line-pitch snapping.
- **Minimal:** kinsoku is one hard-coded list; the `w:kinsoku`, `overflowPunct` and `topLinePunct` settings are ignored.
- **Missing:**
  - punctuation compression (标点挤压)
  - the character grid: `charSpace` is parsed but unused, which matters for 公文
  - ruby
  - true vertical text: `eaVert` is treated as a rotation
  - `.ttc` font collections
- **Fonts:** CJK fonts are Regular weight only, and FangSong/KaiTi map to Noto Serif SC.

## Scores
| Fidelity | CJK | Code quality | Extensibility | Health | License |
|---|---|---|---|---|---|
| 3.5 | 2.5 | 3.5 | 3.5 | 2.5 | 4.5 |

## Univer (desk review only)
- A TypeScript plugin/DI monorepo with a canvas renderer, a formula engine and a headless Node runtime. Its document model is its own JSON, not OOXML.
- **Import/export of docx, xlsx and pptx is in the paid "Pro" tier,** as are collaboration, printing, charts and pivot tables. The open-source packages contain no OOXML parser.
- Fit for Wenshu: an optional web front end for Sheets at most, and only with our own OOXML↔JSON bridge. It is a weak fit for Docs.
