"""J5: does a .docx saved by Wenshu open in WPS unchanged? For every corpus document compare the
original with Wenshu's save (out/rt/files): the text of the body, headers, footers and notes, the
number of tables, the East Asian font names used, and WPS's renders (page count, page scores
against WPS's render of the original, corpus/ref). python -I -X utf8 out/roundtrip.py"""
import json, os, re, sys, zipfile
from PIL import Image, ImageChops

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ORIG = os.path.join(ROOT, 'corpus', 'files')
SAVED = os.path.join(ROOT, 'out', 'rt', 'files')
REF_ORIG = os.path.join(ROOT, 'corpus', 'ref')
REF_SAVED = os.path.join(ROOT, 'out', 'rt', 'ref')
THRESHOLD = 48


def parts(path):
    z = zipfile.ZipFile(path)
    names = list(dict.fromkeys(z.namelist()))
    out = {}
    for n in names:
        if re.match(r'word/(document|header\d*|footer\d*|footnotes|endnotes)\.xml$', n):
            out[n] = z.read(n).decode('utf-8', errors='replace')
    return out


def unfield(xml):
    # Field results are recalculated by the reader (page numbers, dates): drop what lies between
    # a field's separate and end marks, and fldSimple results.
    xml = re.sub(r'<w:fldSimple[^>]*/>', '', xml)
    xml = re.sub(r'<w:fldSimple.*?</w:fldSimple>', '', xml, flags=re.S)
    out, depth, skip, pos = [], 0, [], 0
    for m in re.finditer(r'<w:fldChar [^>]*w:fldCharType="(begin|separate|end)"[^>]*/>', xml):
        kind = m.group(1)
        if not (skip and skip[-1]):
            out.append(xml[pos:m.start()])
        pos = m.end()
        if kind == 'begin':
            skip.append(False)
        elif kind == 'separate' and skip:
            skip[-1] = True
        elif kind == 'end' and skip:
            skip.pop()
    if not (skip and skip[-1]):
        out.append(xml[pos:])
    return ''.join(out)


def text(xml):
    # Visible text runs, VML fallbacks (a second copy of text boxes) and field results left out,
    # entities decoded, whitespace dropped.
    import html
    xml = re.sub(r'<mc:Fallback>.*?</mc:Fallback>', '', xml, flags=re.S)
    xml = re.sub(r'<w:delText[^>]*>.*?</w:delText>', '', xml, flags=re.S)
    xml = unfield(xml)
    t = ''.join(re.findall(r'<w:t(?: [^>]*)?>([^<]*)</w:t>', xml))
    return re.sub(r'\s+', '', html.unescape(t))


def ea_fonts(xml):
    return set(re.findall(r'w:eastAsia="([^"]+)"', ' '.join(re.findall(r'<w:rFonts [^>]*>', xml))))


def kind(n):
    return re.sub(r'\d+', '', n.split('/')[-1])


def summary(path):
    p = parts(path)
    by = {}
    for n, x in p.items():
        k = kind(n)
        by.setdefault(k, '')
        by[k] += text(x)
    doc = p.get('word/document.xml', '')
    return by, doc.count('<w:tbl>'), ea_fonts(doc)


def pngs(d, stem):
    out = []
    i = 1
    while os.path.exists(os.path.join(d, f'{stem}.p{i}.png')):
        out.append(os.path.join(d, f'{stem}.p{i}.png'))
        i += 1
    return out


def score(a, b):
    ia = Image.open(a).convert('L')
    ib = Image.open(b).convert('L').resize(ia.size)
    d = ImageChops.difference(ia, ib).point(lambda v: 255 if v > THRESHOLD else 0)
    return 1 - d.histogram()[255] / (ia.size[0] * ia.size[1])


rows = []
for f in sorted(os.listdir(ORIG)):
    if not f.lower().endswith('.docx'):
        continue
    stem = f[:-5]
    o, s = os.path.join(ORIG, f), os.path.join(SAVED, f)
    r = {'doc': stem}
    try:
        (bo, to, fo), (bs, ts, fs) = summary(o), summary(s)
        r['text'] = all(bo.get(k, '') == bs.get(k, '') for k in set(bo) | set(bs))
        r['text_diff'] = [k for k in set(bo) | set(bs) if bo.get(k, '') != bs.get(k, '')]
        r['tables'] = (to, ts)
        r['fonts_lost'] = sorted(fo - fs)
    except Exception as e:
        r['error'] = str(e)
    po, ps = pngs(REF_ORIG, stem), pngs(REF_SAVED, stem)
    r['pages'] = (len(po), len(ps))
    if po and ps:
        sc = [score(a, b) for a, b in zip(po, ps)]
        r['score'] = sum(sc) / len(sc)
    rows.append(r)

n = len(rows)
text_ok = sum(1 for r in rows if r.get('text'))
tables_ok = sum(1 for r in rows if r.get('tables') and r['tables'][0] == r['tables'][1])
fonts_ok = sum(1 for r in rows if r.get('fonts_lost') == [])
pages_ok = sum(1 for r in rows if r['pages'][0] and r['pages'][0] == r['pages'][1])
scored = [r['score'] for r in rows if 'score' in r]
same = sum(1 for r in rows if r.get('score', 0) > 0.995 and r['pages'][0] == r['pages'][1])
print(f'documents {n}')
print(f'text identical (body, headers, footers, notes): {text_ok}')
print(f'table count kept: {tables_ok}')
print(f'East Asian font names kept: {fonts_ok}')
print(f'WPS page count unchanged: {pages_ok}')
print(f'mean WPS page score saved vs original: {sum(scored) / max(1, len(scored)):.4f}; pixel-identical (>0.995): {same}')
bad = [r for r in rows if not (r.get('text') and r.get('fonts_lost') == [] and r['pages'][0] == r['pages'][1] and r.get('tables', (0, 1))[0] == r.get('tables', (0, 1))[1])]
for r in sorted(bad, key=lambda r: r.get('score', 0))[:40]:
    print(' ', r['doc'], 'text' if not r.get('text') else '', r.get('text_diff', ''), 'tables', r.get('tables'), 'fonts lost', r.get('fonts_lost'), 'pages', r['pages'], f"score {r.get('score', 0):.3f}", r.get('error', ''))
json.dump(rows, open(os.path.join(ROOT, 'out', 'rt', 'report.json'), 'w', encoding='utf-8'), ensure_ascii=False, indent=1, default=list)
