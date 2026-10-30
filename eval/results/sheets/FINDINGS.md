# Sheets kits — GridCraft vs IronCalc (2026-10-30)

Both engines computed all 21 target values in `03_sales.xlsx` correctly. **Recommendation: reuse IronCalc's engine and contribute upstream. Re-evaluate GridCraft in about 3 months and use it as a reference for charts and pivot tables.**

Scripts, logs, JSON results and round-trip files are in this folder; the IronCalc example crate is in `ironcalc_eval/`.

| | GridCraft | IronCalc |
|---|---|---|
| Architecture | 12 crates, engine kept separate from UI; egui desktop app (forked egui), WASM web app, CLI, MCP server | Engine + xlsx crates; Python, Node and WASM bindings; separate React web UI |
| Functions (counted in code) | 514, about 9 of them stubs | 497 enum variants, all implemented |
| Formats / charts | XLSX, CSV, XLS/ODS import, PDF export; charts | XLSX only; no ODS; **no charts** |
| Build (rustc 1.94.1) | OK (CLI release build 5.5 min) | OK |
| Corpus values | 21/21 | 21/21 |
| Round-trip (openpyxl) | Sheet names, 34/34 formulas, `0.00%` format, merged cells and chart kept | Same, but **chart dropped** |
| Tests | 1,406 pass, 0 fail. The engine crate's tests don't compile on Windows (Unix-only call) | 2,821 + 87 pass. Excel test suite doesn't compile on Windows (backslash paths in build.rs); a custom harness ran all 242 files and all pass |
| CJK cases | 15/20; Chinese defined names and LET variables work | 15/20; Chinese defined names and LET variables rejected (issue #1632) |
| Health | 23 days old, 2,299 commits, 126 contributors, 795 open issues + 723 open PRs, ~450 commits co-authored by Claude | Since 2023; 331 commits in the last 90 days, 51 contributors, 261 open issues + 51 open PRs |
| License | MIT OR Apache-2.0 (ArtCraft brand trademarked) | MIT OR Apache-2.0 (+ Unicode-3.0 for locale data) |

## CJK detail
- **Both pass:** LEN, LEFT, MID and RIGHT count characters, not bytes. Chinese sheet names in formulas, VLOOKUP on a Chinese key, and quoted Chinese text in number formats all work.
- **Both fail:** LENB double-byte counting (`LENB("中文")` gives 2, should be 4), `[DBNum1]` Chinese-numeral formats, NUMBERSTRING.
- **GridCraft only:** ASC does nothing on full-width text. Number formats are en-US only; about 12% of the UI is translated into Chinese; Chinese IME input is untested.
- **IronCalc only:** unquoted `yyyy年m月d日` in a TEXT format gives #VALUE!. It ships locales for en, en-GB, de, es, fr and it, and adding zh-CN is mostly a data job.

## Scores
| | Fidelity | CJK | Code quality | Extensibility | Health | License |
|---|---|---|---|---|---|---|
| GridCraft | 4 | 3 | 3 | 4 | 2 | 5 |
| IronCalc | 3 | 3 | 4 | 4 | 4 | 5 |

GridCraft's code-quality score reflects 198k lines written by agents in 23 days. Its docs repeat a line with conflicting function counts, and open issue #2950 says some source files are no longer compiled. It describes itself as alpha.

## Work to send upstream to IronCalc
zh-CN locale · non-Latin defined names · double-byte LENB functions · `[DBNum]` Chinese-numeral formats · **charts** (the largest gap)
