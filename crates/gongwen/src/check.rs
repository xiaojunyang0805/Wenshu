//! Check a DOCX against GB/T 9704: page size, margins, the 22 × 28 grid and the body font.
//!
//! The body font is the East Asian font and size carrying the most text, from direct run
//! formatting and the document defaults. Style inheritance isn't resolved yet, so a document that
//! sets its body font only through a paragraph style reports the default.

use std::collections::HashMap;
use std::io::Read;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::spec::*;

/// One rule's result.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Finding {
    /// Short rule id, e.g. `page.size`.
    pub rule: &'static str,
    pub ok: bool,
    pub expected: String,
    pub actual: String,
}

/// Check the DOCX file at `path`.
pub fn check_docx(path: &std::path::Path) -> Result<Vec<Finding>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    check_bytes(&bytes)
}

/// The findings for a DOCX held in memory as a report, one rule a block, in Chinese and English
/// (the desktop and web apps show it from 检查公文格式). An error when it isn't a DOCX.
pub fn report(docx: &[u8]) -> Result<String, String> {
    let findings = check_bytes(docx)?;
    let failed = findings.iter().filter(|f| !f.ok).count();
    let mut out = if failed == 0 {
        format!(
            "符合 GB/T 9704—2012 的 {} 项检查。
Passes all {} GB/T 9704-2012 checks.

",
            findings.len(),
            findings.len()
        )
    } else {
        format!(
            "{failed} 项不符合 GB/T 9704—2012。
{failed} of {} GB/T 9704-2012 checks fail.

",
            findings.len()
        )
    };
    for f in &findings {
        let mark = if f.ok { "✔" } else { "✘" };
        out.push_str(&format!(
            "{mark} {}
    应为 expected: {}
    实际 actual: {}
",
            f.rule, f.expected, f.actual
        ));
    }
    Ok(out)
}

/// Check a DOCX package held in memory.
pub fn check_bytes(bytes: &[u8]) -> Result<Vec<Finding>, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let document = read_part(&mut zip, "word/document.xml")?
        .ok_or("not a Word document: no word/document.xml")?;
    let styles = read_part(&mut zip, "word/styles.xml")?.unwrap_or_default();
    let defaults = run_defaults(&styles);
    let scan = scan_document(&document, &defaults);
    Ok(findings(&scan))
}

fn read_part<R: Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<Option<String>, String> {
    let Ok(mut f) = zip.by_name(name) else {
        return Ok(None);
    };
    let mut s = String::new();
    f.read_to_string(&mut s)
        .map_err(|e| format!("{name}: {e}"))?;
    Ok(Some(s))
}

fn attr(e: &BytesStart, key: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

fn num(e: &BytesStart, key: &[u8]) -> Option<u32> {
    attr(e, key).and_then(|v| v.parse().ok())
}

#[derive(Clone, Debug, Default)]
struct RunProps {
    font: Option<String>,
    size: Option<u32>,
}

/// The `w:docDefaults` run font (East Asian, else ASCII) and size.
fn run_defaults(styles: &str) -> RunProps {
    let mut reader = Reader::from_str(styles);
    let mut in_defaults = false;
    let mut props = RunProps::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"w:rPrDefault" => in_defaults = true,
            Ok(Event::End(e)) if e.name().as_ref() == b"w:rPrDefault" => break,
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) if in_defaults => match e.name().as_ref() {
                b"w:rFonts" => {
                    props.font = attr(&e, b"w:eastAsia").or_else(|| attr(&e, b"w:ascii"))
                }
                b"w:sz" => props.size = num(&e, b"w:val"),
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    props
}

#[derive(Debug, Default)]
struct Scan {
    page: Option<(u32, u32)>,
    margins: Option<(u32, u32, u32, u32)>,
    grid: Option<(String, u32)>,
    /// Characters of text per (East Asian font, half-point size).
    text_by_font: HashMap<(String, u32), usize>,
}

/// One section's page settings.
#[derive(Debug, Default)]
struct Section {
    page: Option<(u32, u32)>,
    margins: Option<(u32, u32, u32, u32)>,
    grid: Option<(String, u32)>,
}

/// The page, margins and grid are the first section's: a 公文 opens with its header and body
/// (GB/T 9704's 首页), and attachments that follow in sections of their own (a landscape table)
/// may be laid out otherwise. The last section's settings, read before, judged a document by its
/// attachment; the section with the most text could be a large table too.
fn scan_document(xml: &str, defaults: &RunProps) -> Scan {
    let mut reader = Reader::from_str(xml);
    let mut scan = Scan::default();
    let mut run: Option<RunProps> = None;
    let mut in_text = false;
    let mut section = Section::default();
    let mut body: Option<Section> = None;
    // Paragraphs open (text boxes nest them), and a paragraph's own `w:sectPr` waiting for the
    // paragraph to end: that section includes the paragraph's text, which follows its properties.
    let mut para_depth = 0usize;
    let mut pending: Option<(usize, Section)> = None;
    let close = |section: Section, body: &mut Option<Section>| {
        if body.is_none() {
            *body = Some(section);
        }
    };
    loop {
        let event = match reader.read_event() {
            Ok(Event::Eof) | Err(_) => break,
            Ok(e) => e,
        };
        match event {
            Event::Start(e) if e.name().as_ref() == b"w:p" => para_depth += 1,
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                b"w:r" => run = Some(RunProps::default()),
                b"w:rFonts" => {
                    if let Some(r) = run.as_mut() {
                        r.font = attr(&e, b"w:eastAsia").or_else(|| attr(&e, b"w:ascii"));
                    }
                }
                b"w:sz" => {
                    if let Some(r) = run.as_mut() {
                        r.size = num(&e, b"w:val");
                    }
                }
                b"w:t" => in_text = run.is_some(),
                b"w:pgSz" => section.page = num(&e, b"w:w").zip(num(&e, b"w:h")),
                b"w:pgMar" => {
                    if let (Some(t), Some(r), Some(b), Some(l)) = (
                        num(&e, b"w:top"),
                        num(&e, b"w:right"),
                        num(&e, b"w:bottom"),
                        num(&e, b"w:left"),
                    ) {
                        section.margins = Some((t, r, b, l));
                    }
                }
                b"w:docGrid" => {
                    section.grid = Some((
                        attr(&e, b"w:type").unwrap_or_default(),
                        num(&e, b"w:linePitch").unwrap_or(0),
                    ))
                }
                _ => {}
            },
            Event::End(e) => match e.name().as_ref() {
                b"w:sectPr" if para_depth > 0 => {
                    pending = Some((para_depth, std::mem::take(&mut section)))
                }
                b"w:sectPr" => close(std::mem::take(&mut section), &mut body),
                b"w:p" => {
                    if pending.as_ref().is_some_and(|(d, _)| *d == para_depth)
                        && let Some((_, s)) = pending.take()
                    {
                        close(s, &mut body);
                    }
                    para_depth = para_depth.saturating_sub(1);
                }
                b"w:r" => run = None,
                b"w:t" => in_text = false,
                _ => {}
            },
            Event::Text(t) if in_text => {
                let n = std::str::from_utf8(&t)
                    .map(|s| s.chars().filter(|c| !c.is_whitespace()).count())
                    .unwrap_or(0);
                if let Some(r) = &run {
                    let font = r
                        .font
                        .clone()
                        .or_else(|| defaults.font.clone())
                        .unwrap_or_default();
                    let size = r.size.or(defaults.size).unwrap_or(21);
                    *scan.text_by_font.entry((font, size)).or_default() += n;
                }
            }
            _ => {}
        }
    }
    if let Some(b) = body {
        scan.page = b.page;
        scan.margins = b.margins;
        scan.grid = b.grid;
    }
    scan
}

/// `w:docGrid/@w:type`, whose absence means `default`: no grid.
fn grid_name(t: &str) -> &str {
    if t.is_empty() { "default (no grid)" } else { t }
}

fn within(actual: u32, expected: u32, tolerance: u32) -> bool {
    actual.abs_diff(expected) <= tolerance
}

fn findings(scan: &Scan) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut push = |rule, ok, expected: String, actual: String| {
        out.push(Finding {
            rule,
            ok,
            expected,
            actual,
        })
    };
    let page = scan.page.unwrap_or_default();
    push(
        "page.size",
        within(page.0, PAGE_W, 20) && within(page.1, PAGE_H, 20),
        format!("A4 {PAGE_W}×{PAGE_H} twips"),
        format!("{}×{}", page.0, page.1),
    );
    let (t, r, b, l) = scan.margins.unwrap_or_default();
    let mm_of = |v: u32| f64::from(v) / TWIPS_PER_MM;
    push(
        "margin.top",
        within(t, MARGIN_TOP, MARGIN_TOLERANCE),
        "37 mm ± 1".into(),
        format!("{:.1} mm", mm_of(t)),
    );
    push(
        "margin.left",
        within(l, MARGIN_LEFT, MARGIN_TOLERANCE),
        "28 mm ± 1 (订口)".into(),
        format!("{:.1} mm", mm_of(l)),
    );
    let type_w = page.0.saturating_sub(l + r);
    let type_h = page.1.saturating_sub(t + b);
    push(
        "type_area",
        within(type_w, TYPE_W, MARGIN_TOLERANCE) && within(type_h, TYPE_H, MARGIN_TOLERANCE),
        "156 × 225 mm".into(),
        format!("{:.1} × {:.1} mm", mm_of(type_w), mm_of(type_h)),
    );
    let (grid_type, pitch) = scan.grid.clone().unwrap_or_default();
    push(
        "grid.lines",
        grid_type.starts_with("lines") && within(pitch, LINE_PITCH, 3),
        format!("{LINES_PER_PAGE} lines/page (linePitch {LINE_PITCH})"),
        if scan.grid.is_none() {
            "no document grid".into()
        } else {
            format!("type {}, linePitch {pitch}", grid_name(&grid_type))
        },
    );
    push(
        "grid.chars",
        grid_type == "linesAndChars",
        format!("{CHARS_PER_LINE} chars/line (linesAndChars grid)"),
        format!("type {}", grid_name(&grid_type)),
    );
    let body = scan
        .text_by_font
        .iter()
        .max_by_key(|(_, n)| **n)
        .map(|((f, s), _)| (f.clone(), *s));
    let (font, size) = body.unwrap_or_default();
    let fangsong = crate::spec::FONT_BODY.trim_end_matches("_GB2312");
    push(
        "body.font",
        font.contains(fangsong) || font.to_lowercase().contains("fangsong"),
        "仿宋 (FangSong)".into(),
        font,
    );
    push(
        "body.size",
        size == SIZE_3HAO,
        "3号 (16 pt)".into(),
        format!("{} pt", f64::from(size) / 2.0),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sect(w: u32, h: u32, top: u32) -> String {
        format!(
            r#"<w:sectPr><w:pgSz w:w="{w}" w:h="{h}"/><w:pgMar w:top="{top}" w:right="1474" w:bottom="1984" w:left="1588"/><w:docGrid w:type="linesAndChars" w:linePitch="579"/></w:sectPr>"#
        )
    }

    fn para(text: &str, sect_pr: &str) -> String {
        format!(r#"<w:p><w:pPr>{sect_pr}</w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#)
    }

    /// A landscape attachment after the body (its settings in the body's closing `w:sectPr`),
    /// however much text it holds, doesn't stand for the document: the first section does.
    #[test]
    fn the_body_section_is_checked_not_a_landscape_attachment() {
        let body = "正文".repeat(200);
        let xml = format!(
            "<w:document><w:body>{}{}{}</w:body></w:document>",
            para(&body, &sect(11906, 16838, 2098)),
            para(&"附件表格".repeat(500), ""),
            sect(16838, 11906, 1800)
        );
        let scan = scan_document(&xml, &RunProps::default());
        assert_eq!(scan.page, Some((11906, 16838)));
        assert_eq!(scan.margins.map(|m| m.0), Some(2098));
        // One section: its settings, as before.
        let one = format!(
            "<w:document><w:body>{}{}</w:body></w:document>",
            para(&body, ""),
            sect(11906, 16838, 2098)
        );
        assert_eq!(
            scan_document(&one, &RunProps::default()).page,
            Some((11906, 16838))
        );
    }
}
