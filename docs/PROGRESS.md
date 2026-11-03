# Wenshu progress log

Newest first. One entry per working session: what landed, what was verified, what's next.

## 2026-11-03 (10): the 30 documents a page off: empty pages at section ends, 華康 fonts: 174 / 200

- **Triage** (`out/triage.py`): of the 30, nine had an empty page in Wenshu's layout. Their causes,
  each probed in WPS:
  - a kept empty paragraph closing a section (a heading style on it) moved over, then the
    section break began another page (`aa35f17a03`); patch 0041, F26;
  - cover, page break, an empty next-page section mark: WPS starts the section on the break's page
    whenever the file has `w:compat` (`ef14cd2f88`, `1aa06d7905`; found by cutting the document
    down until removing the compatibility settings brought WPS's blank page back); patch 0042.
- **F27:** two church bulletins in 華康細圓體 were half as long: WPS draws nine 華康 names in
  Microsoft JhengHei (whose taller line takes two grid pitches) and three in SimHei, by a table of
  names (24 probed). `wenshu-cjk` follows it.
- **Fidelity: 170 → 174 / 200**, mean 0.9367 → 0.9395; gains `aa35f17a03`, `ef14cd2f88`,
  `1aa06d7905`, `f899568bb2`; no losses.

Tests: layout 605, cjk 8; fmt and clippy clean; the queue 0001–0042 applies on the pin and
reproduces the tested tree.

**Later the same session (F28, patches 0043-0044): 175 / 200.** A rating-scale header in turned
cells with distributed text filled a page (`efacf8571b` now matches). WPS moves a paragraph of
only a page break that doesn't fit at the page bottom (and leaves a blank page); Wenshu now does
too, with no corpus gain yet. Tests: layout 607; the queue 0001-0044 reproduces the tested tree.

**Then (F29, patch 0045): 176 / 200.** With kinsoku off WPS breaks after `《 ` (`ea10dc7972`).
On the way, a space-width "rule" measured with probes lacking a settings part turned out to be
WPS's handling of such files only; the change was reverted, and probes now always carry one.

**Then (F30, patches 0046-0047): still 176 / 200.** A 理 written as a compatibility ideograph
(U+F9E4) fell back to a Japanese UI font with a taller line; WPS uses Microsoft JhengHei UI and
keeps the line height. `92262161df` matches; `2d6a9da333`, off elsewhere, lost its lucky match.
`9c189a35a9` (a chart with top-and-bottom wrap): WPS and Wenshu both respond non-monotonically as
the chart moves; left for a study of floats and the line grid.

**2026-11-04 (F31, patch 0048): 177 / 200.** A missing Arial Unicode MS keeps its own 1.74 em
line in WPS (`377b4bf861` matches).

**Then (F32, patch 0049): 177 / 200, mean 0.9401.** Missing Founder faces keep WPS's line box
(公文 titles in 方正小标宋简体); ten documents score higher. Open: Latin text in 新細明體 runs drawn
in Segoe Print by WPS (three references).

**Where the 23 stand (2026-11-04):**
- `29a7772dfb` (+2): cut down to 16 blocks (`out/bis/reduce2.py … 第二部分2023`); WPS keeps a
  break-only paragraph after text ending at y 724, Wenshu (725) sends it over (empty page). Probe:
  on a 312-twip grid WPS needs the full grid line for that paragraph (Wenshu fits 13.6 pt in 14),
  so Wenshu must lose more than 2 pt of room above it, in its exact 30 pt lines: trace line boxes.
  Traced (2026-11-04): every baseline on page 3 matches WPS within 0.05 pt; Wenshu's body ends at
  the footer top (734.9: a footer of two empty 11 pt paragraphs runs above the 745.5 margin), its
  break line is 31.2 pt (16 pt 仿宋 on the grid), so 7.9 pt of room. WPS keeps the line. Not
  settings (minimal settings: same), not the header, the first-page footer or the page-number text
  box; removing the table of contents before it changes the page boundaries altogether. Probes
  with the same geometry, footer and room push the line over in WPS too, so the cause is still
  unknown.
- `8f343b7812` (−1), `9c189a35a9` (−1): floats with top-and-bottom wrap (in table cells; under a
  chart); WPS's offsets vary with how rows split / where the float sits, non-monotonically.
- `cd82c17bf5`, `228229e95c`, `82eb70e86d`: the empty page is a symptom of earlier line-height or
  break differences.
- `f7d3afdfdb` (−1): Latin text in Arial Unicode MS drawn by WPS in a font of its own (widths).
- `173ff7b3a5`, `ffe3dc89e6`: WPS draws Latin text in 新細明體 runs in Segoe Print (F32, open).
- The rest not yet traced: `a9f763a2b4`, `a180b9d578`, `2d6a9da333`, `1f522d22f6`, `493a360281`,
  `e67370879a`, `aa1a5e719d`, `40991f97da`, `7abe5be4e9`, `1403def07e`, `39de0888eb`,
  `a5beccd773`, `85cb8e4b30`, `9c1f3849ea`.

**Next:** continue down that list (`493a360281`, `cd82c17bf5`, `228229e95c`, `ea10dc7972`,
`82eb70e86d`: tables and trailing empty paragraphs), then line-height differences.

## 2026-11-03 (9): no empty page before a heading that opens with a page break (F25): 170 / 200

- **gov-nmpa-5** was four pages long: an empty page before each attachment (附件1–4), each a
  kept heading 3 opening with a page break. Probes in WPS: keep with next and keep lines together
  don't apply to a paragraph with a break inside; Wenshu moved it whole, then broke again. And a
  `w:bookmarkEnd` between paragraphs, carried onto the heading, stopped the break line from
  staying at the bottom of a full page (found by shrinking the document to 21 blocks, then
  removing parts one at a time). Patch 0040, with a test of both that fails without it.
- **Fidelity: 166 → 170 / 200**, mean 0.9354 → 0.9367; gains `gov-nmpa-1`, `-5`, `-6`,
  `dc-technical-c2c92639c7`; no losses.
- Upstream: 0027 + 0033 opened as draft
  [#5470](https://github.com/storytold/wordcraft/pull/5470) (cherry-picked onto `main`; its engine
  test's family renamed, as "JetBrains Scop" matches JetBrains Mono there).

Tests: layout 603; fmt and clippy clean; the queue 0001–0040 applies on the pin and reproduces the
tested tree.

**Next:** the 30 documents still a page off (the largest gaps: `dc-administrative-82eb70e86d`,
`dc-forms-173ff7b3a5`, `dc-policies-efacf8571b`, `dc-reports-1aa06d7905` at +2; `a9f763a2b4`,
`f899568bb2` at −2); picture fills for shapes (the last J5 text loss).

## 2026-11-03 (8): v0.1.2; J5 measured right: 199 / 200 texts kept; an invisible watermark (F24)

- **v0.1.2** tagged (Cargo 0.1.2); release.yml built all eight packages into a draft release
  (publishing is the user's call; v0.1.1 is a draft too).
- **J5's text comparison fixed** (`corpus/tools/roundtrip.py`): it dropped every field result
  (hyperlinks written back as `w:hyperlink` counted as new text), counted text boxes twice
  (`mc:Fallback` tags carry namespace declarations), missed text before a `w:br` inside `w:t`,
  and joined header parts in file-name order. Now only recalculated fields (page numbers, dates)
  are left out and parts compare as a set. With today's build: text identical **199 / 200**,
  tables 200 / 200, East Asian fonts 199 / 200, WPS page count unchanged 197 / 200, mean page
  score 0.9932 (161 pixel-identical).
- **F24, patch 0039:** a WPS tracing watermark (VML WordArt with fill opacity 0, names and phone
  numbers) was drawn, and after a save WPS showed it across the page. The fill opacity now carries
  over as a transparent `w14:textFill`. WPS's render of the saved page: 0.99995 (was 0.984).
- **Upstream:** 0037 and 0039 opened as drafts on upstream `main` `ae5c65b9`:
  [#5462](https://github.com/storytold/wordcraft/pull/5462) and
  [#5463](https://github.com/storytold/wordcraft/pull/5463) (layout tests pass except three
  hyphenation tests that fail on clean `main` here too). **v0.1.2** published as a pre-release.

Tests: docx 373 + 134; fmt and clippy clean; the queue 0001–0039 applies on the pin and reproduces
the tested tree.

**Open:** the text box in a picture-filled shape (`aae80ea709`; the shape model has no image fill,
so it is read as a picture and its text dropped: the last J5 text and font loss); the three page
changes of J5 (documents Wenshu itself lays out a page off).

**Next:** (0027 + 0033 opened as draft [#5470](https://github.com/storytold/wordcraft/pull/5470)); picture fills for shapes; the 34
documents still a page off WPS.

## 2026-11-03 (7): M2 — PDF export measured (J4); field results in two fonts (F23)

- **J4 measured** (`corpus/tools/pdf_check.py`): every corpus document converted by the CLI (a
  process each), then the PDF read back. 200 / 200 written; page counts equal to Wenshu's layout
  200 / 200; every font embedded and subset 200 / 200; text extractable with mean recall
  **0.9986** (Chinese 0.9984), every character in 169 / 200; pages match the screen render
  (worst page above 0.99 in 198 / 200). The two others differ by screen-only markup (tracked
  changes, field shading), which WPS's PDF leaves out too. Median 0.2 s, 164 KB.
- **Found by it, patch 0037:** a field result in more than one font (a footer
  `TIME \@ "yyyy年M月d日星期W"`) drew every glyph in the first font with the other font's glyph
  ids, so the screen showed wrong glyphs and the PDF couldn't embed that face (drawn as outlines,
  so no text; three documents lost 15–50% of their extractable text). Now a cluster per font, with
  WPS's automatic space between digits and 年月日.
- **F23, patch 0038:** WPS refreshes `DATE`/`TIME` to the render date; `W` is the Chinese weekday
  (二 / 周二 / 星期二). WPS doesn't support `CREATEDATE`.
- The CLI now prints library warnings to stderr (`WENSHU_LOG`, default warn), so a font drawn as
  outlines shows up.
- Fidelity unchanged (166 / 200, mean 0.935; no document moved by more than 0.002).

Tests: layout 602, engine 1095, pdf 42 + 65; fmt and clippy clean; the queue 0001–0038 applies on
the pin and reproduces the tested tree.

**Open from J4:** a vertical (`btLr`) text box drawn empty (`aae80ea709`; WPS's own page 1 is blank
too); hollow (`w14:textFill noFill`) text not extractable; `dddd`/`ddd` day names in Chinese
without a language tag (F23).

**Next (M2):** release v0.1.2; offer 0027 + 0033 + 0037 upstream; then the J5 text differences.

## 2026-11-03 (6): M2 — desktop smoke test, the .docx round-trip (J5)

- **Desktop smoke test** (`corpus/tools/desktop_smoke.py`, control channel): five corpus
  documents open in one session with the CLI's page counts, typing, Save As and reopen work. It
  found an app bug the isolated CLI hid: after a document embedding 仿宋_GB2312 was opened,
  another document's 仿宋_GB2312 text drew in a sans (the embedded face's localized name made the
  family look installed). Fixed: patch 0033; the same document now renders identically before
  and after.
- **J5 measured** (`corpus/tools/roundtrip.py`): every corpus document saved by Wenshu, rendered
  by WPS, compared with WPS's render of the original. Three reader gaps found and fixed on the
  way, each also a fidelity gain: breaks inside `w:t` (0034), VML text boxes with an empty
  `v:imagedata` (0035), spacing in lines `w:beforeLines`/`w:afterLines` (0036, F22).
  Now: WPS opens all 200 saves; **page count unchanged 197 / 200**, mean page score saved vs
  original **0.993**, 160 pixel-identical; tables kept 200 / 200; East Asian font names 199 / 200;
  text identical 175 / 200 (most of the rest are hyperlink fields written as hyperlinks, which my
  comparison counts as a change). The three page changes (`aa1a5e719d`, `29a7772dfb`,
  `ef14cd2f88`) are documents Wenshu itself lays out a page off.
- **Fidelity:** 163 → **166 / 200**, mean 0.9348 → 0.9354 (the three itinerary documents).

Tests: layout 600, fonts 109, doc 242, docx 373 + 134, engine 1095; fmt and clippy clean; the
queue 0001–0036 applies on the pin and reproduces the tested tree.

**Next (M2):** J4 (PDF export across the corpus: embedded fonts, selectable text); release
v0.1.2; offer 0027 + 0033 upstream; then the J5 text differences one by one.

## 2026-11-03 (5): the seven remaining documents (F20, F21)

Line-break diffs against WPS (`out/breakdiff2.py`, tolerant of symbol and right-to-left text;
`out/pagelines.py` for page ends), each cause cut down to one paragraph and probed in WPS.

- **F20, patch 0031:** closing marks hang whole under `w:overflowPunct` in Chinese/Japanese/
  Korean text, adding to the squeeze (off under `w:useAltKinsokuLineBreakRules`); kerned text
  closes up adjacent full-width marks by half an em (unless `w:noPunctuationKerning`); the middle
  dot of transliterated names binds its neighbours.
- **F21, patch 0032:** spaces take `w:spacing` twice under balance; a picture-only line has no
  descent; multiple spacing's leading may run into a footer taller than the margin.

Re-scored (isolated): **156 → 163 / 200**, mean **0.9329 → 0.9348**; no document lost its match
in any step. Of the seven, `b6747d9df3`, `be4a26d95e`, `f9e6e0e0e6` and `b8de3ae296` now match,
and `9e27fa4072`, `7380cfbea3` and `43c2246cf6` matched on the way. Still off: `92262161df`, `7abe5be4e9` (a converted form) and
`ea10dc7972` (an opening mark overhanging with kinsoku off, unmeasured). Tests: layout 599, fonts
108, doc 242, docx 370 + 134, engine 1095; fmt and clippy clean; the queue 0001–0032 applies on
the pin and reproduces the tested tree (0031 also builds without 0032).

**Next:** `ea10dc7972`'s opening-mark overhang; the automatic space under `w:spacing`; stretching
justified Chinese lines (visual only); then the remaining 37 mismatched documents by line-break
diff.

## 2026-11-03 (4): the open items: embedded fonts per document, grids, defaults (F17–F19)

Each rule measured in WPS first; scores are isolated (one process per document).

- **Embedded fonts per document (0027):** a face a document embeds has an owner; layout enters
  its document's fonts (`DocumentFonts`), other documents never see them, fallbacks never use
  them. Found on the way: WPS embeds whole SimSun files as 黑体 / 等线 / 仿宋_GB2312 (`ce628d9760`),
  and such a copy kept the installed SimSun from loading (fixed in `load_cataloged` too).
- **No styles part (0026, F19):** single spacing, no space after, 10.5 pt Calibri / 宋体.
- **標楷體 / DFKai-SB (0030):** missing, they keep a 0.95 / 0.35 em box; DFKai-SB by its English
  name draws in Microsoft JhengHei Light (Wenshu's font table).
- **F18 corrected (0024):** table cells *do* snap to the line grid under
  `adjustLineHeightInTable` (the first probe had the setting outside `w:compat`). The setting is
  now read and written back. `bd6b721e4e` back to 12 pages.
- **Grid multiple spacing (0029):** max(whole pitches, m pitches), not pitches × m. 18 probes
  match; `f526989771`'s red header and a dozen long documents shorten to their reference.
- **SimSun first for Chinese fallbacks (0028)** even when YaHei was loaded before it.
- **1.3 × in whole font units (0022 amended):** SimSun 1.2969 em, YaHei 1.7153; long-paragraph
  pitches now within 0.03% of WPS (were 0.3%). `de24610147` 88 → 86 pages, its reference.

Re-scored (isolated): **144 → 156 / 200**, mean **0.9227 → 0.9329** (the 0020 baseline was 137 /
0.9117). Of the 13 documents 0021–0025 had lost, 6 are back; 7 remain a page or two off
(`f9e6e0e0e6`, `92262161df`, `b8de3ae296`, `be4a26d95e`, `7abe5be4e9`, `b6747d9df3`, `ea10dc7972`),
and their line heights now match WPS, so the cause is elsewhere. Tests: fonts 108, layout 594,
doc 242, docx 370 + 134, engine 1095, cjk 7; fmt and clippy clean; the queue 0001–0030 applies
on the pin and reproduces the tested tree (0027 also builds without 0028).

**Next:** the 7 documents above, by line-break diffs (`out/breakall.py`) against the 0030 build.

**To resume:** build with `CARGO_TARGET_DIR=$PWD/eval/target/wordcraft cargo build --release -p
wenshu-cli` (the current binary is `out/wenshu-o6.exe`, isolated scores in `out/fid-o6`); score
with `wenshu fidelity corpus out/fid-X` (one process per document now) and compare with
`python out/cmpfid.py fid-o6 fid-X`. Older binaries: `out/fidiso.py <absolute exe> corpus out/X`.
Run one corpus score at a time (two at once ran the machine out of memory). WordCraft's own
tests run from `upstream/wordcraft` with `CARGO_TARGET_DIR=…/eval/target/wc-up`. Probe
generators for F17–F19 are in `corpus/tools/lh_probe_*.py`. Upstream PRs: 0019–0030 are not
opened yet (see `docs/UPSTREAM.md`); 0027 is a plain bug fix worth offering first.

## 2026-11-03 (3): East Asian line height (F17), grids in tables (F18), harness isolation

- **Harness:** `wenshu fidelity` now lays each document out in a process of its own (hidden
  `fidelity-doc` command). Twelve corpus documents embed fonts (subsets of 仿宋_GB2312,
  方正小标宋简体…), and the embedded faces stayed loaded for every later document asking for the
  family: `gov-mct-keji` after `gov-fj-gaoxin` drew its title in YaHei. The same leak exists in
  the desktop app between documents of one session (`FontDb::add_document_fonts` is
  process-wide); not fixed yet. `out/fidiso.py` scores older binaries the same way. Re-scored
  isolated, the 0020 build is 137 / 200, mean **0.9117** (0.9085 in one process).
- **0021 (F8 addendum):** line-grid `*Chars` side indents count the Normal size with
  `snapToGrid=0` too (`dc-policies-39de0888eb`).
- **Wenshu font table:** 標楷體, 新細明體, 細明體, PMingLiU, MingLiU → the Song list (WPS draws
  them as 宋体); 微軟正黑體 → YaHei. `dc-policies-a5beccd773`'s digits now match.
- **F17, patches 0022–0023:** East Asian faces take 1.3 × their win height, split evenly above
  and below; Chinese text no font covers falls back to SimSun before YaHei. 40 line-height probe
  cases match WPS within 0.1 pt.
- **F18, patches 0024–0025:** no line-grid snapping in table cells; a grid line's pitch below its
  text may run into the bottom margin.

Re-scored, both isolated (0020 → 0025 + font table): page-count matches **137 → 144 / 200**,
mean **0.9117 → 0.9227**, 117 documents better, 30 worse. 20 documents now match, 13 no longer
do (`dc-forms-7abe5be4e9`, `dc-educational-92262161df`, `dc-administrative-f526989771` and
`-f9e6e0e0e6`, `dc-educational-be4a26d95e`, `dc-policies-38bfeee828`, `-b6747d9df3`,
`-ea10dc7972`, `-de24610147` a page or two long; `dc-administrative-bd6b721e4e` 12 → 8,
`dc-reports-e1c1c56064`, `dc-educational-a9249ea5f3` short; `dc-educational-b8de3ae296` a
page long). Layout 593 tests, fonts and cjk tests pass, fmt and clippy clean; the queue
0001–0025 applies on the pin and reproduces the tested tree.

**Next:** the 13 lost documents (start with `bd6b721e4e`, 4 pages short, and the table-heavy
forms); 標楷體's baseline (0.9 pt lower in WPS); documents without `styles.xml` default to 1.15
spacing where WPS uses single; scope embedded fonts to their document.

## 2026-11-03 (2): releases, upstream, spacing per byte (F15)

- **v0.1.0** published as a pre-release. **v0.1.1** (Cargo 0.1.1, tag pushed): release.yml made
  a draft with all 7 packages; the portable zip's checksum and `gov-mct-keji` → 2 pages checked
  (so it carries 0015). Publishing the draft is the user's call.
- **Upstream:** #5004 + LB19a quote breaks (`db13aa60`, new test); #5006 + 0012/0014/0015 and
  0013's line-end mark (`0ac59db8`, stacked on #5004's commit; the full-width quote case waits
  for #5005's hint); #5003 + 0011 (`902f8efe`); 0016 opened as draft #5327 on current main. Each
  new test fails without its change; fmt, clippy and the crates' tests pass on each branch.
  Comments on #5003/#5006 say the evidence is WPS only. 0018 and 0013's grid indents stay local.
- **F15, patch 0019:** `gov-sh-sheitc`'s open line (F14) was character spacing: under
  `balanceSingleByteDoubleByteWidth` WPS counts `w:spacing` twice on East Asian characters.

Re-scored (0018 → 0019): page-count matches **136 → 137 / 200**, mean 0.9083 → **0.9084**, 10
documents better, 2 slightly worse (−0.0015, −0.0005). `dc-reports-7f0a1221f0` now matches,
`dc-policies-228229e95c` is a page closer. Layout 590 tests; fmt and clippy clean; the queue
0001–0019 applies on the pin and reproduces the tested tree.

- **F16, patch 0020:** lines ending in a digit run (`…给予300`, `…病床120`) squeeze an em in WPS.
  The pool is half an em plus what the last unit is wider than an em, which also reproduces
  every F12 sweep (it replaces F12's count of marks). Re-scored (0019 → 0020): matches 137 / 200
  unchanged, mean 0.9084 → **0.9085**, 6 documents better, 3 slightly worse (worst −0.003).
  Layout 591 tests; the queue 0001–0020 reproduces the tested tree.

**Next:** items 2–3 of the entry below (the one-character-short and one-longer groups), with
`out/breakall.py` against the 0020 build; then fold 0019/0020 into #5006 and 0017/0019 with 0009.

## 2026-11-03: autospace squeeze, whole grid cells (F13, F14); three smaller fixes

Next-step 1 from the last entry: line-break diffs over the 65 documents whose page count
differs (`out/breakall.py`, with `breakdiff.py` now skipping TOC leaders and page-number lines).
Groups found and fixed, each measured in WPS first:

- **F13, patch 0015:** an automatic space beside digits shares the squeeze pool, giving up at
  most half itself (mode 14; `sweep_lines.py … autospace`). `gov-mct-keji` now matches (3 → 2
  pages). Mode 15 measured but left open (no consistent rule yet).
- **F14, patch 0018:** on a character grid the section's text width is whole pitches, and a
  closing mark ending a line may reach into the rest (`gov-nmpa-*`: one paragraph cut out and
  bisected in WPS; only removing the grid changed it). Line ends now match WPS in `gov-nmpa-1`,
  `gov-nmpa-6`, `gov-sh-sheitc` and `dc-correspondence-6ce199cdd6`. The score is flat (5
  documents −0.001 to −0.003, 2 better): those documents still differ elsewhere.
- **Patch 0016:** a custom style with the id `Normal` no longer replaces the default paragraph
  style (`dc-administrative-82eb70e86d`: 12 pt instead of 10.5 pt for unstyled text and the grid).
- **Patch 0017:** under `balanceSingleByteDoubleByteWidth` a list label's suffix space is half
  an em (F7; `dc-policies-730d6a61a9`).

Re-scored (0014 → 0018): page-count matches **135 → 136 / 200**, mean page score 0.9082 →
**0.9083**. Layout 589 tests, docx 368 + 134; fmt and clippy clean on the touched crates; the
queue 0001–0018 applies on the pin and reproduces the tested tree.

**Next:**
1. ~~`gov-sh-sheitc`'s 26-character line~~ (F15, done).
2. The remaining one-character-short group (`af80`, `2e6f`, `39de`, `a5be`, `493a`, `377b`,
   `6562`, `8f34`, `f7d3`): most are justified mode-15 lines with digits or Latin text, so F13's
   mode-15 question first.
3. The one-longer group (`gov-nmpa-1/5/6`, `730d`, `896c`, `efac`, `7f0a`, `gov-bj-jtw`,
   `gov-yn-kjt`), with `out/breakall.py` against the 0018 build.
4. Still open from before: folding into #5003/#5004/#5006, v0.1.0 publishing, v0.1.1 tag.

## 2026-11-02 (8): the squeeze pool and the last unit (F12); end of day

F11's open question answered: a line's squeeze pool grows by an em for each compressible mark in
its last unit (`《瑞`, `中。`, `“中`), half an em otherwise (F12, patch 0014, replacing 0012's
two additions). Found with `corpus/tools/sweep_lines.py`, which sweeps any line: the real
`2e6f27b909` line, then variants removing one ingredient at a time.

Re-scored (0013 → 0014): page-count matches **133 → 135 / 200**, mean page score 0.9080 →
**0.9082** (the day's best; it was 0.9042 this morning, with 122 matches). Layout 586 tests; fmt
and clippy clean; the queue 0001–0014 applies on the pin and reproduces the tested tree.

**Where things stand (end of 2026-11-02):** v0.1.0 is a draft release, built before 0008–0014
(publishing it, as a pre-release, is the user's call; a v0.1.1 would carry this work). Patches
0008–0014 are measured in WPS only and not opened upstream; 0011, 0012 and 0014 change code that
is in the open PRs #5003/#5006 and should be folded in with their next push (the user's call).

**Next:**
1. The 65 documents still differing in page count: `out/breakdiff.py` over all of them, then the
   largest group.
2. F12 on justified mode-15 lines (only the line-end cases measured).
3. Fold 0011/0012/0014 into #5003/#5006 and 0013's quote breaks into #5004, once the user agrees.
4. A v0.1.1 tag with the fidelity work, after publishing v0.1.0.

## 2026-11-02 (7): character-grid indents, curly-quote breaks (F11)

A new first-divergence pass (`out/breakdiff.py`: compares line-break offsets in each document's
whole text stream, so span splits and block order don't matter) found 瑞安/台州 documents whose
lines held a character more than WPS's: their first-line indent was one 10.05 pt grid cell where
WPS indents by the 16 pt text's advance (probe 36, F11). With that fixed, lines ending before an
opening curly quote came out a character short: `unicode-linebreak` has no Unicode 15.1 LB19a
breaks beside curly quotes. Both fixed in patch 0013, with the rule that the mark ending a line
keeps its blank (seen in the sweeps).

Re-scored (0012 → 0013): mean page score 0.9077 → 0.9080, 7 documents better, 2 slightly worse;
page-count matches **135 → 133**: four documents lengthened by a page (two had matched). Their
first divergences are now later in the text, at bracket-heavy lines where WPS squeezes more than
F10's half em (open question in F11). Kept, because each change is measured; the half-em limit
is the next thing to revisit. Layout 585 tests; fmt and clippy clean; the queue 0001–0013 applies
on the pin and reproduces the tested tree.

**Next:** what lets bracket-heavy lines squeeze ~0.1 em per side (F11 open question).

## 2026-11-02 (6): the punctuation squeeze limit is per line (F10)

Our generated 函 (`gen-gongwen-04-han`) lost `来。` to a second line: WPS squeezed its one `，` by half
an em, which F1's "0.1 em per mark" forbids. Variants (compatibility modes, alignment) and
width sweeps (`trim_probes.py sweep-k`, about 60 sweeps of 51 widths) replaced the rule: the
limit is per line, ½ em in mode 14 and older, more when the line ends in a closing mark or has
closing-opening meetings, and in mode 15 only on justified lines (F10, patch 0012). The model
also explains F1's original ten-comma data.

Re-scored (0011 → 0012): page-count matches **128 → 135 / 200**, mean 0.9061 → 0.9077, 44
documents better, 5 slightly worse (worst −0.019); all 13 page-count changes moved towards WPS
(7 new matches, none lost). Layout 583 tests; fmt and clippy clean; the queue 0001–0012 applies
on the pin and reproduces the tested tree.

**Next:** the 65 documents still differing in page count; F10's mode-15 additions measured
precisely; 0011 and 0012 into #5003 and #5006.

## 2026-11-02 (5): grid advances keep scale and spacing (F9)

A first-divergence pass over the 57 documents longer than WPS (`out/firstdiv.py`: first line
whose text differs from WPS's, per document) showed one clear group: red-header titles losing
their last character (瑞安市教育局文件 ×5, 济宁市人民政府, 台州市住房和城乡建设局文件, …). On
a character grid, 0002's advance ignored `w:w` and `w:spacing`; probe 35 measured WPS adding the
grid adjustment to the scaled size, spacing on top (F9, patch 0011). Most other first divergences
are text-extraction order (tables, text boxes) or TOC leaders, not line breaks.

Re-scored: page-count matches **125 → 128 / 200**, mean 0.9060 → 0.9061, 7 documents better, one
worse (`dc-policies-af80f1c025`, −0.016, whose red header now matches but whose page count moved
from 19 to 18 against WPS's 17, shifting the page pairing). Layout 581 tests; fmt and clippy
clean; the queue 0001–0011 applies on the pin and reproduces the tested tree.

**Next:** the remaining first-divergence groups (`dc-administrative-82eb70e86d`-style body lines
one character short; TOC pages); 0011 into #5003.

## 2026-11-02 (4): half-em spaces (F7) and line-grid indents (F8)

Following up the worst document from (3), `dc-correspondence-31310a47c6`: its title wasn't breaking
per character after all (`ycmp`-style pairing by sort order misled on a two-column landscape
page). The header's spaces were too narrow. Variants of the document rendered in WPS pinned it on
`w:balanceSingleByteDoubleByteWidth`, which 123 of the 200 corpus documents carry and WordCraft
ignored. Probe 33: with it, a space is half an em, whatever the font or hint; nothing else
changes width (F7, patch 0009: read, written back, laid out).

That alone pushed the sample letter to 10 pages: its signature line (two spaces) no longer fit.
Probe 34 found why: on a line grid with no character pitch, WPS counts `leftChars`/`rightChars`
in the Normal style's size (10.5 pt) and `firstLineChars` in the mark's (16 pt); WordCraft used the
mark's for all three (F8, patch 0010).

Re-scored against WPS (0008 → 0008 + 0009 + 0010): page-count matches **122 → 125 / 200**, mean
page score 0.9056 → 0.9060, 8 documents better, 2 slightly worse (at most −0.003). Probes 31–34
are in `corpus/tools/probes.py` (not in the scored corpus). Layout 580, doc 242, docx 134 + 367
tests; fmt and clippy clean; the queue 0001–0010 applies on the pin and reproduces the tested tree.

**Next:** the 57 documents that still differ in page count; at-least spacing's baseline;
`*Chars` indents with no grid.

## 2026-11-02 (3): v0.1.0 draft release; the 3.7 pt offset found

**v0.1.0** tagged at `b76cb32` (the user's call). The release workflow built and smoke-tested all
seven packages and made a draft release (run 41417997944). Publishing is the user's call; tick
"pre-release", since the version carries no `-alpha` suffix.

**Exact line spacing (F6, patch 0008).** The 3.7 pt vertical offset in `dc-correspondence-195b80f877`
(deferred list) comes from where an exact line puts its baseline. Probe 31, rendered in WPS: always
0.8 × the line height below the top, whatever the font, size or mix (24 combinations, ±0.1 pt).
WordCraft used height minus the font's descent. Patch 0008 changes `baseline_offset` for
`Exactly` (and the empty last line on a grid). The letter's baselines now match WPS within
0.07 pt; they were 3.7 pt low.

Re-scored against WPS: mean page score 0.9042 → 0.9056; page matches unchanged at 122/200 (line
heights don't change); 30 documents better (up to +0.027), 4 worse (the worst,
`dc-correspondence-31310a47c6`, −0.009, is already ~60 pt off for another reason: its title
公告招租 breaks one character per line where WPS keeps it on one line). Layout 578 tests, fmt and
clippy clean; the queue 0001–0008 applies on the pin. Not opened upstream: Word is unmeasured.

**Next:** the 公告招租 title breaking per character (`dc-correspondence-31310a47c6`); the 60
documents still longer than WPS; at-least spacing's baseline.

## 2026-11-02 (2): LoongArch packages

`packaging/linux/package.sh --arch loongarch64` cross-builds on x86_64 with cargo-zigbuild 0.20.1
and Zig 0.14.1 (0.13 fails the final link without a message), against glibc 2.36, and writes
`wenshu-<v>-linux-loongarch64.tar.gz` and `.deb` (Architecture `loong64`, `libc6 (>= 2.36)`).
That is the new-world ABI (loader `/lib64/ld-linux-loongarch-lp64d.so.1`): deepin 23+,
openKylin 2, Loongnix 25, Debian loong64, AOSC, openEuler, Anolis 23. Old-world systems (UOS 20,
Kylin V10, Loongnix 20) need libLoL. No AppImage: AppImage publishes no LoongArch runtime. Only
two crates need a C compiler (`ring` for the update check's TLS, `wayland-backend`), and Zig
covers both. A cross build strips through the linker; `file` checks the binaries' machine.
`WENSHU_STAGE_DIR` moves staging off NTFS (dpkg-deb rejects mode 777 under `/mnt/d`).

Tested under qemu-user 10.0.13 with Debian sid loong64 libraries, on 2026-11-02:

- **CLI:** `gongwen new`, `check` (8/8), `render` and `convert` to PDF. The .docx and the page
  PNG are byte-identical to the x86_64 build's.
- **Desktop:** started from the .deb on WSLg (X11, Mesa lavapipe Vulkan, a minimal debootstrap
  root as the qemu prefix). wgpu picked llvmpipe, the Chinese interface came up as on x86_64
  (`out/linux-shots/la64-zh-1-blank.png`), and the 通知 template opened and checked 8/8 through
  the control channel. The screenshot after the check timed out (emulated software rendering).

Not tested on LoongArch hardware. Sizes: .deb 42 MB, tar.gz 64 MB. The full cross build takes
14 minutes in WSL.

`release.yml` gains a `linux-loongarch64` job (ubuntu-22.04): the same cross build, then the CLI
writes, checks and renders a 公文 under qemu (qemu-user and the loong64 glibc fetched from
snapshot.debian.org by hash, SHA-256-checked).

**Next:** a run on real LoongArch hardware (a 3A5000/3A6000 machine on deepin 23 or Loongnix 25),
and libLoL on an old-world system.

## 2026-11-02: release workflow; open 公文 fonts in the Linux packages

**Release workflow** (`.github/workflows/release.yml`). A `v*` tag builds every package (Windows
zip + MSI on windows-2022, Linux tar.gz/.deb/AppImage on ubuntu-22.04), runs smoke tests and
attaches the packages and `SHA256SUMS` to a **draft** release. A maintainer publishes it by hand.
Running it by hand gives workflow artifacts only. The tag must match the Cargo version. The
binaries are not code-signed.

**Fonts.** Linux systems rarely have a FangSong or Kai design, so 公文 bodies in 仿宋_GB2312 fell back
to Noto Serif CJK (a Song). The Linux packages now ship two OFL-1.1 families, unmodified, in
`/usr/share/wenshu/fonts`: 朱雀仿宋 Zhuque Fangsong v0.212 (8.8 MB) and 霞鹜文楷 GB
LXGW WenKai GB v1.530 (26 MB). Both cover all 6,763 GB2312 hanzi. `package.sh` downloads them at
pinned SHA-256s, with their licences into `notices/`. Patch 0007 (`FontDb::add_font_dirs`) lets
`wenshu-core::init` add `fonts` and `../share/wenshu/fonts` beside the executable (and
`WENSHU_FONTS_DIR`) to the system font scan, so a font loads only when a document uses it.
`wenshu-cjk` lists the two before Fandol and Noto, after the Windows and macOS designs, so Windows
keeps FangSong and KaiTi. Windows packages don't carry the fonts.

Seen in WSL (no FangSong or Kai installed): the 通知 template's body, rendered by `wenshu render`
and in the AppImage, is now in 朱雀仿宋; before, it was Song. **Difference:** Windows FangSong
draws `× · ° ± — … “ ” ‘ ’` full-width; Zhuque draws `×` at 0.506 em and the curly quotes at
0.331 em, so the `××××` placeholders look small and lines holding them can break differently
from WPS on Windows. Fixing that would mean a modified build of the font (OFL allows it).

Package sizes grow: .deb 31 → 45 MB, AppImage 44 → 62 MB, tar.gz 47 → 66 MB. Font tests 103,
wenshu-core and -cjk pass; fmt and clippy clean; the queue (0001–0007) applies on the pin.

## 2026-11-01: the MSI builds and installs; Linux packaging written

**MSI** (`packaging/windows/wenshu.wxs`). With the user's go-ahead, the .NET 8 SDK (8.0.427, winget)
and WiX 5.0.2 (`dotnet tool install --global wix`) are now on this PC. The first build failed:
per-user components key on HKCU values, and WiX generates no GUID for a component that holds
files plus a registry keypath (WIX0230). ICE64 also wanted `RemoveFolder` for `Programs` and
`notices`. Fixed with fixed component GUIDs, `RemoveFolder` entries, and one component for the
notices, each file listed by name. `package.ps1` now fails if the staged notices differ from that
list, so a new upstream font licence can't silently miss the MSI. `wix msi validate` passes; the
only warnings are ICE91, which is expected for a per-user-only package.

Tested `wenshu-0.1.0-windows-x64.msi` (39 MB): a silent per-user install into
`%LOCALAPPDATA%\Programs\Wenshu` with a Start menu shortcut and HKCU keys, no admin prompt. The
installed app, driven through the control channel, seeds the 公文 templates, and 检查公文格式
passes 通知 (8/8). The silent uninstall removes the folder, the shortcut and the keys. Not tested:
a major upgrade from an older MSI (there is none yet).

**Linux** (`packaging/linux/package.sh`, desktop entry, AppStream metainfo, postinst). It follows
WordCraft's script, and builds tar.gz, .deb (with `dpkg-deb`, so nfpm isn't needed) and an AppImage
(appimagetool and the runtime at WordCraft's pinned SHA-256s). The app id is
`io.github.xiaojunyang0805.wenshu`, and names are in Chinese and English.

Built in WSL Ubuntu 22.04 (glibc 2.35; rustup and the build packages installed with the user's
go-ahead; a clone in `~/wenshu`, release build 9m21s): tar.gz 47 MB, .deb 31 MB, AppImage 44 MB.
desktop-file-validate is clean. appstreamcli passes with one note: 0.15 doesn't know `<developer>`,
and WordCraft's metainfo uses it too. The binaries link only libc, libm and libgcc_s.

**Found:** the first install panicked at start on X11 with `libxkbcommon-x11.so could not be loaded`.
WordCraft's dependency list lacks it too. The .deb now depends on `libxkbcommon-x11-0`. The
AppImage doesn't bundle it, so it needs the host's copy, as WordCraft's does.

Tested under WSLg through the control channel: the .deb in `zh_CN` (Chinese interface, 文档1,
Times New Roman 10.5 justified) and in `en_US` (English interface, Document1, Aptos 12: WordCraft's
defaults), and the AppImage in `zh_CN`. Each run seeds the five 公文 templates, and 检查公文格式
passes 通知 8/8.

**Next**

1. A GitHub Actions release workflow (Windows zip + MSI, Linux tar/deb/AppImage) once there's a
   first release to make (the user's call).
2. Open the embedding hook upstream as a PR (the user's call).
3. The deferred WPS work.

## 2026-10-31 (9): M2 next steps done; a Windows package

The user's note: **English is also a basic function of the MVP.** The interface follows the system
language (Chinese or English; File › Options switches it). A blank document gets Chinese defaults
only in a Chinese region, and every other region keeps WordCraft's English defaults (tested in
`wenshu-core`).

1. **Blank documents** (`engine::set_blank_document_hook` in patch 0006, `wenshu_core::blank_defaults`).
   In a `zh` region: 宋体 for East Asian text (新細明體 in TW/HK/MO) and Times New Roman for Latin,
   10.5 pt (五号), English proofing for Latin text with zh-CN as the East Asian language, single
   spacing with no space after, justified, 2.54/3.17 cm margins and a 15.6 pt line grid.
   Seen in the app: Times New Roman 10.5, justified, A4. Pitfall: Git Bash sets
   `LANG=en_US.UTF-8`, so the app launched from it sees an English region; drive it with the
   `LANG`/`LC_*` variables removed.
2. **Community chrome hidden** (`Brand::community`, `Brand::updates`): no Discord button, no
   File › Discord community, no Help › Community Discord, no Check for Updates; the commands
   refuse. The comment-author badge and the Zotero tab stay: they're features, not branding.
3. **检查公文格式** (`services::AppTool`, `AppConfig::app_tools`): Review › 文枢 › 检查公文格式 saves
   the document as DOCX, runs `wenshu-gongwen`'s checker and shows a report, ✔/✘ per rule in
   Chinese and English. Seen: a blank page fails 7 of 8, the 通知 template passes all 8. Reports
   use the interface font; monospace had no CJK glyphs.
4. **Windows package** (`packaging/windows/package.ps1`): a statically linked (+crt-static)
   portable zip with `wenshu-desktop.exe`, `wenshu.exe`, the licences, WordCraft's
   NOTICE/ATTRIBUTION/licences and the bundled fonts' OFL licences (47 MB). The zip is written
   with Windows' bsdtar for portable paths. Tested: it extracts with unzip, the CLI runs, and the
   app starts from it, seeds its templates and runs the check. `wenshu.wxs` (per-user MSI) is
   written but **not built**: no .NET SDK or WiX on this PC, and installing them is the user's
   call. Linux packages need a Linux machine.

Upstream branch `embed/app-library`: app 52 + 3, engine integration 42, ui-egui 1,012 (the one
failure is the pre-existing Windows backstage test), control-key 4, clippy clean.

**Next (after the break)**

1. Install WiX (with the user's go-ahead) and build and test the MSI.
2. Linux packages (AppImage or .deb) on a Linux machine or CI.
3. Open the embedding hook upstream as a PR (the user's call).
4. The deferred WPS work (below).

## 2026-10-31 (8): M2 started — the desktop alpha runs

**Upstream hook, patch 0006** (`embed/app-library` in the worktree, not opened upstream yet):
`apps/wordcraft` becomes a library. `wordcraft::run(AppConfig)` is the former `main()`, and
`main.rs` runs it with WordCraft's defaults. `AppConfig` carries:

- the product (`ui_egui::brand::Brand`: name, version and About text, used in window titles, the
  About box and the unsaved-changes prompt);
- the executable name (`--version`, command-line errors);
- the settings folder (`control_key::set_folder_name`; Rust 2024 forbids setting
  `WORDCRAFT_PROFILE_DIR` from inside the process without `unsafe`);
- the app id and icon;
- a start-up hook that runs after logging is installed;
- the update check on or off.

WordCraft's own behaviour is unchanged; app 52 + 3 tests, control-key 4, clippy clean.

**`apps/wenshu-desktop`** (`wenshu-desktop`): brand 文枢, an original icon drawn in code, settings
in `%APPDATA%\Wenshu`, the update check off. `wenshu_core::init()` runs at start-up. Five 公文
templates (通知 请示 报告 函 纪要) are seeded into `Templates/公文` on first start: built with
`wenshu-gongwen`, saved as `.dotx` through the engine, never overwriting an existing folder. A test
checks that they pass `gongwen check`. The workspace mirrors upstream's egui/winit patches;
`sync-upstream.sh` checks out craft-i18n at WordCraft's pin, and `.cargo/config.toml` (now
committed) sets `CRAFT_I18N_DIR`.

**Seen running** (control channel, scratch profile): the whole interface in Chinese
(the system language), the window titled 文档1, and File › New › Personal listing the 公文 category
with its five templates.

**Next (M2)**

1. Blank documents with Chinese defaults: A4, 宋体/等线 五号, zh-CN proofing language. This needs
   a hook for the blank document, or a Normal template from the app.
2. Hide upstream's community chrome (Discord button, account badge, Zotero tab) through the brand.
3. A "检查公文格式" command (`wenshu-gongwen::check`) on the ribbon.
4. A Windows installer, then Linux packages.

## Deferred (come back later)

Set aside on 2026-10-31 to move on to M2, at the user's request:

- **The 60 documents still longer than WPS** (mostly on line and character grids). (The 3.7 pt
  vertical offset in `dc-correspondence-195b80f877` is done: F6, patch 0008.) Start from
  `out/sumref.py`-style tag statistics and the `ycmp.py` line comparison.
- `w:topLinePunct` (F5) and kana compression.
- #5005: Latin Extended-A to a GB2312/Big5 East Asian font (F4).
- A Word reference set (MVP open question 1).

## 2026-10-31 (7): patches 0002 and 0005 reworked to the measurements

**0002 / #5003** (`8e2323d`): grid text advances its size plus the grid adjustment
(`Grid::advance`), not whole cells. Probe 30 confirmed it for smaller text too: 12 pt text
advances 11.8 pt.

**0005 / #5006** (`c275702`): the pair model is gone. Compressible full-width marks carry a 0.1 em
squeeze capacity (`ParaLayout::squeeze`). The line fitter counts it like shrinkable spaces, and
`squeeze_line` shares an overflow across the marks so the line ends at the margin. A closing
mark's blank half goes into `auto_space`, which takes no room at a line's end. Last round's
line-end patch-ups in `lines.rs` are gone again (back to upstream). A line-end probe settled the
set: `。，、：；！）》」』】〕` get the free half; `” ’ ？` don't. It applies on character grids.
`wenshu-cjk::punct` follows the same model (`squeeze_em`, `line_end_blank_em`).

**Re-scored against WPS:** page-count matches **112 → 122 / 200**, mean page score 0.903 → 0.904,
28 documents better, 4 slightly worse (the worst by −0.022). The sample letter
(`dc-correspondence-195b80f877`) now breaks every line where WPS does: 9 pages, was 11.

Upstream: branches pushed, #5003/#5006 descriptions rewritten, notes posted with
`@coderabbitai review`; #5005 rebased. Combined branch: layout 578, fonts 102, doc 242,
docx 500, engine 1094; fmt and clippy clean.

**Still long against WPS** (60 documents): mostly line grid (36/82) and character grid (17/27).
In the sample letter, everything from the first body paragraph sits 3.7 pt lower than in WPS: a
paragraph-spacing or grid-snap difference, the next lead.

**Next**

1. Find the 3.7 pt vertical offset and the remaining line-grid overshoot.
2. #5005: Latin Extended-A to a GB2312/Big5 East Asian font (F4).
3. Watch CodeRabbit on the reworked PRs; then mark them ready (the user's call).

## 2026-10-31 (6): how WPS compresses punctuation, measured

`corpus/tools/trim_probes.py` gives 2,361 marks measured across six fonts, four sizes and four
font setups, compression on against off, plus a width sweep. `wps_render.py` gained `--files`,
`--out` and `--no-png` for scratch sets. **The model** ([WPS-FINDINGS.md](WPS-FINDINGS.md) F1):

- **Overflowing lines only:** when the next unit would overflow, every compressible side on the
  line shrinks equally, up to **0.1 em** each, just enough to end at the margin. Lines that fit
  are untouched, and **adjacent pairs are never closed up**.
- **Line end:** a closing mark ending a line doesn't need room for its blank half.
- **Compressible:** the right side of `。，、：；！）》」』】〕”’·` and the left side of
  `（《「『【〔“`, when the glyph is full-width. Never `？ — …`.
- The same on character grids. The probe 05 "hint" puzzle was a long unbreakable chunk that
  couldn't fit even at full squeeze.

This replaces 0005's layout half (pair compression) with a line-fitting rule. The setting's
read/write half stays as it is. Upstream's breaker already shrinks spaces for justification
(`shrink_spaces`); the squeeze would be a second shrinkable resource.

**Next**

1. Rework 0005: replace pair compression with the squeeze and the line-end half; and 0002: the
   delta rule for grid text. Re-score the corpus.
2. Comment on #5003/#5006 with the measurements before maintainers review.
3. Word references (MVP open question 1).

## 2026-10-31 (5): WPS reference renders and first measurements

**Done:** `corpus/tools/wps_render.py` drove WPS 12.1 through COM. All 200 documents rendered
with no failures: 1,952 pages, a PDF for each (local, `corpus/ref/`, 573 MB).

**Scored baseline:** mean page score **0.903**; page counts match on **112 / 200**. Real WPS
documents score 0.909 with 34/76 page matches; real Word documents 0.888 with 32/71; generated
documents 0.969 with 41/42. Where counts differ we are nearly always longer.

**Findings** (details and numbers in [WPS-FINDINGS.md](WPS-FINDINGS.md)):

1. **Compression (0005 / #5006) follows the wrong model.** WPS trims a little (about 0.1 em)
   from the blank side of *every* full-width mark, whatever its neighbour, and does so on
   character grids too. We take half an em only between adjacent pairs, and nothing on a grid.
   This is the main cause of the longer page counts: WPS fits one more character on most lines.
   Odd: a run with `w:hint="eastAsia"` got no trimming at all.
2. **Grid large text (0002 / #5003) is wrong too.** WPS gives every character the grid delta
   (advance = own size + pitch − default size): 22 pt text is 21.8 pt on a 15.8 pt grid, not two
   cells. This matches ECMA-376's definition of `w:charSpace`.
3. **Grid indents (0002) confirmed:** two characters is exactly two pitches, whatever the mark
   size.
4. **Hint (0003 / #5005):** WPS also sends Latin Extended-A (pinyin `ě ī ō ū ā`) to a GB2312
   East Asian font, without any language tag. It keeps `á à ü ǎ` in the Latin font even with
   zh-CN.

**What this means for the upstream PRs:** keep #5003, #5005 and #5006 as drafts. #5003 needs the
delta rule, which the spec backs. #5005 can add the Extended-A row once the app can say which
fonts are GB2312/Big5. #5006 needs a new compression model, and the trim amounts must be
measured first. All of this is WPS evidence. Upstream targets Word, so a Word reference (or the
spec) is needed before claiming it for Word. #5002 and #5004 are unaffected.

**Next**

1. Measure the compression trim properly: per mark, size and font, from new probes. Explain
   why the hint disables it.
2. Rework 0002 (delta rule) and 0005 (trim model) in the patch queue, then re-run the scored
   corpus to see page matches move.
3. Tell the maintainers on #5003/#5006 what the measurements show, before they review.
4. Find a way to get Word references (MVP open question 1).

## 2026-10-31 (4): the 200-document corpus

**Landed:** `corpus/` now holds 200 Chinese documents with one manifest (`corpus/manifest.csv`)
and its tools. See [corpus/README.md](../corpus/README.md).

| Tier | Count | Source | In git |
|---|---|---|---|
| `dc` | 137 | Real web documents from [docx-corpus](https://github.com/superdoc/docx-corpus) (Common Crawl, ~35,000 `zh` files), picked from a seeded 1,420-file pool | No: URL + SHA-256, `tools/fetch.py` |
| `gov` | 21 | Attachments from Chinese government sites (PRC Copyright Law art. 5), URLs in `corpus/sources/gov.tsv` | Yes |
| `gen` | 42 | 30 probes (one per layout question), 10 公文 of ten kinds, 2 M0 samples | Yes |

The 158 real documents were written by WPS (76), Word (71) and others (11).

**The survey shows what Chinese documents use** (1,420-file pool): punctuation compression is on
in 77% and `w:hint="eastAsia"` appears in 83%, so 0003 and 0005 are mainstream behaviour.
`*Chars` indents 64%, line grid 70%, character grid 6%, exact spacing 59%. Vertical text,
columns and kinsoku-off are rare (1–3%), but each is in the pick.

**Baseline run** (`wenshu fidelity`, no references yet): all 200 open and render, no failures;
layout median 5 ms, slowest 0.3 s. Against the page count each authoring app saved
(`docProps/app.xml`, a rough proxy), we match on 52 of 148 real documents and nearly always
come out **longer**. The overshoot follows the document grid: 63% of documents on a line grid
run long (81% on a character grid), against 23% with no grid and 24% with compression off. So
line-grid snapping making lines too tall is the first thing to check against WPS. Missing 方正
fonts on this PC are the second suspect.

A probe bug was found and fixed on the way: python-docx's 11 pt default made the 16 pt grid
probes measure the wrong base. Probes on a 公文 grid now set 16 pt 仿宋 as the default, as real
公文 do.

**Next**

1. Run `corpus/tools/wps_render.py` (after warning the user; WPS may show windows) to fill
   `corpus/ref/`, then the harness gives real scores and page-count matches.
2. Investigate the line-grid page overshoot against those references.
3. Answer the probe questions (01–10) from the WPS renders.
4. Mark #5002–#5006 ready (the user's call).

## 2026-10-31 (3): status review

**Where M1 stands** (day 2 of a 3–5 week milestone)

| Part | State |
|---|---|
| Layout patches 0002–0005 (+ hook 0001) | Done. The fourth CodeRabbit round found nothing new and every thread is closed or answered. Drafts, so no maintainer has looked yet |
| Corpus | 4 synthetic files in `corpus/files` against a target of 200 real documents |
| WPS reference renders | Not started (WPS 12.1 with COM is on the dev PC) |
| Open layout questions | 6, all waiting for renders (below) |

Upstream `main` is 12 commits past the pin `3b7f7ed`. The PRs still report mergeable.

**Questions only a reference render can settle**

1. Compression on a `linesAndChars` grid. 0005 says off, so 公文 bodies are unchanged.
2. `。”` with the quote in a Latin font: compressed (0005) or not.
3. Line-start and line-end compression (`w:topLinePunct`), and fitting a pair at its compressed
   width.
4. Large text on a grid: `ceil(size / default)` cells.
5. `*Chars` indents: one grid pitch per character whatever the mark size.
6. The `w:hint="eastAsia"` rows that need a Chinese `w:lang` (pinyin vowels, Latin Extended).

**Next** (in order)

1. **Mark #5002–#5006 ready for review** (the user's call). #5002, #5003 and #5004 stand alone;
   #5005 stacks on #5003 and #5006 on #5004. Then follow the maintainers' review and rebase on
   upstream `main` when asked. Bump the pin to match once they merge.
2. **WPS render script**: docx → PDF → PNG through `KWps.Application` COM. Warn the user first,
   because WPS may open windows. Start `corpus/ref/` (local only) with the 公文 sample and one
   test document per question above.
3. **Settle the six questions** from the renders, and fix the patches where WPS disagrees.
4. **Grow the corpus** to 200 real documents. Sources need the user's input: public government
   公文 (many are PDF or OFD only), the user's own documents, or generated variants. Then run a
   first fidelity scoreboard (`wenshu fidelity`) as the M1 baseline.
5. **Follow-ups** after the PRs merge: read `w:lang` for the hint rows; line-end compression;
   the upstream docs (gaps, layout-parity, ROADMAP) in the last PR to merge.
6. **Prepare M2** (desktop alpha) once the M1 corpus is in place.

## 2026-10-31 (2): third review round

CodeRabbit withdrew the `full(n)` finding. New points:

- #5006 (5b7b229): a split pair's line end x (and so selection, `cl_right`) covers the whole
  first mark. Declined: fitting a pair at its compressed width. The fitter can't yet know
  whether the line breaks between the two marks; that is part of the line-end compression work
  (`topLinePunct`). Declined: bundling a CJK test font.
- #5005 (6bb57f1): a named East Asian font that resolved to the generic fallback is not used
  as the East Asian face (the check compares against the fallback face, which avoids touching
  #5002's `resolve`). Its test runs only where the name really falls back; on this Windows box
  Leelawadee UI covers Khmer, so the test returns early here.
- #5002 (4eba2f2): the provider is asked for Symbol on `Symbol (PCL6)`, through one
  `base_family` helper.
- #5003: docs again; same answer.

Upstream layout 577, fonts 102, doc 242, docx 500, engine 1094; fmt and clippy clean; queue applies on the pin.

## 2026-10-31: second review round on #5002–#5006

CodeRabbit resolved every first-round thread and raised new points. Still no human reviews.

- #5006 (fd4aa1d): compression is undone at a line's end, like auto-spacing's room (both now
  live in `ParaLayout::auto_space`). A `）|（` pair split across lines leaves `）` at full width.
  Declined: requiring the second mark full-width. In `。”` the blank that goes is `。`'s own,
  whatever font the quote is in. Also declined: asserting that the fixture compresses, because
  machines with no CJK font have no full-width marks.
- #5005 (3f8059f): `w:hint="eastAsia"` follows Word's whole block table (MS-OI29500 §2.1.88):
  Greek, Cyrillic, U+2000–27BF, the listed Latin-1 signs, PUA. The rows that need a Chinese run
  language (pinyin vowels, Latin Extended) wait until layout reads `w:lang`.
- #5003 (e6b7a3d), #5002 (8cbf960): test coverage only. Grid indents are checked with a large
  paragraph mark and on the left and right. The provider test can tell exact-name, base-name
  and built-in results apart.
- Docs: still deferred to the last PR to merge. CodeRabbit leaves the timing to the maintainers.

The queue is regenerated and applies on `3b7f7ed`. Upstream layout 576, fonts 102, doc 242,
docx 500 and engine 1094 tests pass; fmt and clippy clean.

**Open for WPS renders**: `。”` with a Latin-font quote (compress or not), and the pinyin-vowel
hint rows.

## 2026-10-30 (3): first review round on #5002–#5006

CodeRabbit reviewed the drafts (triggered with `@coderabbitai review`). There are no human reviews
yet. Fixed in the `cjk/*` branches and the patch queue:

- #5006: compression now changes the layout-cache key (`env_hash`), so toggling it relays out
  cached paragraphs. A proportional first mark (Latin “ before 《) keeps its width.
- #5004: the kinsoku sets apply only beside East Asian text, so `rose 5 ‰` keeps its break.
- #5003: the cell tolerance is relative 1e-4, so 54.5 pt on a 54 pt grid takes 2 cells.
  Character indents are now one character pitch per character, whatever the mark size; to be
  confirmed with a WPS render.
- #5002: the provider is asked with the document's own font name first; when none of its
  candidates is installed, the built-in lists still apply.
- Declined: putting 仿宋_GB2312-style mappings into WordCraft (#5005). That data comes from the
  app through the #5002 hook.
- Docs (gaps, layout-parity, ROADMAP): offered in the last PR to merge, or a follow-up, to avoid
  five-way conflicts on the parity doc's revision line.

Upstream tests on the merged branches: layout 574, docx 500, doc 242, fonts 102, engine 1094;
fmt and clippy clean.

## 2026-10-30 (2): M1 Chinese layout patches, upstream PRs

**Landed** (`fd5bca0`, `b5c920b`)

- WordCraft patch queue grows from 1 to 5 (`patches/wordcraft/`, details in [UPSTREAM.md](UPSTREAM.md)):
  - 0002 character grid: large text takes whole cells (22 pt titles no longer overlap);
    `*Chars` indents count grid cells (公文 body lines hold 28 characters).
  - 0003 `w:hint="eastAsia"` read, written and applied: “” —— …… drawn full-width in the
    East Asian font; named substitutes such as 仿宋_GB2312 → FangSong kept as the EA face.
  - 0004 kinsoku: ” ’ 〃 〆 ‰ ℃ never start a line, “ ‘ never end one.
  - 0005 punctuation compression (标点挤压): `w:characterSpacingControl` is read and written.
    Before this, every save wrote `doNotCompress`, so a WPS document's setting was lost.
    Adjacent full-width marks (`。”` `）、` `”（` `《“`) now share one blank half.
- `wenshu-cjk::punct`: dashes and ellipses may start a line (`no_break_between` keeps them
  whole); the compression pairs match 0005.
- Upstream: draft PRs [#5002](https://github.com/storytold/wordcraft/pull/5002)–[#5006](https://github.com/storytold/wordcraft/pull/5006)
  on storytold/wordcraft from `xiaojunyang0805/wordcraft` branches `cjk/*`. #5005 builds on
  #5003 and #5006 on #5004.

**Verified**

- Queue applies in order on the pin `3b7f7ed`, and each PR applies on upstream `5b5810f`.
- On the pin: `wordcraft-layout` 571, `-docx` 500 and `-doc` 242 tests pass; Wenshu workspace
  tests pass.
- Upstream `cargo xtask ci` on all five merged: fmt, clippy, wasm, assets and layers pass.
  Three tests fail on Windows on unmodified upstream `main` too (mcp `automation_roots` dead
  code, `proof` hunspell, `ui-egui` backstage); they are listed in each PR.

**Decided**

- Standard East Asian typography sets (kinsoku, compression pairs) go upstream with the layout
  code; regional data (font names) stays in Wenshu behind hooks.
- Compression is off on a `linesAndChars` grid, like auto-spacing, so 公文 bodies are
  unchanged until a reference render says otherwise.
- Upstream credit: GitHub username only.

**Found**

- A licensed WPS Office 12.1.0.29161 is installed on the dev PC, with COM automation
  (`KWps.Application`) registered. This answers MVP open question 1 for local renders.

**Next**

1. Script WPS docx → PDF/PNG through COM, and start `corpus/ref/` with the 公文 sample and
   the quote/punctuation cases.
2. Use those renders to settle three open layout questions: compression on a character grid,
   line-start/end compression (`w:topLinePunct`), and the `ceil` cell rule for large text.
3. Grow the corpus towards 200 real documents.
4. Answer review comments on #5002–#5006, and drop each patch from the queue once it merges.

## 2026-10-30 (1): M0 foundations

`2b856ed`. Covers the kit evaluation, the move to WordCraft plus a patch queue, the rename from
OpenWPS, the workspace and crates (`wenshu-cjk`, `-gongwen`, `-core`, `-fidelity`, `wenshu` CLI),
patch 0001 (font substitute hook), the 公文 generator and checker, and the fidelity harness.
See [MVP.md](MVP.md) and [adr/](adr/).
