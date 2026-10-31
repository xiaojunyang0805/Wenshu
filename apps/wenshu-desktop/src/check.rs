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

/// The checker's findings for a DOCX, one line a rule.
pub fn report(docx: &[u8]) -> Result<String, String> {
    let findings = wenshu_gongwen::check::check_bytes(docx)?;
    let failed = findings.iter().filter(|f| !f.ok).count();
    let mut out = if failed == 0 {
        format!(
            "符合 GB/T 9704—2012 的 {} 项检查。\nPasses all {} GB/T 9704-2012 checks.\n\n",
            findings.len(),
            findings.len()
        )
    } else {
        format!(
            "{failed} 项不符合 GB/T 9704—2012。\n{failed} of {} GB/T 9704-2012 checks fail.\n\n",
            findings.len()
        )
    };
    for f in &findings {
        let mark = if f.ok { "✔" } else { "✘" };
        out.push_str(&format!(
            "{mark} {}\n    应为 expected: {}\n    实际 actual: {}\n",
            f.rule, f.expected, f.actual
        ));
    }
    Ok(out)
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
