//! Windows only: embed Wenshu's icon and version info into `wenshu-desktop.exe` (Explorer, the
//! taskbar, Alt-Tab). Elsewhere this does nothing; a missing resource compiler is a warning.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/wenshu.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/wenshu.ico")
        .set("ProductName", "文枢 Wenshu")
        .set("FileDescription", "文枢 Wenshu 文字处理")
        .set("LegalCopyright", "Copyright (c) the Wenshu authors. Apache-2.0 OR MIT. Built on WordCraft (MIT OR Apache-2.0).")
        .set("OriginalFilename", "wenshu-desktop.exe")
        .set("InternalName", "wenshu-desktop");
    if let Err(e) = res.compile() {
        println!("cargo:warning=wenshu-desktop.exe built without icon/version resources: {e}");
    }
}
