# 文枢网页版 · Wenshu in the browser

这是文枢的自托管网页版：一个静态网站，放在任意 Web 服务器上即可使用，内网也可以。文档只在浏览器中打开、编辑和导出，不会上传到服务器或任何其他地方。

This is Wenshu's self-hostable browser app: a static site that runs from any web server, an intranet one included. Documents are opened, edited and exported in the browser; they are never uploaded to the server or anywhere else.

## 部署 · Hosting

把整个文件夹复制到 Web 服务器的一个目录下（例如 `/var/www/wenshu`），用浏览器打开该目录的地址。

Copy the whole folder into a directory your web server serves (for example `/var/www/wenshu`) and open that address in a browser.

- 服务器需以 `application/wasm` 类型发送 `.wasm` 文件（nginx、Apache、IIS 和 Caddy 的新版本默认如此）。
  The server must send `.wasm` files as `application/wasm` (current nginx, Apache, IIS and Caddy do by default).
- 请为 `.wasm` 和 `.js` 开启 gzip 或 brotli 压缩：程序文件约 34 MB，压缩后约 11–14 MB。
  Turn on gzip or brotli compression for `.wasm` and `.js`: the app is about 34 MB, 11–14 MB compressed.
- 字体在 `fonts/` 中，打开后在后台按需加载（WOFF2，约 30 MB）。请允许浏览器缓存它们（例如 `Cache-Control: max-age=2592000`），以后打开会更快。
  Fonts are in `fonts/` and load in the background after the app opens (WOFF2, about 30 MB). Let browsers cache them (for example `Cache-Control: max-age=2592000`) so later visits start faster.
- 不能直接双击 `index.html` 打开（浏览器禁止从本地文件加载 WebAssembly 和字体）。本机试用可以运行 `python -m http.server 8000`，再打开 <http://localhost:8000>。
  Opening `index.html` from disk doesn't work (browsers block WebAssembly and font loading from local files). To try it on one machine, run `python -m http.server 8000` in this folder and open <http://localhost:8000>.

nginx 示例 · nginx example:

```nginx
location /wenshu/ {
    alias /var/www/wenshu/;
    types { application/wasm wasm; font/woff2 woff2; }
    gzip on; gzip_types application/wasm application/javascript;
    location ~* \.(wasm|js|woff2)$ { add_header Cache-Control "public, max-age=2592000"; }
}
```

## 浏览器 · Browsers

需要支持 WebGPU 或 WebGL2 的浏览器：Chrome、Edge 和基于 Chromium 的国产浏览器（360、QQ 浏览器等）的新版本，以及 Firefox 和 Safari。

A browser with WebGPU or WebGL2: current Chrome, Edge and Chromium-based browsers (360, QQ Browser…), Firefox and Safari.

## 字体 · Fonts

网页版看不到电脑上安装的字体，因此自带开源字体：朱雀仿宋（仿宋）、霞鹜文楷 GB（楷体）、Noto Serif SC（宋体），以及与 Times New Roman、Arial、Calibri、Cambria 等宽的 Liberation、Carlito、Caladea。也可以在“文件 › 选项”中添加自己的字体；在 Chrome 和 Edge 中还可以允许使用本机已安装的字体。

The browser app can't see the computer's installed fonts, so it brings open ones: 朱雀仿宋 (FangSong), 霞鹜文楷 GB (Kai), Noto Serif SC (Song), and Liberation, Carlito and Caladea, metric-compatible with Times New Roman, Arial, Calibri and Cambria. You can add your own fonts in File › Options, and in Chrome and Edge allow the computer's installed fonts.

## 许可 · Licences

文枢以 Apache-2.0 或 MIT 许可发布，基于 WordCraft（MIT 或 Apache-2.0）。字体均为 SIL Open Font License 1.1。各许可证见 `licenses/`。

Wenshu is Apache-2.0 OR MIT, built on WordCraft (MIT OR Apache-2.0). The fonts are under the SIL Open Font License 1.1. The licences are in `licenses/`.
