"""Line-height probe documents (F17/F18): python -I <this> <out dir>, then render with wps_render.py."""
import zipfile, sys, os
out = sys.argv[1]
CT = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'
RELS = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>'
TEXT = '本書編印之目的旨在報導本區經常性收支及資本性收支有關議題之統計數據俾以提供施政考核及釐訂施政計畫之參考本書所列資料來源係根據本所預決算書直接蒐集之資料加以整編而成'
def doc(ea, ascii_, line):
    sp = f'<w:spacing w:line="{line}" w:lineRule="auto"/>' if line else ''
    rpr = f'<w:rFonts w:ascii="{ascii_}" w:hAnsi="{ascii_}" w:eastAsia="{ea}"/><w:sz w:val="27"/><w:lang w:eastAsia="zh-TW"/>'
    p = f'<w:p><w:pPr>{sp}<w:rPr>{rpr}</w:rPr></w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t>{TEXT}</w:t></w:r></w:p>'
    return ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>'
            + p*3 + '<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="851" w:footer="992" w:gutter="0"/></w:sectPr></w:body></w:document>')
cases = {'song': '宋体', 'pming': '新細明體', 'kai': '標楷體', 'mingliu': '細明體', 'jheng': '微軟正黑體'}
for k, ea in cases.items():
    for asc in ('Calibri', 'Times New Roman'):
        for line in (0, 333):
            name = f'tc-{k}-{asc[:3].lower()}-{line}.docx'
            with zipfile.ZipFile(os.path.join(out, name), 'w') as z:
                z.writestr('[Content_Types].xml', CT); z.writestr('_rels/.rels', RELS); z.writestr('word/document.xml', doc(ea, asc, line))
