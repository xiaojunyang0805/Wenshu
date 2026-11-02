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

> **Superseded in part by F10 (2026-11-02):** the limit is per line, not "0.1 em per mark", and
> it depends on the compatibility mode. The sides table and the line-end half still stand.

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

## F6. Exact line spacing puts the baseline at 0.8 of the line

Probe 31 (`gen-probe-31-exact-baseline`): one-line paragraphs with exact spacing of 12, 20, 30 and
40 pt in 宋体 10.5, 仿宋_GB2312 16, 黑体 22, Times New Roman 12, 方正小标宋简体 22 and 宋体 36 pt,
plus a 30 pt line mixing 16 and 22 pt. In every case WPS puts the baseline 0.80 × the line height
below the line's top (±0.1 pt). The font's ascent and descent don't move it; 36 pt text in a 12 pt
line also sits at 9.6 pt and overflows upwards. Wenshu used the line height minus the font's
descent, 1–6.5 pt lower. That was the 3.7 pt offset in `dc-correspondence-195b80f877` (16 pt 仿宋
in 30 pt lines). Patch 0008 makes it 0.8; the letter's baselines now match within 0.07 pt.

## F7. `balanceSingleByteDoubleByteWidth` makes a space half an em

123 of the 200 corpus documents carry `w:balanceSingleByteDoubleByteWidth` in `w:compat`.
`dc-correspondence-31310a47c6` showed it: its header's run of spaces before 招 租 公 告 spans 280 pt
in WPS (half an em of 26 pt each), 126 pt in Wenshu (Calibri's spaces). Variants of that document
(hint off, explicit fonts, no `w:lang`, left-aligned, no header style, no indent) changed nothing;
dropping its `w:compat` did. Probe 33 sets only this switch: in WPS a U+0020 space is then
**0.5 em of its size**, whatever the font (Calibri, Times New Roman) or hint; letters, digits and
punctuation (`iiiii 11111 WWWWW ,.;()`) keep their widths. Without the switch spaces keep their
font's width (probe 32). Patch 0009 reads, writes and lays it out.

## F8. `*Chars` indents on a line grid: side indents count the default size

Probe 34, a lines-only grid (no character pitch) and a 16 pt paragraph mark: `leftChars=100` indents
10.44 pt, whatever `w:left` says (210 or 1000 twips); `leftChars=200` beside a 22 pt mark 21.0 pt;
`firstLineChars=200` 32.04 pt. So the left and right indents count the grid's default font size
(the Normal style's, 10.5 pt here) and the first line counts the mark's size. On a character grid
all three count the pitch (F3). Patch 0010. Found through `dc-correspondence-195b80f877`, whose
signature line only fits with F7's wider spaces when its `leftChars=100` is 10.5 pt, not 16.

## F9. On a character grid, `w:w` and `w:spacing` still count

Probe 35, the 公文 grid (pitch 15.79 pt for 16 pt, adjustment −0.21 pt): 16 pt 宋体 at `w:w` 50%
advances 7.80 pt, at 90% 14.16, at 200% 31.80; with 2 pt `w:spacing` 17.78; 54 pt at 90% 48.31;
22 pt at 90% with 2 pt spacing 21.55. So an East Asian character advances its **scaled size plus
the grid adjustment plus its spacing**. 0002 used the unscaled size and dropped the spacing, so
red-header titles (瑞安市教育局文件: 54 pt at 90%, `jc=distribute` in a 411 pt cell) took 53.8 pt
a character and lost their last one to a second line. Patch 0011.

## F10. The squeeze limit is per line, and depends on the compatibility mode

Width sweeps (`trim_probes.py sweep-k`, 20-unit lines narrowed from 20 em in 0.05 em steps, 16 pt
仿宋, rendered in WPS) with 1–8 commas, mixed marks, marks one ideograph apart, closing pairs,
closing-opening pairs, with and without a closing mark at the line end, left-aligned and justified,
no grid, a line grid and the 公文 character grid, in compatibility modes 14 and 15:

- **Mode 14 and older, any alignment, any grid:** a line may squeeze **½ em in all**, however many
  marks (1, 2, 3, 5 or 8 commas, eight different marks, or three closing pairs: always 19.5 em).
  The marks share it equally (one comma gives up 0.5 em, two 0.25, five 0.1).
- **A closing mark at the line end** (its blank half free, as in F1) adds ½ em: three or five
  commas then fit 18.5 em. The end mark itself isn't squeezed.
- **Closing marks meeting opening marks** (`：“` `”（` `、《` `》“`): every two such places add ½ em
  (four of them, line ending in an ideograph: 18.5 em, 1.5 em of squeeze; one: ½ em).
- **Mode 15:** nothing is squeezed on a left-aligned line and a line-end mark gets no free half.
  On justified lines the base is **1 − 1/(2k) em** for k marks (0.5, 0.75, 0.8, 0.85, 0.85, 0.9,
  0.9 em measured for k = 1, 2, 3, 4, 5, 6, 8), with the same additions.

F1's "0.1 em per mark" came from ten commas on a line ending in `，`: ½ em free plus ½ + ½ em of
squeeze, spread over ten marks. `gen-gongwen-04-han` (no `w:compat`, so legacy mode) showed it:
WPS squeezes a lone `，` by half an em to keep `来。` on the line. Patch 0012 implements the rule
(`SqueezeRule`, `ParaLayout::squeeze_room`). The documents without `w:compat` (16 of 200) are
legacy mode, as WordCraft already reads them.

## F11. Character-grid indents count the text; quotes break; the last mark keeps its blank

- **`*Chars` indents on a character grid** (probe 36: Normal 10.5 pt, `charSpace=-1839`, pitch
  10.05 pt): `firstLineChars=200` is 2 × 15.54 pt beside 16 pt text whatever the mark's size
  (20.04 pt for 10.5 pt text, 43.08 for 22 pt), so it counts the first run's size plus the grid
  adjustment; `leftChars=200` gave 31.08 pt for text/mark sizes 16/16, 16/10.5 and 10.5/16, the
  larger of the two. F3's probe had a 16 pt Normal style, where pitch and text agree.
- **Curly quotes:** WPS breaks `确保|“双减”` before the opening quote. That is Unicode 15.1 LB19a;
  WordCraft's `unicode-linebreak` 0.1.5 implements 15.0, which never breaks beside a curly quote,
  so such lines lost a character (0004 had assumed the breaks existed).
- **The mark ending a line isn't squeezed**, whether its blank half is free (`。`) or not (`”`).

Patch 0013. **Open:** some bracket-heavy lines squeeze more than F10 allows: about 0.11 em off
each of seven sides in `dc-policies-2e6f27b909` (0.78 em in all; `〔2020〕22号` with autospace
beside the digits), 0.52 em in `dc-policies-af80f1c025`, and probes 01/02. Comma sweeps, eight
mixed closing marks and an opening bracket among commas all stay at exactly 0.5 em (a 0.005 em
sweep: 19.500 em fits, 19.495 doesn't). The variable that separates them isn't found yet.

## Still to measure

- Whether every right-compressible mark gets the free line-end half, not only `，` (F1).
- How the squeeze interacts with justification (`jc=both`) and kinsoku.
- `w:topLinePunct` (F5).
- `*Chars` indents with no document grid at all (F8 covers line grids).
- F10's additions on justified mode-15 lines (measured roughly: 1.33 em for three commas and an
  end mark, 1.8 em for four closing-opening meetings), and marks of mixed sizes on one line.
- Where at-least spacing puts the baseline when the minimum is larger than the text (F6 covers
  exact only).
- Word's behaviour for each finding. Upstream targets Word, so changes to #5003/#5006 need
  either Word references or an argument from the spec. F2 has one; F1 does not yet.
