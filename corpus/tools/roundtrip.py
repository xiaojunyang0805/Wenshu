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


# Fields whose result the reader recalculates (WPS and Word on open or when laying out): their
# saved results may differ without anything being lost. Other results (hyperlinks, REF, TOC
# entries) are text the reader shows as it is, and are compared.
VOLATILE = {'PAGE', 'NUMPAGES', 'SECTIONPAGES', 'SECTION', 'DATE', 'TIME', 'PRINTDATE', 'SAVEDATE',
            'CREATEDATE', 'EDITTIME', 'FILENAME', 'FILESIZE', 'NUMWORDS', 'NUMCHARS', 'PAGEREF', '='}


def field_name(instr):
    import html
    m = re.match(r'\s*(=|[A-Za-z]+)', html.unescape(instr))
    return m.group(1).upper() if m else ''


def unfield(xml):
    # Drop the results of VOLATILE fields: what lies between a field's separate and end marks
    # (nested fields included), and such fldSimple results.
    def simple(m):
        return '' if field_name(m.group(1)) in VOLATILE else m.group(0)
    xml = re.sub(r'<w:fldSimple [^>]*w:instr="([^"]*)"[^>]*/>', '', xml)
    xml = re.sub(r'<w:fldSimple [^>]*w:instr="([^"]*)"[^>]*>.*?</w:fldSimple>', simple, xml, flags=re.S)
    out, stack, pos = [], [], 0  # stack: [instruction text, skipping the result]
    tok = re.compile(r'<w:fldChar [^>]*w:fldCharType="(begin|separate|end)"[^>]*/>|<w:instrText[^>]*>([^<]*)</w:instrText>')
    for m in tok.finditer(xml):
        if not any(f[1] for f in stack):
            out.append(xml[pos:m.start()])
        pos = m.end()
        kind = m.group(1)
        if kind is None:  # instruction text, for the innermost field still collecting its code
            if stack and not stack[-1][1]:
                stack[-1][0] += m.group(2)
        elif kind == 'begin':
            stack.append(['', False])
        elif kind == 'separate' and stack:
            stack[-1][1] = field_name(stack[-1][0]) in VOLATILE
        elif kind == 'end' and stack:
            stack.pop()
    if not any(f[1] for f in stack):
        out.append(xml[pos:])
    return ''.join(out)


def text(xml):
    # Visible text runs, VML fallbacks (a second copy of text boxes; the tag often carries
    # namespace declarations) and recalculated field results left out, entities decoded,
    # whitespace dropped.
    import html
    xml = re.sub(r'<mc:Fallback\b.*?</mc:Fallback>', '', xml, flags=re.S)
    xml = re.sub(r'<w:delText[^>]*>.*?</w:delText>', '', xml, flags=re.S)
    xml = unfield(xml)
    # A break or tab inside w:t (against the schema, from some generators) still leaves its text
    inner = re.compile(r'(<w:t(?: [^>]*)?>[^<]*)<w:(?:br|cr|tab)\b[^>]*/>')
    while inner.search(xml):
        xml = inner.sub(r'\1', xml)
    t = ''.join(re.findall(r'<w:t(?: [^>]*)?>([^<]*)</w:t>', xml))
    # WordArt's text is its v:textpath string (a writer may keep it as runs instead)
    t += ''.join(re.findall(r'<v:textpath [^>]*string="([^"]*)"', xml))
    return re.sub(r'\s+', '', html.unescape(t))


def ea_fonts(xml):
    return set(re.findall(r'w:eastAsia="([^"]+)"', ' '.join(re.findall(r'<w:rFonts [^>]*>', xml))))


def kind(n):
    return re.sub(r'\d+', '', n.split('/')[-1])


def summary(path):
    p = parts(path)
    # Headers and footers by kind, as a sorted list of their parts' texts: a writer may number the
    # parts differently (header1 ↔ header2) without changing any of them.
    by = {}
    for n, x in p.items():
        by.setdefault(kind(n), []).append(text(x))
    by = {k: '|'.join(sorted(v)) for k, v in by.items()}
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
