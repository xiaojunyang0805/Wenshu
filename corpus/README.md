# Wenshu fidelity corpus

200 Chinese .docx documents that Wenshu must open, lay out and save the way WPS and Word do. The
fidelity harness (`wenshu fidelity corpus out/fidelity`) renders each one and, when a reference
render exists, compares each page with it.

## Composition

| Tier | Files | Count | Source | Licence basis | In git |
|---|---|---|---|---|---|
| `dc` | `dc-<type>-<id>.docx` | 137 | Real documents from the web, picked from the Chinese part of [docx-corpus](https://github.com/superdoc/docx-corpus) (Common Crawl; ~35,000 `zh` files) | Copyright of their authors | **No**: fetched by URL and SHA-256 (`tools/fetch.py`) |
| `gov` | `gov-<slug>.docx` | 21 | Attachments published on Chinese government sites; URLs in `sources/gov.tsv` | PRC Copyright Law art. 5: state-organ documents of an administrative nature are outside copyright | Yes |
| `gen` | `gen-probe-*.docx` | 30 | One small document per layout question (`tools/probes.py`) | Wenshu's own text | Yes |
| `gen` | `gen-gongwen-*.docx` | 10 | 公文 of ten kinds and 1–4 pages, built with `wenshu gongwen new` (`tools/gongwen_variants.py`) | Wenshu's own text | Yes |
| `gen` | `01_gongwen.docx`, `02_mixed_cjk.docx` | 2 | The M0 evaluation samples (`make_corpus.py`) | Wenshu's own text | Yes |

`manifest.csv` has one row per document: tier, source and URL, SHA-256, licence, the app that
wrote it, its page count, its type and its feature tags. The `.xlsx` and `.pptx` in `files/` are
for M3 and are not part of the Writer corpus.

The real documents (`dc` + `gov`, 158 in all) were written by WPS (76), Word (71) and others
(11, such as PHPWord and LibreOffice).

## What the real documents told us

A survey of a random 1,420-document pool from docx-corpus (`tools/survey.py`) shows what Chinese
documents actually use, and so what the layout must get right:

| Feature | Share of the pool | Note |
|---|---|---|
| `w:characterSpacingControl` = `compressPunctuation` | 77% | Punctuation compression (0005) applies to most documents; only 22% say `doNotCompress` |
| `w:hint="eastAsia"` | 83% | The hint (0003) is the norm, not an edge case |
| `*Chars` indents | 64% | Character indents (0002) |
| Line grid (`lines`) / character grid (`linesAndChars`) | 70% / 6% | Character grids are mostly 公文 |
| Exact line spacing | 59% | |
| Tables / merged cells | 54% / 35% | |
| Written by WPS / Word | 36% / 52% | 11% are machine-made (PHPWord) |
| Vertical text, columns, kinsoku off | 1.8% / 1% / 2.7% | Rare, but each is in the pick |

The pick (`tools/select.py`) is deterministic. It keeps Chinese documents (CJK share ≥ 0.3) of
at most 40 pages, drops textual duplicates, gives every rare feature at least three documents,
and then fills app quotas (45% WPS, 45% Word) with whatever adds the most uncovered tags.

## Probes

Each probe isolates one behaviour, so a single reference render answers a single question.
Probes 01–10 are the open questions in [docs/PROGRESS.md](../docs/PROGRESS.md).

| File | Question |
|---|---|
| `gen-probe-01-compress-on-char-grid` | Does compression apply on a linesAndChars grid (公文 body)? |
| `gen-probe-02-compress-line-grid` | Compression on a lines-only grid: which pairs close up, by how much? |
| `gen-probe-03-compress-off` | Control for 02: doNotCompress. |
| `gen-probe-04-quote-latin-font` | 。” and ”（ with the quotes in the Latin font (no hint): compressed? |
| `gen-probe-05-quote-hinted` | Same as 04 with w:hint=eastAsia: quotes full-width and compressed. |
| `gen-probe-06-line-end-pairs` | A ）（ pair split by the wrap: does the line end keep the blank half? |
| `gen-probe-07-top-line-punct` | Same as 06 with w:topLinePunct: line-start/end punctuation compressed? |
| `gen-probe-08-grid-large-text` | How many grid cells does text larger than the default take (18/22/26/54.5 pt)? |
| `gen-probe-09-char-indents` | *Chars indents: one grid pitch per character, whatever the paragraph mark size? |
| `gen-probe-10-hint-pinyin` | w:hint=eastAsia with w:lang eastAsia=zh-CN: pinyin vowels, Greek, ½ in which font? |
| `gen-probe-11-kinsoku` | Kinsoku: no ，。、”）》 at line start, no “（《 at line end. |
| `gen-probe-12-kinsoku-off` | Same as 11 with w:kinsoku=0. |
| `gen-probe-13-autospace` | autoSpaceDE/DN on and off between Chinese, Latin and digits. |
| `gen-probe-14-dash-ellipsis` | —— and …… at line breaks: kept whole, may start a line? |
| `gen-probe-15-emphasis-scaling` | 着重号 (w:em), character scaling (w:w) and spacing (w:spacing). |
| `gen-probe-16-combine-vert` | 双行合一 (eastAsianLayout combine) and 纵横混排 (vert). |
| `gen-probe-17-ruby` | 拼音指南 ruby above Chinese characters. |
| `gen-probe-18-vertical` | 竖排: a tbRl section. |
| `gen-probe-19-columns` | Two columns with a separator line, balanced on the last page. |
| `gen-probe-20-numbering` | 公文 heading levels 一、（一）1.（1） typed by hand. |
| `gen-probe-21-table-on-grid` | A table with CJK text and merged cells on a character grid. |
| `gen-probe-22-fonts-by-name` | Font names as documents use them: which substitute does each get? |
| `gen-probe-23-line-spacing` | Exact 28.95 pt, multiple 1.5, at least 20 pt; with and without snap to grid. |
| `gen-probe-24-fullwidth-forms` | Full-width letters and digits, ideographic-space indents, half-width katakana. |
| `gen-probe-25-footnotes` | CJK footnotes with circled numbers. |
| `gen-probe-26-red-header` | 红头: red organ mark, document number and the red separator rule (paragraph border). |
| `gen-probe-27-traditional` | Traditional Chinese text with 新細明體/PMingLiU and 「」『』 brackets. |
| `gen-probe-28-kana-compression` | compressPunctuationAndJapaneseKana: kana closes up too? |
| `gen-probe-29-snap-off-in-grid` | Paragraphs with snapToGrid off inside a character-grid section. |
| `gen-probe-30-mixed-sizes-line` | One line mixing 16 pt body, 22 pt and 12 pt runs on a line grid: line height. |

## Reference renders (`ref/`, local only)

`ref/<stem>.p<N>.png` are page images of each document as WPS (and later Word) lays it out,
made on a machine with a licence. They stay out of git (`.gitignore`) until it's settled whether
WPS output may be shared. The harness scores a page as `1 − share of differing pixels` and
reports page-count matches separately.

## Rebuilding

```sh
python corpus/tools/fetch.py                     # download the dc tier (checks each SHA-256)
wenshu fidelity corpus out/fidelity              # render and score
```

To regenerate or change the corpus:

```sh
python corpus/tools/probes.py corpus/files                       # probes
python corpus/tools/gongwen_variants.py <path-to-wenshu> corpus/files
python corpus/tools/survey.py <dir> > survey.jsonl              # survey any .docx folder
python corpus/tools/select.py survey.jsonl pool.tsv 137 > picked.jsonl
python corpus/tools/manifest.py --picked picked.jsonl --pool <pool> --gov <dir> --gen <dirs…>
```

The pool for the current pick was 1,420 documents drawn with seed `20261031` from the docx-corpus
`lang=zh` manifests, weighted by type (forms, policies, reports and administrative documents
first).

## Rules

- Never commit a `dc` file. Its row in the manifest (URL + hash) is enough to fetch it again.
- A takedown request for a `dc` document: drop its row and its file, and tell docx-corpus
  (`help@docxcorp.us`).
- `gov` documents carry their URL. Add only documents a state organ itself published.
- Probes and generated 公文 contain only our own text. Never paste text from elsewhere into them.
