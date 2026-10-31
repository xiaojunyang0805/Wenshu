# Wenshu 文枢

An open-source, Chinese-first office suite in Rust: offline, ad-free, Apache-2.0 OR MIT. The
Writer is built on [WordCraft](https://github.com/storytold/wordcraft). Wenshu adds Chinese
typography, GB/T 9704 official documents (公文), OFD and a Chinese fidelity corpus.

Wenshu is an independent project, not affiliated with Kingsoft or Microsoft. It opens and saves files
made in WPS Office and Microsoft Word.

**Status: M0 (foundations) done; M1 (Chinese layout) in progress:** 4 layout patches done and reviewed
in draft PRs upstream; a 200-document corpus is assembled ([corpus/README.md](corpus/README.md)); WPS reference renders are next. See [docs/PROGRESS.md](docs/PROGRESS.md), [docs/MVP.md](docs/MVP.md) and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Build

```sh
scripts/sync-upstream.sh                 # fetch WordCraft at the pinned rev + apply patches/
cargo build --release -p wenshu-cli      # ~12 min the first time
cargo build --release -p wenshu-desktop  # the desktop app (文枢), Chinese interface
```

`scripts/sync-upstream.sh` also checks out the interface translations (craft-i18n), which
`.cargo/config.toml` points the build at. `packaging/windows/package.ps1` builds the Windows
portable zip (and an MSI when WiX v5 is installed) into `dist/`.

Needs Rust ≥ 1.95 (tested on 1.99.1; on Windows use the MSVC toolchain), Git and about 10 GB of disk for the build.

## Use

```sh
wenshu gongwen new examples/gongwen-sample.json notice.docx   # write a GB/T 9704 document
wenshu gongwen check notice.docx                              # check it against the standard
wenshu render notice.docx page1.png --page 1                  # render a page
wenshu convert notice.docx notice.pdf                         # export PDF (fonts embedded)
wenshu fidelity corpus out/fidelity                           # corpus scoreboard
```

## Documents

- [Feasibility report](docs/Wenshu_Feasibility_Report.md): landscape, cost and time model
- [Kit evaluation](eval/EVALUATION_SUMMARY.md): WordCraft, DeckCraft, GridCraft, IronCalc, BetterOffice, Univer
- [Architecture](docs/ARCHITECTURE.md) · [MVP](docs/MVP.md) · [Upstream policy](docs/UPSTREAM.md) · [Decisions](docs/adr/) · [Progress log](docs/PROGRESS.md)
