# Wenshu architecture

*Status: MVP architecture, v0.1 · 2026-10-30*

Wenshu is an open-source, Chinese-first office suite. It **does not build a new engine from
scratch**. It builds on open Rust engines, chosen in the [kit evaluation](../eval/EVALUATION_SUMMARY.md),
and owns the layer those engines lack: Chinese typography, GB/T 9704 official documents (公文), OFD,
and a Chinese fidelity corpus. Everything Wenshu writes is Apache-2.0 OR MIT and offered upstream
first.

## 1. Big picture

```
┌──────────────────────────── apps ─────────────────────────────┐
│ wenshu-cli (now)    wenshu desktop (M2)    wenshu-web (M4) │
└──────────────┬───────────────────┬─────────────────────────────┘
               │                   │  WordCraft egui UI, rebranded +
               ▼                   ▼  zh-CN defaults (M2)
┌──────────────────────── Wenshu crates ────────────────────────┐
│ wenshu-core      Doc facade: open/render/save + installs CJK  │
│                   defaults into the engine (the only crate     │
│                   that knows engine internals)                 │
│ wenshu-gongwen   GB/T 9704: spec, DOCX generator, checker     │
│ wenshu-fidelity  corpus runner, page diff, scoreboard         │
│ wenshu-cjk       pure data: font substitutes, punctuation     │
│                   classes (clreq / GB/T 15834), no deps        │
│ wenshu-ofd       OFD (GB/T 33190) writer; wenshu-core::ofd     │
│                   walks the layout's display list into it      │
└──────────────┬─────────────────────────────────────────────────┘
               │ path deps + thin patch queue (patches/<engine>/)
┌──────────────▼──────────── upstream engines ────────────────────┐
│ WordCraft  (Writer)  storytold/wordcraft  @ upstream.lock       │
│ DeckCraft  (Slides)  M3 · IronCalc (Sheets engine) M3           │
└─────────────────────────────────────────────────────────────────┘
```

## 2. Principles

1. **Upstream first, thin fork.** Engines are used as they are, pinned in `upstream.lock`. Any change
   to them lives in `patches/<engine>/NNNN-topic.patch`. Each patch is small, generic, follows the
   upstream's rules (no panics, tests, clean room), and is sent upstream as a PR. A merged patch is
   deleted here. Target: **at most 10 open patches per engine.**
2. **Hooks upstream, data here.** We don't put Chinese tables into the engine. We add a generic
   extension point upstream (for example `set_substitute_provider`) and supply the data from
   `wenshu-cjk`. The engines stay language-neutral, and Wenshu owns the Chinese knowledge.
3. **One seam to the engine.** Only `wenshu-core` touches `wordcraft-*` APIs. Apps, the 公文
   tooling and the fidelity harness go through `wenshu_core::Doc`. Swapping or upgrading an engine
   then touches one crate.
4. **OOXML-level tools don't need the engine.** `wenshu-gongwen` reads and writes DOCX directly
   (zip + quick-xml), so it is fast, testable and reusable by others, such as WPS-plugin authors or
   web services.
5. **Measured fidelity.** Every claim about rendering is backed by the corpus scoreboard
   (`wenshu fidelity`). Reference renders from WPS or Word go in `corpus/ref/`, kept locally
   unless their licence allows sharing.
6. **Same safety bar as upstream.** `unsafe_code = forbid`; `unwrap`/`expect`/`panic`/`todo` are
   denied in non-test code. Errors are `Result<_, String>` at crate edges.

## 3. Repository layout

```
Wenshu/
├─ Cargo.toml              workspace; mirrors upstream [patch.crates-io]
├─ (rust-version = 1.95 in Cargo.toml; tested on 1.99.1)
├─ upstream.lock           <engine> <git url> <pinned rev>
├─ patches/wordcraft/      the patch queue (0001-fonts-substitute-provider-hook.patch)
├─ scripts/sync-upstream.sh  clone at the pinned rev + apply patches (and --export)
├─ upstream/               (gitignored) patched engine checkouts
├─ crates/
│  ├─ cjk/                 wenshu-cjk
│  ├─ core/                wenshu-core
│  ├─ gongwen/             wenshu-gongwen
│  ├─ ofd/                 wenshu-ofd
│  └─ fidelity/            wenshu-fidelity
├─ apps/wenshu-cli/       `wenshu` binary
├─ apps/wenshu-desktop/   `wenshu-desktop`: the WordCraft front end as 文枢 (M2)
├─ corpus/                 200-document fidelity corpus: files/, manifest.csv, tools/ (survey, select, probes, fetch, WPS renders), ref/ (local); see corpus/README.md
├─ examples/               gongwen-sample.json
├─ docs/                   this file, MVP.md, UPSTREAM.md, adr/, feasibility report
└─ eval/                   kit evaluation (summary + findings kept; clones and targets ignored)
```

## 4. Crates

| Crate | Depends on | Responsibility | State |
|---|---|---|---|
| `wenshu-cjk` | — | Chinese document-font substitutes by design class (仿宋/楷/宋/小标宋/黑/雅黑), normalised names (`_GB2312`, `简体`, Founder IDs); punctuation classes, line-start/end prohibitions, compression amounts | ✅ 6 tests |
| `wenshu-core` | cjk, ofd, wordcraft-engine, wordcraft-fonts, wordcraft-layout | `init()` installs the substitute provider and the OFD save format; `Doc::{open, info, pages, render_png, save_as, run}`; `ofd::export` (the page display list → `wenshu-ofd`); zh-CN default region (`WENSHU_REGION`) | ✅ |
| `wenshu-gongwen` | zip, quick-xml | `spec` (A4, 37/28 mm, 156×225 mm, 22×28 grid, 3号 仿宋…), `write_docx` (organ mark, number, red rule, title, recipients, body with 一、/（一） headings, signature, date, odd/even "— N —" footers), `check_docx` (8 rules) | ✅ 5 tests |
| `wenshu-fidelity` | core, image | Render `corpus/files/*`, compare against `corpus/ref/<stem>.p<N>.png` (luminance diff), page-count match, `report.json` | ✅ 1 test |
| `wenshu-cli` | core, gongwen, fidelity | `wenshu info · render · convert · gongwen new/check · fidelity` | ✅ |
| `wenshu-desktop` | core, gongwen, wordcraft (app, patch 0006), wordcraft-control-key | `wordcraft::run` with Wenshu's brand (文枢), icon, settings folder (`%APPDATA%\Wenshu`), `wenshu_core::init()` at start-up (font substitutes; Chinese blank-document defaults in a Chinese region), 公文 templates seeded into File › New › Personal, and Review › 文枢 › 检查公文格式 (`wenshu-gongwen` checker). Packaged by `packaging/windows/package.ps1` (zip, MSI) and `packaging/linux/package.sh` (tar.gz, .deb, AppImage; tar.gz and .deb for LoongArch). The one crate that embeds the UI shell; document work still goes through `wenshu-core` | ✅ alpha |
| `wenshu-ofd` | zip, subsetter | OFD (GB/T 33190) writer, engine-free: pages of text, paths and pictures in mm, fonts embedded as subsets with their own `cmap` (`CGTransform` only where a glyph isn't the cmap's), the OFD.xml / Document.xml / resources / page content package. `wenshu-core::ofd` walks WordCraft's page display list into it, and `init()` registers `.ofd` as a save format (patch 0065) for the CLI, Save As and File › Export | ✅ M3 (2026-11-05) |

## 5. How the engine is extended

**Font substitution, done (patch 0001):**

```
wordcraft-fonts::word::resolve(family)
  └─ family installed? → use it
  └─ substitute(base) → substitutes_by_name(base)
        └─ SUBSTITUTE_PROVIDER (set by wenshu-core::init → wenshu_cjk::fonts::substitutes)
              仿宋_GB2312 → [FangSong, 仿宋, STFangsong, 华文仿宋, FandolFang, Noto Serif CJK SC…]
        └─ built-in Latin/Thai/Hebrew/… tables
```

Result on `corpus/files/01_gongwen.docx`: the body went from Microsoft YaHei (sans) to **FangSong**,
and the title from YaHei to **STZhongsong**. Both are verified in the exported PDF's font list.

**Planned hooks, each one small upstream PR:**

| Hook | Upstream crate | Wenshu data | Why |
|---|---|---|---|
| ✅ Punctuation compression (标点挤压, patch 0005) | doc, docx, `wordcraft-layout` | — (standard pairs, upstream; `wenshu_cjk::punct` mirrors them) | `w:characterSpacingControl` was dropped on save and never applied |
| ✅ Kinsoku sets (patch 0004) | `wordcraft-layout::kinsoku` | — (standard sets, upstream) | ” ’ could start a line once quotes become break opportunities |
| ✅ Character-grid fixes (patch 0002) | `wordcraft-layout` | corpus cases | Body lines gave 27 instead of 28 chars on the 公文 grid; large text squeezed to the pitch |
| ✅ `w:hint="eastAsia"` + named EA substitutes (patch 0003) | doc, docx, layout | `wenshu_cjk::fonts` via 0001 | “” drawn half-width in the Latin font; 仿宋_GB2312 as `w:eastAsia` dropped to generic fallback |

## 6. Data flow (CLI)

```
wenshu gongwen new spec.json out.docx   →  wenshu-gongwen (no engine)
wenshu gongwen check file.docx          →  wenshu-gongwen (no engine)
wenshu render|convert|info file         →  wenshu-core::Doc → WordCraft Session commands
                                             (file.info, file.exportPng, file.save)
wenshu fidelity corpus out              →  wenshu-fidelity → Doc → PNG + diff → report.json
```

## 7. Desktop and web (after the CLI MVP)

- **Desktop (M2):** build WordCraft's `apps/wordcraft` with `wenshu-core::init()` called at
  start-up, the zh-Hans interface by default, Wenshu branding (ArtCraft marks removed, as their
  trademark rule requires), 公文 templates in File › New, and a "Check 公文" command. Implemented as
  one more upstream hook: an app-embedding entry point (`run_app(config)`), so we don't fork the
  UI crate.
- **Web (M4):** WordCraft already compiles to WASM, and the same `wenshu-core` init applies.
  Chinese fonts are loaded on demand: subset web fonts, Noto/Source Han.
- **Slides and Sheets (M3):** DeckCraft (same patch-queue model) and IronCalc (engine reuse; zh-CN
  locale and LENB/DBNum contributed upstream).

## 8. Quality gates

`cargo fmt --check` · `cargo clippy -D warnings` · `cargo test` for Wenshu crates · `wenshu
gongwen check` on generated samples · `wenshu fidelity corpus` (scores never drop; page-count
matches never drop) · `scripts/sync-upstream.sh` applies cleanly at the pinned revision.

See [MVP.md](MVP.md) for scope and milestones, [UPSTREAM.md](UPSTREAM.md) for the patch policy, and
[adr/](adr/) for the decisions.
