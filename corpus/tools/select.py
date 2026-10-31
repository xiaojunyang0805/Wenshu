"""Pick the real-world part of the corpus from a surveyed candidate pool.

    python corpus/tools/select.py survey.jsonl pool.tsv N > picked.jsonl

Greedy and deterministic: first every rare feature gets up to `RARE_MIN` documents, then the app
quotas are filled, each time taking the document that adds the most not-yet-covered tags
(features, grid type, compression mode, font families, page band, document type). Documents
must be Chinese (CJK share ≥ 0.3), at most `MAX_PAGES` long, and not textual duplicates.
"""
import json
import sys

MAX_PAGES = 40
RARE_MIN = 3
# Share of the pick per producing app: the files clerks receive come from WPS and Word.
APP_SHARE = {"wps": 0.45, "word": 0.45, "other": 0.04, "libreoffice": 0.02, "unknown": 0.04}
RARE = ["vertical", "columns", "math", "ruby", "kinsoku_off", "nested_tables", "toc", "footnotes",
        "charts", "tracked", "comments", "sdt", "textboxes", "floating", "fields"]


def band(pages):
    if not pages:
        return "pages?"
    return "p1" if pages == 1 else "p2-4" if pages <= 4 else "p5-10" if pages <= 10 else "p11+"


def tags(r, kind):
    t = {f"f:{k}" for k in r.get("features", {})}
    grid = (r.get("grid") or [None])[0]
    t.add(f"grid:{grid}")
    t.add(f"csc:{r.get('compression')}")
    t |= {f"font:{f}" for f in r.get("font_families", [])}
    t.add(band(r.get("app_pages")))
    t.add(f"type:{kind}")
    if r.get("gongwen_like"):
        t.add("gongwen")
    if (r.get("page") or [None, None, None])[2] == "landscape":
        t.add("landscape")
    return t


def main(survey, pool, n):
    kind = {}
    for line in open(pool, encoding="utf-8"):
        k, url = line.rstrip("\n").split("\t")
        kind[url.rsplit("/", 1)[1]] = k
    seen, cands = set(), []
    for line in open(survey, encoding="utf-8"):
        r = json.loads(line)
        if "error" in r or r.get("cjk_share", 0) < 0.3 or (r.get("app_pages") or 0) > MAX_PAGES:
            continue
        if r["text_sha"] in seen or r.get("chars", 0) < 50:
            continue
        seen.add(r["text_sha"])
        r["type"] = kind.get(r["file"], "unknown")
        r["tags"] = tags(r, r["type"])
        cands.append(r)
    cands.sort(key=lambda r: r["file"])
    picked, covered = [], {}

    def gain(r):
        return sum(1.0 / (1 + covered.get(t, 0)) for t in r["tags"])

    def take(r):
        picked.append(r)
        cands.remove(r)
        for t in r["tags"]:
            covered[t] = covered.get(t, 0) + 1

    for feat in RARE:
        have = [r for r in cands if f"f:{feat}" in r["tags"]]
        while covered.get(f"f:{feat}", 0) < RARE_MIN and have and len(picked) < n:
            best = max(have, key=gain)
            have.remove(best)
            take(best)
    for app, share in APP_SHARE.items():
        want = round(n * share) - sum(r["app"] == app for r in picked)
        for _ in range(max(0, want)):
            pool_app = [r for r in cands if r["app"] == app]
            if not pool_app or len(picked) >= n:
                break
            take(max(pool_app, key=gain))
    while len(picked) < n and cands:
        take(max(cands, key=gain))
    for r in picked:
        r["tags"] = sorted(r["tags"])
        print(json.dumps(r, ensure_ascii=False))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]))
