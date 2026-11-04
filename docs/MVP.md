# Wenshu MVP

*v0.2 · 2026-11-03* (v0.1 2026-10-30)

## Goal

**A free, offline word processor that a Chinese government or enterprise clerk can use to open,
write, check and export a GB/T 9704 official document (公文), with the right fonts and layout.**
Writer only. Slides and Sheets come after the MVP.

## Users and core jobs

| # | Job | MVP acceptance | Status (2026-11-03) |
|---|---|---|---|
| J1 | Open a .docx received from WPS/Word and read it correctly | ≥ 90% of corpus docs open without error; FangSong/Kai/Song/Hei never fall back to sans; page count matches the WPS reference on ≥ 80% | **Met.** 200 / 200 open without error; page count matches on **186 / 200 (93%)**, mean page score 0.943 (2026-11-04; baseline 2026-10-31: 112 / 200, 0.903). Fonts: 仿宋/楷体/宋体/黑体 and the Traditional ones (標楷體, 新細明體) map to their designs; Chinese text with no font of its own falls back to 宋体, as in WPS |
| J2 | Write a new 公文 from a template | `wenshu gongwen new` (CLI) and File › New › 公文 (desktop) produce a document that passes all 8 `gongwen check` rules | **Met.** Both paths; a test checks the five templates pass |
| J3 | Check a 公文 against the standard | `wenshu gongwen check` reports page, margins, type area, grid, body font and size | **Met** (CLI and 检查公文格式 on the ribbon) |
| J4 | Export PDF to send or print | PDF with embedded subset fonts, selectable Chinese text | **Met, measured 2026-11-03** (`corpus/tools/pdf_check.py`): 200 / 200 PDFs written, page counts equal to Wenshu's layout 200 / 200; every font embedded and subset 200 / 200; text extractable: mean recall 0.9986 (Chinese 0.9984), all characters in 169 / 200, the rest above 0.995 except 7 (a vertical text box, hollow text, a few others); pages look like the screen (worst page > 0.99 in 198 / 200; the two others show screen-only markup, absent from WPS's PDF too); median 0.2 s and 164 KB. Found on the way: mixed-font field results drew wrong glyphs and broke font embedding (patch 0037) |
| J5 | Save back to .docx that WPS opens unchanged | Text, tables, headers and East Asian font names survive a round-trip (100% on corpus) | **Measured 2026-11-03** (`corpus/tools/roundtrip.py`): WPS opens all 200 saves; text identical **199 / 200** (body, headers, footers, notes; recalculated fields such as page numbers left out), tables 200 / 200, East Asian font names 199 / 200; page count unchanged 197 / 200, mean page score saved vs original 0.993, 161 pixel-identical. The one text loss: a text box in a picture-filled shape (`aae80ea709`), read as a picture. Found on the way: an invisible WordArt watermark became visible after a save (patch 0039). Not yet 100% |

## Milestones

| M | Content | Effort (agents + 1–2 engineers) | Status |
|---|---|---|---|
| **M0 Foundations** | Workspace, patch queue, `wenshu-core`, CJK font hook, 公文 generator and checker, fidelity harness, CLI | 1 day | ✅ **done 2026-10-30** |
| **M1 Chinese layout** | Upstream PRs: grid fixes (28 chars/line) ✅, full-width quotes ✅, kinsoku ✅, punctuation compression ✅. Grow the corpus to 200 real documents with WPS reference renders ✅. Page counts within J1's 80% ✅ | 3–5 weeks | **done** (2026-11-03): corpus of 200 with WPS references (2026-10-31); patch queue 0002–0032, every rule measured in WPS first (docs/WPS-FINDINGS.md F1–F21: grids, punctuation squeeze, autospace, East Asian line height, font fallback, embedded fonts per document); page counts 112 → **166 / 200**, mean 0.903 → **0.935** (one process per document, 2026-11-03). Upstream: #5002–#5006 updated with 0011–0015, 0016 opened as draft #5327; 0017–0032 not opened yet. J1's 80% reached; left: 37 documents with a different page count |
| **M2 Desktop alpha** | Wenshu desktop build (WordCraft UI with `init()`, zh-Hans default, branding), 公文 templates in File › New, "Check 公文" command, Windows/Linux (incl. LoongArch) installers | 3–4 weeks | **alpha 2026-10-31**: `wenshu-desktop` runs (zh-Hans or English UI, 文枢 brand, own settings folder, Chinese blank documents in a Chinese region, 公文 templates in File › New, 检查公文格式 on the ribbon), Windows portable zip and per-user MSI, Linux tar.gz/.deb/AppImage (x86_64), all install-tested 2026-11-02; LoongArch tar.gz/.deb (new-world, glibc ≥ 2.36) cross-built and tested under qemu 2026-11-02; CI builds all of them into a draft release on a tag; **v0.1.0** published as a pre-release, **v0.1.1** (the fidelity work through patch 0018) built as a draft; real LoongArch hardware still untested |
| **M3 Formats** | OFD export (`wenshu-ofd`); DeckCraft slides with the same font hook; IronCalc zh-CN locale | 4–6 weeks | |
| **M4 Public beta** | Web build, signed releases, public fidelity scoreboard | 2–3 weeks | |

Token budget (from the feasibility model): M1–M4 come to roughly **0.3–1B tokens, about
$500–1,500 on Opus 5.5**, because most of the engine already exists.

## Out of scope for the MVP

Real-time collaboration, macros, cloud accounts, mobile, AI assistant, legacy .wps files, printing
beyond what upstream offers.

## Open questions

1. **Reference renders:** ~~who has WPS and Word licences?~~ A licensed WPS Office 12.1 is on the dev PC
   (2026-10-30) with COM automation, so renders can be made locally. Still open: a Word reference,
   and whether WPS renders may be shared publicly (until then `corpus/ref/` stays local, per
   `.gitignore`). Upstream targets Word, so most WPS-measured patches (0008–0030) need Word
   evidence or a spec argument before they can be offered as PRs.
2. **Fonts:** ~~bundle open FangSong/Kai designs, or rely on the system fonts?~~ Both (2026-11-02): the
   Linux packages ship 朱雀仿宋 Zhuque Fangsong and 霞鹜文楷 GB (OFL-1.1, unmodified,
   `/usr/share/wenshu/fonts`); Windows uses its own FangSong and KaiTi. Still open: no open 小标宋
   (titles fall back to Noto Serif CJK), and Zhuque's `×` and curly quotes are narrower than
   Windows FangSong's full-width ones. **Linux measured 2026-11-04** (WSL Ubuntu 22.04: Noto
   CJK, DejaVu, the packaged 朱雀仿宋 and 霞鹜文楷 GB, no Windows fonts; same corpus and WPS
   references): 115 / 200 page matches at first (mean 0.911; Windows 186, 0.943), nearly all
   documents running long. Patch 0064 keeps the missing Windows Chinese faces' own line box
   (Noto CJK's is taller): 150 / 200. With the metric-compatible Latin fonts (Liberation 2,
   Carlito, Caladea), now Recommends of the .deb: 167 / 200; with missing fonts drawn as a Windows
   face (新細明體 as 宋体) taking that face's box too: **173 / 200, mean 0.923** (Windows 186).
   Still open: Cambria Math, the other 12 documents running long, and bundling the Latin fonts
   in the tar.gz and AppImage.
3. **Governance:** work as a CJK working group inside the storytold/ArtCraft community, or as a
   separate organisation? This affects branding and CLA.
4. **Engine risk:** WordCraft is 3 weeks old. If upstream stalls, the patch queue becomes a fork.
   `wenshu-core` keeps that cost contained.
