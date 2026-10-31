"""Survey .docx files: which app wrote them and which layout features they use.

    python corpus/tools/survey.py <dir-or-files...> > survey.jsonl

One JSON object per file. Reads the OOXML parts only (no rendering), so it is quick and safe on
hostile files: size-capped, and a broken file yields {"error": ...} instead of a crash.
"""
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

from lxml import etree

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
NS = {"w": W}
MAX_PART = 64 << 20  # bytes; larger parts are skipped
CJK = re.compile(r"[㐀-鿿豈-﫿]")
GONGWEN_NO = re.compile(r"〔\s*\d{4}\s*〕\s*\d+\s*号")
FONT_FAMILIES = {
    "fangsong": ("仿宋", "FangSong", "FangSong_GB2312"),
    "kaiti": ("楷体", "KaiTi", "KaiTi_GB2312"),
    "songti": ("宋体", "SimSun", "NSimSun", "华文中宋", "Songti"),
    "heiti": ("黑体", "SimHei", "微软雅黑", "Microsoft YaHei", "等线", "DengXian"),
    "xiaobiaosong": ("小标宋",),
}


def part(z, name):
    try:
        info = z.getinfo(name)
    except KeyError:
        return None
    if info.file_size > MAX_PART:
        return None
    return z.read(name)


def xml(z, name):
    data = part(z, name)
    if data is None:
        return None
    try:
        return etree.fromstring(data, etree.XMLParser(resolve_entities=False, huge_tree=False, recover=True))
    except etree.XMLSyntaxError:
        return None


def app_of(z):
    root = xml(z, "docProps/app.xml")
    if root is None:
        return "unknown", None, None
    get = lambda tag: next((e.text for e in root.iter() if etree.QName(e).localname == tag), None)
    name, ver, pages = get("Application") or "unknown", get("AppVersion"), get("Pages")
    low = name.lower()
    family = (
        "wps" if "wps" in low or "kingsoft" in low
        else "word" if "microsoft" in low
        else "libreoffice" if "libreoffice" in low or "openoffice" in low
        else "other" if name != "unknown" else "unknown"
    )
    try:
        pages = int(pages) if pages else None
    except ValueError:
        pages = None
    return family, f"{name} {ver or ''}".strip(), pages


def survey(path):
    p = Path(path)
    r = {"file": p.name, "bytes": p.stat().st_size}
    try:
        z = zipfile.ZipFile(p)
    except (zipfile.BadZipFile, OSError) as e:
        return {**r, "error": f"not a zip: {e}"}
    with z:
        names = set(z.namelist())
        if "word/document.xml" not in names:
            return {**r, "error": "no word/document.xml"}
        doc = xml(z, "word/document.xml")
        if doc is None:
            return {**r, "error": "document.xml unreadable"}
        settings = xml(z, "word/settings.xml")
        styles = xml(z, "word/styles.xml")
        r["app"], r["app_name"], r["app_pages"] = app_of(z)
        text = "".join(t.text or "" for t in doc.iter(f"{{{W}}}t"))
        r["chars"] = len(text)
        r["cjk_share"] = round(len(CJK.findall(text)) / max(1, len(text)), 3)
        r["text_sha"] = hashlib.sha256(re.sub(r"\s+", "", text).encode()).hexdigest()[:16]
        count = lambda root, q: len(root.xpath(q, namespaces=NS)) if root is not None else 0
        sects = doc.xpath("//w:sectPr", namespaces=NS)
        r["sections"] = len(sects)
        s0 = sects[-1] if sects else None
        if s0 is not None:
            pg = s0.find("w:pgSz", NS)
            mar = s0.find("w:pgMar", NS)
            grid = s0.find("w:docGrid", NS)
            a = lambda e, n: e.get(f"{{{W}}}{n}") if e is not None else None
            r["page"] = [a(pg, "w"), a(pg, "h"), a(pg, "orient")]
            r["margins"] = [a(mar, k) for k in ("top", "right", "bottom", "left")]
            r["grid"] = [a(grid, "type"), a(grid, "linePitch"), a(grid, "charSpace")]
        f = {}
        f["tables"] = count(doc, "//w:tbl")
        f["nested_tables"] = count(doc, "//w:tbl//w:tbl")
        f["merged_cells"] = count(doc, "//w:vMerge|//w:gridSpan")
        f["images"] = count(doc, "//w:drawing//*[local-name()='blip']") + count(doc, "//*[local-name()='imagedata']")
        f["textboxes"] = count(doc, "//*[local-name()='txbx' or local-name()='textbox']")
        f["shapes"] = count(doc, "//*[local-name()='wsp']|//*[local-name()='shape']")
        f["charts"] = count(doc, "//*[local-name()='chart']")
        f["floating"] = count(doc, "//*[local-name()='anchor']")
        f["numbering"] = count(doc, "//w:numPr")
        f["fields"] = count(doc, "//w:fldSimple|//w:instrText")
        f["toc"] = int(bool(doc.xpath("//w:instrText[contains(., 'TOC')]", namespaces=NS)))
        f["footnotes"] = count(doc, "//w:footnoteReference|//w:endnoteReference")
        f["headers"] = len([n for n in names if re.match(r"word/(header|footer)\d*\.xml", n)])
        f["columns"] = count(doc, "//w:cols[@w:num > 1]")
        f["vertical"] = count(doc, "//w:textDirection[@w:val='tbRl' or @w:val='tbRlV']")
        f["ruby"] = count(doc, "//w:ruby")
        f["math"] = count(doc, "//*[local-name()='oMath']")
        f["tracked"] = count(doc, "//w:ins|//w:del")
        f["comments"] = count(doc, "//w:commentReference")
        f["sdt"] = count(doc, "//w:sdt")
        f["hint_ea"] = count(doc, "//w:rFonts[@w:hint='eastAsia']") + count(styles, "//w:rFonts[@w:hint='eastAsia']")
        f["char_indent"] = count(doc, "//w:ind[@w:firstLineChars or @w:leftChars or @w:hangingChars]")
        f["snap_off"] = count(doc, "//w:snapToGrid[@w:val='0' or @w:val='false']")
        f["kinsoku_off"] = count(doc, "//w:kinsoku[@w:val='0' or @w:val='false']")
        f["exact_spacing"] = count(doc, "//w:spacing[@w:lineRule='exact']")
        f["red_text"] = count(doc, "//w:color[@w:val='FF0000' or @w:val='E60000' or @w:val='C00000']")
        r["features"] = {k: v for k, v in f.items() if v}
        csc = settings.find("w:characterSpacingControl", NS) if settings is not None else None
        r["compression"] = csc.get(f"{{{W}}}val") if csc is not None else None
        fonts = set()
        for root in (doc, styles):
            if root is not None:
                for e in root.iter(f"{{{W}}}rFonts"):
                    for k in ("eastAsia", "ascii"):
                        v = e.get(f"{{{W}}}{k}")
                        if v:
                            fonts.add(v)
        r["fonts"] = sorted(fonts)[:40]
        r["font_families"] = sorted({fam for fam, keys in FONT_FAMILIES.items() if any(k in fn for fn in fonts for k in keys)})
        r["gongwen_like"] = bool(GONGWEN_NO.search(text)) or ("fangsong" in r["font_families"] and "xiaobiaosong" in r["font_families"])
        return r


def main(args):
    paths = []
    for a in args:
        p = Path(a)
        paths.extend(sorted(p.glob("*.docx")) if p.is_dir() else [p])
    for p in paths:
        try:
            r = survey(p)
        except Exception as e:  # a survey never stops on one bad file
            r = {"file": p.name, "error": f"{type(e).__name__}: {e}"}
        print(json.dumps(r, ensure_ascii=False))


if __name__ == "__main__":
    main(sys.argv[1:])
