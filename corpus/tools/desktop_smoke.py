"""Smoke-test wenshu-desktop on corpus documents through the control channel: open several in one
session (embedded fonts must not leak between them), compare page counts with the CLI's, type,
save as .docx, reopen, screenshot. python -I -X utf8 out/drive_smoke.py"""
import json, os, shutil, socket, subprocess, sys, time, pathlib

root = pathlib.Path(r"D:\27_Project\Wenshu")
prof = root / "out" / "smoke-profile"
shots = root / "out" / "smoke"
shutil.rmtree(prof, ignore_errors=True)
shutil.rmtree(shots, ignore_errors=True)
shots.mkdir(parents=True)
port = 7991
env = {k: v for k, v in os.environ.items() if not k.startswith(("LANG", "LC_"))}
env["WORDCRAFT_PROFILE_DIR"] = str(prof)
exe = root / "eval/target/wordcraft/release/wenshu-desktop.exe"
cli = root / "eval/target/wordcraft/release/wenshu.exe"
p = subprocess.Popen([str(exe), "--control", str(port)], env=env)
key = None
for _ in range(120):
    ks = list(prof.glob("control-key*")) if prof.exists() else []
    if ks:
        key = ks[0].read_text().strip()
        break
    time.sleep(0.5)
if not key:
    print("no key file"); p.kill(); sys.exit(1)
s = socket.create_connection(("127.0.0.1", port), timeout=120)
f = s.makefile("rw", encoding="utf-8")
n = 0


def call(method, params=None):
    global n
    n += 1
    f.write(json.dumps({"id": n, "key": key, "method": method, "params": params or {}}) + "\n")
    f.flush()
    return json.loads(f.readline())


def ex(command, params=None, doc=None):
    q = {"command": command, "params": params or {}}
    if doc is not None:
        q["doc"] = doc
    return call("engine.execute", q)


def cli_pages(path):
    out = subprocess.run([str(cli), "info", str(path)], capture_output=True, text=True, encoding="utf-8").stdout
    try:
        return json.loads(out)["pages"]
    except Exception:
        return None


def pages_of(r):
    res = r.get("result") or {}
    for k in ("pages", "pageCount"):
        if isinstance(res, dict) and k in res:
            return res[k]
    return None


time.sleep(2)
problems = []
docs = ["gov-fj-gaoxin", "gov-mct-keji", "dc-policies-b6747d9df3", "dc-legal-ce628d9760", "dc-educational-b8de3ae296"]
for i, name in enumerate(docs):
    src = root / "corpus" / "files" / f"{name}.docx"
    r = ex("file.open", {"path": str(src)})
    if r.get("error"):
        problems.append(f"{name}: open failed {r['error']}"); print(name, "open error", r["error"]); continue
    doc = (r.get("result") or {}).get("id")
    time.sleep(2)
    info = ex("document.inspect", {}, doc)
    app_pages = pages_of(info)
    want = cli_pages(src)
    rr = call("ui.render", {"path": str(shots / f"{i}-{name}.png"), "page": 1, "scale": 1, "doc": doc})
    if rr.get("error"):
        print("render error", rr["error"])
    print(f"{name}: app pages {app_pages}, cli pages {want}", "" if app_pages == want else "  <-- differs")
    if app_pages != want:
        problems.append(f"{name}: app {app_pages} vs cli {want}")

# Edit, save as .docx, reopen (the last document opened).
name = docs[2]
src = root / "corpus" / "files" / f"{name}.docx"
r = ex("view.switchWindows", {})
ids = {d.get("path", "").lower(): d["id"] for d in (r.get("result") or {}).get("documents", [])}
doc = next((v for k, v in ids.items() if name.lower() in k), None)
print("edit doc id", doc)
ex("caret.documentEnd", {}, doc)
print("type", ex("text.insert", {"text": "测试文字，保存后重开。"}, doc).get("error"))
saved = shots / f"{name}-saved.docx"
print("saveAs", ex("file.saveAs", {"path": str(saved)}, doc).get("error"))
time.sleep(1)
if saved.exists():
    print("saved size", saved.stat().st_size, "cli pages after save", cli_pages(saved))
    r = ex("file.open", {"path": str(saved)})
    print("reopen", r.get("error") or (r.get("result") or {}).get("id"))
    time.sleep(2)
    print("shot", call("ui.screenshot", {"path": str(shots / "9-window.png")}).get("error"))
else:
    problems.append("save produced no file")

r = ex("view.switchWindows", {})
print("open documents:", len((r.get("result") or {}).get("documents", [])))
call("app.quit", {"force": True})
try:
    p.wait(20)
except Exception:
    p.kill()
print("exit", p.returncode)
print("PROBLEMS:", problems or "none")
