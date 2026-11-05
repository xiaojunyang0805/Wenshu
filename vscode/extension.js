// Wenshu Document Preview: a read-only editor that shows a document's pages as Wenshu (文枢)
// lays them out. The pages are drawn by the `wenshu` command-line tool
// (`wenshu render <file> <dir> --all`, one PNG per page, one JSON line per page as it is done),
// so the preview is what the Wenshu desktop app shows and prints.
'use strict';

const vscode = require('vscode');
const cp = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

const t = (s, ...args) => vscode.l10n.t(s, ...args);
const exe = (name) => (process.platform === 'win32' ? `${name}.exe` : name);

/** The `wenshu` CLI: the setting, the installed Wenshu, then `PATH`. */
function findCli() {
  const configured = vscode.workspace.getConfiguration('wenshu').get('cliPath', '').trim();
  if (configured) {
    return fs.existsSync(configured) ? configured : null;
  }
  const candidates = [];
  if (process.platform === 'win32') {
    const local = process.env.LOCALAPPDATA;
    if (local) candidates.push(path.join(local, 'Programs', 'Wenshu', 'wenshu.exe'));
  } else {
    candidates.push('/usr/bin/wenshu', '/usr/local/bin/wenshu', path.join(os.homedir(), '.local', 'bin', 'wenshu'));
  }
  for (const dir of (process.env.PATH || '').split(path.delimiter)) {
    if (dir) candidates.push(path.join(dir, exe('wenshu')));
  }
  return candidates.find((c) => {
    try {
      return fs.statSync(c).isFile();
    } catch {
      return false;
    }
  }) || null;
}

class Preview {
  constructor(context, uri, panel) {
    this.context = context;
    this.uri = uri;
    this.panel = panel;
    this.proc = null;
    this.generation = 0;
    this.root = vscode.Uri.joinPath(context.globalStorageUri, 'renders');
    this.dir = null;
    this.disposables = [];

    panel.webview.options = { enableScripts: true, localResourceRoots: [] };
    panel.webview.html = this.html();
    this.disposables.push(
      panel.webview.onDidReceiveMessage((m) => this.onMessage(m)),
      panel.onDidDispose(() => this.dispose()),
    );
    // Re-render when the file changes on disk (saved from Wenshu, WPS or Word).
    const watcher = vscode.workspace.createFileSystemWatcher(
      new vscode.RelativePattern(vscode.Uri.joinPath(uri, '..'), path.basename(uri.fsPath)),
    );
    let timer = null;
    const later = () => {
      clearTimeout(timer);
      timer = setTimeout(() => this.render(), 600);
    };
    this.disposables.push(watcher, watcher.onDidChange(later), watcher.onDidCreate(later));
  }

  html() {
    const csp = [
      "default-src 'none'",
      // Pages arrive as bytes and are shown through blob: URLs: the webview resource loader
      // (asWebviewUri + localResourceRoots) left them broken on some Windows installs.
      'img-src blob:',
      "style-src 'unsafe-inline'",
      `script-src 'nonce-${this.nonce()}'`,
    ].join('; ');
    const strings = {
      rendering: t('Laying out…'),
      pages: t('{0} pages'),
      page: t('1 page'),
      zoomIn: t('Zoom in'),
      zoomOut: t('Zoom out'),
      fit: t('Fit width'),
      refresh: t('Refresh'),
      openDesktop: t('Open in Wenshu'),
      settings: t('Settings'),
    };
    return `<!DOCTYPE html>
<html lang="${vscode.env.language}">
<head>
<meta charset="UTF-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  :root { --zoom: 1; }
  body { margin: 0; padding: 0; background: var(--vscode-editorWidget-background, #e6e6e6); color: var(--vscode-foreground); font-family: var(--vscode-font-family); }
  #bar { position: sticky; top: 0; z-index: 1; display: flex; gap: 6px; align-items: center; padding: 6px 12px;
         background: var(--vscode-editor-background); border-bottom: 1px solid var(--vscode-panel-border, #8884); font-size: 12px; }
  #bar button { background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground);
                border: none; padding: 3px 8px; border-radius: 3px; cursor: pointer; font: inherit; }
  #bar button:hover { background: var(--vscode-button-secondaryHoverBackground); }
  #status { flex: 1; opacity: .85; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  #pages { display: flex; flex-direction: column; align-items: center; gap: 16px; padding: 16px; }
  #pages img { display: block; background: #fff; box-shadow: 0 1px 4px #0005; max-width: none; }
  body.fit #pages img { width: min(100%, calc(var(--w) * 1px)); height: auto; }
  body:not(.fit) #pages img { width: calc(var(--w) * var(--zoom) * 1px); }
  #error { display: none; margin: 24px; padding: 12px 16px; border-left: 3px solid var(--vscode-errorForeground);
           background: var(--vscode-inputValidation-errorBackground, #f003); white-space: pre-wrap; }
</style>
</head>
<body class="fit">
<div id="bar">
  <span id="status">${strings.rendering}</span>
  <button id="out" title="${strings.zoomOut}">−</button>
  <button id="fit" title="${strings.fit}">⇔</button>
  <button id="in" title="${strings.zoomIn}">+</button>
  <button id="refresh" title="${strings.refresh}">⟳</button>
  <button id="desktop" title="${strings.openDesktop}">${strings.openDesktop}</button>
</div>
<div id="error"></div>
<div id="pages"></div>
<script nonce="${this.nonce()}">
  const vscode = acquireVsCodeApi();
  const S = ${JSON.stringify(strings)};
  const pages = document.getElementById('pages');
  const status = document.getElementById('status');
  const error = document.getElementById('error');
  let zoom = 1, count = 0, gen = -1;
  const setZoom = (z) => { zoom = Math.min(4, Math.max(0.25, z)); document.body.classList.remove('fit');
                           document.documentElement.style.setProperty('--zoom', zoom); };
  document.getElementById('in').onclick = () => setZoom(document.body.classList.contains('fit') ? 1.25 : zoom * 1.25);
  document.getElementById('out').onclick = () => setZoom(document.body.classList.contains('fit') ? 0.8 : zoom / 1.25);
  document.getElementById('fit').onclick = () => document.body.classList.add('fit');
  document.getElementById('refresh').onclick = () => vscode.postMessage({ type: 'refresh' });
  document.getElementById('desktop').onclick = () => vscode.postMessage({ type: 'openDesktop' });
  const clear = () => {
    for (const img of pages.querySelectorAll('img')) URL.revokeObjectURL(img.src);
    pages.textContent = ''; delete pages.dataset.stale;
  };
  const label = (n) => n === 1 ? S.page : S.pages.replace('{0}', n);
  window.addEventListener('message', ({ data: m }) => {
    if (m.type === 'start') {
      gen = m.generation; count = 0; error.style.display = 'none';
      status.textContent = S.rendering;
      // Keep the old pages until new ones arrive, so a refresh doesn't flash.
      pages.dataset.stale = '1';
    } else if (m.type === 'page' && m.generation === gen) {
      if (pages.dataset.stale) clear();
      const img = document.createElement('img');
      img.src = URL.createObjectURL(new Blob([m.data], { type: 'image/png' })); img.alt = 'page ' + m.page; img.style.setProperty('--w', m.width / m.scale);
      pages.appendChild(img);
      count = m.page;
      status.textContent = S.rendering + ' ' + count;
    } else if (m.type === 'done' && m.generation === gen) {
      if (pages.dataset.stale) clear();
      status.textContent = label(m.pages) + (m.note ? ' · ' + m.note : '');
    } else if (m.type === 'error') {
      status.textContent = '';
      error.textContent = m.text; error.style.display = 'block';
    }
  });
  vscode.postMessage({ type: 'ready' });
</script>
</body>
</html>`;
  }

  nonce() {
    if (!this._nonce) this._nonce = [...Array(32)].map(() => Math.floor(Math.random() * 36).toString(36)).join('');
    return this._nonce;
  }

  post(m) {
    this.panel.webview.postMessage(m);
  }

  onMessage(m) {
    if (m.type === 'ready' || m.type === 'refresh') this.render();
    else if (m.type === 'openDesktop') this.openDesktop();
  }

  openDesktop() {
    const cli = findCli();
    const desktop = cli && path.join(path.dirname(cli), exe('wenshu-desktop'));
    if (!desktop || !fs.existsSync(desktop)) {
      vscode.window.showWarningMessage(t('The Wenshu desktop app was not found next to {0}.', cli || 'wenshu'));
      return;
    }
    const child = cp.spawn(desktop, [this.uri.fsPath], { detached: true, stdio: 'ignore' });
    child.on('error', (e) => vscode.window.showErrorMessage(String(e)));
    child.unref();
  }

  stop() {
    if (this.proc) {
      this.proc.kill();
      this.proc = null;
    }
  }

  async newDir() {
    const old = this.dir;
    this.dir = vscode.Uri.joinPath(this.root, `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`);
    await vscode.workspace.fs.createDirectory(this.dir);
    // The previous render's images stay until the new ones replace them in the view.
    if (old) setTimeout(() => vscode.workspace.fs.delete(old, { recursive: true }).then(undefined, () => {}), 10000);
  }

  async render() {
    if (this.uri.scheme !== 'file') {
      this.post({ type: 'error', text: t('Only files on disk can be previewed.') });
      return;
    }
    const cli = findCli();
    if (!cli) {
      this.post({
        type: 'error',
        text: t(
          'The Wenshu command-line tool (wenshu) was not found.\n\nInstall Wenshu (https://github.com/xiaojunyang0805/Wenshu/releases) or set "wenshu.cliPath" in the settings.',
        ),
      });
      const choice = await vscode.window.showWarningMessage(t('Wenshu was not found.'), t('Settings'));
      if (choice) vscode.commands.executeCommand('workbench.action.openSettings', 'wenshu.cliPath');
      return;
    }
    this.stop();
    const generation = ++this.generation;
    await this.newDir();
    const scale = vscode.workspace.getConfiguration('wenshu').get('scale', 1.5);
    this.post({ type: 'start', generation });
    // Pages are read in order (a chain), so they reach the view in order.
    let chain = Promise.resolve();
    const page = (r) => {
      chain = chain.then(async () => {
        if (generation !== this.generation) return;
        let bytes;
        try {
          bytes = await fs.promises.readFile(r.path);
        } catch (e) {
          this.post({ type: 'error', text: String(e) });
          return;
        }
        if (generation !== this.generation) return;
        const data = new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        this.post({ type: 'page', generation, page: r.page, data, width: r.width, scale });
      });
    };
    const started = Date.now();
    const ok = await this.renderAll(cli, scale, generation, page);
    await chain;
    if (generation !== this.generation) return;
    if (ok !== null) {
      const secs = ((Date.now() - started) / 1000).toFixed(1);
      this.post({ type: 'done', generation, pages: ok, note: t('laid out by Wenshu in {0} s', secs) });
    }
  }

  /** `wenshu render --all`; an older CLI without `--all` renders page by page. */
  renderAll(cli, scale, generation, onPage) {
    return new Promise((resolve) => {
      const args = ['render', this.uri.fsPath, this.dir.fsPath, '--all', '--scale', String(scale)];
      const proc = cp.spawn(cli, args, { windowsHide: true });
      this.proc = proc;
      let buf = '';
      let err = '';
      let seen = 0;
      let total = null;
      proc.stdout.setEncoding('utf8');
      proc.stdout.on('data', (d) => {
        buf += d;
        let i;
        while ((i = buf.indexOf('\n')) >= 0) {
          const line = buf.slice(0, i).trim();
          buf = buf.slice(i + 1);
          if (!line) continue;
          try {
            const r = JSON.parse(line);
            if (generation !== this.generation) continue;
            if (typeof r.page === 'number' && r.path) {
              seen++;
              onPage(r);
            } else if (typeof r.pages === 'number') {
              total = r.pages;
            }
          } catch {
            // not a JSON line: ignore
          }
        }
      });
      proc.stderr.setEncoding('utf8');
      proc.stderr.on('data', (d) => (err += d));
      proc.on('error', (e) => {
        this.post({ type: 'error', text: `${cli}: ${e.message}` });
        resolve(null);
      });
      proc.on('close', async (code) => {
        if (this.proc === proc) this.proc = null;
        if (generation !== this.generation) return resolve(null);
        if (code === 0 && total !== null) return resolve(total);
        if (seen === 0 && /usage|missing argument|invalid|\.png/i.test(err)) {
          // An older CLI: no `--all`. Count the pages, then render them one at a time.
          return resolve(await this.renderEach(cli, scale, generation, onPage));
        }
        this.post({ type: 'error', text: t('Wenshu could not lay out this document.') + `\n\n${err.trim() || `exit ${code}`}` });
        resolve(null);
      });
    });
  }

  run(cli, args) {
    return new Promise((resolve) => {
      cp.execFile(cli, args, { windowsHide: true, maxBuffer: 16 * 1024 * 1024 }, (e, stdout, stderr) =>
        resolve({ ok: !e, stdout, stderr: stderr || (e && e.message) || '' }),
      );
    });
  }

  async renderEach(cli, scale, generation, onPage) {
    const info = await this.run(cli, ['info', this.uri.fsPath]);
    let pages = 0;
    try {
      pages = JSON.parse(info.stdout).pages;
    } catch {
      this.post({ type: 'error', text: t('Wenshu could not lay out this document.') + `\n\n${info.stderr.trim()}` });
      return null;
    }
    for (let n = 1; n <= pages; n++) {
      if (generation !== this.generation) return null;
      const out = path.join(this.dir.fsPath, `page-${n}.png`);
      const r = await this.run(cli, ['render', this.uri.fsPath, out, '--page', String(n), '--scale', String(scale)]);
      if (!r.ok) {
        this.post({ type: 'error', text: r.stderr.trim() });
        return null;
      }
      let width = 0;
      try {
        width = JSON.parse(r.stdout).width;
      } catch {
        // keep 0: the view fits it
      }
      onPage({ page: n, path: out, width });
    }
    return pages;
  }

  dispose() {
    this.stop();
    this.generation++;
    for (const d of this.disposables) d.dispose();
    if (this.dir) vscode.workspace.fs.delete(this.dir, { recursive: true }).then(undefined, () => {});
    Preview.open.delete(this);
  }
}
Preview.open = new Set();

class Provider {
  constructor(context) {
    this.context = context;
  }
  openCustomDocument(uri) {
    return { uri, dispose() {} };
  }
  resolveCustomEditor(document, panel) {
    const p = new Preview(this.context, document.uri, panel);
    Preview.open.add(p);
    this.context.subscriptions.push({ dispose: () => p.dispose() });
  }
}

function activate(context) {
  // Renders left behind by a session that ended abruptly (older than a day: another window may
  // be showing newer ones).
  const root = path.join(context.globalStorageUri.fsPath, 'renders');
  fs.readdir(root, { withFileTypes: true }, (e, entries) => {
    if (e) return;
    for (const d of entries) {
      const p = path.join(root, d.name);
      fs.stat(p, (err, st) => {
        if (!err && Date.now() - st.mtimeMs > 24 * 3600 * 1000) fs.rm(p, { recursive: true, force: true }, () => {});
      });
    }
  });
  const provider = new Provider(context);
  const options = { webviewOptions: { retainContextWhenHidden: true }, supportsMultipleEditorsPerDocument: true };
  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider('wenshu.preview', provider, options),
    vscode.window.registerCustomEditorProvider('wenshu.previewText', provider, options),
    vscode.commands.registerCommand('wenshu.openPreview', (uri) => {
      const target = uri || (vscode.window.activeTextEditor && vscode.window.activeTextEditor.document.uri);
      if (!target) return;
      const view = /\.rtf$/i.test(target.fsPath) ? 'wenshu.previewText' : 'wenshu.preview';
      vscode.commands.executeCommand('vscode.openWith', target, view);
    }),
    vscode.commands.registerCommand('wenshu.refresh', () => {
      for (const p of Preview.open) if (p.panel.active) p.render();
    }),
  );
}

function deactivate() {}

module.exports = { activate, deactivate, findCli };
