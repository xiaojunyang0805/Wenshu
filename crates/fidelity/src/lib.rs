//! The fidelity harness: render every document of a corpus, compare each page with a reference
//! image when there is one, and write a scoreboard.
//!
//! Layout: `<corpus>/files/*.docx` are the documents; `<corpus>/ref/<stem>.p<N>.png` are reference
//! renders (from WPS or Word, made by people with licences; never committed when their licence
//! forbids it). A page scores `1 - diff`, where `diff` is the share of pixels whose luminance
//! differs by more than [`PIXEL_THRESHOLD`] after scaling the reference to the render's size. A
//! page count that differs from the reference is reported separately: it's the first thing a
//! reader notices.

use std::path::{Path, PathBuf};

use image::{GrayImage, imageops::FilterType};
use serde::Serialize;
use wenshu_core::Doc;

/// Luminance difference (0–255) below which two pixels count as equal (anti-aliasing noise).
pub const PIXEL_THRESHOLD: u8 = 48;

const EXTENSIONS: &[&str] = &["docx", "doc", "odt", "rtf"];

#[derive(Debug, Serialize)]
pub struct PageScore {
    pub page: usize,
    pub render: PathBuf,
    /// `None` when there's no reference image for this page.
    pub score: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct DocReport {
    pub file: PathBuf,
    pub pages: usize,
    /// Pages in the reference set, when there is one.
    pub reference_pages: Option<usize>,
    pub layout_ms: Option<f64>,
    pub page_scores: Vec<PageScore>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub documents: Vec<DocReport>,
    /// Mean page score over pages that have references.
    pub mean_score: Option<f64>,
    pub page_count_matches: usize,
    pub page_count_compared: usize,
    pub failures: usize,
}

/// Run the corpus at `corpus` and write renders under `out`.
pub fn run(corpus: &Path, out: &Path, scale: f64) -> Result<Report, String> {
    let files_dir = corpus.join("files");
    let ref_dir = corpus.join("ref");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&files_dir)
        .map_err(|e| format!("{}: {e}", files_dir.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| EXTENSIONS.contains(&x.to_ascii_lowercase().as_str()))
        })
        .collect();
    files.sort();
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let documents: Vec<DocReport> = files
        .iter()
        .map(|f| run_one(f, &ref_dir, out, scale))
        .collect();
    let scores: Vec<f64> = documents
        .iter()
        .flat_map(|d| d.page_scores.iter().filter_map(|p| p.score))
        .collect();
    let compared: Vec<&DocReport> = documents
        .iter()
        .filter(|d| d.reference_pages.is_some())
        .collect();
    Ok(Report {
        mean_score: (!scores.is_empty()).then(|| scores.iter().sum::<f64>() / scores.len() as f64),
        page_count_matches: compared
            .iter()
            .filter(|d| d.reference_pages == Some(d.pages))
            .count(),
        page_count_compared: compared.len(),
        failures: documents.iter().filter(|d| d.error.is_some()).count(),
        documents,
    })
}

fn run_one(file: &Path, ref_dir: &Path, out: &Path, scale: f64) -> DocReport {
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let reference_pages = count_reference_pages(ref_dir, &stem);
    let mut report = DocReport {
        file: file.into(),
        pages: 0,
        reference_pages,
        layout_ms: None,
        page_scores: Vec::new(),
        error: None,
    };
    let result = (|| -> Result<(), String> {
        let mut doc = Doc::open(file, None)?;
        let info = doc.info()?;
        report.layout_ms = info.get("layoutMs").and_then(serde_json::Value::as_f64);
        report.pages = doc.pages()?;
        for page in 1..=report.pages {
            let render = out.join(format!("{stem}.p{page}.png"));
            doc.render_png(page, scale, &render)?;
            let reference = ref_dir.join(format!("{stem}.p{page}.png"));
            let score = reference
                .is_file()
                .then(|| compare(&render, &reference))
                .transpose()?;
            report.page_scores.push(PageScore {
                page,
                render,
                score,
            });
        }
        Ok(())
    })();
    report.error = result.err();
    report
}

fn count_reference_pages(ref_dir: &Path, stem: &str) -> Option<usize> {
    let n = (1..)
        .take_while(|p| ref_dir.join(format!("{stem}.p{p}.png")).is_file())
        .count();
    (n > 0).then_some(n)
}

fn load_gray(path: &Path) -> Result<GrayImage, String> {
    Ok(image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_luma8())
}

/// The share of matching pixels between a render and its reference, in 0–1.
pub fn compare(render: &Path, reference: &Path) -> Result<f64, String> {
    let a = load_gray(render)?;
    let b = image::imageops::resize(
        &load_gray(reference)?,
        a.width(),
        a.height(),
        FilterType::Triangle,
    );
    Ok(score(&a, &b))
}

fn score(a: &GrayImage, b: &GrayImage) -> f64 {
    let total = a.pixels().len().max(1);
    let differing = a
        .pixels()
        .zip(b.pixels())
        .filter(|(p, q)| p.0[0].abs_diff(q.0[0]) > PIXEL_THRESHOLD)
        .count();
    1.0 - differing as f64 / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_pages_score_one_and_inverted_zero() {
        let a = GrayImage::from_fn(8, 8, |x, _| image::Luma([if x < 4 { 0 } else { 255 }]));
        let inverted = GrayImage::from_fn(8, 8, |x, _| image::Luma([if x < 4 { 255 } else { 0 }]));
        assert_eq!(score(&a, &a), 1.0);
        assert_eq!(score(&a, &inverted), 0.0);
    }
}
