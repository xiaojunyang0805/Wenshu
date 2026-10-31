# Wenshu progress log

Newest first. One entry per working session: what landed, what was verified, what's next.

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
