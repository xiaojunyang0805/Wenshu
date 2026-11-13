//! 公文 templates in File › New › Personal: on first start, one `.dotx` per common kind goes into
//! `<settings>/Templates/公文` (the folder name is the category New shows). Built with
//! `wenshu-gongwen`, so they pass `wenshu gongwen check`, and saved through the engine so they are
//! real Word templates. A `公文` folder that exists is left alone: the user may have changed or
//! removed templates in it.

use std::path::Path;

/// The category folder's name.
const CATEGORY: &str = "公文";

/// Seed the templates folder if it has no 公文 category yet. Failures are logged, never fatal.
pub fn seed() {
    let Some(dir) =
        wordcraft_control_key::settings_dir().map(|d| d.join("Templates").join(CATEGORY))
    else {
        return;
    };
    if dir.exists() {
        return;
    }
    match write_all(&dir) {
        Ok(n) => log::info!("公文 templates: {n} written to {}", dir.display()),
        Err(e) => log::warn!("公文 templates: {e}"),
    }
}

/// The templates, by file name (without `.dotx`): shared with the web app.
pub use wenshu_gongwen::templates::kinds;

/// Write every template into `dir`; the number written.
fn write_all(dir: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let tmp = std::env::temp_dir().join(format!("wenshu-gongwen-{}.docx", std::process::id()));
    let mut n = 0;
    for (name, g) in kinds() {
        let written = (|| {
            let file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
            wenshu_gongwen::write_docx(&g, file)?;
            let mut doc = wenshu_core::Doc::open(&tmp, None)?;
            doc.save_as(&dir.join(format!("{name}.dotx")))
        })();
        match written {
            Ok(()) => n += 1,
            Err(e) => log::warn!("公文 template {name}: {e}"),
        }
    }
    let _ = std::fs::remove_file(&tmp);
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every template is a 公文 that passes the format checker, and saves as a real template.
    #[test]
    fn templates_pass_the_checker_and_save_as_dotx() {
        let dir =
            std::env::temp_dir().join(format!("wenshu-templates-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(write_all(&dir).unwrap(), kinds().len());
        for (name, _) in kinds() {
            let path = dir.join(format!("{name}.dotx"));
            let findings = wenshu_gongwen::check_docx(&path).unwrap();
            let failed: Vec<_> = findings.iter().filter(|f| !f.ok).collect();
            assert!(failed.is_empty(), "{name}: {failed:?}");
            assert!(
                wenshu_core::Doc::open(&path, None)
                    .unwrap()
                    .pages()
                    .unwrap()
                    >= 1,
                "{name}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
