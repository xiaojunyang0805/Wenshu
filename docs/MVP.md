# Wenshu MVP

*v0.3 · 2026-11-04* (v0.2 2026-11-03, v0.1 2026-10-30); milestones updated 2026-11-04 evening

## Goal

**A free, offline word processor that a Chinese government or enterprise clerk can use to open,
write, check and export a GB/T 9704 official document (公文), with the right fonts and layout.**
Writer only. Slides and Sheets come after the MVP.

## Users and core jobs

| # | Job | MVP acceptance | Status (2026-11-04) |
|---|---|---|---|
| J1 | Open a .docx received from WPS/Word and read it correctly | ≥ 90% of corpus docs open without error; FangSong/Kai/Song/Hei never fall back to sans; page count matches the WPS reference on ≥ 80% | **Met.** 200 / 200 open without error. Page count matches on **186 / 200 (93%)** on Windows, mean page score 0.943 (2026-11-04; baseline 2026-10-31: 112 / 200, 0.903), and on **173 / 200** on Linux with the packaged fonts (mean 0.923; see open question 2). Fonts: 仿宋/楷体/宋体/黑体 and the Traditional ones (標楷體, 新細明體) map to their designs (on Linux: 朱雀仿宋, 霞鹜文楷 GB, Noto Serif CJK, Noto Sans CJK); Chinese text with no font of its own falls back to 宋体, as in WPS. The 14 documents still a page off on Windows are analysed in docs/PROGRESS.md (2026-11-04); that work is now in maintenance mode (see Next) |
| J2 | Write a new 公文 from a template | `wenshu gongwen new` (CLI) and File › New › 公文 (desktop) produce a document that passes all 8 `gongwen check` rules | **Met.** Both paths; a test checks the five templates pass |
| J3 | Check a 公文 against the standard | `wenshu gongwen check` reports page, margins, type area, grid, body font and size | **Met** (CLI and 检查公文格式 on the ribbon) |
| J4 | Export PDF to send or print | PDF with embedded subset fonts, selectable Chinese text | **Met, measured 2026-11-03** (`corpus/tools/pdf_check.py`): 200 / 200 PDFs written, page counts equal to Wenshu's layout 200 / 200; every font embedded and subset 200 / 200; text extractable: mean recall 0.9986 (Chinese 0.9984), all characters in 169 / 200, the rest above 0.995 except 7 (a vertical text box, hollow text, a few others); pages look like the screen (worst page > 0.99 in 198 / 200; the two others show screen-only markup, absent from WPS's PDF too); median 0.2 s and 164 KB. Found on the way: mixed-font field results drew wrong glyphs and broke font embedding (patch 0037). Measured at patch 0038; not re-measured since (patches 0039–0064 change layout and fonts, not PDF writing) |
| J5 | Save back to .docx that WPS opens unchanged | Text, tables, headers and East Asian font names survive a round-trip (100% on corpus) | **Measured 2026-11-03** (`corpus/tools/roundtrip.py`): WPS opens all 200 saves; text identical **199 / 200** (body, headers, footers, notes; recalculated fields such as page numbers left out), tables 200 / 200, East Asian font names 199 / 200; page count unchanged 197 / 200, mean page score saved vs original 0.993, 161 pixel-identical. The one text loss: a text box in a picture-filled shape (`aae80ea709`), read as a picture. Found on the way: an invisible WordArt watermark became visible after a save (patch 0039). Not yet 100%; not re-measured since patch 0039 |

## Milestones

| M | Content | Effort (agents + 1–2 engineers) | Status |
|---|---|---|---|
| **M0 Foundations** | Workspace, patch queue, `wenshu-core`, CJK font hook, 公文 generator and checker, fidelity harness, CLI | 1 day | ✅ **done 2026-10-30** |
| **M1 Chinese layout** | Upstream PRs: grid fixes (28 chars/line) ✅, full-width quotes ✅, kinsoku ✅, punctuation compression ✅. Grow the corpus to 200 real documents with WPS reference renders ✅. Page counts within J1's 80% ✅ | 3–5 weeks | ✅ **done 2026-11-03**, continued to 2026-11-04. Corpus of 200 with WPS references (2026-10-31). Patch queue 0002–0064, the layout rules measured in WPS first (docs/WPS-FINDINGS.md F1–F43: grids, punctuation squeeze and hanging, autospace, East Asian line height, font fallback, embedded fonts per document, tables, sections). Page counts 112 → **186 / 200** on Windows (mean 0.903 → **0.943**), **173 / 200** on Linux with the packaged fonts. The remaining 14 were analysed (docs/PROGRESS.md, groups A–J): single-document quirks, so maintenance only from here. Upstream: drafts #5002–#5006, #5327, #5462, #5463, #5470 open, no human review yet; most later patches are WPS-measured and some depart from Word (docs/UPSTREAM.md), so not opened |
| **M2 Desktop alpha** | Wenshu desktop build (WordCraft UI with `init()`, zh-Hans default, branding), 公文 templates in File › New, "Check 公文" command, Windows/Linux (incl. LoongArch) installers | 3–4 weeks | ✅ **alpha 2026-10-31**, re-checked 2026-11-04. `wenshu-desktop`: zh-Hans or English UI, 文枢 brand, own settings folder, 公文 templates in File › New, 检查公文格式 on the ribbon. Re-checked on the current code (patches through 0064, through the control channel): 13 / 13 pass (zh-Hans start-up; the 5 templates seeded and each passing all 8 GB/T 9704 checks; three corpus documents with the same page count as the CLI; typing, Save As .docx, reopen; clean quit). Chinese blank documents now match WPS's own (Calibri for Latin text, punctuation compression, compatibility mode 14; measured 2026-11-04, on `main` after v0.1.3). Installers: Windows portable zip and per-user MSI, Linux tar.gz/.deb/AppImage (x86_64), all install-tested 2026-11-02; LoongArch tar.gz/.deb (new-world, glibc ≥ 2.36) cross-built and tested under qemu 2026-11-02; the Linux packages bundle 朱雀仿宋, 霞鹜文楷 GB and the metric-compatible Latin fonts. CI builds them all into a draft release on a tag. Releases: v0.1.0 (patches through 0007) and v0.1.2 (through 0038) published as pre-releases 2026-11-03, v0.1.1 left a draft; **v0.1.3** (through 0064) tagged 2026-11-04, built by CI into a draft (8 assets, checksums verified); install-tested the same day with the 13 desktop checks: Windows portable zip and per-user MSI, Linux .deb, tar.gz and AppImage (WSL Ubuntu 22.04, 22 bundled fonts) all 13 / 13; the MSI also upgrades v0.1.2 in place (one entry, 13 / 13, clean uninstall); the installed Linux package embeds its bundled fonts (Carlito, Caladea, 朱雀仿宋) with no setup. LoongArch desktop under qemu: 11 / 13 (start-up, templates, checks, save and reopen pass; one document is 2 pages longer because the emulated root has no Chinese system fonts, so 宋体 falls back to the bundled 霞鹜文楷; the quit check times out under emulation). Published as a pre-release 2026-11-04. Since v0.1.3 the Linux packages also bundle a Song fallback (Noto Serif SC Regular and Bold, OFL) for systems with no Chinese fonts: in the LoongArch root with only DejaVu that document is back to 28 pages, as on x86_64 (fonts folder 41 → 64 MB). Editing pass 2026-11-04 (scripted through the control channel, the commands the keyboard and ribbon use): 18 / 18: a Chinese blank document saves with WPS's settings; typing with Enter, Backspace, undo, redo and undo back to empty; bold, 黑体, 16 pt and centring on a selection only; a 公文 template edited (title replaced, a paragraph added) still passes 检查公文格式 in the app and the CLI, and WPS lays the saved file out with the app's page count. Still open: real LoongArch hardware; typing through a real Chinese IME (needs a person at the keyboard) |
| **M3 Formats** | OFD export (`wenshu-ofd`) for the MVP; after the MVP, DeckCraft slides with the same font hook and IronCalc zh-CN locale (Goal: Writer only first) | 4–6 weeks | **OFD export done 2026-11-05** (`wenshu-ofd` + `wenshu-core::ofd`, patch 0065): CLI, Save As and File › Export; text as text in embedded subset fonts with their own cmap; checked in WPS's OFD reader; corpus: 200 / 200 convert with the PDF's page count, character recall 1.0000, 0.9991 of characters within 1 pt of the PDF's; outline (the PDF's entries in 200 / 200) and links (link annotations, followed in WPS). Slides and sheets after the MVP |
| **M4 Public beta** | Web build, signed releases, public fidelity scoreboard | 2–3 weeks | Not started. The fidelity rescore as a regression gate (Next, 4) is the first step toward the scoreboard; WPS renders can't be published yet (open question 1) |

Token budget (from the feasibility model): M1–M4 come to roughly **0.3–1B tokens, about
$500–1,500 on Opus 5.5**, because most of the engine already exists.

## Next (decided 2026-11-04)

J1–J4 are met and J5 nearly. The page-count work on the corpus has reached the long tail (one
document per patch, some fixes reverted), so it is in maintenance mode: broad, cheaply measured
rules or problems from real documents only. In order:

1. **Linux fidelity**: measured, 115 → 173 / 200 (open question 2); next, the 12 documents
   still running long on Linux.
2. **Desktop editing pass**: scripted pass done 2026-11-04 (18 / 18, M2); still to do by hand:
   typing through a Chinese IME.
3. **OFD export** (M3): done 2026-11-05 (CLI, Save As, File › Export; checked in WPS's OFD
   reader and across the corpus), with outline and links.
4. **Regression gate**: the fidelity rescore, Windows and Linux, run before each release so the
   page counts can't slip unnoticed.

Then the business plan review.

## Out of scope for the MVP

Real-time collaboration, macros, cloud accounts, mobile, AI assistant, legacy .wps files, printing
beyond what upstream offers.

## Open questions

1. **Reference renders:** ~~who has WPS and Word licences?~~ A licensed WPS Office 12.1 is on the dev PC
   (2026-10-30) with COM automation, so renders can be made locally. Still open: a Word reference,
   and whether WPS renders may be shared publicly (until then `corpus/ref/` stays local, per
   `.gitignore`). Upstream targets Word, so most WPS-measured patches (0008–0064) need Word
   evidence or a spec argument before they can be offered as PRs; some (0053, 0056) knowingly
   depart from Word.
2. **Fonts:** ~~bundle open FangSong/Kai designs, or rely on the system fonts?~~ Both (2026-11-02): the
   Linux packages ship 朱雀仿宋 Zhuque Fangsong and 霞鹜文楷 GB (OFL-1.1, unmodified,
   `/usr/share/wenshu/fonts`); Windows uses its own FangSong and KaiTi. **Linux measured
   2026-11-04** (WSL Ubuntu 22.04: Noto CJK, DejaVu, the packaged fonts, no Windows fonts; same
   corpus and WPS references):

   | Linux setup | Page matches | Mean |
   |---|---|---|
   | Packaged 朱雀仿宋 and 霞鹜文楷 GB, queue through 0063 | 115 / 200 | 0.911 |
   | + patch 0064: missing Windows Chinese faces keep their own line box (Noto CJK's is taller) | 150 / 200 | 0.915 |
   | + metric-compatible Latin fonts (Liberation 2, Carlito, Caladea) | 167 / 200 | 0.921 |
   | + 0064 extended: missing fonts drawn as a Windows face (新細明體 as 宋体) take its box | **173 / 200** | **0.923** |
   | Windows, for comparison | 186 / 200 | 0.943 |

   Every package now bundles the Latin fonts as well (OFL, SHA-256-pinned, 2026-11-04; the .tar.gz
   and AppImage can't pull them in as dependencies); a run with only Wenshu's own font folder and
   no system Latin fonts gives the same 173 / 200. A development build finds the packaged fonts only with
   `WENSHU_FONTS_DIR`. A Song fallback (Noto Serif SC) is bundled too, for systems with no Chinese
   fonts. Still open: no open 小标宋 (titles fall back to Noto Serif CJK), Zhuque's
   `×` and curly quotes are narrower than Windows FangSong's, Cambria Math, and the 12 documents
   still running long on Linux.
3. **Governance:** work as a CJK working group inside the storytold/ArtCraft community, or as a
   separate organisation? This affects branding and CLA.
4. **Engine risk:** WordCraft is young (a few weeks old when Wenshu pinned it on 2026-10-30). If
   upstream stalls, the patch queue (64 patches by 2026-11-04) becomes a fork; none has been
   merged upstream yet. `wenshu-core` keeps that cost contained.
