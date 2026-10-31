# Wenshu MVP

*v0.1 · 2026-10-30*

## Goal

**A free, offline word processor that a Chinese government or enterprise clerk can use to open,
write, check and export a GB/T 9704 official document (公文), with the right fonts and layout.**
Writer only. Slides and Sheets come after the MVP.

## Users and core jobs

| # | Job | MVP acceptance |
|---|---|---|
| J1 | Open a .docx received from WPS/Word and read it correctly | ≥ 90% of corpus docs open without error; FangSong/Kai/Song/Hei never fall back to sans; page count matches the WPS reference on ≥ 80% |
| J2 | Write a new 公文 from a template | `wenshu gongwen new` (CLI) and File › New › 公文 (desktop) produce a document that passes all 8 `gongwen check` rules |
| J3 | Check a 公文 against the standard | `wenshu gongwen check` reports page, margins, type area, grid, body font and size |
| J4 | Export PDF to send or print | PDF with embedded subset fonts, selectable Chinese text |
| J5 | Save back to .docx that WPS opens unchanged | Text, tables, headers and East Asian font names survive a round-trip (100% on corpus) |

## Milestones

| M | Content | Effort (agents + 1–2 engineers) | Status |
|---|---|---|---|
| **M0 Foundations** | Workspace, patch queue, `wenshu-core`, CJK font hook, 公文 generator and checker, fidelity harness, CLI | 1 day | ✅ **done 2026-10-30** |
| **M1 Chinese layout** | Upstream PRs: grid fixes (28 chars/line) ✅, full-width quotes ✅, kinsoku ✅, punctuation compression ✅. Grow the corpus to 200 real documents with WPS reference renders ⏳ | 3–5 weeks | **in progress** (patches 0002–0005 done; upstream PRs #5002–#5006 through 4 bot review rounds, awaiting maintainers; corpus 4/200, reference renders not started) |
| **M2 Desktop alpha** | Wenshu desktop build (WordCraft UI with `init()`, zh-Hans default, branding), 公文 templates in File › New, "Check 公文" command, Windows/Linux (incl. LoongArch) installers | 3–4 weeks | |
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
   `.gitignore`).
2. **Fonts:** bundle open FangSong/Kai designs (Fandol is GPL with a font exception; LXGW WenKai is
   OFL) in the installer, or rely on the system fonts?
3. **Governance:** work as a CJK working group inside the storytold/ArtCraft community, or as a
   separate organisation? This affects branding and CLA.
4. **Engine risk:** WordCraft is 3 weeks old. If upstream stalls, the patch queue becomes a fork.
   `wenshu-core` keeps that cost contained.
