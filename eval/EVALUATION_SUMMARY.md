# Wenshu kit evaluation — summary (2026-10-30)

**Corpus:** `corpus/files/` holds 4 Chinese-heavy files: 公文 (official document) docx, mixed CJK docx, an xlsx with formulas and a chart, and a pptx. Per-kit details are in `results/*/FINDINGS.md`.

## Scoreboard (1–5)

| Kit | Role | Fidelity | CJK | Code | Extensibility | Health | License | Corpus result |
|---|---|---|---|---|---|---|---|---|
| **WordCraft** | Writer | 4 | 3.5 | 4.5 | 4 | 4.5 | 5 | ✅ both docx correct: layout, header, lists, table, PDF, lossless text round-trip. ❌ 仿宋/小标宋 fall back to YaHei (sans) |
| **DeckCraft** | Slides | 4 | 4 | 4 | 4 | 3 | 5 | ✅ Chinese renders correctly, lossless round-trip, CJK PDF |
| **GridCraft** | Sheets | 4 | 3 | 3 | 4 | 2 | 5 | ✅ 21/21 values; chart kept on save |
| **IronCalc** | Sheets engine | 3 | 3 | 4 | 4 | 4 | 5 | ✅ 21/21 values; chart dropped |
| **BetterOffice** | OOXML engines | 3.5 | 2.5 | 3.5 | 3.5 | 2.5 | 4.5 | ✅ 公文 page good, xlsx 34/34. ❌ header missing, xlsx CJK tofu |
| Univer | Web UI (TS) | — | — | — | — | — | Pro-gated | OOXML import/export is paid; poor fit |

WordCraft was built and run on 2026-10-30 with Rust 1.99.1. Its unit-test run was stopped because the system ran low on memory.

## Main findings
1. **AI agents are already building Rust office suites.** WordCraft, GridCraft and DeckCraft (ArtCraft) total about 760k lines of Rust in 3 weeks, are Apache/MIT licensed, and have strong test discipline (no panics, no `unsafe`, thousands of tests). BetterOffice is about 400k lines of real crate code in 4 months. This confirms the cost model in the first report. It also means **we should not start from zero**.
2. **No kit is Chinese-ready.** Common gaps:
   - **Typography:** punctuation compression (标点挤压); the character grid for the 公文 28×22 layout (partial in WordCraft, absent in BetterOffice); kinsoku coverage of Chinese closing punctuation.
   - **Fonts:** mapping for 仿宋_GB2312 and 方正小标宋.
   - **Spreadsheet functions:** LENB/`[DBNum]`/NUMBERSTRING.
   - **Locale:** zh-CN number formats.
   - **China-specific formats:** OFD; GB/T 9704 templates.

   **These gaps are the space Wenshu can own.**
3. **The Craft suite has no shared core.** Each app has its own text, layout and render copies. A unified "Wenshu" suite would need a shared CJK text layer, or the same fixes would have to be applied in three places.
4. **Toolchain:** the kits need Rust ≥1.95–1.96. This machine is now on 1.99.1.

## Recommended direction
| App | Base | How |
|---|---|---|
| Writer | **WordCraft** | Contribute upstream (CJK typography, 公文 grid, Chinese corpus CI). Ship a Wenshu zh-CN distribution with 公文 templates and OFD export |
| Slides | **DeckCraft** | Same model; it is the most fidelity-ready for Chinese |
| Sheets | **IronCalc** engine (+ GridCraft UI as reference) | Contribute the zh-CN locale, CJK defined names, LENB/DBNum and charts. Re-evaluate GridCraft in 3 months |
| Shared | **BetterOffice crates** (`docx-parse`, `xlsx-calc`, `ooxml-text` metric tables) | Reuse selectively as reference or oracle |
| New (ours) | `wenshu-cjk` crate + OFD writer + Chinese fidelity corpus | Apache-2.0, offered to all the projects above |

**Next steps:**
- [x] Upgrade Rust (now 1.99.1). Build WordCraft and run the 公文 and mixed-CJK docx files through it.
- [ ] First upstream PR to WordCraft: map 仿宋_GB2312→FangSong/仿宋, 楷体_GB2312→KaiTi, 方正小标宋→a serif fallback, plus full-width CJK quotes.
- [ ] Re-run the WordCraft core tests when the machine has free memory.
- [ ] Grow the corpus to more than 200 real Chinese documents.
- [ ] Open a discussion with the storytold/ArtCraft maintainers about a CJK working group.
