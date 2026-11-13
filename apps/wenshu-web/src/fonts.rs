//! Wenshu's open fonts in the browser: a browser build can't see the system's fonts, so the
//! server hosting it serves them beside the app (`fonts/`, written by `scripts/build-web.sh`).
//! `fonts/fonts.json` lists the files in the order they are wanted (公文 body FangSong first, the
//! large Kai design last); each is fetched, added as an app font and the page repainted, so the
//! document lays out again with it. A file that fails is logged and skipped: the document keeps
//! its fallback face.

/// The most files a manifest may list.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] // fetched only in the browser
const MAX_FILES: usize = 64;

/// The font files `json` (the manifest: `{"files": ["a.woff2", …]}`) lists, in order. Only plain
/// file names: a path, a parent reference or a URL is left out, so the app fetches only from its
/// own `fonts/` folder.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] // fetched only in the browser
pub fn manifest_files(json: &str) -> Vec<String> {
    #[derive(serde::Deserialize)]
    struct Manifest {
        files: Vec<String>,
    }
    let Ok(m) = serde_json::from_str::<Manifest>(json) else {
        return Vec::new();
    };
    m.files
        .into_iter()
        .filter(|f| {
            !f.is_empty()
                && f.len() <= 128
                && !f.starts_with('.')
                && f.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        })
        .take(MAX_FILES)
        .collect()
}

/// Fetch the manifest and then each font, adding them as they arrive (in the background).
#[cfg(target_arch = "wasm32")]
pub fn fetch_all(ctx: egui::Context) {
    wasm_bindgen_futures::spawn_local(async move {
        let Some(manifest) = fetch("fonts/fonts.json")
            .await
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
        else {
            log::warn!("文枢: no fonts/fonts.json beside the app; documents use fallback faces");
            return;
        };
        for file in manifest_files(&manifest) {
            match fetch(&format!("fonts/{file}")).await {
                Ok(bytes) if bytes.len() <= wordcraft_fonts::MAX_FONT_FILE_BYTES => {
                    let added = wordcraft_fonts::FontDb::global().add_app_fonts(vec![bytes]);
                    log::info!("文枢: font {file}: {added} faces");
                    ctx.request_repaint();
                }
                Ok(_) => log::warn!("文枢: font {file} is too large; skipped"),
                Err(e) => log::warn!("文枢: font {file}: {e}"),
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_all(_ctx: egui::Context) {}

/// GET `url` (relative to the page), its body.
#[cfg(target_arch = "wasm32")]
async fn fetch(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast as _;
    let window = web_sys::window().ok_or("no window")?;
    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: web_sys::Response = resp.dyn_into().map_err(|e| format!("{e:?}"))?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let buf =
        wasm_bindgen_futures::JsFuture::from(resp.array_buffer().map_err(|e| format!("{e:?}"))?)
            .await
            .map_err(|e| format!("{e:?}"))?;
    Ok(js_sys::Uint8Array::new(&buf).to_vec())
}

#[cfg(test)]
mod tests {
    use super::manifest_files;

    #[test]
    fn only_plain_file_names_in_order() {
        let m = r#"{"files": ["ZhuqueFangsong-Regular.woff2", "../secret", "https://x.example/f.ttf", "a/b.ttf", ".hidden", "", "Carlito-Bold.woff2"]}"#;
        assert_eq!(
            manifest_files(m),
            ["ZhuqueFangsong-Regular.woff2", "Carlito-Bold.woff2"]
        );
        assert!(manifest_files("not json").is_empty());
        let many = format!(
            r#"{{"files": [{}]}}"#,
            (0..100)
                .map(|i| format!("\"f{i}.ttf\""))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert_eq!(manifest_files(&many).len(), 64);
    }
}
