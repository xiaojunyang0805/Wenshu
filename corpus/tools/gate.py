"""Regression gate: the fidelity rescore against a committed baseline, run before each release on
Windows and on Linux, so page counts and page scores can't slip unnoticed.

    python -X utf8 corpus/tools/gate.py run    [--platform P] [--exe wenshu] [--out DIR]
    python -X utf8 corpus/tools/gate.py check  REPORT.json [--platform P]
    python -X utf8 corpus/tools/gate.py update REPORT.json [--platform P]

`run` renders the corpus with `wenshu fidelity` (each document in its own process) into
out/gate-<platform> and checks the report; `check` checks an existing report; `update` makes a
report the new baseline (corpus/baseline/<platform>.json) after a change that is meant to move the
numbers. The platform defaults to this machine's (windows or linux). The references are WPS's
renders in corpus/ref (local), so the gate runs where they are, not in CI. The baseline holds
numbers only (page counts and scores), no renders.

The gate fails (exit 1) when:
- a document fails to render;
- a document whose page count matched WPS's in the baseline no longer does;
- fewer page counts match than in the baseline;
- the mean score falls by more than MEAN_SLACK;
- a document's mean page score falls by more than DOC_SLACK.
Documents that got better are listed too: update the baseline to keep them.
"""
import argparse, json, os, platform, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BASE = os.path.join(ROOT, 'corpus', 'baseline')
MEAN_SLACK = 0.002
DOC_SLACK = 0.02


def this_platform():
    return 'windows' if os.name == 'nt' else 'linux'


def default_exe():
    """The release CLI in the shared target folder (.cargo/config.toml), else cargo's default."""
    name = 'wenshu.exe' if os.name == 'nt' else 'wenshu'
    for t in (os.path.join('eval', 'target', 'wordcraft'), 'target'):
        exe = os.path.join(ROOT, t, 'release', name)
        if os.path.exists(exe):
            return exe
    return name


def summary(report):
    """Per document: pages, WPS's pages, mean page score, error."""
    docs = {}
    for d in report['documents']:
        name = os.path.basename(d['file'].replace('\\', '/'))
        scores = [s['score'] for s in d['page_scores'] if s.get('score') is not None]
        docs[name] = {
            'pages': d['pages'],
            'ref': d['reference_pages'],
            'score': round(sum(scores) / len(scores), 4) if scores else None,
            'error': d['error'],
        }
    return {
        'mean_score': round(report['mean_score'], 4) if report['mean_score'] is not None else None,
        'page_count_matches': report['page_count_matches'],
        'page_count_compared': report['page_count_compared'],
        'documents': docs,
    }


def check(report_path, plat):
    with open(report_path, encoding='utf-8') as f:
        now = summary(json.load(f))
    base_path = os.path.join(BASE, f'{plat}.json')
    if not os.path.exists(base_path):
        print(f'no baseline {base_path}: run `gate.py update {report_path} --platform {plat}` first')
        return 1
    with open(base_path, encoding='utf-8') as f:
        base = json.load(f)
    fails, better = [], []
    for name, b in sorted(base['documents'].items()):
        n = now['documents'].get(name)
        if n is None:
            fails.append(f'{name}: missing from the report')
            continue
        if n['error']:
            fails.append(f"{name}: {n['error'][:120]}")
            continue
        was, isnow = b['pages'] == b['ref'], n['pages'] == n['ref']
        if was and not isnow:
            fails.append(f"{name}: pages {n['pages']} (WPS {n['ref']}), matched before")
        elif isnow and not was:
            better.append(f"{name}: pages now match WPS ({n['ref']}; was {b['pages']})")
        if b['score'] is not None and n['score'] is not None:
            if n['score'] < b['score'] - DOC_SLACK:
                fails.append(f"{name}: score {n['score']:.3f} (was {b['score']:.3f})")
            elif n['score'] > b['score'] + DOC_SLACK:
                better.append(f"{name}: score {n['score']:.3f} (was {b['score']:.3f})")
    for name in sorted(set(now['documents']) - set(base['documents'])):
        better.append(f'{name}: new in the corpus')
    if now['page_count_matches'] < base['page_count_matches']:
        fails.append(f"page counts {now['page_count_matches']}/{now['page_count_compared']} (baseline {base['page_count_matches']})")
    if base['mean_score'] is not None and (now['mean_score'] or 0) < base['mean_score'] - MEAN_SLACK:
        fails.append(f"mean score {now['mean_score']:.4f} (baseline {base['mean_score']:.4f})")
    print(f"{plat}: page counts {now['page_count_matches']}/{now['page_count_compared']} (baseline {base['page_count_matches']}), "
          f"mean score {now['mean_score']} (baseline {base['mean_score']})")
    for b in better:
        print('  better:', b)
    for f in fails:
        print('  FAIL:', f)
    if better and not fails:
        print('  (better than the baseline: `gate.py update` to keep it)')
    print('PASS' if not fails else f'FAIL ({len(fails)})')
    return 1 if fails else 0


def update(report_path, plat):
    with open(report_path, encoding='utf-8') as f:
        s = summary(json.load(f))
    s['platform'] = plat
    s['machine'] = platform.platform()
    os.makedirs(BASE, exist_ok=True)
    path = os.path.join(BASE, f'{plat}.json')
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        json.dump(s, f, ensure_ascii=False, indent=1, sort_keys=True)
        f.write('\n')
    print(f"wrote {path}: page counts {s['page_count_matches']}/{s['page_count_compared']}, mean score {s['mean_score']}")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('cmd', choices=['run', 'check', 'update'])
    ap.add_argument('report', nargs='?')
    ap.add_argument('--platform', default=this_platform(), choices=['windows', 'linux'])
    ap.add_argument('--exe', default=default_exe())
    ap.add_argument('--out')
    a = ap.parse_args()
    if a.cmd == 'run':
        out = a.out or os.path.join(ROOT, 'out', f'gate-{a.platform}')
        r = subprocess.run([a.exe, 'fidelity', os.path.join(ROOT, 'corpus'), out], cwd=ROOT,
                           stdout=subprocess.DEVNULL)
        report = os.path.join(out, 'report.json')
        if not os.path.exists(report):
            print(f'fidelity run failed (exit {r.returncode})')
            return 1
        return check(report, a.platform)
    if not a.report:
        ap.error(f'{a.cmd} needs a report.json')
    return check(a.report, a.platform) if a.cmd == 'check' else update(a.report, a.platform)


if __name__ == '__main__':
    sys.exit(main())
