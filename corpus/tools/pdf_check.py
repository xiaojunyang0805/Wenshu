"""J4: PDF export across the corpus. For every corpus document, `wenshu convert` writes a PDF (a
fresh process each, so fonts a document embeds can't leak into the next), then:

- fonts: every font in the PDF embedded, and subset (a six-letter tag before the name);
- text: the text of the .docx (body, headers, footers, notes; field results left out) found in the
  PDF's extractable text: characters (recall), Chinese characters, and pairs of neighbouring
  characters (a rough check of order); replacement and private-use characters counted;
- pages: the PDF's pages rasterized and compared with Wenshu's own render (out/fid-r3, what the
  app shows) and with WPS's render (corpus/ref), scored the way `wenshu fidelity` scores.

python -X utf8 corpus/tools/pdf_check.py [wenshu.exe] [screen-renders-dir]
Writes out/pdf/<stem>.pdf, out/pdf/report.json and out/pdf/summary.txt."""
import html, json, os, re, subprocess, sys, time, unicodedata, zipfile
from collections import Counter

import fitz  # PyMuPDF
from PIL import Image, ImageChops

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EXE = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.join(ROOT, 'eval', 'target', 'wordcraft', 'release', 'wenshu.exe')
SCREEN = os.path.abspath(sys.argv[2]) if len(sys.argv) > 2 else os.path.join(ROOT, 'out', 'fid-r3')
FILES = os.path.join(ROOT, 'corpus', 'files')
REF = os.path.join(ROOT, 'corpus', 'ref')
OUT = os.path.join(ROOT, 'out', 'pdf')
THRESHOLD = 48  # crates/fidelity PIXEL_THRESHOLD
SUBSET = re.compile(r'^[A-Z]{6}\+')


# ---- .docx text (as corpus/tools/roundtrip.py) ----
def unfield(xml):
    xml = re.sub(r'<w:fldSimple[^>]*/>', '', xml)
    xml = re.sub(r'<w:fldSimple.*?</w:fldSimple>', '', xml, flags=re.S)
    out, skip, pos = [], [], 0
    for m in re.finditer(r'<w:fldChar [^>]*w:fldCharType="(begin|separate|end)"[^>]*/>', xml):
        k = m.group(1)
        if not (skip and skip[-1]):
            out.append(xml[pos:m.start()])
        pos = m.end()
        if k == 'begin':
            skip.append(False)
        elif k == 'separate' and skip:
            skip[-1] = True
        elif k == 'end' and skip:
            skip.pop()
    if not (skip and skip[-1]):
        out.append(xml[pos:])
    return ''.join(out)


def docx_text(path):
    z = zipfile.ZipFile(path)
    t = []
    for n in dict.fromkeys(z.namelist()):
        if re.match(r'word/(document|header\d*|footer\d*|footnotes|endnotes)\.xml$', n):
            x = z.read(n).decode('utf-8', errors='replace')
            # (the tag often carries namespace declarations)
            x = re.sub(r'<mc:Fallback\b.*?</mc:Fallback>', '', x, flags=re.S)
            x = re.sub(r'<w:delText[^>]*>.*?</w:delText>', '', x, flags=re.S)
            # hidden runs aren't drawn
            x = re.sub(r'<w:r>(?:(?!</w:r>).)*?<w:vanish/>(?:(?!</w:r>).)*?</w:r>', '', x, flags=re.S)
            x = unfield(x)
            t.append(''.join(re.findall(r'<w:t(?: [^>]*)?>([^<]*)</w:t>', x)))
    return norm(html.unescape(''.join(t)))


def norm(s):
    # compatibility forms fold (PDF extraction may give ﬁ for fi, full-width forms stay as they are)
    s = s.replace(' ', ' ').replace('　', ' ')
    s = ''.join(c for c in s if c not in '﻿​‌‍­')
    s = re.sub(r'\s+', '', s)
    return unicodedata.normalize('NFKC', s)


def cjk(c):
    return '㐀' <= c <= '鿿' or '豈' <= c <= '﫿'


def recall(want, got):
    if not want:
        return None
    w, g = Counter(want), Counter(got)
    return sum(min(n, g[k]) for k, n in w.items()) / sum(w.values())


def score(a, b, blur=False):
    b = b.resize(a.size)
    if blur:  # forgive anti-aliasing differences between two rasterizers
        from PIL import ImageFilter
        a, b = a.filter(ImageFilter.GaussianBlur(1)), b.filter(ImageFilter.GaussianBlur(1))
    d = ImageChops.difference(a, b).point(lambda v: 255 if v > THRESHOLD else 0)
    return 1 - d.histogram()[255] / (a.size[0] * a.size[1])


def pngs(d, stem):
    out, i = [], 1
    while os.path.exists(os.path.join(d, f'{stem}.p{i}.png')):
        out.append(os.path.join(d, f'{stem}.p{i}.png'))
        i += 1
    return out


def check(f):
    stem = os.path.splitext(f)[0]
    src, pdf = os.path.join(FILES, f), os.path.join(OUT, stem + '.pdf')
    r = {'doc': stem}
    t0 = time.time()
    p = subprocess.run([EXE, 'convert', src, pdf], capture_output=True, text=True, encoding='utf-8', errors='replace')
    r['seconds'] = round(time.time() - t0, 2)
    if p.returncode or not os.path.exists(pdf):
        r['error'] = (p.stderr or '').strip()[-300:] or f'exit {p.returncode}'
        return r
    r['bytes'] = os.path.getsize(pdf)
    doc = fitz.open(pdf)
    r['pages'] = doc.page_count
    fonts, text = {}, []
    for page in doc:
        for xref, ext, ftype, base, *_ in page.get_fonts(full=True):
            fonts[xref] = (ext, ftype, base)
        text.append(page.get_text())
    r['fonts'] = len(fonts)
    r['not_embedded'] = sorted({b for e, t, b in fonts.values() if e in ('n/a', '') and t != 'Type3'})
    r['type3'] = sum(1 for e, t, b in fonts.values() if t == 'Type3')
    r['not_subset'] = sorted({b for e, t, b in fonts.values() if e not in ('n/a', '') and not SUBSET.match(b)})
    got = norm(''.join(text))
    want = docx_text(src)
    r['chars'] = len(want)
    r['recall'] = recall(want, got)
    r['cjk_recall'] = recall([c for c in want if cjk(c)], [c for c in got if cjk(c)])
    r['pair_recall'] = recall([want[i:i + 2] for i in range(len(want) - 1)], [got[i:i + 2] for i in range(len(got) - 1)])
    r['bad_chars'] = sum(1 for c in got if c == '�' or '' <= c <= '')
    if r['recall'] is not None and r['recall'] < 1:
        miss = Counter(want) - Counter(got)
        r['missing'] = ''.join(k * min(n, 3) for k, n in miss.most_common(12))
    # pages against the screen render and WPS
    scr, ref = pngs(SCREEN, stem), pngs(REF, stem)
    s_scr, s_ref = [], []
    for i, page in enumerate(doc):
        pm = page.get_pixmap(matrix=fitz.Matrix(1, 1), colorspace=fitz.csGRAY, alpha=False)
        img = Image.frombytes('L', (pm.width, pm.height), pm.samples)
        if i < len(scr):
            a = Image.open(scr[i]).convert('L')
            s_scr.append(score(a, img, blur=True))
        if i < len(ref):
            s_ref.append(score(img, Image.open(ref[i]).convert('L')))
    r['screen_pages'] = len(scr)
    r['ref_pages'] = len(ref)
    r['vs_screen'] = min(s_scr) if s_scr else None
    r['vs_wps'] = sum(s_ref) / len(s_ref) if s_ref else None
    # the screen render's own score against WPS, over the same pages
    fs = FID.get(stem)
    if fs and s_ref:
        own = [p['score'] for p in fs['page_scores'][:len(s_ref)] if p['score'] is not None]
        r['screen_vs_wps'] = sum(own) / len(own) if own else None
    return r


def load_fid():
    try:
        rep = json.load(open(os.path.join(SCREEN, 'report.json'), encoding='utf-8'))
    except OSError:
        return {}
    return {os.path.splitext(os.path.basename(d['file'].replace(chr(92), '/')))[0]: d for d in rep['documents']}


FID = load_fid()


def main():
    os.makedirs(OUT, exist_ok=True)
    files = sorted(f for f in os.listdir(FILES) if f.lower().endswith(('.docx',)))
    rows = []
    for f in files:
        try:
            r = check(f)
        except Exception as e:  # keep going; the report lists it
            r = {'doc': f[:-5], 'error': f'{type(e).__name__}: {e}'}
        rows.append(r)
        print(r['doc'], r.get('pages'), r.get('recall') and round(r['recall'], 4), r.get('vs_screen') and round(r['vs_screen'], 4), r.get('error', ''), flush=True)
    json.dump(rows, open(os.path.join(OUT, 'report.json'), 'w', encoding='utf-8'), ensure_ascii=False, indent=1)

    ok = [r for r in rows if 'error' not in r]
    n = len(rows)
    lines = [f'documents {n}; PDFs written {len(ok)}; failures {n - len(ok)}']
    lines.append(f"page count = Wenshu's screen layout: {sum(r['pages'] == r['screen_pages'] for r in ok)} / {len(ok)}")
    lines.append(f"page count = WPS: {sum(r['pages'] == r['ref_pages'] for r in ok if r['ref_pages'])} / {sum(1 for r in ok if r['ref_pages'])}")
    lines.append(f"all fonts embedded: {sum(not r['not_embedded'] for r in ok)} / {len(ok)}; all subset: {sum(not r['not_subset'] for r in ok)} / {len(ok)}; Type3 fonts in {sum(1 for r in ok if r['type3'])}")
    wt = [r for r in ok if r['recall'] is not None]
    lines.append(f"text: all characters extractable {sum(r['recall'] >= 1 for r in wt)} / {len(wt)}; mean recall {sum(r['recall'] for r in wt) / len(wt):.4f}; "
                 f"Chinese {sum(r['cjk_recall'] for r in wt if r['cjk_recall'] is not None) / max(1, sum(1 for r in wt if r['cjk_recall'] is not None)):.4f}; "
                 f"pairs {sum(r['pair_recall'] for r in wt if r['pair_recall'] is not None) / len(wt):.4f}; documents with U+FFFD / private-use characters {sum(1 for r in ok if r['bad_chars'])}")
    vs = [r['vs_screen'] for r in ok if r['vs_screen'] is not None]
    vw = [r['vs_wps'] for r in ok if r['vs_wps'] is not None]
    lines.append(f"PDF vs screen render: worst page per document, mean {sum(vs) / len(vs):.4f}; documents with every page > 0.99: {sum(v > 0.99 for v in vs)} / {len(vs)}")
    both = [r for r in ok if r.get('vs_wps') is not None and r.get('screen_vs_wps') is not None]
    lines.append(f"PDF vs WPS: mean page score {sum(vw) / len(vw):.4f}; same pages, screen render vs WPS {sum(r['screen_vs_wps'] for r in both) / max(1, len(both)):.4f}, "
                 f"PDF vs WPS {sum(r['vs_wps'] for r in both) / max(1, len(both)):.4f}; PDF more than 0.005 below the screen render: {sum(r['vs_wps'] < r['screen_vs_wps'] - 0.005 for r in both)}")
    lines.append(f"size: median {sorted(r['bytes'] for r in ok)[len(ok) // 2] / 1024:.0f} KB; time: median {sorted(r['seconds'] for r in ok)[len(ok) // 2]:.1f} s")
    lines.append('')
    bad = [r for r in rows if 'error' in r or r['not_embedded'] or r['not_subset'] or (r['recall'] or 1) < 0.995 or r['bad_chars']
           or (r['vs_screen'] or 1) < 0.99 or r['pages'] != r['screen_pages']]
    for r in sorted(bad, key=lambda r: (r.get('recall') or 0)):
        lines.append(f"  {r['doc']}: " + (r['error'] if 'error' in r else
                     f"pages {r['pages']}/{r['screen_pages']} recall {r['recall'] or 0:.3f} cjk {r['cjk_recall'] or 0:.3f} bad {r['bad_chars']} "
                     f"vs_screen {r['vs_screen'] or 0:.3f} not_embedded {r['not_embedded']} not_subset {r['not_subset']} missing {r.get('missing', '')!r}"))
    s = '\n'.join(lines)
    open(os.path.join(OUT, 'summary.txt'), 'w', encoding='utf-8').write(s + '\n')
    print(s)


if __name__ == '__main__':
    main()
