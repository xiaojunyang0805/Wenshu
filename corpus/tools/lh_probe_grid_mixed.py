"""Line-height probe documents (F17/F18): python -I <this> <out dir>, then render with wps_render.py."""
import zipfile, sys, os
out = sys.argv[1]
CT = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'
RELS = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>'
ZH = '中文字体行高测试文字' * 8
EN = 'The quick brown fox jumps over the lazy dog. ' * 4
def doc(runs, line, grid):
    pr = f'<w:spacing w:line="{line}" w:lineRule="auto"/>'
    body = ''.join(f'<w:r><w:rPr><w:rFonts w:ascii="{a}" w:hAnsi="{a}" w:eastAsia="{e}"/><w:sz w:val="{sz}"/></w:rPr><w:t xml:space="preserve">{t}</w:t></w:r>' for a,e,sz,t in runs)
    p = f'<w:p><w:pPr>{pr}</w:pPr>{body}</w:p>'
    g = '<w:docGrid w:type="lines" w:linePitch="312"/>' if grid else ''
    return ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>'
            + p*2 + f'<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="851" w:footer="992" w:gutter="0"/>{g}</w:sectPr></w:body></w:document>')
cases = {}
for sz in (24, 26, 28, 30, 32):
    cases[f'grid-song-{sz}'] = ([('宋体','宋体',sz,ZH)], 240, True)
cases['grid-times-28'] = ([('Times New Roman','宋体',28,EN)], 240, True)
cases['times-en-28'] = ([('Times New Roman','Times New Roman',28,EN)], 240, False)
cases['times-en-ea-song-28'] = ([('Times New Roman','宋体',28,EN)], 240, False)
cases['song-en-28'] = ([('宋体','宋体',28,EN)], 240, False)
cases['calibri-en-28'] = ([('Calibri','Calibri',28,EN)], 240, False)
cases['mixed-28'] = ([('Times New Roman','宋体',28,'Mixed text 中文混排 ' * 10)], 240, False)
cases['zh-in-times-28'] = ([('Times New Roman','Times New Roman',28,ZH)], 240, False)
for k,(runs,line,grid) in cases.items():
    with zipfile.ZipFile(os.path.join(out, f'{k}.docx'), 'w') as z:
        z.writestr('[Content_Types].xml', CT); z.writestr('_rels/.rels', RELS); z.writestr('word/document.xml', doc(runs,line,grid))
