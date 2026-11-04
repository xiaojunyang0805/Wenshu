//! The Wenshu document core: opens, renders and saves documents through the WordCraft engine,
//! with Wenshu's Chinese defaults installed first.
//!
//! Everything above this crate (CLI, desktop shell, fidelity harness) goes through [`Doc`], so the
//! engine underneath can change without touching them.

use std::path::{Path, PathBuf};
use std::sync::Once;

use serde_json::{Value, json};
use wordcraft_engine::Session;

pub use wenshu_cjk as cjk;

/// The region used for documents that don't give a paper size (text, HTML, Markdown): mainland
/// China, A4. `WENSHU_REGION` (a BCP 47 tag) overrides it.
pub const DEFAULT_REGION: &str = "zh-CN";

static INIT: Once = Once::new();

/// Install Wenshu's defaults into the engine: Chinese document-font substitutes, and Chinese
/// defaults for new blank documents in a Chinese region ([`blank_defaults`]). Idempotent;
/// [`Doc::open`] calls it.
pub fn init() {
    INIT.call_once(|| {
        let _ = wordcraft_fonts::word::set_substitute_provider(wenshu_cjk::fonts::substitutes);
        let _ = wordcraft_engine::set_blank_document_hook(blank_defaults);
        wordcraft_fonts::FontDb::global().add_font_dirs(app_font_dirs());
    });
}

/// The folders of fonts shipped with Wenshu (open FangSong and Kai designs for 公文 on systems
/// without them), cataloged like installed fonts and loaded only when a document uses them:
/// `WENSHU_FONTS_DIR` if set, then `fonts` beside the executable (Windows, portable) and
/// `../share/wenshu/fonts` from it (the Linux packages: `/usr/bin` → `/usr/share/wenshu/fonts`,
/// and the same inside the AppImage and the tar tree). Only folders that exist.
pub fn app_font_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("WENSHU_FONTS_DIR")
        .map(PathBuf::from)
        .into_iter()
        .collect();
    if let Some(bin) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    {
        dirs.push(bin.join("fonts"));
        dirs.push(bin.join("../share/wenshu/fonts"));
    }
    dirs.retain(|d| d.is_dir());
    dirs
}

/// A new blank document's defaults when the region is Chinese (`zh-*`), as a clerk expects them
/// from WPS and Chinese Word: 五号 (10.5 pt) text in 宋体 (新細明體 in Taiwan, Hong Kong and Macau),
/// Times New Roman for Latin text, single spacing with no space after, justified, A4 with 2.54 cm
/// top and bottom and 3.17 cm side margins, and a 15.6 pt line grid. Latin text keeps English
/// proofing (`en-US`), with Chinese as the East Asian language. Any other region keeps WordCraft's
/// own defaults, so an English document starts as it does in WordCraft.
pub fn blank_defaults(region: &wordcraft_engine::Region, d: &mut wordcraft_doc::Document) {
    if region.language != "zh" {
        return;
    }
    let traditional = matches!(region.territory.as_str(), "TW" | "HK" | "MO");
    // As WPS Office's own blank document (2026-11-04, created through COM and saved): Latin text
    // in Calibri (Carlito on Linux), 宋体, 10.5 pt, English proofing for Latin text, punctuation
    // compression on, compatibility mode 14, header 1.5 cm and footer 1.75 cm.
    let c = &mut d.styles.default_chr;
    c.font = Some("Calibri".into());
    c.font_ea = Some(
        if traditional {
            "新細明體"
        } else {
            "宋体"
        }
        .into(),
    );
    c.size = Some(10.5);
    c.lang = Some("en-US".into());
    c.lang_ea = Some(if traditional { "zh-TW" } else { "zh-CN" }.into());
    let p = &mut d.styles.default_para;
    p.space_after = Some(0.0);
    p.line_spacing = Some(wordcraft_doc::props::LineSpacing::Multiple(1.0));
    p.align = Some(wordcraft_doc::Align::Justify);
    d.settings.char_spacing = wordcraft_doc::CharSpacing::CompressPunctuation;
    d.settings.compat_mode = 14;
    let s = &mut d.last_section;
    (s.margin_top, s.margin_bottom, s.margin_left, s.margin_right) = (72.0, 72.0, 90.0, 90.0);
    (s.header, s.footer) = (42.55, 49.6);
    s.columns.space = 21.25;
    s.doc_grid = Some(wordcraft_doc::section::DocGrid {
        kind: wordcraft_doc::section::DocGridType::Lines,
        line_pitch: 15.6,
        char_space: 0,
    });
}

fn region() -> wordcraft_engine::Region {
    let tag = std::env::var("WENSHU_REGION").unwrap_or_else(|_| DEFAULT_REGION.to_string());
    wordcraft_engine::Region::from_tag(&tag)
}

/// An open document.
pub struct Doc {
    session: Session,
}

impl Doc {
    /// Open a document (docx, doc, odt, rtf, html, md, txt…), with `password` for a protected one.
    pub fn open(path: &Path, password: Option<&str>) -> Result<Doc, String> {
        init();
        let region = region();
        let (doc, encrypted) = wordcraft_engine::io::open_path_in(path, password, &region)?;
        let mut session = Session::new(doc);
        session.system_region = region;
        session.path = Some(PathBuf::from(path));
        if encrypted {
            session.password = password.map(wordcraft_engine::io::Password::new);
        }
        Ok(Doc { session })
    }

    /// Run an engine command by id (see `wordcraft-cli commands`).
    pub fn run(&mut self, command: &str, params: &Value) -> Result<Value, String> {
        self.session.run(command, params).map_err(|e| e.to_string())
    }

    /// Page count, paragraphs, properties and layout time.
    pub fn info(&mut self) -> Result<Value, String> {
        self.run("file.info", &json!({}))
    }

    /// The number of laid-out pages.
    pub fn pages(&mut self) -> Result<usize, String> {
        let info = self.info()?;
        let pages = info
            .get("pages")
            .and_then(Value::as_u64)
            .ok_or("file.info gave no page count")?;
        usize::try_from(pages).map_err(|e| e.to_string())
    }

    /// Render page `page` (from 1) to a PNG at `scale` × 96 dpi.
    pub fn render_png(&mut self, page: usize, scale: f64, out: &Path) -> Result<Value, String> {
        self.run(
            "file.exportPng",
            &json!({"path": out, "page": page, "scale": scale}),
        )
    }

    /// Save or export by the extension of `out` (docx, pdf, odt, rtf, html, md, txt…).
    pub fn save_as(&mut self, out: &Path) -> Result<(), String> {
        self.run("file.save", &json!({"path": out})).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use wordcraft_engine::{Region, blank_document};

    #[test]
    fn chinese_regions_get_chinese_blank_documents_and_english_ones_keep_wordcraft_s() {
        super::init();
        let zh = blank_document(&Region::from_tag("zh-CN"));
        assert_eq!(zh.styles.default_chr.font_ea.as_deref(), Some("宋体"));
        assert_eq!(zh.styles.default_chr.size, Some(10.5));
        assert_eq!(
            zh.styles.default_chr.lang.as_deref(),
            Some("en-US"),
            "English text keeps English proofing"
        );
        assert!((zh.last_section.page_w - 595.3).abs() < 1.0, "A4");
        assert!(zh.last_section.doc_grid.is_some());
        // WPS's own blank document: Calibri for Latin text, compression on, mode 14.
        assert_eq!(zh.styles.default_chr.font.as_deref(), Some("Calibri"));
        assert_eq!(
            zh.settings.char_spacing,
            wordcraft_doc::CharSpacing::CompressPunctuation
        );
        assert_eq!(zh.settings.compat_mode, 14);
        assert!(
            (zh.last_section.header - 42.55).abs() < 0.01
                && (zh.last_section.footer - 49.6).abs() < 0.01
        );
        let tw = blank_document(&Region::from_tag("zh-TW"));
        assert_eq!(tw.styles.default_chr.font_ea.as_deref(), Some("新細明體"));
        // English stays a first-class function: an English region's blank document is WordCraft's.
        let en = blank_document(&Region::from_tag("en-US"));
        let plain = wordcraft_doc::Document::new();
        assert_eq!(en.styles.default_chr.font, plain.styles.default_chr.font);
        assert_eq!(en.styles.default_chr.size, plain.styles.default_chr.size);
        assert_eq!(en.last_section.doc_grid, plain.last_section.doc_grid);
    }

    /// A missing font Wenshu draws as a Windows Chinese face keeps that face's line box, not its
    /// substitute's (Linux: 新細明體 as 宋体 there is Noto Serif CJK, whose line is far taller).
    #[test]
    fn a_missing_font_drawn_as_simsun_keeps_simsuns_line() {
        super::init();
        let mut r = wordcraft_fonts::word::resolve("Arial", false, false);
        r.substituted = true;
        let simsun = Some((258.0 / 256.0, 74.0 / 256.0));
        for name in ["新細明體", "华文宋体", "方正黑体", "华文楷体"] {
            assert_eq!(
                wordcraft_fonts::word::missing_font_line(name, &r),
                simsun,
                "{name}"
            );
        }
        assert_eq!(
            wordcraft_fonts::word::missing_font_line("No Such Font", &r),
            None
        );
    }

    #[test]
    fn init_is_idempotent() {
        super::init();
        super::init();
        assert!(!wordcraft_fonts::word::set_substitute_provider(
            wenshu_cjk::fonts::substitutes
        ));
    }
}
