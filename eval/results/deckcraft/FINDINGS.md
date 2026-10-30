# DeckCraft — evaluation (2026-10-30)

**Recommendation: reuse crates + contribute; don't fork yet.**

## Architecture
- 221,775 lines of Rust in 24 crates plus 3 apps (desktop, CLI, web).
- Layers: geom/color → model → fonts/text (own layout on harfrust + skrifa) → render (vello_cpu) → pptx (20k) / format / pdf → engine (40.7k, 200+ commands with undo) → mcp → ui-egui (43k, a storytold fork of egui).
- About 43k lines are video/audio codecs copied in from FilmCraft.
- No ODP and no legacy .ppt. Interface translation about 8%. Self-described alpha, never tested on a corpus of real decks.
- License: MIT OR Apache-2.0. The ArtCraft logos are trademarks and must be removed in forks.

## Shared core with WordCraft and GridCraft: none
- All three take pinned git dependencies on `storytold/craft-libs` (MIT/Apache). That covers small things only: a ComboBox widget, PDF writing, the updater, printing and window helpers.
- All three patch egui to the same fork revision.
- text, layout, model, render, geom and fonts are **separate copies per app that have drifted apart**. For example, metafile is 7.2k lines here and 3.6k in WordCraft.
- AGENTS.md: "repos don't share code".

## Build and tests
- The repo requires rustc **1.95**; we have 1.94.1. With `--ignore-rust-version` the release CLI built in 6m42s.
- Core crate tests (pptx, text, model, fonts, render, geom): **651 passed, 0 failed.**
- Engine and UI tests were not run.

## Corpus: 04_deck.pptx
- All 3 slides render correctly with **no tofu boxes**: Chinese text falls back to Microsoft YaHei, and mixed CJK/Latin text, full-width punctuation, layout and table geometry are right.
- One deviation: the table header is not bold. The cause is probably the bold face being lost on the CJK fallback font; this is unconfirmed.
- PDF export: fonts embedded and subset; the Chinese text extracts exactly.
- Round-trip: python-pptx dump identical, re-rendered slides pixel-identical. Only the thumbnail and printer settings were dropped.

## Code quality
- `unsafe_code = "forbid"`; crate roots deny unwrap/expect/panic.
- 1 TODO in 222k lines; 2,116 `#[test]`.
- Sampled files are clean and reference the spec.
- Chinese handling is real: separate East Asian fonts, kinsoku, upright vertical text, CJK numbering.
- By policy, Noto CJK and Source Han are not bundled.

## Health
- 4.4k★, 2.5k forks, 66 contributors; one dominant committer with 619 commits.
- 306 open issues and 339 open PRs, with 607 PRs merged in 3 weeks.
- Issues are early-alpha editor bugs; none about file corruption.

## Scores
| Fidelity | CJK | Code quality | Extensibility | Health | License |
|---|---|---|---|---|---|
| 4 | 4 | 4 | 4 | 3 | 5 |

Artifacts in this folder: `png/`, `png_rt/`, `04_deck.pdf`, `04_deck_roundtrip.pptx`, `info.json`, build and test logs.
