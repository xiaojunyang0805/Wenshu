"""M3: OFD export across the corpus. For every corpus document, `wenshu convert` writes an OFD and a
PDF (a fresh process each), then:

- package: OFD.xml → DocRoot → Document.xml → pages and resources all present and well formed;
- fonts: every font has an embedded FontFile that opens as a font on its own (fontTools), and
  every character drawn in it is in its cmap (or has a CGTransform);
- pages: as many as the PDF export;
- text and place: each page's characters against the PDF's (PyMuPDF): recall both ways, and the
  share of OFD characters within 1 pt of the same character in the PDF (both exports draw the
  same layout, so they should coincide);
- outline and links: the OFD's outline entries and link annotations against the PDF's outline
  and links (each OFD Goto must name a page of the document).

python -X utf8 corpus/tools/ofd_check.py [wenshu.exe] [stem-filter]
Writes out/ofd/<stem>.ofd, out/ofd/<stem>.pdf, out/ofd/report.json and out/ofd/summary.txt."""
import io, json, os, re, subprocess, sys, time, zipfile
from collections import Counter

import fitz  # PyMuPDF
from fontTools.ttLib import TTFont
from lxml import etree
import logging
logging.getLogger("fontTools").setLevel(logging.ERROR)

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EXE = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.join(ROOT, 'eval', 'target', 'wordcraft', 'release', 'wenshu.exe')
FILTER = sys.argv[2] if len(sys.argv) > 2 else ''
FILES = os.path.join(ROOT, 'corpus', 'files')
OUT = os.path.join(ROOT, 'out', 'ofd')
NS = {'ofd': 'http://www.ofdspec.org/2016'}
PT = 72 / 25.4  # points per mm


def floats(s):
    """OFD number lists, with the `g n v` repeat shorthand."""
    v, t = [], (s or '').split()
    i = 0
    while i < len(t):
        if t[i] == 'g' and i + 2 < len(t):
            v += [float(t[i + 2])] * int(t[i + 1])
            i += 3
        else:
            v.append(float(t[i]))
            i += 1
    return v


def read_ofd(path):
    z = zipfile.ZipFile(path)
    names = set(z.namelist())
    problems = []
    xml = lambda n: etree.fromstring(z.read(n))
    root = xml('OFD.xml')
    doc_loc = root.findtext('.//ofd:DocRoot', namespaces=NS)
    base = os.path.dirname(doc_loc)
    doc = xml(doc_loc)
    fonts = {}
    for rl in doc.findall('.//ofd:PublicRes', NS) + doc.findall('.//ofd:DocumentRes', NS):
        res = xml(f'{base}/{rl.text}')
        rbase = f"{base}/{res.get('BaseLoc')}"
        for f in res.findall('.//ofd:Font', NS):
            ff = f.findtext('ofd:FontFile', namespaces=NS)
            cmap = None
            if not ff:
                problems.append(f"font {f.get('FontName')} not embedded")
            elif f'{rbase}/{ff}' not in names:
                problems.append(f'missing {rbase}/{ff}')
            else:
                try:
                    tt = TTFont(io.BytesIO(z.read(f'{rbase}/{ff}')))
                    cmap = tt.getBestCmap() or {}
                    tt['hhea'], tt['hmtx'], tt['maxp'], tt['head']
                except Exception as e:  # noqa: BLE001
                    problems.append(f"font {f.get('FontName')}: {e}")
            fonts[f.get('ID')] = (f.get('FontName'), cmap)
        for m in res.findall('.//ofd:MultiMedia', NS):
            mf = f"{rbase}/{m.findtext('ofd:MediaFile', namespaces=NS)}"
            if mf not in names:
                problems.append(f'missing {mf}')
    page_ids = [p.get('ID') for p in doc.findall('.//ofd:Pages/ofd:Page', NS)]
    outline = len(doc.findall('.//ofd:Outlines//ofd:OutlineElem', NS))
    links = 0
    gotos = doc.findall('.//ofd:Outlines//ofd:Dest', NS)
    al = doc.findtext('ofd:Annotations', namespaces=NS)
    if al:
        ann = xml(f'{base}/{al}')
        abase = os.path.dirname(f'{base}/{al}')
        for pg in ann.findall('ofd:Page', NS):
            pa = xml(f"{abase}/{pg.findtext('ofd:FileLoc', namespaces=NS)}")
            links += len(pa.findall('ofd:Annot', NS))
            gotos += pa.findall('.//ofd:Dest', NS)
    for g in gotos:
        if g.get('PageID') not in page_ids:
            problems.append(f"Goto to unknown page {g.get('PageID')}")
    pages = []
    for p in doc.findall('.//ofd:Pages/ofd:Page', NS):
        pg = xml(f"{base}/{p.get('BaseLoc')}")
        chars = []
        for t in pg.iter('{%s}TextObject' % NS['ofd']):
            fid = t.get('Font')
            if fid not in fonts:
                problems.append(f'text in unknown font {fid}')
                continue
            ctm = [float(v) for v in (t.get('CTM') or '1 0 0 1 0 0').split()]
            bx = [float(v) for v in t.get('Boundary').split()]
            covered = set()
            kids = list(t)
            for k, el in enumerate(kids):
                if el.tag != '{%s}TextCode' % NS['ofd']:
                    continue
                cg = set()
                j = k - 1
                while j >= 0 and kids[j].tag == '{%s}CGTransform' % NS['ofd']:
                    pos, cnt = int(kids[j].get('CodePosition')), int(kids[j].get('CodeCount'))
                    cg.update(range(pos, pos + cnt))
                    j -= 1
                text = el.text or ''
                x, y = float(el.get('X')), float(el.get('Y'))
                dx, dy = floats(el.get('DeltaX')), floats(el.get('DeltaY'))
                cmap = fonts[fid][1]
                for i, ch in enumerate(text):
                    if cmap is not None and i not in cg and ord(ch) not in cmap and not ch.isspace():
                        problems.append(f'{ch!r} not in the cmap of {fonts[fid][0]}')
                    a, b, c, d, e, f = ctm
                    px, py = a * x + c * y + e + bx[0], b * x + d * y + f + bx[1]
                    chars.append((ch, px * PT, py * PT))
                    if i < len(text) - 1:
                        x += dx[i] if i < len(dx) else 0
                        y += dy[i] if i < len(dy) else 0
        pages.append(chars)
    return pages, problems, outline, links


def pdf_nav(path):
    with fitz.open(path) as d:
        return len(d.get_toc()), sum(len(p.get_links()) for p in d)


def pdf_chars(path):
    out = []
    with fitz.open(path) as d:
        for page in d:
            cs = []
            for b in page.get_text('rawdict')['blocks']:
                for l in b.get('lines', []):
                    for s in l['spans']:
                        for c in s['chars']:
                            cs.append((c['c'], c['origin'][0], c['origin'][1]))
            out.append(cs)
    return out


def compare(o, p):
    co = Counter(c for c, _, _ in o if not c.isspace())
    cp = Counter(c for c, _, _ in p if not c.isspace())
    both = sum((co & cp).values())
    near = 0
    index = {}
    for c, x, y in p:
        index.setdefault(c, []).append((x, y))
    for c, x, y in o:
        if c.isspace():
            continue
        if any(abs(x - px) <= 1 and abs(y - py) <= 1 for px, py in index.get(c, ())):
            near += 1
    n = sum(co.values())
    return both, sum(cp.values()), n, near


def main():
    os.makedirs(OUT, exist_ok=True)
    rows = []
    stems = sorted(f for f in os.listdir(FILES) if f.endswith('.docx') and FILTER in f)
    for f in stems:
        stem = f[:-5]
        src = os.path.join(FILES, f)
        ofd, pdf = os.path.join(OUT, stem + '.ofd'), os.path.join(OUT, stem + '.pdf')
        row = {'file': f}
        t0 = time.time()
        r = subprocess.run([EXE, 'convert', src, ofd], capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=600)
        row['ofd_s'] = round(time.time() - t0, 2)
        if r.returncode != 0:
            row['error'] = r.stderr.strip()[-300:]
            rows.append(row)
            print(f, 'ERROR', row['error'])
            continue
        subprocess.run([EXE, 'convert', src, pdf], capture_output=True, timeout=600)
        row['ofd_kb'] = round(os.path.getsize(ofd) / 1024)
        row['pdf_kb'] = round(os.path.getsize(pdf) / 1024) if os.path.exists(pdf) else None
        try:
            pages, problems, row['outline'], row['links'] = read_ofd(ofd)
        except Exception as e:  # noqa: BLE001
            row['error'] = f'unreadable: {e}'
            rows.append(row)
            print(f, 'ERROR', row['error'])
            continue
        ppages = pdf_chars(pdf)
        row['pdf_outline'], row['pdf_links'] = pdf_nav(pdf)
        row['pages'], row['pdf_pages'] = len(pages), len(ppages)
        row['problems'] = sorted(set(problems))[:20]
        row['problem_count'] = len(problems)
        tb = tp = to = tn = 0
        for o, p in zip(pages, ppages):
            b, np_, no, near = compare(o, p)
            tb, tp, to, tn = tb + b, tp + np_, to + no, tn + near
        row['recall'] = round(tb / tp, 4) if tp else 1.0  # PDF characters found in the OFD
        row['precision'] = round(tb / to, 4) if to else 1.0  # OFD characters found in the PDF
        row['placed'] = round(tn / to, 4) if to else 1.0
        rows.append(row)
        print(f, row['pages'], row['recall'], row['precision'], row['placed'], row['problem_count'], f"{row['ofd_s']}s")
    with open(os.path.join(OUT, 'report.json'), 'w', encoding='utf-8') as fh:
        json.dump(rows, fh, ensure_ascii=False, indent=1)
    ok = [r for r in rows if 'error' not in r]
    lines = [
        f'documents: {len(rows)}, converted: {len(ok)}, errors: {len(rows) - len(ok)}',
        f"pages equal to the PDF's: {sum(r['pages'] == r['pdf_pages'] for r in ok)}",
        f"no package/font problems: {sum(r['problem_count'] == 0 for r in ok)}",
        f"mean recall {sum(r['recall'] for r in ok) / max(len(ok), 1):.4f}, precision {sum(r['precision'] for r in ok) / max(len(ok), 1):.4f}, placed {sum(r['placed'] for r in ok) / max(len(ok), 1):.4f}",
        f"documents with placed < 0.99: {sum(r['placed'] < 0.99 for r in ok)}",
        f"outline entries: OFD {sum(r['outline'] for r in ok)}, PDF {sum(r['pdf_outline'] for r in ok)}; documents with the PDF's count: {sum(r['outline'] == r['pdf_outline'] for r in ok)}",
        f"links: OFD {sum(r['links'] for r in ok)} annotations, PDF {sum(r['pdf_links'] for r in ok)} link areas; documents with links in both or neither: {sum((r['links'] > 0) == (r['pdf_links'] > 0) for r in ok)}",
        f"size: OFD {sum(r['ofd_kb'] for r in ok)} KB, PDF {sum(r['pdf_kb'] or 0 for r in ok)} KB; OFD time {sum(r['ofd_s'] for r in rows):.0f} s",
    ]
    with open(os.path.join(OUT, 'summary.txt'), 'w', encoding='utf-8') as fh:
        fh.write('\n'.join(lines) + '\n')
    print('\n'.join(lines))


if __name__ == '__main__':
    main()
