# Wenshu Document Preview (文枢文档预览)

Preview Word, WPS and OpenDocument documents in VS Code, page by page, as
[Wenshu (文枢)](https://github.com/xiaojunyang0805/Wenshu) lays them out: Chinese typography,
GB/T 9704 official documents (公文), fonts and page breaks matched against WPS Office.

- Opens `.docx`, `.docm`, `.dotx`, `.doc` and `.odt` files in a preview tab (and `.rtf` through
  **Open With… › Wenshu Preview**, or **Open with Wenshu Preview** in the Explorer menu).
- Pages appear as they are laid out; zoom in and out or fit the width.
- Refreshes when the file changes on disk (saved from Wenshu, WPS or Word).
- **Open in Wenshu** opens the document in the Wenshu desktop app for editing.

在 VS Code 中逐页预览 Word、WPS 和 OpenDocument 文档，排版与文枢桌面程序一致；文件在磁盘上改动后自动刷新，
“在文枢中打开”可直接编辑。

## Requirements

The extension draws pages with the `wenshu` command-line tool, which comes with Wenshu:

- **Windows**: the Wenshu installer (MSI) puts it in `%LOCALAPPDATA%\Programs\Wenshu\wenshu.exe`;
  the portable zip has `wenshu.exe` next to `wenshu-desktop.exe`.
- **Linux**: the `.deb` puts it in `/usr/bin/wenshu`.

Downloads: <https://github.com/xiaojunyang0805/Wenshu/releases>. Elsewhere (the portable zip, a
build from source), set **Wenshu: Cli Path** (`wenshu.cliPath`) to the tool. Wenshu 0.1.4 and later
draw all pages in one run; older versions work too, a page at a time.

## Settings

| Setting | Default | |
|---|---|---|
| `wenshu.cliPath` | (empty) | Path to `wenshu`; empty: the installed Wenshu, then `PATH` |
| `wenshu.scale` | `1.5` | Page image scale (1 = 96 dpi): sharper or faster |

## Privacy

Everything stays on your machine: the document is laid out by the local `wenshu` tool, and the
page images are kept in the extension's storage folder until the preview closes.

## Licence

Apache-2.0 OR MIT, as Wenshu.
