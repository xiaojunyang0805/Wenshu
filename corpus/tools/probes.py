"""Generate the probe documents: one small .docx per Chinese layout question or feature.

    python corpus/tools/probes.py corpus/files

Each probe isolates one behaviour so a reference render (WPS, Word) answers one question. The text
is our own; files are named `gen-probe-NN-topic.docx`. The table at the bottom lists what each
probe asks. Built from raw WordprocessingML on a python-docx base (for its styles part).
"""
import sys
from pathlib import Path
from xml.sax.saxutils import escape

from docx import Document
from lxml import etree

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"

# 公文 page (GB/T 9704): A4, margins 37/26/35/28 mm, 22 lines of 28 characters at 16 pt.
GONGWEN_SECT = (
    '<w:pgSz w:w="11906" w:h="16838"/>'
    '<w:pgMar w:top="2098" w:right="1474" w:bottom="1984" w:left="1587" w:header="851" w:footer="992" w:gutter="0"/>'
    '<w:docGrid w:type="linesAndChars" w:linePitch="579" w:charSpace="-842"/>'
)
PLAIN_SECT = (
    '<w:pgSz w:w="11906" w:h="16838"/>'
    '<w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="851" w:footer="992" w:gutter="0"/>'
    '<w:docGrid w:type="lines" w:linePitch="312"/>'
)
FS = '<w:rFonts w:ascii="仿宋_GB2312" w:hAnsi="仿宋_GB2312" w:eastAsia="仿宋_GB2312"/><w:sz w:val="32"/><w:szCs w:val="32"/>'
SONG = '<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="宋体"/><w:sz w:val="21"/>'

PAIRS = "他说：“你好。”（再见）、《书名》“引文”。他问：“是吗？”【注】〔2026〕“甲”，“乙”；（丙）（丁）。"


def r(text, rpr=""):
    return f'<w:r><w:rPr>{rpr}</w:rPr><w:t xml:space="preserve">{escape(text)}</w:t></w:r>'


def p(*runs, ppr=""):
    return f'<w:p><w:pPr>{ppr}</w:pPr>{"".join(runs)}</w:p>'


def body_par(text, rpr=FS, ppr='<w:ind w:firstLineChars="200" w:firstLine="640"/>'):
    return p(r(text, rpr), ppr=ppr)


class Probe:
    def __init__(self, sect=GONGWEN_SECT, compression="compressPunctuation", settings_extra=""):
        self.sect, self.compression, self.settings_extra = sect, compression, settings_extra
        self.parts = []

    def add(self, *xml):
        self.parts.extend(xml)
        return self

    def save(self, path):
        d = Document()
        body = d.element.body
        for child in list(body):
            body.remove(child)
        xml = f'<w:body xmlns:w="{W}">{"".join(self.parts)}<w:sectPr>{self.sect}</w:sectPr></w:body>'
        for child in etree.fromstring(xml):
            body.append(child)
        # The grid's character pitch is relative to the default size: a 公文 sets 16 pt 仿宋 as
        # the default (python-docx's template says 11 pt), a plain page keeps 10.5 pt 宋体.
        size, font = (32, "仿宋_GB2312") if self.sect.startswith(GONGWEN_SECT[:20]) and "linesAndChars" in self.sect else (21, "宋体")
        rpr = d.styles.element.find(f"{{{W}}}docDefaults/{{{W}}}rPrDefault/{{{W}}}rPr")
        if rpr is not None:
            for tag in ("sz", "szCs", "rFonts"):
                for e in rpr.findall(f"{{{W}}}{tag}"):
                    rpr.remove(e)
            for e in etree.fromstring(
                f'<x xmlns:w="{W}"><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="{font}"/>'
                f'<w:sz w:val="{size}"/><w:szCs w:val="{size}"/></x>'
            ):
                rpr.append(e)
        normal = [e for st_ in d.styles.element.findall(f"{{{W}}}style") if st_.get(f"{{{W}}}styleId") == "Normal" for e in st_.findall(f"{{{W}}}rPr")]
        for e in normal:
            for tag in ("sz", "szCs", "rFonts"):
                for x in e.findall(f"{{{W}}}{tag}"):
                    e.remove(x)
        st = d.settings.element
        for tag in ("characterSpacingControl",):
            for e in st.findall(f"{{{W}}}{tag}"):
                st.remove(e)
        extra = etree.fromstring(
            f'<x xmlns:w="{W}"><w:characterSpacingControl w:val="{self.compression}"/>{self.settings_extra}</x>'
        )
        for e in extra:
            st.append(e)
        cp = d.core_properties
        cp.author, cp.title, cp.comments = "Wenshu corpus", path.stem, "Generated probe; text is Wenshu's own."
        d.save(path)


def title(text, size=44, font="方正小标宋简体"):
    rpr = f'<w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="{font}"/><w:sz w:val="{size}"/><w:szCs w:val="{size}"/>'
    return p(r(text, rpr), ppr='<w:jc w:val="center"/>')


def note(text):
    """A grey caption saying what the probe asks (kept outside the measured text)."""
    return p(r(text, '<w:color w:val="808080"/><w:sz w:val="18"/>'), ppr='<w:snapToGrid w:val="0"/>')


PROBES = {}


def probe(name, question):
    def deco(fn):
        PROBES[name] = (fn, question)
        return fn
    return deco


@probe("01-compress-on-char-grid", "Does compression apply on a linesAndChars grid (公文 body)?")
def _():
    return Probe().add(note("compressPunctuation on a 28×22 character grid"), *[body_par(PAIRS) for _ in range(3)])


@probe("02-compress-line-grid", "Compression on a lines-only grid: which pairs close up, by how much?")
def _():
    return Probe(sect=PLAIN_SECT).add(note("compressPunctuation, lines grid"), *[body_par(PAIRS, SONG) for _ in range(3)])


@probe("03-compress-off", "Control for 02: doNotCompress.")
def _():
    return Probe(sect=PLAIN_SECT, compression="doNotCompress").add(note("doNotCompress"), *[body_par(PAIRS, SONG) for _ in range(3)])


@probe("04-quote-latin-font", "。” and ”（ with the quotes in the Latin font (no hint): compressed?")
def _():
    rpr = '<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="宋体"/><w:sz w:val="24"/>'
    return Probe(sect=PLAIN_SECT).add(note("quotes take the ascii/hAnsi font"), body_par(PAIRS, rpr), body_par("他说“好。”（是）“对”。", rpr))


@probe("05-quote-hinted", "Same as 04 with w:hint=eastAsia: quotes full-width and compressed.")
def _():
    rpr = '<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="宋体" w:hint="eastAsia"/><w:sz w:val="24"/>'
    return Probe(sect=PLAIN_SECT).add(note("hint=eastAsia"), body_par(PAIRS, rpr), body_par("他说“好。”（是）“对”。", rpr))


def line_end_text():
    return "（甲）（乙）（丙）（丁）（戊）（己）（庚）（辛）（壬）（癸）" * 4


@probe("06-line-end-pairs", "A ）（ pair split by the wrap: does the line end keep the blank half?")
def _():
    ppr = '<w:jc w:val="both"/>'
    return Probe(sect=PLAIN_SECT).add(note("justified, compression on, topLinePunct off"), body_par(line_end_text(), SONG, ppr), p(r(line_end_text(), SONG), ppr='<w:jc w:val="right"/>'))


@probe("07-top-line-punct", "Same as 06 with w:topLinePunct: line-start/end punctuation compressed?")
def _():
    ppr = '<w:jc w:val="both"/><w:topLinePunct/>'
    return Probe(sect=PLAIN_SECT).add(note("topLinePunct on"), body_par(line_end_text(), SONG, ppr), p(r(line_end_text(), SONG), ppr='<w:jc w:val="right"/><w:topLinePunct/>'))


@probe("08-grid-large-text", "How many grid cells does text larger than the default take (18/22/26/54.5 pt)?")
def _():
    pr = Probe().add(note("16 pt grid; each line one size"))
    for half in (36, 44, 52, 109):
        rpr = f'<w:rFonts w:eastAsia="仿宋_GB2312" w:ascii="仿宋_GB2312" w:hAnsi="仿宋_GB2312"/><w:sz w:val="{half}"/>'
        pr.add(p(r("关于加强管理工作", rpr)))
    return pr.add(body_par("正文一行二十八个字的基准行，用于对齐上方的各行文字，看格子数。"))


@probe("09-char-indents", "*Chars indents: one grid pitch per character, whatever the paragraph mark size?")
def _():
    big_mark = '<w:ind w:firstLineChars="200"/><w:rPr><w:sz w:val="44"/></w:rPr>'
    text = "首行缩进两个字符，段落标记为二号字。" * 3
    return Probe().add(
        note("firstLineChars=200 with a 22 pt mark; leftChars/rightChars=200; hangingChars=200"),
        p(r(text, FS), ppr=big_mark),
        body_par("左右各缩进两个字符的段落。" * 4, FS, '<w:ind w:leftChars="200" w:rightChars="200" w:left="640" w:right="640"/>'),
        body_par("悬挂缩进两个字符的段落。" * 4, FS, '<w:ind w:hangingChars="200" w:hanging="640" w:leftChars="200" w:left="640"/>'),
    )


@probe("10-hint-pinyin", "w:hint=eastAsia with w:lang eastAsia=zh-CN: pinyin vowels, Greek, ½ in which font?")
def _():
    hinted = '<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="宋体" w:hint="eastAsia"/><w:sz w:val="28"/>'
    lang = hinted + '<w:lang w:val="en-US" w:eastAsia="zh-CN"/>'
    text = "Běijīng shì Zhōngguó de shǒudū α β ж ½ ¡ ¿ ā á ǎ à ü"
    return Probe(sect=PLAIN_SECT).add(note("first: hint only; second: hint + zh-CN; third: no hint"), p(r(text, hinted)), p(r(text, lang)), p(r(text, SONG)))


@probe("11-kinsoku", "Kinsoku: no ，。、”）》 at line start, no “（《 at line end.")
def _():
    t = "这是一段用于测试避头尾规则的文字，在行末与行首处安排了标点符号：“引号”、（括号）、《书名》。" * 4
    return Probe(sect=PLAIN_SECT).add(note("kinsoku on (default)"), body_par(t, SONG))


@probe("12-kinsoku-off", "Same as 11 with w:kinsoku=0.")
def _():
    t = "这是一段用于测试避头尾规则的文字，在行末与行首处安排了标点符号：“引号”、（括号）、《书名》。" * 4
    return Probe(sect=PLAIN_SECT).add(note("kinsoku off"), body_par(t, SONG, '<w:kinsoku w:val="0"/>'))


@probe("13-autospace", "autoSpaceDE/DN on and off between Chinese, Latin and digits.")
def _():
    t = "使用WPS Office打开DOCX文件，共有128页和3张表，版本为12.1。"
    return Probe(sect=PLAIN_SECT).add(
        note("on / DE off / DN off / both off"),
        body_par(t, SONG, ""),
        body_par(t, SONG, '<w:autoSpaceDE w:val="0"/>'),
        body_par(t, SONG, '<w:autoSpaceDN w:val="0"/>'),
        body_par(t, SONG, '<w:autoSpaceDE w:val="0"/><w:autoSpaceDN w:val="0"/>'),
    )


@probe("14-dash-ellipsis", "—— and …… at line breaks: kept whole, may start a line?")
def _():
    t = "他想了想——还是算了……这件事情——说来话长……我们改日再谈——好吗……" * 4
    return Probe(sect=PLAIN_SECT).add(note("dashes and ellipses near line ends"), body_par(t, SONG))


@probe("15-emphasis-scaling", "着重号 (w:em), character scaling (w:w) and spacing (w:spacing).")
def _():
    return Probe(sect=PLAIN_SECT).add(
        p(r("加着重号的文字", SONG + '<w:em w:val="dot"/>'), r("和普通文字。", SONG)),
        p(r("缩放百分之六十六", SONG + '<w:w w:val="66"/>'), r("缩放百分之一百五十", SONG + '<w:w w:val="150"/>')),
        p(r("字间距加宽三磅", SONG + '<w:spacing w:val="60"/>'), r("字间距紧缩一磅", SONG + '<w:spacing w:val="-20"/>')),
    )


@probe("16-combine-vert", "双行合一 (eastAsianLayout combine) and 纵横混排 (vert).")
def _():
    return Probe(sect=PLAIN_SECT).add(
        p(r("前文", SONG), r("双行合一示例", SONG + '<w:eastAsianLayout w:id="1" w:combine="1" w:combineBrackets="round"/>'), r("后文", SONG)),
        p(r("纵横混排", SONG), r("12", SONG + '<w:eastAsianLayout w:id="2" w:vert="1"/>'), r("月", SONG)),
    )


@probe("17-ruby", "拼音指南 ruby above Chinese characters.")
def _():
    ruby = lambda base, rt: (
        '<w:r><w:ruby><w:rubyPr><w:rubyAlign w:val="distributeSpace"/><w:hps w:val="10"/><w:hpsRaise w:val="18"/>'
        '<w:hpsBaseText w:val="21"/><w:lid w:val="zh-CN"/></w:rubyPr>'
        f'<w:rt><w:r><w:rPr><w:sz w:val="10"/></w:rPr><w:t>{rt}</w:t></w:r></w:rt>'
        f'<w:rubyBase><w:r><w:rPr><w:rFonts w:eastAsia="宋体"/><w:sz w:val="21"/></w:rPr><w:t>{base}</w:t></w:r></w:rubyBase></w:ruby></w:r>'
    )
    return Probe(sect=PLAIN_SECT).add(p(r("注音：", SONG), ruby("文", "wén"), ruby("枢", "shū"), r("是办公软件。", SONG)))


@probe("18-vertical", "竖排: a tbRl section.")
def _():
    sect = PLAIN_SECT + '<w:textDirection w:val="tbRl"/>'
    return Probe(sect=sect).add(*[body_par("竖排文字从右往左排列，标点符号应旋转或居中。“引号”与（括号）。", SONG) for _ in range(4)])


@probe("19-columns", "Two columns with a separator line, balanced on the last page.")
def _():
    sect = PLAIN_SECT + '<w:cols w:num="2" w:space="425" w:sep="1"/>'
    return Probe(sect=sect).add(*[body_par("分栏排版测试，文字在两栏之间流动，最后一页应当平衡。" * 3, SONG) for _ in range(6)])


@probe("20-numbering", "公文 heading levels 一、（一）1.（1） typed by hand.")
def _():
    hs = [("一、总体要求", "黑体"), ("（一）指导思想", "楷体_GB2312"), ("1.坚持问题导向", "仿宋_GB2312"), ("（1）加强统筹协调", "仿宋_GB2312")]
    pr = Probe()
    for text, font in hs:
        rpr = f'<w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="{font}"/><w:sz w:val="32"/>'
        pr.add(body_par(text, rpr), body_par("各单位要结合实际，认真贯彻落实，确保各项工作取得实效。", FS))
    return pr


@probe("21-table-on-grid", "A table with CJK text and merged cells on a character grid.")
def _():
    cell = lambda t, extra="": f'<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/>{extra}</w:tcPr>{body_par(t, FS, "")}</w:tc>'
    border = "".join(f'<w:{s} w:val="single" w:sz="4" w:space="0" w:color="000000"/>' for s in ("top", "left", "bottom", "right", "insideH", "insideV"))
    rows = [
        cell("序号") + cell("项目名称") + cell("责任单位") + cell("完成时限"),
        cell("1") + cell("办公软件替代", '<w:vMerge w:val="restart"/>') + cell("市信息中心") + cell("2027年6月"),
        cell("2") + cell("", "<w:vMerge/>") + cell("各区政府") + cell("2027年12月"),
    ]
    tbl = f'<w:tbl><w:tblPr><w:tblW w:w="8000" w:type="dxa"/><w:jc w:val="center"/><w:tblBorders>{border}</w:tblBorders></w:tblPr>' + "".join(f"<w:tr>{x}</w:tr>" for x in rows) + "</w:tbl>"
    return Probe().add(body_par("附表如下："), tbl, body_par("表后正文。"))


@probe("22-fonts-by-name", "Font names as documents use them: which substitute does each get?")
def _():
    names = ["仿宋_GB2312", "仿宋", "FangSong", "楷体_GB2312", "楷体", "方正小标宋简体", "方正仿宋_GBK", "华文中宋", "华文仿宋", "宋体", "黑体", "微软雅黑", "等线", "新宋体"]
    pr = Probe(sect=PLAIN_SECT)
    for n in names:
        rpr = f'<w:rFonts w:ascii="{n}" w:hAnsi="{n}" w:eastAsia="{n}"/><w:sz w:val="28"/>'
        pr.add(p(r(f"{n}：中华人民共和国 2026 ABC “引号”", rpr)))
    return pr


@probe("23-line-spacing", "Exact 28.95 pt, multiple 1.5, at least 20 pt; with and without snap to grid.")
def _():
    t = "行距测试文字，观察每行之间的距离以及与文档网格的关系。" * 3
    return Probe().add(
        body_par(t, FS, '<w:spacing w:line="579" w:lineRule="exact"/>'),
        body_par(t, FS, '<w:spacing w:line="360" w:lineRule="auto"/>'),
        body_par(t, FS, '<w:spacing w:line="400" w:lineRule="atLeast"/>'),
        body_par(t, FS, '<w:spacing w:line="360" w:lineRule="auto"/><w:snapToGrid w:val="0"/>'),
    )


@probe("24-fullwidth-forms", "Full-width letters and digits, ideographic-space indents, half-width katakana.")
def _():
    return Probe(sect=PLAIN_SECT).add(
        body_par("全角字母ＡＢＣｄｅｆ与全角数字１２３４５６，半角ABC def 123456。", SONG, ""),
        body_par("　　用两个全角空格缩进的段落，而不是首行缩进。", SONG, ""),
        body_par("半角片假名ｶﾀｶﾅと全角カタカナ。", SONG, ""),
    )


@probe("25-footnotes", "CJK footnotes with circled numbers.")
def _():
    # Footnote parts need a footnotes.xml; kept simple: circled numbers typed as text.
    return Probe(sect=PLAIN_SECT).add(body_par("正文引用①和②，以及③。", SONG), body_par("①　注释一。②　注释二。③　注释三。", SONG + '<w:sz w:val="18"/>'))


@probe("26-red-header", "红头: red organ mark, document number and the red separator rule (paragraph border).")
def _():
    red = '<w:rFonts w:ascii="方正小标宋简体" w:hAnsi="方正小标宋简体" w:eastAsia="方正小标宋简体"/><w:color w:val="FF0000"/><w:w w:val="80"/><w:sz w:val="72"/>'
    rule = '<w:jc w:val="center"/><w:pBdr><w:bottom w:val="single" w:sz="12" w:space="8" w:color="FF0000"/></w:pBdr>'
    return Probe().add(
        p(r("某某市人民政府办公室文件", red), ppr='<w:jc w:val="center"/>'),
        p(r("某政办发〔2026〕12号", FS), ppr=rule),
        title("关于开展办公软件测试工作的通知"),
        body_par("各有关单位：", FS, ""),
        body_par("为做好测试工作，现将有关事项通知如下。"),
    )


@probe("27-traditional", "Traditional Chinese text with 新細明體/PMingLiU and 「」『』 brackets.")
def _():
    rpr = '<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="新細明體"/><w:sz w:val="24"/><w:lang w:eastAsia="zh-TW"/>'
    return Probe(sect=PLAIN_SECT).add(body_par("他說：「你好。」『書名』（註）、。這是繁體中文的測試段落，標點居中或靠左因地區而異。" * 2, rpr))


@probe("28-kana-compression", "compressPunctuationAndJapaneseKana: kana closes up too?")
def _():
    rpr = '<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="MS Mincho"/><w:sz w:val="24"/><w:lang w:eastAsia="ja-JP"/>'
    return Probe(sect=PLAIN_SECT, compression="compressPunctuationAndJapaneseKana").add(body_par("「こんにちは。」（さようなら）、ショッピング。" * 3, rpr))


@probe("29-snap-off-in-grid", "Paragraphs with snapToGrid off inside a character-grid section.")
def _():
    return Probe().add(body_par("对齐网格的段落，每个字占一个格子。" * 2), body_par("不对齐网格的段落，字距按字体本身。" * 2, FS, '<w:ind w:firstLineChars="200"/><w:snapToGrid w:val="0"/>'))


@probe("30-mixed-sizes-line", "One line mixing 16 pt body, 22 pt and 12 pt runs on a line grid: line height.")
def _():
    big = FS.replace('w:val="32"', 'w:val="44"')
    small = FS.replace('w:val="32"', 'w:val="24"')
    return Probe().add(p(r("正文", FS), r("大字", big), r("小字", small), r("正文继续。", FS), ppr='<w:ind w:firstLineChars="200"/>'), body_par("下一行正文。"))


EXACT_FONTS = [("宋体", 21), ("仿宋_GB2312", 32), ("黑体", 44), ("Times New Roman", 24), ("方正小标宋简体", 44), ("宋体", 72)]


def exact_par(font, size, height_pt, extra=""):
    rpr = f'<w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="{font}"/><w:sz w:val="{size}"/><w:szCs w:val="{size}"/>'
    ppr = f'<w:spacing w:before="0" w:after="0" w:line="{height_pt * 20}" w:lineRule="exact"/><w:ind w:left="0" w:firstLine="0"/>'
    return p(r(f"{font}{size / 2:g}/{height_pt}汉字Abg", rpr), extra, ppr=ppr)


@probe("31-exact-baseline", "Exact line spacing: where does the baseline sit in the line (heights 12/20/30/40 pt, six fonts)?")
def _():
    pars = [exact_par(f, s, h) for h in (12, 20, 30, 40) for f, s in EXACT_FONTS]
    big = '<w:rFonts w:ascii="黑体" w:hAnsi="黑体" w:eastAsia="黑体"/><w:sz w:val="44"/><w:szCs w:val="44"/>'
    pars.append(exact_par("仿宋_GB2312", 32, 30, r("大字", big)))
    return Probe(sect=PLAIN_SECT).add(*pars)


@probe("32-hint-spaces", "Which font sets U+0020 spaces: hint on/off, between Chinese or Latin, in their own run?")
def _():
    def rpr(hint):
        h = ' w:hint="eastAsia"' if hint else ""
        return f'<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="宋体"{h}/><w:sz w:val="40"/><w:szCs w:val="40"/>'
    sp = " " * 10
    cases = []
    for hint in (True, False):
        cases.append(p(r(f"汉字{sp}汉字", rpr(hint))))
        cases.append(p(r(f"abc{sp}abc", rpr(hint))))
        cases.append(p(r("汉字", rpr(hint)), r(sp, rpr(hint)), r("汉字", rpr(hint))))
        cases.append(p(r(f"abc{sp}汉字", rpr(hint))))
    return Probe(sect=PLAIN_SECT).add(*cases)


@probe("33-balance-sbcs-dbcs", "w:balanceSingleByteDoubleByteWidth: which single-byte characters get half an em (spaces, letters, digits, punctuation)?")
def _():
    def rpr(font, hint=True):
        h = ' w:hint="eastAsia"' if hint else ""
        return f'<w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="宋体"{h}/><w:sz w:val="40"/><w:szCs w:val="40"/>'
    sp = " " * 10
    pars = []
    for font in ("Calibri", "Times New Roman"):
        pars.append(p(r(f"汉字{sp}汉字", rpr(font))))
        pars.append(p(r(f"abc{sp}abc", rpr(font))))
        pars.append(p(r(f"abc{sp}abc", rpr(font, hint=False))))
        pars.append(p(r("汉字", rpr(font)), r("iiiii", rpr(font)), r("汉字", rpr(font))))
        pars.append(p(r("汉字", rpr(font)), r("11111", rpr(font)), r("汉字", rpr(font))))
        pars.append(p(r("汉字", rpr(font)), r("WWWWW", rpr(font)), r("汉字", rpr(font))))
        pars.append(p(r("汉字", rpr(font)), r(",.;()", rpr(font)), r("汉字", rpr(font))))
    return Probe(sect=PLAIN_SECT, settings_extra='<w:compat><w:balanceSingleByteDoubleByteWidth/></w:compat>').add(*pars)


@probe("34-chars-indent-line-grid", "*Chars indents on a lines-only grid (no character pitch): mark size, default size, or the twips value?")
def _():
    long = "试验文字" * 14
    def par(ind, size=32):
        rpr = f'<w:rFonts w:ascii="仿宋_GB2312" w:hAnsi="仿宋_GB2312" w:eastAsia="仿宋_GB2312"/><w:sz w:val="{size}"/><w:szCs w:val="{size}"/>'
        return p(r(long, rpr), ppr=f'<w:ind {ind}/><w:rPr>{rpr}</w:rPr>')
    return Probe(sect=PLAIN_SECT).add(
        par('w:leftChars="100"'),
        par('w:leftChars="100" w:left="210"'),
        par('w:leftChars="100" w:left="1000"'),
        par('w:firstLineChars="200"'),
        par('w:firstLineChars="200" w:firstLine="200"'),
        par('w:leftChars="200"', size=44),
        par('w:left="400"'),
    )


@probe("35-grid-scale-spacing", "Character grid: how do w:w (scale) and w:spacing combine with the grid advance?")
def _():
    def run(extra, size=32):
        return r("汉字测试文字", f'<w:rFonts w:ascii="宋体" w:hAnsi="宋体" w:eastAsia="宋体"/>{extra}<w:sz w:val="{size}"/><w:szCs w:val="{size}"/>')
    pars = [p(run(e, sz), ppr='<w:ind w:firstLine="0"/>') for e, sz in [("", 32), ('<w:w w:val="50"/>', 32), ('<w:w w:val="90"/>', 32), ('<w:w w:val="200"/>', 32), ('<w:spacing w:val="40"/>', 32), ('<w:w w:val="90"/>', 108), ('<w:w w:val="90"/><w:spacing w:val="40"/>', 44)]]
    return Probe().add(*pars)


@probe("36-chars-indent-char-grid-small-normal", "*Chars indents on a character grid whose Normal size (10.5 pt) is smaller than the text: grid pitch, run size or mark size?")
def _():
    long = "试验文字" * 16
    def par(ind, run=32, mark=32):
        rr = f'<w:rFonts w:ascii="仿宋_GB2312" w:hAnsi="仿宋_GB2312" w:eastAsia="仿宋_GB2312"/><w:sz w:val="{run}"/><w:szCs w:val="{run}"/>'
        mr = f'<w:rFonts w:ascii="仿宋_GB2312" w:hAnsi="仿宋_GB2312" w:eastAsia="仿宋_GB2312"/><w:sz w:val="{mark}"/><w:szCs w:val="{mark}"/>'
        return p(r(long, rr), ppr=f'<w:ind {ind}/><w:rPr>{mr}</w:rPr>')
    sect = PLAIN_SECT.replace('<w:docGrid w:type="lines" w:linePitch="312"/>', '<w:docGrid w:type="linesAndChars" w:linePitch="289" w:charSpace="-1839"/>')
    return Probe(sect=sect).add(
        par('w:firstLineChars="200"'),
        par('w:firstLineChars="200"', mark=21),
        par('w:firstLineChars="200"', run=21),
        par('w:firstLineChars="200"', run=44, mark=44),
        par('w:leftChars="200"'),
        par('w:leftChars="200"', mark=21),
        par('w:leftChars="200"', run=21),
    )


def main(out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    for name, (fn, _) in PROBES.items():
        fn().save(out / f"gen-probe-{name}.docx")
    print(f"{len(PROBES)} probes in {out}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "corpus/files")
