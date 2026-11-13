# Wenshu 文枢

An open-source, Chinese-first office suite in Rust: offline, ad-free, Apache-2.0 OR MIT. The
Writer is built on [WordCraft](https://github.com/storytold/wordcraft). Wenshu adds Chinese
typography, GB/T 9704 official documents (公文), OFD and a Chinese fidelity corpus.

Wenshu is an independent project, not affiliated with Kingsoft or Microsoft. It opens and saves files
made in WPS Office and Microsoft Word.

**Status: MVP jobs met (v0.1.5).** M0 foundations, M1 Chinese layout (page counts
match WPS on 187 / 200 corpus documents on Windows, 182 on Linux), M2 desktop alpha and M3's OFD
export are done, and a self-hostable browser version is available (M4 started); [releases](https://github.com/xiaojunyang0805/Wenshu/releases) are published as
pre-releases. The layout patches are in draft PRs upstream, reworked to WPS measurements across a 200-document corpus
([corpus/README.md](corpus/README.md), [docs/WPS-FINDINGS.md](docs/WPS-FINDINGS.md)). The desktop
app 文枢 has a Chinese or English interface, 公文 templates and a 公文 format check, and builds
into a Windows zip and MSI, Linux tar.gz, .deb and AppImage packages (x86_64), and LoongArch
tar.gz and .deb packages. See [docs/PROGRESS.md](docs/PROGRESS.md), [docs/MVP.md](docs/MVP.md) and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Build

```sh
scripts/sync-upstream.sh                 # fetch WordCraft at the pinned rev + apply patches/
cargo build --release -p wenshu-cli      # ~12 min the first time
cargo build --release -p wenshu-desktop  # the desktop app (文枢), Chinese interface
```

`scripts/sync-upstream.sh` also checks out the interface translations (craft-i18n), which
`.cargo/config.toml` points the build at.

Packages go to `dist/`:

- `packaging/windows/package.ps1`: a portable zip, and a per-user MSI when WiX v5 is installed
  (`dotnet tool install --global wix --version 5.0.2`, which needs the .NET SDK).
- `packaging/linux/package.sh`: tar.gz, .deb (`dpkg-deb`) and AppImage. Build on Ubuntu 22.04
  (or WSL) with WordCraft's Linux build packages; the script's header lists them.
  `--arch loongarch64` cross-builds tar.gz and .deb for LoongArch (glibc ≥ 2.36, new-world ABI)
  with [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) and Zig 0.14.
- `.github/workflows/release.yml`: all of the above on a `v*` tag, attached to a draft release.

Before tagging a release, run the regression gate on Windows and on Linux (it needs the WPS
references in `corpus/ref`, kept locally, so it runs where they are, not in CI):

```sh
python -X utf8 corpus/tools/gate.py run                     # fidelity rescore vs corpus/baseline/<platform>.json
python -X utf8 corpus/tools/gate.py update out/gate-windows/report.json   # after a change meant to move the numbers
```

It fails when a document stops matching WPS's page count, the matches drop, a document's score
falls by more than 0.02 or the mean by more than 0.002. On Linux, set
`WENSHU_FONTS_DIR=/usr/share/wenshu/fonts` (the packaged fonts); from Windows,
`corpus/tools/gate_wsl.sh` does the Linux half in WSL.
`corpus/tools/pdf_check.py` and `corpus/tools/ofd_check.py` check the PDF and OFD exports.

`scripts/build-web.sh` builds the self-hostable browser app (文枢网页版) into `dist/web`: copy
that folder to any static web server, an intranet one too, and documents are opened, edited and
exported in the browser without being uploaded ([apps/wenshu-web/HOSTING.md](apps/wenshu-web/HOSTING.md)).
It needs trunk (`cargo install trunk --locked`) and the `wasm32-unknown-unknown` target.

`python vscode/package.py` packages the VS Code preview extension as
`dist/wenshu-preview-<version>.vsix` (install it with `code --install-extension <file>`); it draws
the pages with the `wenshu` CLI ([vscode/README.md](vscode/README.md)).

Needs Rust ≥ 1.95 (tested on 1.99.1; on Windows use the MSVC toolchain), Git and about 10 GB of disk for the build.

## Use

```sh
wenshu gongwen new examples/gongwen-sample.json notice.docx   # write a GB/T 9704 document
wenshu gongwen check notice.docx                              # check it against the standard
wenshu render notice.docx page1.png --page 1                  # render a page
wenshu render notice.docx pages --all                         # render every page into pages/
wenshu convert notice.docx notice.pdf                         # export PDF (fonts embedded)
wenshu convert notice.docx notice.ofd                         # export OFD (GB/T 33190)
wenshu fidelity corpus out/fidelity                           # corpus scoreboard
```

## Documents

- [Feasibility report](docs/Wenshu_Feasibility_Report.md): landscape, cost and time model
- [Kit evaluation](eval/EVALUATION_SUMMARY.md): WordCraft, DeckCraft, GridCraft, IronCalc, BetterOffice, Univer
- [Architecture](docs/ARCHITECTURE.md) · [MVP](docs/MVP.md) · [Upstream policy](docs/UPSTREAM.md) · [Decisions](docs/adr/) · [Progress log](docs/PROGRESS.md)
