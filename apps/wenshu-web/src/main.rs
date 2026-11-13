//! Wenshu in the browser: WordCraft's web front end presented as 文枢, with Wenshu's Chinese
//! defaults (document-font substitutes, Chinese blank documents, OFD export) installed before the
//! app starts, 检查公文格式 on the ribbon, and Wenshu's open fonts fetched from the server it is
//! hosted on ([`fonts`]).
//!
//! A self-hostable static site: `scripts/build-web.sh` builds it with trunk into `dist/web`
//! (the app, `fonts/` and the licences); any static web server can host that folder, on an
//! intranet too. Documents never leave the browser.

mod fonts;

const ABOUT: &str = "中文优先的开源文字处理软件，支持 GB/T 9704 公文格式。\n\
A Chinese-first, open-source word processor (Apache-2.0 OR MIT).\n\
Built on WordCraft (MIT OR Apache-2.0); not affiliated with Kingsoft or Microsoft.\n\
文档只在浏览器中处理，不会上传。Documents stay in your browser.";

/// Wenshu's configuration of the browser front end.
fn config() -> wordcraft_web::WebConfig {
    wordcraft_web::WebConfig {
        brand: wordcraft_web::Brand {
            name: "文枢",
            version: env!("CARGO_PKG_VERSION"),
            about: Some(ABOUT),
            community: false,
            updates: false,
        },
        on_start: Some(start),
        app_tools: Some(tools),
        on_ready: Some(fonts::fetch_all),
    }
}

/// Before the app starts: Wenshu's defaults, and the 公文 templates in File › New (the desktop
/// app seeds them as files; the browser has no folder for them).
fn start() {
    wenshu_core::init();
    let _ = wordcraft_engine::io::set_app_templates(
        wenshu_gongwen::templates::kinds()
            .into_iter()
            .map(|(name, _)| wordcraft_engine::io::AppTemplate {
                id: name,
                label: name,
                category: "公文",
                docx: wenshu_gongwen::templates::docx,
            })
            .collect(),
    );
}

/// Review › 文枢 › 检查公文格式: the document against GB/T 9704.
fn tools() -> Vec<wordcraft_web::AppTool> {
    vec![wordcraft_web::AppTool {
        id: "wenshu.gongwenCheck",
        label: "检查公文格式",
        icon: "checkErrors",
        run: Box::new(wenshu_gongwen::check::report),
    }]
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wordcraft_web::start(config());
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let _ = config;
    let _ = std::io::Write::write_fmt(
        &mut std::io::stderr(),
        format_args!("wenshu-web only runs in the browser: build it with scripts/build-web.sh\n"),
    );
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_config_is_wenshus() {
        let c = super::config();
        assert_eq!(c.brand.name, "文枢");
        assert!(!c.brand.community && !c.brand.updates);
        let tools = c.app_tools.map(|t| t()).unwrap_or_default();
        assert_eq!(tools.len(), 1);
        assert!((tools[0].run)(b"not a zip").is_err());
    }

    /// The 公文 templates open from File › New: `file.new` with a template's id.
    #[test]
    fn gongwen_templates_open_from_file_new() {
        super::start();
        let ids: Vec<_> = wordcraft_engine::io::app_templates()
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, ["通知", "请示", "报告", "函", "纪要"]);
        let mut s = wordcraft_engine::Session::new(wordcraft_doc::Document::new());
        s.run("file.new", &serde_json::json!({"template": "请示"}))
            .unwrap();
        let text: String = s
            .doc
            .body
            .iter()
            .filter_map(|b| b.as_para())
            .map(|p| p.plain_text())
            .collect();
        assert!(text.contains("关于××××的请示"), "{text}");
    }
}
