# What WPS does: measured findings

*v0.3 · 2026-10-31 · WPS Office 12.1.0.29161 on Windows 11*

Measured from WPS's own PDF exports of the corpus (`corpus/tools/wps_render.py`). PyMuPDF reads
each character's origin from the PDF, so advances are exact to about 0.1 pt. The renders and PDFs
stay local (`corpus/ref/`). Each finding names the probe it comes from (`corpus/files/gen-probe-*`).

## Scored baseline

`wenshu fidelity corpus` against the WPS references, before any change:

| Group | Mean page score | Page count matches |
|---|---|---|
| All 200 | **0.903** | **112 / 200** |
| Real, written by WPS (dc + gov) | 0.909 | 34 / 76 |
| Real, written by Word (dc + gov) | 0.888 | 32 / 71 |
| Real, other apps | 0.858 | 5 / 11 |
| Generated (probes, 公文) | 0.969 | 41 / 42 |

Where pages differ, we are nearly always **longer** (+1: 39, +2: 13, +3 or more: 19; shorter: 17).

| Real documents running longer than WPS | Share |
|---|---|
| Character grid, compression on | 22 / 27 |
| Line grid, compression on | 41 / 82 |
| No grid, compression on | 3 / 13 |
| No grid, compression off | 3 / 31 |

The line height matches. On a line-grid letter that runs two pages long
(`dc-correspondence-195b80f877`), WPS and Wenshu both use a 30 pt pitch, but **WPS fits one more
character on most lines** (26 against 25). The cause is how WPS compresses punctuation (F1).

## F1. Punctuation compression squeezes overflowing lines; it never closes up pairs

Probes 01–07, then `corpus/tools/trim_probes.py`: six fonts × four sizes × four font setups, each
with compression on and off (2,361 measured marks), and a width sweep from 20 em down to 17 em in
0.05 em steps. `w:characterSpacingControl="compressPunctuation"` does exactly two things in WPS:

1. **A line may squeeze its compressible marks to fit more text.** When the next unit would
   overflow, every compressible side on the line gives up the same amount, just enough for the
   line to end exactly at the margin (left-aligned lines too). **The limit is 0.1 em per mark.**
   That held in 宋体 and 仿宋 at 10.5 and 16 pt: ten `，` absorb 1.0 em at a width of 18.5 em, and
   at 18.45 em the line breaks earlier. If the next unit doesn't fit even at the limit, nothing
   is squeezed (probe 05: an unbreakable `〔2026〕“甲”` chunk).
2. **A closing mark that ends a line doesn't need room for its blank half.** Twenty characters
   ending in `，` fit down to 19.5 em with no squeeze. With `doNotCompress` they need the full 20 em.

Lines that fit are left alone. **Adjacent pairs (`。”` `）（` `、《`) keep their full advances**:
there is no JIS-style closing-up of pairs.

Compressible sides, when the mark's glyph is full-width (in 宋体, 仿宋, 楷体 and 黑体 alike):

| Side | Marks |
|---|---|
| Right (the blank after the ink) | `。，、：；！）》」』】〕”’·` |
| Left (the blank before the ink) | `（《「『【〔“` |
| Never | `？ — …` |

In 微软雅黑 and 等线, several marks have proportional glyphs and are left alone, which a glyph-width
check like 0005's (advance ≥ 0.9 em) reproduces.

- **It applies on a character grid too** (probe 01: about 0.1 em off marks on a 15.8 pt pitch).
- `doNotCompress`: no squeeze and no free line-end half.

**Done in 0005 / #5006** (2026-10-31): the squeeze (`ParaLayout::squeeze`, `lines::squeeze_line`)
and the free line-end half (`auto_space`) replace the pair model. Re-scored: page-count matches
112 → 122 of 200, mean page score 0.903 → 0.904, 28 documents better and 4 slightly worse. The
sample letter now breaks every line where WPS does (9 pages, was 11).

## F2. On a character grid, every character gets the grid delta, whatever its size

Probe 08: 16 pt default on a 15.8 pt pitch (`charSpace=-842`).

| Text size | WPS advance | Size + (pitch − default) |
|---|---|---|
| 16 pt | 15.8 | 15.8 |
| 18 pt | 17.8 | 17.8 |
| 22 pt | 21.8 | 21.8 |
| 26 pt | 25.8 | 25.8 |
| 54.5 pt | 54.3 | 54.3 |

Larger text does **not** take whole cells. This is how ECMA-376 defines `w:charSpace`: extra
space added to each character. Upstream's `round` cell rule and 0002's first `ceil` both differed; 0002 now implements this rule.

**Done in 0002 / #5003** (2026-10-31): `Grid::advance` = own size + delta; 12 pt text measured 11.8 pt as well (probe 30).

## F3. `*Chars` indents are grid pitches

Probe 09: `firstLineChars=200` with a 22 pt paragraph mark indents 31.68 pt, which is 2 × 15.84.
`leftChars`/`rightChars=200` indent 31.56 pt. **Confirms 0002's indent unit** (one pitch per
character, whatever the mark size).

## F4. Which characters `w:hint="eastAsia"` sends to the East Asian font

Probe 10: run fonts Times New Roman (ascii/hAnsi) and 宋体 (eastAsia), hint set.

- To 宋体: `ě ī ō ū ā` (Latin Extended-A), Greek, Cyrillic, `½ ¡ ¿`. This happens **without a
  Chinese `w:lang`**: MS-OI29500's "or the East Asian font's character set is GB2312" condition,
  and 宋体 is a GB2312 font.
- Kept in Times New Roman, **even with `w:lang w:eastAsia="zh-CN"`**: `á à ü` (Latin-1 vowels,
  which MS-OI29500 conditions on the language) and `ǎ` (Latin Extended-B).
- Without the hint, everything stays in Times New Roman.

**Consequence for 0003 / #5005:** add Latin Extended-A when the East Asian font is a GB2312 or
Big5 face (a font-charset check, so a font list in the app's data). The language-conditioned
rows can stay out.

## F5. Line ends

Covered by F1: with compression on, a closing mark at a line's end doesn't need room for its blank
half (measured with `，`). `w:topLinePunct` (probe 07) changes the squeeze slightly (about 1.0 pt
instead of 1.4 pt for `）` in 10.5 pt 宋体). Not yet modelled.

## Still to measure

- Whether every right-compressible mark gets the free line-end half, not only `，` (F1).
- How the squeeze interacts with justification (`jc=both`) and kinsoku.
- `w:topLinePunct` (F5).
- Word's behaviour for each finding. Upstream targets Word, so changes to #5003/#5006 need
  either Word references or an argument from the spec. F2 has one; F1 does not yet.
