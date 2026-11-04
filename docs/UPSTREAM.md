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
| 0009-balance-sbcs-dbcs-spaces | wordcraft-doc, -docx, -layout | `w:balanceSingleByteDoubleByteWidth` is read into `Settings::balance_sbcs_dbcs`, written back, and makes every U+0020 space half an em of its size (WPS, probe 33). Tests | not opened: measured in WPS only |
| 0010-layout-line-grid-char-indents | wordcraft-doc, -layout | On a line grid without a character pitch, `leftChars`/`rightChars` count the Normal style's size and `firstLineChars` the mark's (`ResolvedPara::set_char_units`; WPS, probe 34). Test | not opened: builds on 0002 (#5003); measured in WPS only |
| 0011-layout-grid-scale-spacing | wordcraft-layout | On a character grid an East Asian character advances its `w:w`-scaled size plus the grid adjustment plus its `w:spacing` (WPS, probe 35); 0002 used the unscaled size and dropped the spacing. Test | in [#5003](https://github.com/storytold/wordcraft/pull/5003) (`902f8efe`) |
| 0012-layout-punctuation-squeeze-per-line | wordcraft-layout | The punctuation squeeze limit is per line (`SqueezeRule`): ½ em in compatibility mode 14 and older, 1 − 1/(2k) em on justified mode-15 lines, ½ em more for a line ending in a closing mark and per two closing-opening meetings; unjustified mode-15 lines neither squeeze nor get the free line-end half (WPS, F10). Tests | in [#5006](https://github.com/storytold/wordcraft/pull/5006) (`0ac59db8`) |
| 0013-layout-grid-indent-units-quote-breaks | wordcraft-layout | On a character grid `*Chars` indents count the text's advance (first line: the first run's size; sides: the larger of it and the mark's, plus the grid adjustment); Unicode 15.1 LB19a breaks before an opening and after a closing curly quote between East Asian characters (`unicode-linebreak` 0.1.5 lacks them); the mark ending a line is never squeezed (WPS, F11). Tests | quote breaks in [#5004](https://github.com/storytold/wordcraft/pull/5004) (`db13aa60`), the line-end mark in #5006; the indents stay here (need 0010) |
| 0014-layout-squeeze-last-unit-pool | wordcraft-layout | The squeeze pool is ½ em plus 1 em per compressible mark in the line's last unit (justified mode 15: 1 − 1/(2k) em plus ½ em per mark); a free line-end half comes out of it; replaces 0012's additions (WPS, F12). Test | in [#5006](https://github.com/storytold/wordcraft/pull/5006) (`0ac59db8`) |
| 0015-layout-autospace-squeeze | wordcraft-layout | In compatibility mode 14 and older an automatic space beside digits or Latin text shares the line's squeeze pool, giving up at most half itself; `Squeeze` carries its own em size (WPS, F13). Test | in [#5006](https://github.com/storytold/wordcraft/pull/5006) (`0ac59db8`) |
| 0016-docx-normal-style-id-clash | wordcraft-docx | A custom style whose id is `Normal` (named `[Normal]`) beside a default paragraph style with another id (`a`) takes a free id, so unstyled paragraphs keep the default style's formatting (ECMA-376 §17.7.4.17). Test | [#5327](https://github.com/storytold/wordcraft/pull/5327) (draft) |
| 0017-layout-label-suffix-space-balance | wordcraft-layout | Under `balanceSingleByteDoubleByteWidth` a list label's suffix space is half an em, like every other space (WPS, F7). Test | not opened: belongs with 0009 |
| 0018-layout-grid-whole-cells | wordcraft-layout | On a character grid a single-column section's text width is narrowed to whole pitches; a closing mark ending a line may reach into the rest (WPS, F14). Tests | not opened: WPS only so far; Word reference needed |
| 0019-layout-balance-doubles-ea-spacing | wordcraft-layout | Under `balanceSingleByteDoubleByteWidth` an East Asian character counts `w:spacing` twice (per byte), a Latin letter once; on and off the character grid (WPS, F15). Test | not opened: belongs with 0009/0017 |
| 0020-layout-squeeze-pool-last-unit-width | wordcraft-layout | The squeeze pool grows by what the last unit is wider than an em (mode 15: half that), instead of an em per mark in it: a line ending in `120` or `abc` squeezes an em (WPS, F16). Test | not opened: fold into #5006 |
| 0021-layout-line-grid-chars-indent-snap-off | wordcraft-layout | On a line grid, `leftChars`/`rightChars` count the Normal size even when the paragraph has `snapToGrid=0` (WPS, F8 addendum). Test | not opened: with 0010 |
| 0022-fonts-east-asian-line-height | wordcraft-fonts, -layout | Faces with an East Asian OS/2 code page take floor(1.3 × win ascent + descent) font units as a text line's height, the extra split evenly above and below (`word::text_line_metrics`; WPS, F17). Test | not opened: WPS measurement, Word unmeasured |
| 0023-fonts-simsun-fallback | wordcraft-fonts | Chinese text no loaded font covers falls back to SimSun before Microsoft YaHei (WPS, F17). Test | not opened: WPS only |
| 0024-layout-table-cells-no-line-grid | wordcraft-doc, -docx, -layout | `w:adjustLineHeightInTable` is read into `Settings::adjust_line_height_in_table` and written back; lines in table cells snap to the line grid only under it (WPS, F18). Tests | not opened: WPS measurement (matches what Word documents for the setting) |
| 0025-layout-grid-pitch-into-margin | wordcraft-layout | A single-spaced grid line's pitch below its text may run into the bottom margin, as multiple spacing's leading does (WPS, F18). Test | not opened: WPS only |
| 0026-docx-no-styles-part-defaults | wordcraft-docx | A file without a styles part takes single spacing, no space after, 10.5 pt Calibri and 宋体, not a new document's styles (WPS, F19). Test | not opened: WPS measurement |
| 0027-fonts-document-scoped-embedded-fonts | wordcraft-fonts, -doc, -engine, -layout | A document's embedded fonts belong to it (`FontFace::owner`, `DocumentFonts`, `Document::embedded_fonts`): lookups see them only while it is laid out, fallbacks never use them, and an embedded copy keeping an installed font's names doesn't stop that font loading. Tests | draft [#5470](https://github.com/storytold/wordcraft/pull/5470) with 0033 (2026-11-03, cherry-picked onto `main` `ae5c65b9`; its engine test names the family "Embedded Scope", as "JetBrains Scop" matches JetBrains Mono there) |
| 0028-fonts-cjk-fallback-prefers-simsun | wordcraft-fonts | Chinese characters a run's font lacks fall back to SimSun first, loading it when installed, before faces loaded for other reasons (WPS, F17). Test | not opened: WPS only |
| 0029-layout-grid-multiple-spacing | wordcraft-layout | Multiple spacing on a line grid takes the larger of the line's whole pitches and m pitches (`Grid::multiple_height`; WPS, F18). Test | not opened: WPS only |
| 0030-fonts-missing-dfkai-line-box | wordcraft-fonts, -layout | DFKai-SB / 標楷體 not installed keeps its own 0.95 / 0.35 em line box under its substitute (`word::missing_font_line`; WPS, F17). Test | not opened: WPS only |
| 0031-layout-wps-punctuation-rules | wordcraft-doc, -docx, -layout | Closing marks hang whole under `w:overflowPunct` in CJK-language text (off under `w:useAltKinsokuLineBreakRules`), adding to the squeeze; kerned text closes up adjacent full-width marks by half an em (`w:noPunctuationKerning` read); the middle dot binds its neighbours; `ResolvedChar::lang_ea` (WPS, F20). Tests | not opened: WPS measurements |
| 0032-layout-wps-spacing-details | wordcraft-layout | Spaces take `w:spacing` twice under balance; a picture-only line has no descent; multiple spacing's leading may run into a tall footer (WPS, F21; two upstream tests now assert the WPS values). Tests | not opened: WPS measurements, Word differs on the last two |
| 0033-fonts-embedded-names-scoped | wordcraft-fonts | Another document's embedded faces don't name families or styles for this one (`english_family`, the font lists); follows 0027. Test | in draft [#5470](https://github.com/storytold/wordcraft/pull/5470) with 0027 |
| 0034-docx-break-inside-text | wordcraft-docx | A `w:br`/`w:cr`/`w:tab` inside `w:t` breaks the line instead of being dropped. Test | not opened: reader robustness, a candidate upstream |
| 0035-docx-vml-textbox-empty-imagedata | wordcraft-docx | A VML text box with an empty `v:imagedata` is read as a text box (it was dropped). Test | not opened: bug fix, a candidate upstream |
| 0036-layout-spacing-in-lines | wordcraft-doc, -docx, -layout | `w:beforeLines`/`w:afterLines` read, written back and laid out in grid lines (12 pt without a grid), overriding points; 0 ignored (WPS, F22). Tests | not opened: read/write part is spec, the unit is WPS-measured |
| 0037-layout-field-result-fonts | wordcraft-layout | A field result shaped in more than one font (`yyyy年M月d日`: digits Latin, 年月日 East Asian) is a cluster per font, each standing for the whole object; one cluster drew every glyph in the first face with the others' glyph ids (wrong glyphs; the PDF face couldn't be subset). Tests | draft [#5462](https://github.com/storytold/wordcraft/pull/5462) (2026-11-03, on `main` `ae5c65b9`) |
| 0038-engine-date-picture-chinese-weekday | wordcraft-engine | Date picture `W` (either case): the Chinese weekday, 二 / 周二 / 星期二 for 1 / 2 / 3+ letters (WPS, F23). Test | not opened: WPS-measured; Word's Chinese pictures to check |
| 0039-docx-vml-wordart-fill-opacity | wordcraft-docx | VML WordArt letters take the shape fill's `opacity` (a transparent `w14:textFill`) and `filled="f"`; a WPS tracing watermark at opacity 0 was drawn opaque, and after a save WPS showed it too. Test | draft [#5463](https://github.com/storytold/wordcraft/pull/5463) (2026-11-03, on `main` `ae5c65b9`) |
| 0040-layout-page-break-heading-no-empty-page | wordcraft-layout | A paragraph with a page or column break inside is placed as if keep with next / keep lines together were unset; a break line holding only markers still stays at the bottom of a full page (WPS, F25). Tests | not opened: the keep part is WPS-measured (Word to check); the marker part is a plain bug fix |
| 0041-layout-kept-paragraph-closing-section | wordcraft-layout | The last paragraph of a section before a new page: keep with next unset, and keep lines too when empty (WPS, F26). Test | not opened: WPS-measured, Word to check |
| 0042-layout-section-mark-after-page-break | wordcraft-layout | An empty next-page section mark right after a paragraph ending with a page break takes no line on the break page; the section takes that page back (WPS with `w:compat`, F26). Test | not opened: WPS-measured, Word to check |
| 0043-layout-break-only-paragraph-goes-over | wordcraft-layout | A paragraph of only a page break that doesn't fit goes over (its break leaves a blank page); a break opening a paragraph stays at the bottom (WPS, F28). Test | not opened: WPS behaviour, Word keeps both |
| 0044-layout-turned-distributed-cell-length | wordcraft-layout | Turned cell text measured by its own length, not its justified one, and at least an at-least row's height (F28). Test | not opened: bug fix, a candidate upstream |
| 0045-layout-kinsoku-off-break-after-spaced-bracket | wordcraft-layout | Kinsoku off: two East Asian characters with spaces between may break after the spaces (a line may end `根据《 `), WPS (F29). Test | not opened: WPS-measured, Word to check |
| 0046-fonts-cjk-compat-ideograph-fallback | wordcraft-fonts | A CJK compatibility ideograph the run's font lacks falls back to Microsoft JhengHei UI, then Microsoft JhengHei (Windows font linking, WPS, F30). Test | not opened: Windows-specific measurement |
| 0047-layout-linked-face-keeps-line-height | wordcraft-layout | Compatibility ideographs in a stand-in face keep the run font's line height (WPS, F30). Test | not opened: WPS-measured |
| 0048-fonts-missing-arial-unicode-ms-line | wordcraft-fonts | A missing Arial Unicode MS keeps a 1.74 em line box (WPS, F31). Test | not opened: WPS-measured |
| 0049-layout-missing-founder-faces-line-box | wordcraft-fonts, -layout | Missing Founder faces keep WPS's line box, also for text in the run's East Asian font (F32). Test | not opened: WPS-measured, depends on WPS's substitutes |
| 0050-layout-at-least-zero-off-grid | wordcraft-layout | At least 0 (`w:line="0" w:lineRule="atLeast"`) is no rule: natural height, off the document grid (F33). Test | not opened: WPS-measured |
| 0051-doc-hanging-from-character-left-indent | wordcraft-doc | A hanging indent beside `leftChars`/`hangingChars` hangs from the character left indent, whatever `w:left` says (F34). Test | not opened: WPS-measured (Word's `w:left` agrees) |
| 0052-layout-first-line-after-leading-break | wordcraft-layout | Text after leading page or column breaks keeps the first-line or hanging indent (F35). Test | not opened: WPS-measured |
| 0053-layout-space-before-after-leading-break | wordcraft-layout | The space before of a paragraph opening with page or column breaks applies on the new page or column up to mode 14, not in mode 15 (WPS, F36). Tests | not opened: departs from Word in mode 15 (one Word-measured assertion changed) |
| 0054-layout-empty-paragraph-after-nested-table | wordcraft-layout | An empty last paragraph right after a nested table takes no room (WPS, F37). Test | not opened: WPS-measured |
| 0055-docx-unsigned-minus-one-wrap-distance | wordcraft-docx | A wrap distance stored as the unsigned form of −1 (`4294967295`) is no distance (F38). Test | could go upstream: a reader robustness fix |
| 0056-layout-first-line-chars-count-the-first-run | wordcraft-layout | Off a character grid a first-line indent in characters counts the first run with text: size plus twice its spacing (WPS, F39). Tests | not opened: departs from Word (#534 test changed) |
| 0057-layout-lowered-text-keeps-the-line-height | wordcraft-layout | A line all lowered by `w:position` is as tall as an unlowered one (WPS, F40). Test | not opened: WPS-measured |
| 0058-layout-squeeze-limit-shrinks-with-line-length | wordcraft-layout | In mode 15 a justified line squeezes each of k marks at most 1/(k + n/32) em, n its length before the last unit (WPS, F41; replaces F10's 1 − 1/(2k)). Test | not opened: WPS-measured |
| 0059-layout-mode-15-hangs-justified-lines-only | wordcraft-layout | In mode 15 a closing mark hangs past the margin on justified and distributed lines only (WPS, F42). Test | not opened: WPS-measured |
| 0060-layout-punctuation-kerning-on-a-character-grid | wordcraft-layout | Kerned text closes up adjacent full-width marks on a character grid too, by half the cell (WPS, F42). Test | not opened: WPS-measured |
| 0061-layout-even-pages-gutter-with-odd-even-headers | wordcraft-layout | With different odd and even headers (margins not mirrored) the gutter moves to the right of even pages (WPS, F42). Test | not opened: WPS-measured |
| 0062-layout-bookmark-only-section-end-stays | wordcraft-layout | A section-closing paragraph of nothing but markers (a bookmark) counts as empty and stays at the bottom of a full page (F42). Test | could go upstream: generalises an existing rule |

All 62 apply in order on `3b7f7ed` and reproduce the tested tree (checked 2026-11-04); the
`wordcraft-layout` (612), `-fonts` (109), `-doc` (242), `-docx` (375 + 134 integration) and
`-engine` (1095) suites pass with them, and fmt and clippy are clean.

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