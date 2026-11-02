"""Width sweeps of arbitrary first lines in WPS: for each case, paragraphs narrowed from the
line's natural width in 0.02 em steps; reports the narrowest width keeping the whole case text
on the first line, and so the squeeze WPS allowed. 16 pt 仿宋, no grid, left aligned, mode 14.

    python corpus/tools/sweep_lines.py make out/sweepline
    python corpus/tools/wps_render.py --files out/sweepline --out out/sweepline --no-png
    python corpus/tools/sweep_lines.py read out/sweepline

Edit CASES for the lines to test; the 2026-11-02 cases (F12) are the last set used.
"""
import sys, json
from pathlib import Path
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
CASES = {
    "end-open-k3": "甲中，中中中，中中中中，中中中中中中《瑞",
    "end-open-k5": "甲，中中，中中，中中，中中，中中中中中《瑞",
    "end-open-k8": "甲，中，中，中，中，中，中，中，中中中《瑞",
    "end-close-k3": "甲中，中中中，中中中中，中中中中中中瑞。",
    "end-close-k5": "甲，中中，中中，中中，中中，中中中中中瑞。",
    "end-closequote-k3": "甲中，中中中，中中中中，中中中中中中瑞”",
    "end-open2-k5": "甲，中中，中中，中中，中中，中中中中《「瑞",
}
SZ, FONT, EM = 32, "仿宋", 16.0
def width_em(text):
    # natural width estimate in em: ASCII digits 0.5 em, the rest 1 em
    return sum(0.5 if c.isascii() else 1.0 for c in text)
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
            ps.append(f'<w:p><w:pPr><w:jc w:val="left"/><w:ind w:left="0" w:right="{right}"/></w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t xml:space="preserve">{t}国国国国国国国国</w:t></w:r></w:p>')
        sect = '<w:sectPr><w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/><w:pgMar w:top="720" w:right="720" w:bottom="720" w:left="720" w:header="0" w:footer="0" w:gutter="0"/></w:sectPr>'
        for c in etree.fromstring(f'<w:body xmlns:w="{W}">{"".join(ps)}{sect}</w:body>'): body.append(c)
        st = d.settings.element
        for e in st.findall(f"{{{W}}}characterSpacingControl"): st.remove(e)
        st.append(etree.fromstring(f'<w:characterSpacingControl xmlns:w="{W}" w:val="compressPunctuation"/>'))
        for e in st.iter(f"{{{W}}}compatSetting"):
            if e.get(f"{{{W}}}name") == "compatibilityMode": e.set(f"{{{W}}}val", "14")
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
