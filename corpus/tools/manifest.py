"""Assemble corpus/files and write corpus/manifest.csv.

    python corpus/tools/manifest.py --picked out/corpus-pool/picked.jsonl --pool out/corpus-pool \
        --gov out/gov --gen out/probes out/gongwen

Copies each source document into corpus/files under its corpus name, then writes one manifest row
per document in corpus/files (sorted by name): where it came from, its SHA-256, the licence basis,
and the survey's app, pages, type and tags. Re-running is idempotent. `fetch.py` restores the
files that are not committed (tier `dc`) from the manifest.
"""
import argparse
import csv
import hashlib
import json
import shutil
from pathlib import Path

import survey

ROOT = Path(__file__).resolve().parent.parent
FILES = ROOT / "files"
DC_URL = "https://docxcorp.us/documents/{}.docx"
LICENCE = {
    "dc": "third-party, copyright of its authors; local only, fetched by URL (docx-corpus, Common Crawl)",
    "gov": "PRC Copyright Law art. 5 (state organ documents of an administrative nature)",
    "gen": "Wenshu's own (Apache-2.0 OR MIT)",
}
FIELDS = ["file", "tier", "source", "url", "sha256", "licence", "app", "app_name", "pages", "type", "cjk_share", "tags"]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tier_of(name):
    return "dc" if name.startswith("dc-") else "gov" if name.startswith("gov-") else "gen"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--picked")
    ap.add_argument("--pool")
    ap.add_argument("--gov")
    ap.add_argument("--gen", nargs="*", default=[])
    a = ap.parse_args()
    FILES.mkdir(exist_ok=True)
    meta = {}
    if a.picked:
        for line in open(a.picked, encoding="utf-8"):
            r = json.loads(line)
            doc_id = r["file"].removesuffix(".docx")
            name = f"dc-{r['type']}-{doc_id[:10]}.docx"
            shutil.copyfile(Path(a.pool) / "files" / r["file"], FILES / name)
            meta[name] = {"source": "docx-corpus", "url": DC_URL.format(doc_id), "type": r["type"]}
    gov_urls = {}
    for line in open(ROOT / "sources" / "gov.tsv", encoding="utf-8"):
        if line.strip() and not line.startswith("#"):
            slug, url, what = line.rstrip("\n").split("\t")
            gov_urls[f"gov-{slug}.docx"] = (url, what)
    if a.gov:
        for p in sorted(Path(a.gov).glob("gov-*.docx")):
            shutil.copyfile(p, FILES / p.name)
    for d in a.gen:
        for p in sorted(Path(d).glob("gen-*.docx")):
            shutil.copyfile(p, FILES / p.name)
    old = {}
    manifest = ROOT / "manifest.csv"
    if manifest.exists():
        old = {row["file"]: row for row in csv.DictReader(open(manifest, encoding="utf-8"))}
    rows = []
    for p in sorted(FILES.glob("*.docx")):
        s = survey.survey(p)
        tier = tier_of(p.name)
        m = meta.get(p.name) or {k: v for k, v in old.get(p.name, {}).items() if k in ("source", "url", "type")}
        if tier == "gov":
            url, what = gov_urls.get(p.name, ("", ""))
            m = {"source": what, "url": url, "type": "government"}
        elif tier == "gen":
            m = {"source": "corpus/tools (generated)" if p.name.startswith("gen-") else "corpus/make_corpus.py", "url": "", "type": "generated"}
        feats = sorted(s.get("features", {}))
        grid = (s.get("grid") or [None])[0]
        tags = feats + [f"grid:{grid}", f"csc:{s.get('compression')}"] + [f"font:{f}" for f in s.get("font_families", [])]
        if s.get("gongwen_like"):
            tags.append("gongwen")
        rows.append({
            "file": p.name, "tier": tier, "source": m.get("source", ""), "url": m.get("url", ""),
            "sha256": sha(p), "licence": LICENCE[tier], "app": s.get("app", ""), "app_name": s.get("app_name", ""),
            "pages": s.get("app_pages") or "", "type": m.get("type", ""), "cjk_share": s.get("cjk_share", ""),
            "tags": " ".join(tags),
        })
    with open(manifest, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, FIELDS)
        w.writeheader()
        w.writerows(rows)
    by = {}
    for r in rows:
        by[r["tier"]] = by.get(r["tier"], 0) + 1
    print(f"{len(rows)} documents: {by}")


if __name__ == "__main__":
    main()
