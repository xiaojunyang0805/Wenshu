//! Wenshu desktop: WordCraft's desktop front end presented as 文枢, with Wenshu's Chinese defaults
//! (document-font substitutes) installed before the window opens and 公文 templates in
//! File › New › Personal.
//!
//! This is the one crate that embeds WordCraft's UI shell (`wordcraft::run`, patch 0006); document
//! work still goes through `wenshu-core`. Settings, logs and personal templates live in a folder
//! of their own (`%APPDATA%\Wenshu`, `~/.config/wenshu`), apart from WordCraft's.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod check;
mod templates;

/// The window and taskbar icon (original artwork, drawn for Wenshu).
const ICON: &[u8] = include_bytes!("../assets/wenshu-256.png");

const ABOUT: &str = "中文优先的开源文字处理软件，支持 GB/T 9704 公文格式。\n\
A Chinese-first, open-source word processor (Apache-2.0 OR MIT).\n\
Built on WordCraft (MIT OR Apache-2.0); not affiliated with Kingsoft or Microsoft.";

fn main() {
    let config = wordcraft::AppConfig {
        brand: wordcraft::Brand {
            name: "文枢",
            version: env!("CARGO_PKG_VERSION"),
            about: Some(ABOUT),
            // Wenshu's own community and releases, not ArtCraft's Discord and WordCraft's updates.
            community: false,
            updates: false,
        },
        bin: "wenshu-desktop",
        settings_folder: "Wenshu",
        app_id: "io.github.xiaojunyang0805.wenshu",
        icon_png: Some(ICON),
        on_start: Some(start),
        // Wenshu has releases of its own; WordCraft's update check would offer WordCraft's.
        check_updates: false,
        app_tools: Some(check::tools),
    };
    if let Err(e) = wordcraft::run(config) {
        // Unlike `eprintln!`, never panics on a closed stderr.
        let _ = std::io::Write::write_fmt(
            &mut std::io::stderr(),
            format_args!("wenshu-desktop: {e}\n"),
        );
        std::process::exit(1);
    }
}

/// Before the window opens: Chinese document-font substitutes, and the 公文 templates.
fn start() {
    wenshu_core::init();
    templates::seed();
}
