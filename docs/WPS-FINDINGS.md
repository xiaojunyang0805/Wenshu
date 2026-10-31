# What WPS does: measured findings

*v0.1 · 2026-10-31 · WPS Office 12.1.0.29161 on Windows 11*

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

## F1. Punctuation compression is a small trim on every mark, not a half-em between pairs

Probes 01–05; `w:characterSpacingControl="compressPunctuation"`.

- **Every full-width closing mark and 。，、：； loses a little from its blank right side,
  whatever follows it.** `。` followed by 他 is trimmed as much as `。` followed by `”`. Opening
  marks `（《【〔` lose about as much from their blank left side. That shows as a shorter
  advance for the character before them. `？` is not trimmed.
- **The trim is small:** in 10.5 pt 宋体, about 0.9–1.0 pt per mark (about 0.09 em), and about
  2 pt for `、`. In 12 pt 宋体 it is about 0.6 pt. In 16 pt 仿宋 on a 15.8 pt grid it is about
  1.6–1.8 pt. The amount is not a fixed share of the em. It probably depends on each glyph's
  side bearings; more measurements are needed.
- **On a character grid it still applies** (probe 01: `。` 14.0 pt, `、` 12.4 pt on a 15.8 pt
  pitch). Patch 0005 turns compression off there.
- **With `w:hint="eastAsia"` on the run, nothing was trimmed** (probe 05, the same text as
  probe 04). Not yet explained; it needs more probes.
- `doNotCompress` (probe 03): every mark keeps its full advance.

**Consequence for 0005 / #5006:** the JIS-style model (half an em between adjacent pairs, off on
a character grid) does not match WPS. Compression-on documents are 77% of real documents. This
is the main cause of our longer page counts.

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
space added to each character. Upstream's `round` cell rule and 0002's `ceil` both differ.

**Consequence for 0002 / #5003:** replace the cell rule with "advance = own size + delta".

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

## F5. Line ends (probes 06–07)

With compression on, `）` is trimmed (about 1.4 pt in 10.5 pt 宋体) on every line, including at
line ends. With `w:topLinePunct` the trim is about 1.0 pt. A justified line with no room to trim
kept full advances. To be measured more closely together with F1.

## Still to measure

- The trim amount per mark and font (F1): more sizes and fonts (仿宋, 楷体, 黑体, 微软雅黑),
  and whether it comes from glyph bounds.
- Why the hint disables trimming (F1).
- Word's behaviour for each finding. Upstream targets Word, so changes to #5003/#5006 need
  either Word references or an argument from the spec. F2 has one; F1 does not yet.
