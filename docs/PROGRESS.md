# Wenshu progress log

Newest first. One entry per working session: what landed, what was verified, what's next.

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
