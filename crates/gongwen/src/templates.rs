//! The common kinds of 公文 as templates (通知, 请示, 报告, 函, 纪要): each a [`Gongwen`] with
//! placeholder text, so it passes [`crate::check`]. The desktop app seeds them as `.dotx` files in
//! File › New › Personal; the web app, which has no folder for them, adds them to New directly.

use crate::Gongwen;

/// The templates, by file name (without `.dotx`).
pub fn kinds() -> Vec<(&'static str, Gongwen)> {
    let g =
        |organ: &str, number: &str, title: &str, recipients: &str, body: &[&str], signer: &str| {
            Gongwen {
                organ_mark: organ.into(),
                number: number.into(),
                title: title.into(),
                recipients: recipients.into(),
                body: body.iter().map(|s| (*s).to_string()).collect(),
                signer: signer.into(),
                date: "2026年×月×日".into(),
            }
        };
    vec![
        (
            "通知",
            g(
                "××市人民政府文件",
                "×政发〔2026〕×号",
                "关于××××的通知",
                "各区人民政府，市政府各部门：",
                &[
                    "为××××，现就有关事项通知如下。",
                    "一、××××",
                    "正文。",
                    "二、××××",
                    "（一）××××。",
                    "（二）××××。",
                    "三、××××",
                    "正文。",
                ],
                "××市人民政府",
            ),
        ),
        (
            "请示",
            g(
                "××市××局文件",
                "×局〔2026〕×号",
                "关于××××的请示",
                "××市人民政府：",
                &[
                    "为××××，现将有关情况请示如下。",
                    "一、基本情况",
                    "正文。",
                    "二、请示事项",
                    "正文。",
                    "妥否，请批示。",
                ],
                "××市××局",
            ),
        ),
        (
            "报告",
            g(
                "××市××局文件",
                "×局〔2026〕×号",
                "关于××××情况的报告",
                "××市人民政府：",
                &[
                    "现将××××情况报告如下。",
                    "一、主要工作",
                    "正文。",
                    "二、存在问题",
                    "正文。",
                    "三、下一步打算",
                    "正文。",
                    "特此报告。",
                ],
                "××市××局",
            ),
        ),
        (
            "函",
            g(
                "××市××局",
                "×局函〔2026〕×号",
                "关于商请××××的函",
                "××省××厅：",
                &[
                    "为××××，现商请贵厅××××。",
                    "联系人：×××，电话：××××××××。",
                    "特此函商，请予支持。",
                ],
                "××市××局",
            ),
        ),
        (
            "纪要",
            g(
                "××市人民政府办公室",
                "第×期",
                "××××专题会议纪要",
                "",
                &[
                    "2026年×月×日，×××主持召开××××专题会议。现将会议议定事项纪要如下。",
                    "一、××××",
                    "正文。",
                    "二、××××",
                    "正文。",
                ],
                "××市人民政府办公室",
            ),
        ),
    ]
}

/// The template named `name` (one of [`kinds`]) as a Word document.
pub fn docx(name: &str) -> Result<Vec<u8>, String> {
    let (_, g) = kinds()
        .into_iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| format!("no 公文 template named {name}"))?;
    let mut buf = std::io::Cursor::new(Vec::new());
    crate::write_docx(&g, &mut buf)?;
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_template_is_a_passing_gongwen() {
        for (name, _) in super::kinds() {
            let report = crate::check::report(&super::docx(name).unwrap()).unwrap();
            assert!(report.starts_with("符合"), "{name}: {report}");
        }
        assert!(super::docx("不存在").is_err());
    }
}
