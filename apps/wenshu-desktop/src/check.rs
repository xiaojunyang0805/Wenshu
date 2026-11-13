//! Review › 文枢 › 检查公文格式: the document against GB/T 9704 (`wenshu-gongwen`'s checker),
//! as a report the front end shows. Labels are in Chinese and English, as the interface may be
//! either.

use wordcraft::AppTool;

/// The tools Wenshu adds to the ribbon.
pub fn tools() -> Vec<AppTool> {
    vec![AppTool {
        id: "wenshu.gongwenCheck",
        label: "检查公文格式",
        icon: "checkErrors",
        run: Box::new(report),
    }]
}

/// The checker's report for a DOCX ([`wenshu_gongwen::check::report`]).
pub fn report(docx: &[u8]) -> Result<String, String> {
    wenshu_gongwen::check::report(docx)
}

#[cfg(test)]
mod tests {
    use super::report;

    #[test]
    fn a_generated_gongwen_passes_and_a_blank_page_does_not() {
        let mut buf = std::io::Cursor::new(Vec::new());
        let (_, g) = crate::templates::kinds().into_iter().next().unwrap();
        wenshu_gongwen::write_docx(&g, &mut buf).unwrap();
        let ok = report(buf.get_ref()).unwrap();
        assert!(ok.starts_with("符合"), "{ok}");
        assert!(!ok.contains('✘'), "{ok}");
        // Not a DOCX at all: an error, not a report.
        assert!(report(b"not a zip").is_err());
    }
}
