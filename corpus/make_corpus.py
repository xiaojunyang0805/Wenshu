"""Generate a small Chinese-focused test corpus for evaluating office kits."""
from pathlib import Path

from docx import Document
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.oxml.ns import qn
from docx.shared import Pt, RGBColor, Cm
from openpyxl import Workbook
from openpyxl.chart import BarChart, Reference
from pptx import Presentation
from pptx.util import Inches, Pt as PPt

OUT = Path(__file__).parent / "files"
OUT.mkdir(exist_ok=True)


def set_east_asian_font(run, name):
    run.font.name = name
    run._element.rPr.rFonts.set(qn("w:eastAsia"), name)


def gongwen():
    """GB/T 9704-style official document: red header, FangSong body, 28x22 grid."""
    d = Document()
    s = d.sections[0]
    s.page_width, s.page_height = Cm(21), Cm(29.7)
    s.top_margin, s.bottom_margin = Cm(3.7), Cm(3.5)
    s.left_margin, s.right_margin = Cm(2.8), Cm(2.6)
    p = d.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    r = p.add_run("某某市人民政府办公室文件")
    r.font.size = Pt(36)
    r.font.color.rgb = RGBColor(0xE0, 0, 0)
    set_east_asian_font(r, "方正小标宋简体")
    p = d.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    r = p.add_run("某政办发〔2026〕12号")
    set_east_asian_font(r, "仿宋_GB2312")
    r.font.size = Pt(16)
    p = d.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    r = p.add_run("关于进一步加强办公软件国产化替代工作的通知")
    set_east_asian_font(r, "方正小标宋简体")
    r.font.size = Pt(22)
    body = (
        "各区人民政府，市政府各部门、各直属机构：",
        "为深入贯彻落实信息技术应用创新工作部署，推动办公软件自主可控，现就有关事项通知如下。",
        "一、充分认识重要意义。办公软件是政务信息化的基础支撑，各单位要高度重视，切实把思想和行动统一到决策部署上来。",
        "二、明确工作目标。到2027年底，全市党政机关办公软件国产化替代率达到100%，公文格式符合《党政机关公文格式》（GB/T 9704—2012）要求。",
        "三、强化组织保障。各单位主要负责同志要亲自抓，分管负责同志具体抓，确保各项任务落到实处。",
    )
    for i, t in enumerate(body):
        p = d.add_paragraph()
        if i:
            p.paragraph_format.first_line_indent = Pt(32)
        p.paragraph_format.line_spacing = Pt(28.95)
        r = p.add_run(t)
        set_east_asian_font(r, "仿宋_GB2312")
        r.font.size = Pt(16)
    p = d.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    r = p.add_run("某某市人民政府办公室\n2026年10月30日")
    set_east_asian_font(r, "仿宋_GB2312")
    r.font.size = Pt(16)
    d.save(OUT / "01_gongwen.docx")


def mixed():
    """Mixed CJK/Latin, punctuation-heavy text, lists, table, headers, footnote-like refs."""
    d = Document()
    d.sections[0].header.paragraphs[0].text = "Wenshu 测试文档 — 页眉 Header"
    d.add_heading("第一章 中英文混排测试 Mixed Script", level=1)
    d.add_paragraph(
        "WPS Office（金山办公）是中国最流行的办公软件之一，月活设备约6.76亿台。"
        "它支持DOCX、XLSX、PPTX等格式；“引号”、《书名号》、【方括号】、……省略号、——破折号。"
        "行首不应出现：，。、；：？！）」』】等标点（避头尾）。"
    )
    for t in ("第一项：文件格式兼容", "第二项：中文排版（标点挤压）", "第三项：公文模板"):
        d.add_paragraph(t, style="List Number")
    for t in ("要点甲", "要点乙"):
        d.add_paragraph(t, style="List Bullet")
    d.add_heading("第二章 表格", level=1)
    tbl = d.add_table(rows=4, cols=3)
    tbl.style = "Table Grid"
    rows = [("项目", "数量", "金额（元）"), ("笔记本电脑", "10", "65,000.00"),
            ("办公软件授权", "50", "12,500.00"), ("合计", "", "77,500.00")]
    for i, row in enumerate(rows):
        for j, v in enumerate(row):
            tbl.cell(i, j).text = v
    tbl.cell(3, 0).merge(tbl.cell(3, 1))
    d.add_page_break()
    d.add_heading("第三章 长段落分页", level=1)
    para = "这是一个用于测试分页与换行的长段落，包含中文、English words、数字12345和标点符号。" * 12
    for _ in range(8):
        d.add_paragraph(para)
    d.save(OUT / "02_mixed_cjk.docx")


def sheet():
    wb = Workbook()
    ws = wb.active
    ws.title = "销售数据"
    ws.append(["月份", "华东", "华南", "华北", "合计", "增长率"])
    data = [(f"{m}月", 120 + m * 7, 95 + m * 5, 80 + m * 9) for m in range(1, 13)]
    for i, (m, a, b, c) in enumerate(data, start=2):
        ws.append([m, a, b, c, f"=SUM(B{i}:D{i})", None if i == 2 else f"=E{i}/E{i-1}-1"])
        ws[f"F{i}"].number_format = "0.00%"
    ws.append(["总计", "=SUM(B2:B13)", "=SUM(C2:C13)", "=SUM(D2:D13)", "=SUM(E2:E13)"])
    ws["H1"], ws["H2"] = "平均", "=AVERAGE(E2:E13)"
    ws["H3"], ws["H4"] = "最大", "=MAX(E2:E13)"
    ws["H5"], ws["H6"] = "查找6月合计", '=VLOOKUP("6月",A2:E13,5,FALSE)'
    ws["H7"], ws["H8"] = "条件", '=IF(H2>300,"达标","未达标")'
    ws["H9"], ws["H10"] = "文本", '=CONCATENATE(A2,"-",TEXT(B2,"0.0"))'
    ws["H11"], ws["H12"] = "SUMIF", '=SUMIF(B2:B13,">150")'
    ch = BarChart()
    ch.title = "月度销售"
    ch.add_data(Reference(ws, min_col=2, min_row=1, max_col=4, max_row=13), titles_from_data=True)
    ch.set_categories(Reference(ws, min_col=1, min_row=2, max_row=13))
    ws.add_chart(ch, "J2")
    ws2 = wb.create_sheet("跨表引用")
    ws2["A1"], ws2["B1"] = "华东全年", "=销售数据!B14"
    ws2.merge_cells("A3:C3")
    ws2["A3"] = "合并单元格 Merged"
    wb.save(OUT / "03_sales.xlsx")


def deck():
    prs = Presentation()
    s = prs.slides.add_slide(prs.slide_layouts[0])
    s.shapes.title.text = "开放办公套件评估"
    s.placeholders[1].text = "Wenshu 项目 · 2026年10月"
    s = prs.slides.add_slide(prs.slide_layouts[1])
    s.shapes.title.text = "评估要点"
    tf = s.placeholders[1].text_frame
    tf.text = "文件格式兼容性（DOCX/XLSX/PPTX）"
    for t in ("中文排版与字体", "性能与内存", "许可证与社区"):
        tf.add_paragraph().text = t
    s = prs.slides.add_slide(prs.slide_layouts[5])
    s.shapes.title.text = "表格与形状"
    tbl = s.shapes.add_table(3, 3, Inches(1), Inches(2), Inches(6), Inches(1.5)).table
    for i, row in enumerate([("套件", "语言", "许可证"), ("WordCraft", "Rust", "Apache-2.0"), ("IronCalc", "Rust", "MIT/Apache")]):
        for j, v in enumerate(row):
            tbl.cell(i, j).text = v
    box = s.shapes.add_textbox(Inches(1), Inches(4.2), Inches(6), Inches(1))
    box.text_frame.text = "竖排与混排：中文 English 123"
    box.text_frame.paragraphs[0].runs[0].font.size = PPt(24)
    prs.save(OUT / "04_deck.pptx")


if __name__ == "__main__":
    gongwen()
    mixed()
    sheet()
    deck()
    print(sorted(p.name for p in OUT.iterdir()))
