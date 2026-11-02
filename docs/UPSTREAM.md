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
| 0001-fonts-substitute-provider-hook | wordcraft-fonts | `SubstituteProvider` + `set_substitute_provider()`: named substitutes from the embedding app, tried before the built-in lists | [#5002](https://github.com/storytold/wordcraft/pull/5002) (draft) |
| 0002-layout-grid-cells-and-char-indents | wordcraft-layout | Character grid: East Asian text advances its size plus the grid adjustment (`w:charSpace`, ECMA-376 §17.6.5; measured in WPS), not `round` cells (22 pt glyphs had overlapped in one 15.8 pt cell); `w:firstLineChars`/`leftChars`/`rightChars` count pitches, one per character (a 公文 first line held 25 + indent instead of 26 + indent). Tests | [#5003](https://github.com/storytold/wordcraft/pull/5003) (draft) |
| 0003-east-asian-hint-and-named-ea-substitutes | doc, docx, layout | Reads and writes `w:rFonts/@w:hint`; with `hint="eastAsia"`, the characters Word's block table lists (MS-OI29500 §2.1.88: curly quotes, dashes, Greek, Cyrillic, U+2000–27BF, some Latin-1 signs) use the East Asian font; the rows that need a Chinese `w:lang` are not applied yet. An East Asian font substituted *by name* (仿宋_GB2312 → FangSong) is kept instead of dropped to generic fallback, unless it resolved to the generic fallback itself. Round-trip + layout tests | [#5005](https://github.com/storytold/wordcraft/pull/5005) (draft, on #5003) |
| 0004-layout-kinsoku-curly-quotes | wordcraft-layout | Kinsoku sets gain ” ’ 〃 〆 ‰ ℃ (no line start) and “ ‘ (no line end), needed once break opportunities beside quotes exist (Unicode 15.1 LB19a, kinsoku-off paths). Test | [#5004](https://github.com/storytold/wordcraft/pull/5004) (draft) |
| 0005-layout-punctuation-compression | doc, docx, layout | `Settings::char_spacing` (`w:characterSpacingControl`) read and written (it was always saved as `doNotCompress`). Layout as WPS does it (measured, [WPS-FINDINGS.md](WPS-FINDINGS.md) F1): a line that fits is untouched; an overflowing line squeezes its full-width marks by up to 0.1 em each (`ParaLayout::squeeze`, `lines::squeeze_line`); a closing mark ending a line needs no room for its blank half (`auto_space`). On character grids too. 7 layout + 1 docx test, round-trip extended | [#5006](https://github.com/storytold/wordcraft/pull/5006) (draft, on #5004) |

| 0006-app-embedding | app, ui-egui, control-key | The desktop app as a library: `wordcraft::run(AppConfig)` (name/version/About via `ui_egui::brand::Brand`, executable name, settings folder, app id, icon, start-up hook, update check on/off, app tools); `main.rs` runs it with WordCraft's defaults. Also: `engine::set_blank_document_hook` (an app's Normal defaults by region), `Brand::{community, updates}` (hide the ArtCraft Discord links and WordCraft's update check), `services::AppTool` (app tools in a Review group named after the brand, reporting in a dialog), `control_key::set_folder_name`. Tests | not yet opened (M2) |
| 0007-fonts-app-font-dirs | wordcraft-fonts | `FontDb::add_font_dirs`: an app adds font folders (fonts shipped beside its executable) to the system font scan. They are cataloged like installed fonts and loaded only when a document uses them; adding a folder after the first scan rescans and clears remembered fallback misses. Test | not yet opened (M2) |
| 0008-layout-exact-baseline | wordcraft-layout | Exactly spaced lines put the baseline at 0.8 of the line height, whatever the fonts (WPS, measured with probe 31; was height minus the font descent). Test | not opened: measured in WPS only; Word needs references first |

All six apply in order on `3b7f7ed`; the `wordcraft-layout` (577), `-fonts` (102), `-doc` (242),
`-docx` (500) and `-engine` (1094) suites pass with them, and fmt and clippy are clean.

**Review status (2026-10-31):** four CodeRabbit rounds; every thread resolved or answered, and the
fourth round found nothing new. No human review yet: the PRs are still drafts. Upstream `main` is
12 commits past the pin; the PRs report mergeable.

Opened 2026-10-30 from `xiaojunyang0805/wordcraft` branches `cjk/*`, rebased on upstream `5b5810f`.
The patch files are generated from those branches (first line: the PR link). After review fixes,
regenerate them with `git diff` between the branches, as in [PROGRESS.md](PROGRESS.md).
Three tests also fail on unmodified upstream `main` on Windows (mcp `automation_roots` dead code,
`proof` hunspell, `ui-egui` backstage); they are listed in each PR.

## Issues found (status)

- ✅ Character grid: 27 instead of 28 characters per line; large text overlapping. Fixed by 0002,
  reworked 2026-10-31 to the measured rule: each character advances its size plus the grid delta
  ([WPS-FINDINGS.md](WPS-FINDINGS.md) F2); indents are pitches (F3).
- ✅ Curly quotes drawn half-width in hinted East Asian runs. Fixed by 0003. Unhinted runs keep the
  Latin font, as the spec says.
- ✅ Kinsoku for curly quotes. Fixed by 0004. Dashes and ellipses may start a line (GB/T 15834:
  they only must not be split; UAX #14 already keeps —— and …… together).
- ✅ Punctuation compression. 0005 reads and writes the setting, and since 2026-10-31 lays it
  out as WPS does (F1): squeeze on overflowing lines, free line-end blank half, no pair
  closing-up. Corpus page-count matches 112 → 122 of 200. Open: `w:topLinePunct`, kana mode, a
  Word reference.