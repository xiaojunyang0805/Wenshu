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
        on_start: Some(wenshu_core::init),
        app_tools: Some(tools),
        on_ready: Some(fonts::fetch_all),
    }
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
}
