"""Line-height probe documents (F17/F18): python -I <this> <out dir>, then render with wps_render.py."""
import zipfile, sys, os
out = sys.argv[1]
CT = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/></Types>'
RELS = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>'
DRELS = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/></Relationships>'
W='xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'
ZH='中文字体行高测试文字' * 6
def settings(adj, mode):
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:settings {W}>' + ('<w:adjustLineHeightInTable/>' if adj else '') + f'<w:compat><w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="{mode}"/></w:compat></w:settings>'
def doc(sz, grid, table):
    rpr = f'<w:rFonts w:ascii="宋体" w:hAnsi="宋体" w:eastAsia="宋体"/><w:sz w:val="{sz}"/>'
    p = f'<w:p><w:pPr><w:spacing w:line="240" w:lineRule="auto"/></w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t>{ZH}</w:t></w:r></w:p>'
    body = p*2
    if table:
        body = '<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="pct"/><w:tblBorders><w:top w:val="single" w:sz="4"/><w:bottom w:val="single" w:sz="4"/></w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w="8306"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="8306" w:type="dxa"/></w:tcPr>' + p*2 + '</w:tc></w:tr></w:tbl><w:p/>'
    g = {'none':'', 'lines':'<w:docGrid w:type="lines" w:linePitch="312"/>', 'plain':'<w:docGrid w:linePitch="312"/>'}[grid]
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document {W}><w:body>' + body + f'<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="851" w:footer="992" w:gutter="0"/>{g}</w:sectPr></w:body></w:document>'
for mode in (14, 15):
  for adj in (0, 1):
    for grid in ('none', 'lines'):
      for table in (0, 1):
        if table == 0 and adj: continue
        name = f'm{mode}-adj{adj}-{grid}-{"tbl" if table else "body"}'
        with zipfile.ZipFile(os.path.join(out, name + '.docx'), 'w') as z:
            z.writestr('[Content_Types].xml', CT); z.writestr('_rels/.rels', RELS); z.writestr('word/_rels/document.xml.rels', DRELS)
            z.writestr('word/settings.xml', settings(adj, mode)); z.writestr('word/document.xml', doc(26, grid, table))
