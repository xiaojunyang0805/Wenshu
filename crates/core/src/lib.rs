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

/// Install Wenshu's defaults into the engine: Chinese document-font substitutes. Idempotent;
/// [`Doc::open`] calls it.
pub fn init() {
    INIT.call_once(|| {
        let _ = wordcraft_fonts::word::set_substitute_provider(wenshu_cjk::fonts::substitutes);
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
    #[test]
    fn init_is_idempotent() {
        super::init();
        super::init();
        assert!(!wordcraft_fonts::word::set_substitute_provider(
            wenshu_cjk::fonts::substitutes
        ));
    }
}
