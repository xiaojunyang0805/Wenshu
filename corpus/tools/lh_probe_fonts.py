"""Line-height probe documents (F17/F18): python -I <this> <out dir>, then render with wps_render.py."""
import zipfile, sys, os
out = sys.argv[1]
CT = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'
RELS = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>'
TEXT = '中文字体行高测试文字' * 8
def doc(ea, line, sz):
    rpr = f'<w:rFonts w:ascii="{ea}" w:hAnsi="{ea}" w:eastAsia="{ea}"/><w:sz w:val="{sz}"/>'
    p = f'<w:p><w:pPr><w:spacing w:line="{line}" w:lineRule="auto"/><w:rPr>{rpr}</w:rPr></w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t>{TEXT}</w:t></w:r></w:p>'
    return ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>'
            + p*2 + '<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="851" w:footer="992" w:gutter="0"/></w:sectPr></w:body></w:document>')
cases = {'song': '宋体', 'hei': '黑体', 'kai': '楷体', 'fang': '仿宋', 'yahei': '微软雅黑', 'jheng': '微軟正黑體', 'deng': '等线'}
for k, ea in cases.items():
    for line in (240, 480):
        for sz in (21, 32):
            with zipfile.ZipFile(os.path.join(out, f'lh-{k}-{line}-{sz}.docx'), 'w') as z:
                z.writestr('[Content_Types].xml', CT); z.writestr('_rels/.rels', RELS); z.writestr('word/document.xml', doc(ea, line, sz))
