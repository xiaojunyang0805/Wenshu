"""Width sweeps of arbitrary first lines in WPS: for each case, paragraphs narrowed from the
line's natural width in 0.02 em steps; reports the narrowest width keeping the whole case text
on the first line, and so the squeeze WPS allowed. 16 pt 仿宋, no grid, left aligned, mode 14.

    python corpus/tools/sweep_lines.py make out/sweepline
    python corpus/tools/wps_render.py --files out/sweepline --out out/sweepline --no-png
    python corpus/tools/sweep_lines.py read out/sweepline

A third argument picks the case set (default: the last one in SETS):
    python corpus/tools/sweep_lines.py make out/sweepas autospace
"""
import sys, json
from pathlib import Path
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
SETS = {}
SETS["f12"] = {
    "end-open-k3": "甲中，中中中，中中中中，中中中中中中《瑞",
    "end-open-k5": "甲，中中，中中，中中，中中，中中中中中《瑞",
    "end-open-k8": "甲，中，中，中，中，中，中，中，中中中《瑞",
    "end-close-k3": "甲中，中中中，中中中中，中中中中中中瑞。",
    "end-close-k5": "甲，中中，中中，中中，中中，中中中中中瑞。",
    "end-closequote-k3": "甲中，中中中，中中中中，中中中中中中瑞”",
    "end-open2-k5": "甲，中中，中中，中中，中中，中中中中《「瑞",
}
# F13: CJK-digit autospace (0.25 em each side of a digit run) on overflowing lines
SETS["autospace"] = {
    "as-r1": "甲中中中中12中中中中中中中中中中中中中瑞",
    "as-r2": "甲中中中12中中中中中中34中中中中中中中瑞",
    "as-r4": "甲中12中中中34中中中56中中中78中中中中瑞",
    "as-r2-k3": "甲中，中12中中中，中中34中中，中中中中中瑞",
    "as-r2-open": "甲中中中12中中中中中中34中中中中中中《瑞",
    "as-end": "甲中中中中中中中中中中中中中中中中中中12",
    "as-long": "甲中中中中2020中中中中中中中中中中中中中瑞",
}
# Hanging punctuation: does a closing mark ending a line hang a whole em (overflowPunct)?
SETS["hang"] = {
    "plain-end-period": "甲中中中中中中中中中中中中中中中中中瑞。",
    "plain-end-comma": "甲中中中中中中中中中中中中中中中中中瑞，",
    "plain-end-quote": "甲中中中中中中中中中中中中中中中中中瑞”",
    "k1-end-period": "甲中中中中中中中，中中中中中中中中中瑞。",
    "k3-end-period": "甲中，中中中，中中中中，中中中中中中瑞。",
    "plain-end-open": "甲中中中中中中中中中中中中中中中中中《瑞",
}
# Lines ending in a digit run (dc-policies-7380cfbea3: 两个，squeezed 0.545 em, the spaces not)
SETS["enddigits"] = {
    "k2-end120": "甲中中中，中中中中中中中中，中中中中中中中120",
    "k2-mid120": "甲中中中，中中中中120中中中中，中中中中中中瑞",
    "k0-end120": "甲中中中中中中中中中中中中中中中中中中120",
    "k3-end120": "甲中，中中中，中中中中，中中中中中中中中120",
    "k2-endideo": "甲中中中，中中中中中中中中，中中中中中中中中瑞",
    "real7380": "绿色通道，4家中医医院开设了老年病科，设置老年病床120",
}
SETS["enddigits2"] = {
    "k2-end1": "甲中中中，中中中中中中中中，中中中中中中中中1",
    "k2-end12345": "甲中中中，中中中中中中中中，中中中中中12345",
    "k2-endabc": "甲中中中，中中中中中中中中，中中中中中中中abc",
    "k2-endcomma120": "甲中中中，中中中中中中中中，中中中中中中，120",
    "k1-end120": "甲中中中中中中中中中中中中，中中中中中中中120",
    "k5-end120": "甲中，中中，中中，中中，中中，中中中中中中中120",
}
CASES = SETS[sys.argv[3]] if len(sys.argv) > 3 else list(SETS.values())[-1]
SZ, FONT, EM = 32, "仿宋", 16.0
import os
# SWEEP_MODE=15 SWEEP_JC=both for justified mode-15 lines
MODE, JC = os.environ.get("SWEEP_MODE", "14"), os.environ.get("SWEEP_JC", "left")
# SWEEP_PPR: extra paragraph properties, e.g. '<w:overflowPunct w:val="0"/>'
PPR = os.environ.get("SWEEP_PPR", "")
# SWEEP_COMPAT: extra w:compat children, e.g. "<w:balanceSingleByteDoubleByteWidth/>"
COMPAT = os.environ.get("SWEEP_COMPAT", "")
# SWEEP_GRID: a w:docGrid for the section, e.g. '<w:docGrid w:type="linesAndChars" w:linePitch="312"/>'
GRID = os.environ.get("SWEEP_GRID", "")
def width_em(text):
    # width estimate in em: ASCII digits 0.5 em, the rest 1 em, autospace 0.25 em
    w = sum(0.5 if c.isascii() else 1.0 for c in text)
    return w + 0.25 * sum(a.isascii() != b.isascii() for a, b in zip(text, text[1:]))
def make(out):
    from docx import Document
    from lxml import etree
    out = Path(out); out.mkdir(parents=True, exist_ok=True)
    text_w = 16838 - 1440
    for name, t in CASES.items():
        nat = width_em(t)
        d = Document(); body = d.element.body
        for c in list(body): body.remove(c)
        ps = []
        for i in range(151):
            w = nat + 0.1 - 0.02 * i
            right = text_w - round(EM * w * 20)
            rpr = f'<w:rFonts w:ascii="{FONT}" w:hAnsi="{FONT}" w:eastAsia="{FONT}" w:hint="eastAsia"/><w:sz w:val="{SZ}"/>'
            ps.append(f'<w:p><w:pPr>{PPR}<w:jc w:val="{JC}"/><w:ind w:left="0" w:right="{right}"/></w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t xml:space="preserve">{t}国国国国国国国国</w:t></w:r></w:p>')
        sect = f'<w:sectPr><w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/><w:pgMar w:top="720" w:right="720" w:bottom="720" w:left="720" w:header="0" w:footer="0" w:gutter="0"/>{GRID}</w:sectPr>'
        for c in etree.fromstring(f'<w:body xmlns:w="{W}">{"".join(ps)}{sect}</w:body>'): body.append(c)
        st = d.settings.element
        for e in st.findall(f"{{{W}}}characterSpacingControl"): st.remove(e)
        st.append(etree.fromstring(f'<w:characterSpacingControl xmlns:w="{W}" w:val="compressPunctuation"/>'))
        if COMPAT:
            comp = st.find(f"{{{W}}}compat")
            for c in etree.fromstring(f'<x xmlns:w="{W}">{COMPAT}</x>'):
                comp.insert(0, c)
        for e in st.iter(f"{{{W}}}compatSetting"):
            if e.get(f"{{{W}}}name") == "compatibilityMode": e.set(f"{{{W}}}val", MODE)
        d.save(out / f"line-{name}.docx")
    (out / "cases.json").write_text(json.dumps(CASES, ensure_ascii=False), encoding="utf-8")
def read(out):
    import fitz
    cases = json.loads((Path(out) / "cases.json").read_text(encoding="utf-8"))
    for name, t in cases.items():
        pdf = Path(out) / "pdf" / f"line-{name}.pdf"
        firsts = []
        for page in fitz.open(pdf):
            for b in page.get_text("rawdict")["blocks"]:
                for l in b.get("lines", []):
                    cs = [c for s in l["spans"] for c in s["chars"] if c["c"] != " "]
                    s = "".join(c["c"] for c in cs)
                    if s and s[0] == t[0] and len(s) >= 3 and s[:3] == t[:3]:
                        firsts.append((s, cs[0]["origin"][0], cs[-1]["bbox"][2]))
        nat = width_em(t)
        full = [(i, f) for i, f in enumerate(firsts) if f[0].startswith(t.replace(" ", ""))]
        if not full: print(name, "never whole"); continue
        i, f = max(full)
        natural = firsts[0][2] - firsts[0][1]
        kept = nat + 0.1 - 0.02 * i
        print(f"{name:16} natural {natural / EM:6.2f} em, kept down to {kept:6.2f} em -> squeeze {natural / EM - kept:5.2f} em  ({len(firsts)} lines)")
if __name__ == "__main__":
    {"make": make, "read": read}[sys.argv[1]](sys.argv[2])
