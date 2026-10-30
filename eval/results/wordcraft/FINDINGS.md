# WordCraft — evaluation (2026-10-30)

## Build and corpus run (with user approval, Rust 1.99.1)
- `cargo build --release -p wordcraft-cli -p wordcraft` **succeeded in 12m05s**, no errors (`build.log`). Binaries: wordcraft.exe (77 MB) and wordcraft-cli.exe (39 MB).
- **Tests not completed:** the core crate test run (layout, docx, fonts, doc) was stopped by Claude Code because the system ran low on memory. It was not a test failure. Re-run on request.

| File | Result |
|---|---|
| 01_gongwen.docx | 1 page, layout 2.8 s on first run (font scan). **All Chinese renders, no tofu.** Red header, centred document number, first-line indent of 2 chars, 27 chars/line at 16pt, fixed 28.95pt line spacing and the right-aligned signature are all correct. ❌ **Font mapping:** 仿宋_GB2312 and 方正小标宋 fell back to **Microsoft YaHei (a sans)**, although FangSong (simfang.ttf) is installed. The PDF embeds only ArialMT and MicrosoftYaHei. For 公文 this is the #1 visible defect. |
| 02_mixed_cjk.docx | 4 pages, 0.12 s. Header on every page ✅, mixed CJK/Latin with automatic spacing ✅, kinsoku ✅, numbered and bulleted lists ✅, merged table cell ✅, page break ✅, paragraphs split across pages ✅. Minor: curly quotes “” drawn in the Latin font (half-width) instead of full-width CJK. |
| PDF export | Both files export; fonts are embedded and subset, and the Chinese text extracts from the PDF. |
| DOCX round-trip | Paragraph text 9/9 and 18/18 identical, table identical, header kept, eastAsia font names (仿宋_GB2312, 方正小标宋简体) preserved. |

Outputs: `png/`, `*.pdf`, `*_rt.docx`.

## Static review (source + GitHub)

## Facts
- Repo: storytold/wordcraft · **Apache-2.0 / MIT** · created 2026-10-07 · ~16.7k★, 4.5k forks, ~2.2k open issues · stage **alpha v0.4.0** with signed builds for Win/macOS/Linux/BSD/web.
- Built by Claude (Opus 5.5) agents under the ArtCraft "craftrules" process, with human contributors on top (top: 514 / 192 / 159 commits).
- Rust only, **~338k lines total** (the README's "~88.7k" is out of date or counts something narrower). Breakdown: ui-egui 83k, engine 74k, docx 38k, layout 36k, doc 24k, formats 23k, proof 16k, docbin (.doc) 9k, fonts 7k, pdf 4.5k, render 4.5k.
- Architecture layers: geom → doc/fonts → layout/docx/formats → render (vello_cpu)/pdf (krilla)/proof → engine (433-command registry, undo) → MCP server / CLI → **egui** UI (described as swappable).
- Key deps (all permissive): harfrust (shaping), skrifa, unicode-linebreak/bidi/segmentation, quick-xml, zip, cfb, vello_cpu, egui 0.37. Two git deps on `storytold/craft-libs` (craft-pdf, craft-update).
- Quality signals: **4,711 `#[test]`**, proptest, only 4 `unsafe` mentions (`unsafe_code = "forbid"`), 0 TODO/unimplemented. The crate roots deny unwrap/expect/panic. It has a strict clean-room policy (no GPL code, no MS assets) and SonarCloud + CodeRabbit review.
- Self-assessed readiness (ROADMAP, 2026-10-10): **~60% "ready for real work"**. Layout/pagination fidelity 55%, file formats 60%. **Never tested on a real-world DOCX corpus.** Charts, SmartArt and OLE objects are dropped silently, and there is no metric-compatible Aptos font.

## CJK readiness (code + docs)
| Feature | Status |
|---|---|
| eastAsia font slot (`w:rFonts/@eastAsia`), 宋体 ↔ SimSun localized names | ✅ |
| Kinsoku (避头尾) | ✅ basic (`crates/layout/src/kinsoku.rs`, 152 lines). The NO_START list misses some Chinese closers (” ’ … —) |
| autoSpaceDE/DN (中西文间距) | 🟡 |
| Document grid (`docGrid`: lines/page, chars/line, needed for 公文 28×22) | 🟡 landed #391 |
| Vertical text (tbRl), ruby (拼音指南), enclosed chars (带圈字符) | ✅ landed |
| CJK list numbering (一、 第一章 ①) | ✅ |
| zh-Hans UI translation | ✅ ~95% |
| **Punctuation compression (标点挤压)** | ❌ not found |
| **Chinese font fallback for 仿宋_GB2312 / 方正小标宋 / 楷体_GB2312** | ❌ no explicit mapping (only a general installed-font fallback) |
| GB/T 9704 公文 templates, OFD | ❌ |
| Chinese user bugs | open issues: web can't display Chinese, overlap during IME input, inconsistent justification in mixed CJK/Latin, Chinese glyphs in TOC fields |

## Verdict (after the corpus run)
| Fidelity | CJK | Code quality | Extensibility | Health | License |
|---|---|---|---|---|---|
| 4 | 3.5 | 4.5 | 4 (command registry, MCP, swappable UI) | 4.5 (very active; 3 weeks old, governance unproven) | 5 |

**Recommendation: contribute + downstream distribution**, not a hard fork. Its architecture, test discipline and license fit Wenshu's plan almost exactly, and it is far ahead of anything we could build in six months. Wenshu should add the Chinese layer upstream: 标点挤压, Chinese closer kinsoku, 公文 font mapping, GB/T 9704 templates, and a Chinese DOCX corpus with CI, which is its own #1 beta gap. Separately, build OFD export plus a Chinese-localized distribution.
