//! Write a GB/T 9704 official document as DOCX.
//!
//! The parts follow the standard's order (§7): the issuing-organ mark (发文机关标志) in red, the
//! document number (发文字号), the red separator line, the title (标题), the principal recipients
//! (主送机关), the body (正文), the signature (发文机关署名) and date (成文日期). Body lines that
//! start with 一、 or （一） become first- and second-level headings in 黑体 and 楷体.

use std::fmt::Write as _;
use std::io::{Seek, Write};

use zip::write::SimpleFileOptions;

use crate::spec::*;

/// The content of an official document.
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct Gongwen {
    /// 发文机关标志, e.g. "某某市人民政府办公室文件".
    pub organ_mark: String,
    /// 发文字号, e.g. "某政办发〔2026〕12号".
    pub number: String,
    /// 标题.
    pub title: String,
    /// 主送机关, e.g. "各区人民政府，市政府各部门："; empty for none.
    pub recipients: String,
    /// 正文, one paragraph per entry.
    pub body: Vec<String>,
    /// 发文机关署名.
    pub signer: String,
    /// 成文日期, e.g. "2026年10月30日".
    pub date: String,
}

/// Write `doc` as a DOCX package to `out`.
pub fn write_docx<W: Write + Seek>(doc: &Gongwen, out: W) -> Result<(), String> {
    let mut zip = zip::ZipWriter::new(out);
    let opts = SimpleFileOptions::default();
    let parts: [(&str, String); 7] = [
        ("[Content_Types].xml", CONTENT_TYPES.to_string()),
        ("_rels/.rels", ROOT_RELS.to_string()),
        ("word/_rels/document.xml.rels", DOC_RELS.to_string()),
        ("word/settings.xml", SETTINGS.to_string()),
        ("word/styles.xml", styles()),
        ("word/footer1.xml", footer(true)),
        ("word/footer2.xml", footer(false)),
    ];
    for (name, xml) in parts
        .iter()
        .chain(std::iter::once(&("word/document.xml", document(doc))))
    {
        zip.start_file(*name, opts).map_err(|e| e.to_string())?;
        zip.write_all(xml.as_bytes()).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Character spacing for the 28-characters grid, in 4096ths of a point relative to the 16 pt body.
fn char_space() -> i64 {
    let pitch_pt = f64::from(TYPE_W) / f64::from(CHARS_PER_LINE) / 20.0;
    ((pitch_pt - f64::from(SIZE_3HAO) / 2.0) * 4096.0).round() as i64
}

fn document(doc: &Gongwen) -> String {
    let mut b = String::new();
    // Organ mark: red, centred, heavy Song.
    para(
        &mut b,
        Align::Center,
        0,
        false,
        &[run(
            &doc.organ_mark,
            FONT_TITLE,
            SIZE_ORGAN_MARK,
            Some(RED),
            false,
        )],
    );
    para(&mut b, Align::Center, 0, false, &[]);
    para(
        &mut b,
        Align::Center,
        0,
        false,
        &[run(&doc.number, FONT_BODY, SIZE_3HAO, None, false)],
    );
    // The red separator under the number (§7.2.6), drawn as the paragraph's bottom border.
    let _ = write!(
        b,
        r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="12" w:space="1" w:color="{RED}"/></w:pBdr></w:pPr></w:p>"#
    );
    para(&mut b, Align::Center, 0, false, &[]);
    para(
        &mut b,
        Align::Center,
        0,
        false,
        &[run(&doc.title, FONT_TITLE, SIZE_2HAO, None, false)],
    );
    para(&mut b, Align::Left, 0, false, &[]);
    if !doc.recipients.is_empty() {
        para(
            &mut b,
            Align::Left,
            0,
            false,
            &[run(&doc.recipients, FONT_BODY, SIZE_3HAO, None, false)],
        );
    }
    for text in &doc.body {
        let font = match heading_level(text) {
            Some(1) => FONT_H1,
            Some(_) => FONT_H2,
            None => FONT_BODY,
        };
        para(
            &mut b,
            Align::Both,
            200,
            false,
            &[run(text, font, SIZE_3HAO, None, false)],
        );
    }
    para(&mut b, Align::Left, 0, false, &[]);
    para(&mut b, Align::Left, 0, false, &[]);
    // Signature and date: right-aligned, four characters in from the right edge (§7.3.5.1).
    para(
        &mut b,
        Align::Right,
        0,
        true,
        &[run(&doc.signer, FONT_BODY, SIZE_3HAO, None, false)],
    );
    para(
        &mut b,
        Align::Right,
        0,
        true,
        &[run(&doc.date, FONT_BODY, SIZE_3HAO, None, false)],
    );
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<w:document xmlns:w="{w}" xmlns:r="{r}"><w:body>{body}"#,
            r#"<w:sectPr><w:footerReference w:type="default" r:id="rIdF1"/><w:footerReference w:type="even" r:id="rIdF2"/>"#,
            r#"<w:pgSz w:w="{pw}" w:h="{ph}"/>"#,
            r#"<w:pgMar w:top="{mt}" w:right="{mr}" w:bottom="{mb}" w:left="{ml}" w:header="851" w:footer="{ft}" w:gutter="0"/>"#,
            r#"<w:docGrid w:type="linesAndChars" w:linePitch="{lp}" w:charSpace="{cs}"/>"#,
            r#"</w:sectPr></w:body></w:document>"#
        ),
        w = NS_W,
        r = NS_R,
        body = b,
        pw = PAGE_W,
        ph = PAGE_H,
        mt = MARGIN_TOP,
        mr = MARGIN_RIGHT,
        mb = MARGIN_BOTTOM,
        ml = MARGIN_LEFT,
        // Page number 7 mm below the type area (§7.5.1): footer distance from the page edge.
        ft = MARGIN_BOTTOM.saturating_sub(mm(7) + LINE_PITCH),
        lp = LINE_PITCH,
        cs = char_space(),
    )
}

/// 1 for 一、二、…, 2 for （一）（二）…, else `None`.
fn heading_level(text: &str) -> Option<u8> {
    const NUMERALS: &str = "一二三四五六七八九十";
    let mut chars = text.trim_start().chars();
    let first = chars.next()?;
    if first == '（' || first == '(' {
        let n: String = chars
            .clone()
            .take_while(|c| NUMERALS.contains(*c))
            .collect();
        let close = chars.nth(n.chars().count())?;
        return (!n.is_empty() && (close == '）' || close == ')')).then_some(2);
    }
    if NUMERALS.contains(first) {
        let rest: String = std::iter::once(first)
            .chain(chars)
            .skip_while(|c| NUMERALS.contains(*c))
            .take(1)
            .collect();
        return (rest == "、").then_some(1);
    }
    None
}

#[derive(Clone, Copy)]
enum Align {
    Left,
    Center,
    Right,
    Both,
}

fn para(
    b: &mut String,
    align: Align,
    first_line_chars: u32,
    right_indent_4: bool,
    runs: &[String],
) {
    let jc = match align {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
        Align::Both => "both",
    };
    b.push_str("<w:p><w:pPr>");
    // Display lines (organ mark, number, title) are larger than the grid's character pitch: off
    // the grid, as 公文 templates set them, so their characters keep their own width.
    if matches!(align, Align::Center) {
        b.push_str(r#"<w:snapToGrid w:val="0"/>"#);
    }
    if first_line_chars > 0 || right_indent_4 {
        b.push_str("<w:ind");
        if right_indent_4 {
            b.push_str(r#" w:rightChars="400" w:right="1280""#);
        }
        if first_line_chars > 0 {
            let _ = write!(
                b,
                r#" w:firstLineChars="{first_line_chars}" w:firstLine="640""#
            );
        }
        b.push_str("/>");
    }
    let _ = write!(b, r#"<w:jc w:val="{jc}"/></w:pPr>"#);
    for r in runs {
        b.push_str(r);
    }
    b.push_str("</w:p>");
}

fn run(text: &str, font: &str, size: u32, color: Option<&str>, bold: bool) -> String {
    let font = esc(font);
    let mut r = format!(
        r#"<w:r><w:rPr><w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:eastAsia="{font}" w:hint="eastAsia"/>"#
    );
    if bold {
        r.push_str("<w:b/>");
    }
    if let Some(c) = color {
        let _ = write!(r, r#"<w:color w:val="{c}"/>"#);
    }
    let _ = write!(
        r,
        r#"<w:sz w:val="{size}"/><w:szCs w:val="{size}"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
        esc(text)
    );
    r
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn styles() -> String {
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="{w}">"#,
            r#"<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="{f}" w:hAnsi="{f}" w:eastAsia="{f}" w:cs="Times New Roman"/>"#,
            r#"<w:kern w:val="2"/><w:sz w:val="{s}"/><w:szCs w:val="{s}"/><w:lang w:val="en-US" w:eastAsia="zh-CN"/></w:rPr></w:rPrDefault>"#,
            r#"<w:pPrDefault><w:pPr><w:widowControl w:val="0"/><w:spacing w:line="{lp}" w:lineRule="exact"/></w:pPr></w:pPrDefault></w:docDefaults>"#,
            r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:jc w:val="both"/></w:pPr></w:style>"#,
            r#"<w:style w:type="paragraph" w:styleId="Footer"><w:name w:val="footer"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:line="240" w:lineRule="auto"/></w:pPr></w:style>"#,
            r#"</w:styles>"#
        ),
        w = NS_W,
        f = FONT_BODY,
        s = SIZE_3HAO,
        lp = LINE_PITCH,
    )
}

/// Page numbers in 4号 宋体 as "— 1 —": odd pages right, even pages left, one character in (§7.5.1).
fn footer(odd: bool) -> String {
    let (jc, ind) = if odd {
        ("right", r#"w:rightChars="100" w:right="280""#)
    } else {
        ("left", r#"w:leftChars="100" w:left="280""#)
    };
    let f = FONT_PAGE_NUMBER;
    let rpr = format!(
        r#"<w:rPr><w:rFonts w:ascii="{f}" w:hAnsi="{f}" w:eastAsia="{f}"/><w:sz w:val="{SIZE_4HAO}"/></w:rPr>"#
    );
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:ftr xmlns:w="{w}"><w:p><w:pPr><w:pStyle w:val="Footer"/><w:ind {ind}/><w:jc w:val="{jc}"/></w:pPr>"#,
            r#"<w:r>{rpr}<w:t xml:space="preserve">— </w:t></w:r>"#,
            r#"<w:r>{rpr}<w:fldChar w:fldCharType="begin"/></w:r><w:r>{rpr}<w:instrText xml:space="preserve"> PAGE </w:instrText></w:r>"#,
            r#"<w:r>{rpr}<w:fldChar w:fldCharType="separate"/></w:r><w:r>{rpr}<w:t>1</w:t></w:r><w:r>{rpr}<w:fldChar w:fldCharType="end"/></w:r>"#,
            r#"<w:r>{rpr}<w:t xml:space="preserve"> —</w:t></w:r></w:p></w:ftr>"#
        ),
        w = NS_W,
        ind = ind,
        jc = jc,
        rpr = rpr,
    )
}

const NS_W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const NS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

const CONTENT_TYPES: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
    r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>"#,
    r#"<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
    r#"<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#,
    r#"<Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/>"#,
    r#"<Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>"#,
    r#"<Override PartName="/word/footer2.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>"#,
    r#"</Types>"#
);

const ROOT_RELS: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>"#,
    r#"</Relationships>"#
);

const DOC_RELS: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    r#"<Relationship Id="rIdS" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
    r#"<Relationship Id="rIdT" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/>"#,
    r#"<Relationship Id="rIdF1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>"#,
    r#"<Relationship Id="rIdF2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer2.xml"/>"#,
    r#"</Relationships>"#
);

const SETTINGS: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    r#"<w:evenAndOddHeaders/><w:defaultTabStop w:val="420"/><w:characterSpacingControl w:val="compressPunctuation"/>"#,
    r#"</w:settings>"#
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_levels() {
        assert_eq!(heading_level("一、充分认识重要意义。"), Some(1));
        assert_eq!(heading_level("十二、其他"), Some(1));
        assert_eq!(heading_level("（三）加强保障"), Some(2));
        assert_eq!(heading_level("一是要加强领导"), None);
        assert_eq!(heading_level("为深入贯彻落实"), None);
        assert_eq!(heading_level(""), None);
    }

    #[test]
    fn grid_char_space_is_slightly_negative() {
        // 156 mm / 28 is 15.79 pt, a little under the 16 pt body.
        assert!((-900..=-800).contains(&char_space()), "{}", char_space());
    }

    #[test]
    fn writes_a_package_the_checker_accepts() {
        let doc = Gongwen {
            organ_mark: "某某市人民政府办公室文件".into(),
            number: "某政办发〔2026〕1号".into(),
            title: "关于测试的通知".into(),
            recipients: "各有关单位：".into(),
            body: vec!["一、总体要求".into(), "正文 & <特殊字符>".into()],
            signer: "某某市人民政府办公室".into(),
            date: "2026年10月30日".into(),
        };
        let mut buf = std::io::Cursor::new(Vec::new());
        write_docx(&doc, &mut buf).unwrap();
        let findings = crate::check::check_bytes(buf.get_ref()).unwrap();
        let failed: Vec<_> = findings.iter().filter(|f| !f.ok).collect();
        assert!(failed.is_empty(), "{failed:#?}");
    }
}
