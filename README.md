# Wenshu 文枢

An open-source, Chinese-first office suite in Rust: offline, ad-free, Apache-2.0 OR MIT. The
Writer is built on [WordCraft](https://github.com/storytold/wordcraft). Wenshu adds Chinese
typography, GB/T 9704 official documents (公文), OFD and a Chinese fidelity corpus.

Wenshu is an independent project, not affiliated with Kingsoft or Microsoft. It opens and saves files
made in WPS Office and Microsoft Word.

**Status: M0 (foundations) done.** See [docs/MVP.md](docs/MVP.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Build

```sh
scripts/sync-upstream.sh                 # fetch WordCraft at the pinned rev + apply patches/
cargo build --release -p wenshu-cli      # ~12 min the first time
```

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
- [Architecture](docs/ARCHITECTURE.md) · [MVP](docs/MVP.md) · [Upstream policy](docs/UPSTREAM.md) · [Decisions](docs/adr/)
