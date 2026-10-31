"""Measure how WPS (or Word) compresses East Asian punctuation: probes, then analysis of the PDFs.

    python corpus/tools/trim_probes.py make out/trim            # write the probe .docx files
    python corpus/tools/wps_render.py --files out/trim --out out/trim --no-png
    python corpus/tools/trim_probes.py measure out/trim > trim.tsv
    python corpus/tools/trim_probes.py sweep out/sweep              # width sweep (then render, then:)
    python corpus/tools/trim_probes.py sweep-read out/sweep

Every mark sits between two ideographs ("中。中"). Each probe exists twice, with
`compressPunctuation` and with `doNotCompress`, and nothing else differs. The trim on a mark's right
is the drop in its own advance; the trim on its left is the drop in the advance of the ideograph
before it. No document grid and left alignment, so neither stretches anything.
"""
import sys
from pathlib import Path
from xml.sax.saxutils import escape

FONTS = {"song": "宋体", "fang": "仿宋", "kai": "楷体", "hei": "黑体", "yahei": "微软雅黑", "dengxian": "等线"}
SIZES = [21, 24, 32, 44]  # half-points: 10.5, 12, 16, 22 pt
MARKS = "。，、：；？！）》」』】〕”’（《「『【〔“‘·…—"
# Font setups: (name, ascii/hAnsi font or None for the East Asian font itself, hint)
SETUPS = [("ea", None, False), ("ea+hint", None, True), ("tnr", "Times New Roman", False), ("tnr+hint", "Times New Roman", True)]
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"


def text():
    return "中" + "".join(f"{m}中" for m in MARKS)


def paragraphs(font):
    for setup, latin, hint in SETUPS:
        for sz in SIZES:
            yield setup, sz, latin or font, hint


def make(out):
    from docx import Document
    from lxml import etree

    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    for key, font in FONTS.items():
        for mode in ("compressPunctuation", "doNotCompress"):
            d = Document()
            body = d.element.body
            for child in list(body):
                body.remove(child)
            ps = []
            for setup, sz, latin, hint in paragraphs(font):
                h = ' w:hint="eastAsia"' if hint else ""
                rpr = f'<w:rFonts w:ascii="{latin}" w:hAnsi="{latin}" w:eastAsia="{font}"{h}/><w:sz w:val="{sz}"/><w:szCs w:val="{sz}"/>'
                ps.append(f'<w:p><w:pPr><w:jc w:val="left"/></w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t>{escape(text())}</w:t></w:r></w:p>')
            sect = '<w:sectPr><w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/><w:pgMar w:top="720" w:right="720" w:bottom="720" w:left="720" w:header="0" w:footer="0" w:gutter="0"/></w:sectPr>'
            for child in etree.fromstring(f'<w:body xmlns:w="{W}">{"".join(ps)}{sect}</w:body>'):
                body.append(child)
            st = d.settings.element
            for e in st.findall(f"{{{W}}}characterSpacingControl"):
                st.remove(e)
            st.append(etree.fromstring(f'<w:characterSpacingControl xmlns:w="{W}" w:val="{mode}"/>'))
            d.save(out / f"trim-{key}-{'on' if mode.startswith('compress') else 'off'}.docx")
    print(f"{len(FONTS) * 2} probes in {out}")


def chars(pdf):
    import fitz

    out = []
    with fitz.open(pdf) as d:
        for page in d:
            for b in page.get_text("rawdict")["blocks"]:
                for l in b.get("lines", []):
                    for s in l["spans"]:
                        for c in s["chars"]:
                            if not c["c"].isspace():
                                out.append((c["c"], c["origin"][0], round(c["origin"][1], 1), s["font"]))
    return out


def advances(pdf, font):
    """{(setup, size, mark): (advance of the ideograph before, advance of the mark, mark's font)}."""
    cs = chars(pdf)
    t = text()
    res, i = {}, 0
    for setup, sz, _, _ in paragraphs(font):
        seq = cs[i:i + len(t)]
        i += len(t)
        if "".join(c[0] for c in seq) != t:
            raise SystemExit(f"{pdf}: text out of step at {setup} {sz}")
        for k in range(1, len(t) - 1, 2):
            prev, mark, nxt = seq[k - 1], seq[k], seq[k + 1]
            if prev[2] == mark[2] == nxt[2]:  # all three on one line
                res[(setup, sz, mark[0])] = (mark[1] - prev[1], nxt[1] - mark[1], mark[3])
    return res


def measure(d):
    d = Path(d) / "pdf"
    print("font\tsetup\tsize_pt\tmark\tmark_font\tadv_off\tadv_on\ttrim_right\ttrim_left\ttrim_right_em\ttrim_left_em")
    for key, font in FONTS.items():
        on, off = advances(d / f"trim-{key}-on.pdf", font), advances(d / f"trim-{key}-off.pdf", font)
        for k in sorted(on.keys() & off.keys(), key=lambda k: (SETUPS.index(next(s for s in SETUPS if s[0] == k[0])), k[1], MARKS.index(k[2]))):
            setup, sz, m = k
            (pre_on, adv_on, f), (pre_off, adv_off, _) = on[k], off[k]
            right, left, em = adv_off - adv_on, pre_off - pre_on, sz / 2
            print(f"{key}\t{setup}\t{em}\t{m}\t{f}\t{adv_off:.2f}\t{adv_on:.2f}\t{right:.2f}\t{left:.2f}\t{right / em:.3f}\t{left / em:.3f}")


def sweep(out, mode="compressPunctuation"):
    """'中，' pairs on lines narrowed from 20 em to 17 em in 0.05 em steps, one paragraph per width.

    Ten pairs are 20 em wide. The first line's character count and width show how far WPS squeezes
    each mark before it breaks earlier, and whether a line-end mark's blank half counts."""
    from docx import Document
    from lxml import etree

    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    text_w = 16838 - 1440
    for key, font in (("song", "宋体"), ("fang", "仿宋")):
        for sz in (21, 32):
            em = sz / 2
            d = Document()
            body = d.element.body
            for child in list(body):
                body.remove(child)
            ps = []
            for i in range(61):
                right = text_w - round(em * (20 - 0.05 * i) * 20)
                rpr = f'<w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="{font}"/><w:sz w:val="{sz}"/>'
                ps.append(f'<w:p><w:pPr><w:jc w:val="left"/><w:ind w:left="0" w:right="{right}"/></w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t>{"中，" * 14}</w:t></w:r></w:p>')
            sect = '<w:sectPr><w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/><w:pgMar w:top="720" w:right="720" w:bottom="720" w:left="720" w:header="0" w:footer="0" w:gutter="0"/></w:sectPr>'
            for child in etree.fromstring(f'<w:body xmlns:w="{W}">{"".join(ps)}{sect}</w:body>'):
                body.append(child)
            st = d.settings.element
            for e in st.findall(f"{{{W}}}characterSpacingControl"):
                st.remove(e)
            st.append(etree.fromstring(f'<w:characterSpacingControl xmlns:w="{W}" w:val="{mode}"/>'))
            d.save(out / f"sweep-{key}-{sz}{'' if mode.startswith('compress') else '-off'}.docx")


def sweep_read(d):
    import collections

    import fitz

    for pdf in sorted((Path(d) / "pdf").glob("sweep-*.pdf")):
        em = 16 if "-32" in pdf.name else 10.5
        lines = []
        with fitz.open(pdf) as doc:
            for page in doc:
                rows = collections.OrderedDict()
                for b in page.get_text("rawdict")["blocks"]:
                    for l in b.get("lines", []):
                        for s in l["spans"]:
                            for c in s["chars"]:
                                rows.setdefault(round(c["origin"][1]), []).append((c["origin"][0], c["bbox"][2]))
                for cs in rows.values():
                    cs.sort()
                    lines.append((len(cs), cs[-1][1] - cs[0][0]))
        print(pdf.name)
        i = k = 0
        prev = None
        while i < len(lines):
            n, w = lines[i]
            total, i = n, i + 1
            while total < 28 and i < len(lines):
                total += lines[i][0]
                i += 1
            squeeze = (n * em - w) / (n // 2) / em if n else 0
            if (n, round(squeeze, 3)) != prev:
                print(f"  width {20 - 0.05 * k:5.2f} em: {n:2} chars, squeeze per mark {squeeze:.3f} em")
                prev = (n, round(squeeze, 3))
            k += 1


if __name__ == "__main__":
    cmd, arg = sys.argv[1], sys.argv[2]
    if cmd == "sweep-off":
        sweep(arg, "doNotCompress")
    else:
        {"make": make, "measure": measure, "sweep": sweep, "sweep-read": sweep_read}[cmd](arg)
