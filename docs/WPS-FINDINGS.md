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
font's width (probe 32). Patch 0009 reads, writes and lays it out. A list label's suffix space
(`w:suff="space"`) is half an em too: `dc-policies-730d6a61a9`'s `11.` is followed by 8 pt before
16 pt text (patch 0017).

## F8. `*Chars` indents on a line grid: side indents count the default size

Probe 34, a lines-only grid (no character pitch) and a 16 pt paragraph mark: `leftChars=100` indents
10.44 pt, whatever `w:left` says (210 or 1000 twips); `leftChars=200` beside a 22 pt mark 21.0 pt;
`firstLineChars=200` 32.04 pt. So the left and right indents count the grid's default font size
(the Normal style's, 10.5 pt here) and the first line counts the mark's size. On a character grid
all three count the pitch (F3). Patch 0010. Found through `dc-correspondence-195b80f877`, whose
signature line only fits with F7's wider spaces when its `leftChars=100` is 10.5 pt, not 16.

Addendum (2026-11-03): the same holds for a paragraph with `snapToGrid=0` on a line grid
(`dc-policies-39de0888eb`: `leftChars=413` beside 13 pt text indents 49.55 pt = 4.13 × the
12 pt Normal size). Patch 0021.

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

Patch 0013. **Open (resolved by F12):** some bracket-heavy lines squeeze more than F10 allows: about 0.11 em off
each of seven sides in `dc-policies-2e6f27b909` (0.78 em in all; `〔2020〕22号` with autospace
beside the digits), 0.52 em in `dc-policies-af80f1c025`, and probes 01/02. Comma sweeps, eight
mixed closing marks and an opening bracket among commas all stay at exactly 0.5 em (a 0.005 em
sweep: 19.500 em fits, 19.495 doesn't). The variable that separates them isn't found yet.

## F12. The squeeze pool grows with the marks in the line's last unit

Resolves F11's open question and replaces F10's two additions. `corpus/tools/sweep_lines.py` sweeps
arbitrary first lines (0.02 em steps). The real line from `dc-policies-2e6f27b909` squeezes 1.2 em;
taking out its brackets drops that to ½ em, and so does replacing only its final `《` (the line
ends `…定《瑞`). Lines whose last unit (what can't be broken off the end: `《瑞`, `中。`, `“中`)
holds a compressible mark:

| Line | Squeeze |
|---|---|
| ideographs, last unit `《瑞` | 0.49 em (the `《`'s own half) |
| 1 comma, last unit `《瑞` | 0.99 em |
| 3, 5 or 8 commas, last unit `《瑞` / `瑞。` / `瑞”` | 1.49–1.50 em |
| 5 commas, last unit `《「瑞` | 2.50 em |
| the same commas with `《` mid-line, plain last unit | 0.50 em |

**Rule (mode 14 and older):** the line has a pool of ½ em plus 1 em per compressible mark in its
last unit. A closing mark's free blank half at the line end comes out of the pool; the marks
share the rest equally, each at most its blank half (a closing quote ending the line gives
nothing itself). It reproduces every sweep so far, F1's ten-comma line and line-end `”` case,
the closing-opening line (its last unit was `“中`) and the bracket-heavy corpus lines. On
justified mode-15 lines the same shape fits with 1 − 1/(2k) em plus ½ em per last-unit mark
(three commas and `。`: 1.33 em measured, 1.3). Patch 0014.

## F13. Automatic spaces beside digits are squeezed with the marks

WPS's automatic space between Asian text and digits is 0.25 em on a line that fits (a median of
0.25 em over the corpus refs, every size). On an overflowing line it gives some up:
`gov-mct-keji`'s `我部于2020年10月…关` keeps its last ideograph by narrowing four automatic spaces
from 4.0 to about 3.8 pt. Sweeps (`sweep_lines.py … autospace`, mode 14, left aligned, no grid):

| Line | Squeeze |
|---|---|
| one digit run (2 spaces) | 0.24 em |
| two runs (4 spaces) | 0.49 em |
| four runs (8) / two runs and 3 commas | 0.50 em |
| two runs, last unit `《瑞` | 0.99 em |
| digits ending the line (1 space before them) | 0.12 em |

**Rule (mode 14 and older):** each automatic space shares F12's pool with the marks, giving up at
most half itself (0.125 em); it adds nothing to the pool from the last unit. Patch 0015.
**Open:** justified mode-15 lines fit no rule yet: spaces alone squeeze 0.24, 0.27, 0.30 em for
2, 4, 8 spaces, and two runs with three commas only 0.60 em (three commas alone: 0.8). Mode 15 is
left as it was.

## F14. On a character grid the text width is whole cells

`gov-nmpa-1`, `-5`, `-6` (`linesAndChars` grids) fitted a character more per line than WPS. Cut to
one paragraph, `gov-nmpa-6`'s line changed in WPS only when its `w:docGrid` was removed (not with
`overflowPunct`, exact spacing, `w:compat` or the colour). WPS lays text in whole character
pitches: its 442.2 pt text width on a 16 pt pitch holds 27 pitches, so lines end at 432 pt
(508.5, not 518.8). A 10 pt right indent comes off that (lines end at 498.6) and `snapToGrid=0`
paragraphs are narrowed too, so it is the section's width that is rounded. The 公文 grid (28 ×
15.79 pt = 442.2) is unchanged. `gov-sh-sheitc` (pitch 16.3 pt, 27 pitches, 519.3) and
`dc-correspondence-6ce199cdd6` (529.2) match the same rule.

What's left past the last pitch isn't wasted: in `gov-nmpa-1` (29 pitches of 14.83 pt) a line
holds 29 ideographs and then a `，` hanging entirely in that strip, ending at 522.6. Off the grid
a closing mark ending a line frees only its blank half (sweeps of plain lines ending `。` `，` `”`:
0.49 em, with or without `overflowPunct`), so the strip is grid-specific. **Rule:** a single
column on a character grid is as wide as the whole pitches it holds (within 0.01 pitch); a closing
mark ending a line may reach into the rest. Patch 0018. Several columns aren't measured.

`gov-sh-sheitc`'s 26-character line, left open here at first, is F15.

Also found here, not a WPS rule: `dc-administrative-82eb70e86d` has a custom style with the id
`Normal` (named `[Normal]`, 12 pt) beside its default paragraph style `a` (Normal, 10.5 pt);
WordCraft took the custom style for unstyled paragraphs and the grid's default size. Patch 0016.

## F15. `balanceSingleByteDoubleByteWidth` counts character spacing per byte

`gov-sh-sheitc`'s Normal style has `w:spacing w:val="-6"` (−0.3 pt) on a 16.3 pt character grid.
Cut to one paragraph and left-aligned, without compression, WPS advances its ideographs 15.72 pt:
16 + 0.3 (grid) − **0.6**. Without `w:balanceSingleByteDoubleByteWidth` the same text advances
16.0 pt. Probe (16 pt 仿宋 and Times New Roman `H`, spacing 0, −0.3, −1, +1 pt, with and
without the switch, no grid and a +0.3 pt grid): with the switch an East Asian character gets its
spacing **twice** (+1 pt → 18.0, −0.3 → 15.4; on the grid 18.306, 15.706), a Latin letter once;
without it, once each. As if spacing were per byte, a GBK character being two. Patch 0019. With
it, the line fits 26 characters as in WPS.

## F16. The pool grows by what the last unit is wider than an em

`dc-policies-7380cfbea3`'s `绿色通道，4家…设置老年病床120` squeezes its two commas by 0.545 em
in all in WPS, more than F12's half em, and leaves its automatic spaces whole. Its last unit is the
digits `120`. Sweeps (`sweep_lines.py … enddigits`, `enddigits2`, mode 14, natural widths
counted from the text):

| Line | Squeeze |
|---|---|
| 2 commas, last unit `120` / `abc` / `，120` | 1.00 em |
| 3 or 5 commas, last unit `120` | 1.00 em |
| 2 commas, last unit `12345` | 1.12 em (the commas' 1.0 and a space's 0.125: all the room) |
| 2 commas, last unit `1`; or `120` mid-line and `瑞` ending the line | 0.50 em |
| 1 comma, last unit `120` | 0.62 em (the comma's half and a space's eighth) |
| the real line | 0.99 em |

Without automatic spaces the same, less the spaces' share. **Rule (replaces F12's count):** the
pool is half an em plus what the last unit is wider than an em (`《瑞`, `瑞。`: an em, as F12
had; `《「瑞`: two; `120`: half an em; `1` or `瑞`: nothing). It gives every F12 sweep again. In
mode 15 half that is assumed, as F12 measured for marks. Patch 0020.

## F17. East Asian fonts take 1.3 times their height; missing Chinese fonts fall back to 宋体

Probes (`corpus/tools/lh_probe_fonts.py`, `lh_probe_grid_mixed.py`, `lh_probe_traditional.py`; 10.5 and 16 pt, single and double spacing, no grid):
WPS lays a line of 宋体, 黑体, 楷体 or 仿宋 out **1.30 em** tall, Microsoft YaHei 1.714 em,
Microsoft JhengHei 1.726 em, DengXian 1.354 em. Each is **1.3 × usWinAscent + usWinDescent**
(1.000, 1.320, 1.330, 1.042 em), and the extra 0.3 is split evenly: the first baseline sits
win ascent + 0.15 × the height below the line top (宋体 16 pt: 16.13 pt; YaHei 20.09; DengXian
15.41). It follows the font, not the text: 宋体 holding only Latin letters is 1.3 em too, and
a line mixing Times New Roman and 宋体 takes 宋体's. Times New Roman and Calibri keep their usual
heights. The faces it applies to carry an East Asian OS/2 code page (bits 17–21: JIS, GB2312,
Wansung, Big5, Johab). On a line grid the 1.3 counts before snapping: 13 pt 宋体 takes two
15.6 pt pitches. Wenshu used the hhea ascent + descent + line gap, 1.141 em for 宋体, so every
off-grid Chinese line was 12% short. Patch 0022.

Fallbacks, from the same probes and `dc-educational-b8de3ae296`, `dc-forms-56af470ee7`:
Traditional Chinese document fonts this system lacks (標楷體, 新細明體, 細明體) are drawn as 宋体,
glyphs and line height; Chinese text in a font without it (Times New Roman, or an empty theme
East Asian font with an en-US language, `02_mixed_cjk`) is drawn in 宋体 too, where Wenshu took
Microsoft YaHei. Wenshu's substitute table maps the Traditional names to the Song list and the
system fallbacks try SimSun before YaHei (patch 0023).

Refinements (long-paragraph probes, `corpus/tools/lh_probe_fonts.py` style, averaged over 30
lines): the height is **1.3 × the win height in whole font units, rounded down**: SimSun 332 of
256 units (1.2969 em, not 1.3), YaHei 3513 of 2048 (1.7153 em); over `dc-policies-de24610147`'s 86
pages the 0.24% is two pages (patch 0022). The same rule holds below single spacing (`line=150`
to `276` auto). Chinese characters a run's font lacks take SimSun even when another CJK face
(YaHei) was loaded first; Wenshu's fallback preferred loaded faces (patch 0028).
DFKai-SB not installed: WPS keeps its own box, 1.30 em with the baseline 0.95 em down (a 0.8 /
0.2 em font, 1.3 ×), while drawing 標楷體 in 宋体 and **DFKai-SB, by that name, in Microsoft
JhengHei Light** (patch 0030; Wenshu's font table maps the English name to JhengHei Light).

Embedded fonts (`dc-legal-ce628d9760`): WPS embeds whole SimSun files as 黑体, 等线 and
仿宋_GB2312. A document's embedded fonts are now its own (patch 0027): other documents neither
find them nor fall back to them, and a copy that keeps SimSun's names no longer stops the
installed SimSun from loading.

## F18. Grids in table cells, at the page foot, and with multiple spacing

Probes (`corpus/tools/lh_probe_tables.py`: 13 pt 宋体 on a 15.6 pt line grid, modes 14 and 15, with and without
`adjustLineHeightInTable`): lines in a table cell snap to the line grid **only under
`w:adjustLineHeightInTable`** (two pitches with it; 16.9 pt, the font's 1.3 em, without). The
setting sits in `w:compat`; a first probe put it under `w:settings`, where WPS ignores it, and
read as "never snaps" (corrected after `dc-administrative-bd6b721e4e`, whose whole text is in one
table and does snap). Fixed table layout, `textAlignment`, at-least row heights and cell margins
change nothing. Patch 0024, which also reads and writes the setting.

At the page foot (`gov-mct-keji`): a 16 pt line on a 15.6 pt grid takes two pitches with its
text centred, and stays on the page while its text ends 1 pt above the bottom margin and its
pitches end 4.2 pt below it, as multiple spacing's leading may run into the margin. Patch 0025.

Multiple spacing on a line grid (16, 22 and 32 pt 宋体 and YaHei at 1, 1.5 and 2.5): a line takes
**its whole pitches or m pitches, whichever is more**: 16 pt (two pitches) stays at 31.2 pt at 1.5
and takes 2.5 pitches (39.0 pt) at 2.5; 32 pt (three pitches) stays at 46.8 pt at any of them.
Wenshu multiplied the pitches by m (2.5 × 3 pitches for a red-header title,
`dc-administrative-f526989771` a page long). The two agree for text within one pitch. Below
single spacing it is unmeasured on a grid. Patch 0029.

## F19. A file without a styles part

Probe (no `styles.xml`, no run formatting): WPS lays the text out single-spaced with no space
after paragraphs, in 10.5 pt Calibri (Latin) and 宋体 (Chinese). Wenshu took a new document's
styles (12 pt, 1.15 spacing, 8 pt after). Patch 0026.

## F20. Hanging punctuation, punctuation kerning, the middle dot

Found through the seven documents 0021–0025 had left a page or two off; each rule probed in WPS
(16 pt 仿宋 / 12 pt 宋体 lines swept across widths, documents cut down to one paragraph and
bisected by settings, styles and properties).

- **Hanging:** under `w:overflowPunct` (on unless turned off), a closing mark ending a line of
  text whose East Asian language is Chinese, Japanese or Korean hangs **whole** past the margin,
  compressed or not (34 characters and a `，` fit 34 em; ，。、）」”：！ alike). The language may
  come from the run, the paragraph or the document defaults; text tagged en-US (or untagged with
  no default) keeps F1's half. `w:useAltKinsokuLineBreakRules` (in `w:compat`) turns hanging off.
  The hang adds to what the line's other marks squeeze (`…协议、协调意见、`). F1's probes had no
  language, which is why they never showed it. Cause of `dc-administrative-f9e6e0e0e6` and
  `dc-policies-b6747d9df3` lines one character short.
- **Punctuation kerning:** kerned text (`w:kern` at or below the size) closes up two adjacent
  full-width marks by **half an em**, compressed or not, unless the document says
  `w:noPunctuationKerning`: the first mark's blank right half (`》《` `）。` `，《` `。。`), else the
  second's left half after an opening mark (`（《` `（（`). Curly quotes take no part (`）“`
  `”《`). Compression may squeeze the pair further on a full line. F1's "pairs keep their full
  advances" holds for unkerned text. (`b6747d9df3`'s Normal style kerns.)
- **Middle dot:** U+00B7 between East Asian characters binds both neighbours (`艾布·达伍德`:
  WPS moves `布·达` to the next line whole). With F16 that also widens the squeeze pool of a line
  ending in such a name (`dc-educational-be4a26d95e`).

Patch 0031.

## F21. Spaces under balance, picture lines, leading into a tall footer

- Under `balanceSingleByteDoubleByteWidth` a space (half an em, F7) takes `w:spacing` **twice**,
  like an East Asian character (F15): at −1 pt a 14 pt space is 7 − 2 = 5 pt. A 版记 line with 21
  spaces lost its last character to a new page (`b6747d9df3`).
- A line of only pictures is the picture's height: no descent below it, whatever the paragraph
  mark's font and size (a 60 pt picture alone takes 60 pt, where Word adds the font's descent).
  `f9e6e0e0e6`'s title table had two such lines and ran 7 pt taller.
- Multiple spacing's leading below the text may run into a **footer taller than the bottom
  margin**, not only into the margin; only the text must clear the footer (probes with one- and
  four-line footers, single and double spacing). Word keeps the whole line clear.

Patch 0032.

## F22. Spacing in lines; reader gaps the .docx round-trip exposed

- `w:beforeLines` / `w:afterLines` (hundredths of a line) count lines of the section's line grid
  (`afterLines="100"` is 16.45 pt on a 329-twip grid, 31.2 pt on 624) or 12 pt without one, and
  override `w:before` / `w:after` (`after="400" afterLines="100"` is still 16.45 pt). A count of 0
  is ignored (the style's spacing stays). They were neither read nor saved (patch 0036).
- A `w:br` inside `w:t` (`<w:t>苏州-合肥<w:br/></w:t>`, against the schema, from a travel
  itinerary generator) breaks the line in WPS (patch 0034). Three corpus documents gained their
  missing page.
- A VML text box carrying an empty `<v:imagedata o:title=""/>` (WPS writes one) is a text box; it
  was dropped with its text, in groups too (patch 0035; `dc-policies-de24610147` had 220).

## F23. Date fields: when WPS refreshes them, the W picture; PDF export (J4)

Measured 2026-11-03 with probe documents (`DATE` and `CREATEDATE` with the stored result `X`) and
WPS updating fields through COM (`Fields.Update`).

- WPS refreshes `DATE` and `TIME` to the moment it renders: a corpus footer
  `TIME \@ "yyyy年M月d日星期W"` stored as 2018年6月27日星期三 shows the render date, 2026年10月31日星期六.
  `CREATEDATE` keeps its stored result on open, and an explicit update gives
  `错误！未定义书签。` (WPS doesn't support it).
- `W`, either case, is the Chinese weekday: `W` 二, `WW` 周二, `WWW` (or `Www`) 星期二 on a
  Tuesday. Sunday could only be measured by changing the clock; Wenshu writes 日 (patch 0038).
  Without a language tag WPS also gives `dddd` 星期二 and `ddd` 二, where Wenshu writes English
  names (open).
- In the result, digits take the Latin font and 年月日 the East Asian one, with the automatic
  space between them (2026 年 10 月 31 日). Wenshu drew the whole result in the first font's face,
  with the East Asian font's glyph ids: wrong glyphs on screen, and in a PDF that face could not
  be subset and was drawn as outlines without text (15% of one document's text unextractable).
  Fixed in layout, patch 0037.
- WPS's PDF export is the final text: tracked changes are applied and not marked, like Wenshu's
  export (and unlike its screen view, which shows the markup; nine corpus documents have tracked
  changes, all with matching page counts).

## F24. An invisible WordArt watermark (round-trip)

A WPS-generated government notice carries VML WordArt in its header with `fillcolor="gray"` and
`<v:fill opacity="0"/>`: a tracing watermark (names and phone numbers, between 爩灪 and 犪
markers) that WPS doesn't show. Wenshu read the letters in the fill colour and ignored the
opacity, so it drew them, and wrote them back opaque: WPS then showed them across the page. The
opacity now becomes a transparent text fill (`w14:textFill` with `w14:alpha`), and WPS's render of
the saved file matches the original again (page score 0.99995, was 0.984). Patch 0039.

## F25. A heading that opens with a page break; bookmarks between paragraphs

Measured 2026-11-03 with probes (16 pt text, exact 28 pt lines and the corpus document's own
grid and margins; filler lines up to the page bottom, then a heading `<w:br w:type="page"/>附件1`):

- WPS places a paragraph with a page break inside as if `w:keepNext` and `w:keepLines` were not
  set: with either or both, the heading's text starts the next page and nothing more. Wenshu
  moved the whole paragraph when the page was nearly full (keep lines: its two lines didn't fit;
  keep with next: with the next paragraph's first line), then broke again, so an empty page came
  before the heading.
- A line holding only the page break stays at the bottom of a full page in WPS (and Word). In the
  corpus document a `w:bookmarkEnd` between two paragraphs is carried onto the heading's start,
  so that line also held a bookmark marker, and Wenshu then made it need room: it went over and
  left an empty page. Markers (bookmarks, comment ranges) now don't count.

`gov-nmpa-1`, `-5`, `-6` (attachments 附件1–4, each a heading-3 opening with a page break) and
`dc-technical-c2c92639c7` now match their page counts. Patch 0040.

## F26. Section ends: kept closing paragraphs, a mark after a page break

Measured 2026-11-03 with probes (filler lines to the page bottom, then the paragraph closing a
section, next-page and next-column breaks; and the start of a corpus document cut down step by
step):

- The last paragraph of a section before one that starts a new page is placed as if `w:keepNext`
  were unset, and an empty one as if `w:keepLines` were unset too: kept or not, WPS lays them out
  the same. Wenshu moved a kept closing paragraph (a heading style on the section's empty last
  paragraph) to a new page, and the section break then began another (patch 0041).
- "Cover, a paragraph of only a page break, an empty paragraph carrying a next-page section
  break, the next section": WPS starts the next section on the page the break began, in any file
  with `w:compat` (whatever its `compatibilityMode`, 11 to 15); without a `w:compat` element at
  all, WPS leaves that page blank. Wenshu always left it blank. The mark now takes no line there
  (its line sits beside the break's mark on the page before), and the section takes the page
  back (patch 0042). Files without `w:compat` aren't told apart: Wenshu's settings default to the
  current mode.

`dc-reports-aa35f17a03`, `dc-administrative-ef14cd2f88` and `dc-reports-1aa06d7905` (two empty
pages) now match their page counts.

## F27. Missing Traditional Chinese (華康) fonts

WPS substitutes a missing font by a table of names, not by design class (measured name by name,
24 names): 華康細圓體, 華康中圓體, 華康粗圓體, 華康中黑體, 華康粗黑體, 華康標楷體, 華康細明體,
華康中明體 and 華康隸書體 are drawn in Microsoft JhengHei; 華康儷中黑, 華康黑體 and 文鼎粗黑 in
SimHei; everything else (華康圓體, 華康楷書體W5, 華康新特明體, 文鼎中楷, the display faces) in 宋体.
Wenshu drew them all in 宋体; on a line grid Microsoft JhengHei's taller line takes two pitches,
so a church bulletin set in 華康細圓體 came out half as long as in WPS. Wenshu's font table
(`wenshu-cjk`) now follows WPS. `dc-administrative-f899568bb2` matches; `a9f763a2b4` is a page
closer.

## F28. A paragraph of only a page break at the page bottom; turned cells with distributed text

- A paragraph holding nothing but a page break (`<w:p><w:r><w:br w:type="page"/></w:r></w:p>`)
  whose line doesn't fit at the bottom of a page goes over in WPS, and its break then leaves the
  next page blank (measured with exact line heights 28 to 80 pt, with and without space after).
  When the paragraph goes on after the break (a heading opening with one, F25), its break line
  stays at the bottom. Word keeps both, as upstream's comment says; Wenshu now does as WPS does
  (patch 0043). No corpus document gained from it (the one where WPS shows such a blank page,
  `9c189a35a9`, differs earlier), and `29a7772dfb`, already off, moved a page further.
- A turned cell (`w:textDirection tbRlV`) with distributed text (`jc="distribute"`, a rating
  scale's 非常不符合 header) in a row of at-least height: WPS spreads the characters over the row.
  Wenshu measured the text as laid out on a long probe line, where distributing spreads it across
  the whole probe, and gave the row the longest turned length allowed (a page). Its own length,
  and at least the row's height, now (patch 0044). `dc-policies-efacf8571b` matches.

## F29. Kinsoku off: a line may end with an opening bracket and its space; probes need a settings part

- With `w:kinsoku w:val="0"`, WPS breaks `根据《 山东` after `《 ` (the bracket ends the line, the
  space hangs); with kinsoku on it doesn't, and neither does UAX #14 (LB14: no break after an
  opening bracket, spaces or not). So in WPS that rule is part of kinsoku. Wenshu now adds the
  break, with kinsoku off, between two East Asian characters with spaces between them (patch
  0045). `dc-policies-ea10dc7972` (a table cell `…查实违法违规金额       元。根据《 山东省…`, Normal
  style with kinsoku off) matches.
- **Method:** WPS lays out a file with no settings part (`word/settings.xml` missing) differently
  from any file that has one, even an empty one. Without it a space beside Chinese text is set in
  the East Asian font (half an em), and a page break followed by an empty next-page section mark
  leaves a blank page (F26). Real documents always carry the part, so probes must too: a space
  measured without it looked like a WPS rule; with it WPS keeps the Latin space, as Wenshu does
  (the change was made and reverted). The probes behind F25, F26 and F28 were checked against the
  corpus documents they fixed.

## F30. CJK compatibility ideographs: the fallback face and the line height

Measured 2026-11-03 (probes with a settings part). A run in 宋体, 仿宋 or 新細明體 holding a CJK
compatibility ideograph its font lacks (U+F900–FAFF; a 理 written U+F9E4 in a corpus document) is
drawn by WPS in Microsoft JhengHei UI (Microsoft JhengHei for a Microsoft YaHei run), as Windows
font linking gives it; Wenshu took whichever loaded face covered it, Yu Gothic UI (patch 0046).
That face adds no height to the line: a single-spaced line holding it is exactly as tall as the
others (16.8 pt at 13 pt), where Wenshu added the face's own East Asian line height, 1.3 × its
1.27 em (patch 0047). 宋体 reached as a fallback for a run with no East Asian font still counts in
full, as before. `dc-educational-92262161df` matches; `dc-reports-2d6a9da333`, whose line breaks
differ from WPS's elsewhere, lost the page count it had matched by those taller lines.

## F31. A missing Arial Unicode MS keeps its own line box

Measured 2026-11-04 (settings part included): text in Arial Unicode MS (not installed) is laid
out by WPS on a 1.74 em line at 10, 12 and 16 pt, Latin, Chinese or mixed: the font's own win
ascent and descent (2189 and 555 of 2048) times the East Asian 1.3 (its code pages are East
Asian). The glyphs are drawn in substitutes (宋体 for Chinese, a font of WPS's own for Latin).
Wenshu used its substitutes' boxes, about 1.3 em. Patch 0048, beside DFKai-SB (F19).
`dc-creative-377b4bf861` matches; `f7d3afdfdb` is closer (its Latin text in WPS's own font keeps
its line breaks apart).

## F32. Missing Founder (方正) faces: WPS's line box; 新細明體's Latin text in Segoe Print

Measured 2026-11-04 at 16 pt (baseline below the top margin, line pitch; settings part included).
Without the Founder fonts installed, WPS draws 方正小标宋简体 in 方正舒体 (FZSTK) and 方正仿宋_GBK /
方正大标宋简体 in Microsoft YaHei or faces of its own, on boxes of its own: 方正小标宋简体 and
方正小标宋_GBK 1.038 + 0.484 em (24.36 pt), 方正仿宋_GBK 1.031 + 0.416 (23.16), 方正大标宋简体
1.046 + 0.439 (23.76). Wenshu used its substitutes' boxes (华文中宋, 23.65; 仿宋, 20.76). The boxes
are now WPS's, also for text in the run's East Asian font (a 公文 title names 方正小标宋简体 only
there), and only when the font is missing (patch 0049). No page count changed; ten documents
score higher, `01_gongwen` 0.938 → 0.970, mean 0.9394 → 0.9401. FZSTK appears in 63 corpus
references.

Not done: in a run whose Latin font is 新細明體 or 細明體 (missing), WPS draws Latin text in Segoe
Print, and its tall line box (28.2 pt at 16 pt, baseline 20.3) sets the line; Chinese-only lines
are unaffected. Three references show it (`173ff7b3a5`, `ffe3dc89e6`, `aae80ea709`).

## F33. At least 0 is no line rule: off the grid

Measured 2026-11-04 (settings part included; 14 pt and 10.5 pt, 標楷體 and 新細明體, on an 18 pt
line grid). A paragraph with `<w:spacing w:line="0" w:lineRule="atLeast"/>` lays its lines in WPS
at their natural height (18.24 pt at 14 pt, 13.68 at 10.5), off the grid, as if it had no rule;
any other minimum, 0.05 pt included, snaps to whole pitches (36 pt for those same lines). Wenshu
snapped at least 0 as well (patch 0050). Page counts 177 → 179 / 200, mean 0.9401 → 0.9408: gains
`493a360281`, `173ff7b3a5`, `ffe3dc89e6`; `a18e0d6d84` lost a lucky match. Its page 3 now has
WPS's 18 pt pitch, but WPS wraps the text in its narrow table cells at 6 characters a line where
Wenshu fits 10 to 12 (to trace).

## F34. A hanging indent in characters hangs from `leftChars`

Measured 2026-11-04 (probes built on `dc-legal-a18e0d6d84`'s package, 12 pt 標楷體, in the body
and in a table cell, on a line grid, a character grid and no grid; compatibility settings varied
without effect). When a paragraph has `w:leftChars` or `w:hangingChars` and a hanging indent, WPS
starts the first line at the character left indent (at 0 without `leftChars`) and the other lines
a hanging further in; `w:left` is not used. `leftChars=250 hangingChars=274` at 12 pt: first line
30 pt, the rest 62.9 pt (Wenshu had −2.9 and 30); `leftChars=250` with `w:hanging="100"`: 30 and
35; `hangingChars=274` with `w:left="1258"`: 0 and 32.9. Word writes `w:left` as that same sum
(1258 twips = 30 + 32.9 pt), so Word reads it the same way. Twips alone keep the usual rule (first
line at left − hanging). Patch 0051. Page counts 179 → 180 / 200 (`a18e0d6d84`, whose table cells
F33 left wrapping wide), mean 0.9408 → 0.9419; eight more score higher, no losses.

## F35. Text after a leading page or column break is still the first line

Measured 2026-11-04 (two-column probes, 9 pt 宋体). A paragraph that opens with a column break (or
a page break, or several) starts its text in the next column or page with the first-line or
hanging indent (`w:hanging="291"`: x 326.8, not the other lines' 341.4; `w:firstLine="600"`:
345.6); the space before is not repeated there. After a leading line break, or a column break
after text, the text takes the other lines' indent. Wenshu now does the same (patch 0052); the
probes match. No corpus change: found in `dc-forms-7abe5be4e9`, a form converted from PDF whose
page count differs for other reasons (a header holding an OLE object's EMF preview that pushes the
body down in WPS; a 45-shape drawing group that Wenshu doesn't draw).

## F36. Space before after a leading page or column break, by compatibility mode

Measured 2026-11-04 (one- and two-column probes, 9 pt 宋体, `w:before="400"`, compatibility modes
12, 14 and 15). A paragraph that opens with a page or column break gets its space before at the top
of the new page or column in modes up to 14 (text at y 101.09, not 81.05), and not in mode 15.
Word keeps it in mode 15 too (WordCraft's test, measured in Word, expected it), so this departs
from Word: Wenshu follows WPS (patch 0053, not for upstream as it stands). Before, Wenshu kept it
after a page break in every mode and never after a column break. The probes match in all three
modes. Page counts unchanged at 180 / 200; `ffe3dc89e6` scores higher (0.940 → 0.946), no losses.

## F37. An empty paragraph after a nested table takes no room

Measured 2026-11-04 (a borderless outer table whose cell holds a 3×3 nested table, then a body
paragraph; with and without the line grid and `adjustLineHeightInTable`, floating and not). When a
cell ends with a nested table followed by one empty paragraph, WPS ends the cell at the nested
table's bottom: the paragraph takes no room. A paragraph with text there takes its line, and with
two empty paragraphs after the table both count. Wenshu gave the empty one its line (two grid
pitches for a 16 pt one). Patch 0054; the probes match. `dc-forms-40991f97da` (a landscape form
whose title and floating table sit in such a cell) now keeps its note on the page: 4 pages, as WPS.

## F38. A wrap distance of 4294967295 is none

`dc-educational-aa1a5e719d`'s first-page footer holds a rule (a line shape with top-and-bottom
wrap) whose `distT`/`distB` are `4294967295`, the unsigned form of −1 (VML's
`mso-wrap-distance-top:-3e-5mm`). Wenshu read 4.3 × 10⁹ EMU (clamped to 1584 pt), so the rule's
wrap took the whole page and pushed the body after the DOI line to page 2; WPS lays the page as if
there were no distance. Values past `i32::MAX` now read as no distance (patch 0055). Only this
document in the corpus has one. With F37: page counts 180 → 182 / 200, mean 0.9419 → 0.9420; no
losses (`a598d05be9` 0.919 → 0.915, page count unchanged).

## F39. A first-line indent in characters counts the first run, spacing twice

Measured 2026-11-04 (A4 landscape, 宋体 22 pt paragraph mark, 仿宋 text; with a line grid and
without one). Off a character grid, WPS measures `w:firstLineChars` in the first run with text:
its size plus twice its character spacing, whatever the paragraph mark's size. 15 pt text beside a
22 pt mark: `firstLineChars=1000` is 150 pt; spaced 0.4 pt, 158; spaced 2 pt, 190; spaced −1 pt,
130; 22 pt text beside a 16 pt mark, 220. No clamping: 37 characters (584.6 pt) put a six-character
line near the right margin (`dc-administrative-1f522d22f6`'s "金额单位：元"; Wenshu's 37 × 22 pt
mark sent it off the page and the table under it a line down). Word measures these in the mark's
size (WordCraft's #534 test), so this departs from Word. A bookmark or comment range before the
text doesn't count as the first run. Patch 0056; probes match. Page counts 182 → 183 / 200,
mean 0.9420 → 0.9423; no losses.

## Still to measure

- Whether every right-compressible mark gets the free line-end half, not only `，` (F1).
- How the squeeze interacts with justification (`jc=both`) and kinsoku.
- `w:topLinePunct` (F5).
- `*Chars` indents with no document grid at all (F8 covers line grids).
- Whether an opening mark may overhang the margin with kinsoku off (`dc-policies-ea10dc7972`'s
  `根据《` line ends 4 pt past where the line above stopped), and the curly quote “ breaking
  after it with kinsoku off.
- The automatic space beside Latin text and digits under `w:spacing` (WPS: about a quarter of
  the size plus twice the spacing, 3.0 pt at −1 pt and 4.0 at +1 for 14 pt; Wenshu keeps 3.5).
- Justified Chinese lines: WPS stretches every character; Wenshu leaves them unstretched (no
  effect on breaks).
- F12 on justified mode-15 lines beyond the line-end cases (the closing-opening line measured
  1.8 em there), and marks of mixed sizes on one line.
- Where at-least spacing puts the baseline when the minimum is larger than the text (F6 covers
  exact only).
- `dddd` / `ddd` day names by the field's language (F23), and Sunday's `W`.
- Word's behaviour for each finding. Upstream targets Word, so changes to #5003/#5006 need
  either Word references or an argument from the spec. F2 has one; F1 does not yet.
