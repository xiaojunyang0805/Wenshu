"""Download the corpus documents that are not committed (tier `dc`), checking each SHA-256.

    python corpus/tools/fetch.py

Reads corpus/manifest.csv; skips files already present with the right hash. A file whose hash no
longer matches (the source changed or was taken down) is reported and left out, never replaced.
"""
import csv
import hashlib
import sys
import time
from pathlib import Path

import requests

ROOT = Path(__file__).resolve().parent.parent


def main():
    rows = list(csv.DictReader(open(ROOT / "manifest.csv", encoding="utf-8")))
    s = requests.Session()
    got = bad = have = 0
    for r in rows:
        if r["tier"] != "dc" or not r["url"]:
            continue
        path = ROOT / "files" / r["file"]
        if path.exists() and hashlib.sha256(path.read_bytes()).hexdigest() == r["sha256"]:
            have += 1
            continue
        try:
            resp = s.get(r["url"], timeout=60)
            data = resp.content if resp.status_code == 200 else b""
        except requests.RequestException:
            data = b""
        if hashlib.sha256(data).hexdigest() != r["sha256"]:
            print(f"missing or changed: {r['file']} ({r['url']})", file=sys.stderr)
            bad += 1
            continue
        path.write_bytes(data)
        got += 1
        time.sleep(0.1)
    print(f"{have} present, {got} downloaded, {bad} unavailable")


if __name__ == "__main__":
    main()
