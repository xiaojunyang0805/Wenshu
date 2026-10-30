# Wenshu — Feasibility Report

**Can we build an open-source, Rust-based replacement for WPS Office / Microsoft Office? What would it cost in AI tokens, and how long would it take?**

*Date: 2026-10-30 · Status: exploratory, first pass*

---

## 1. Executive summary

| Question | Short answer |
|---|---|
| Are there open-source replacements today? | Yes. LibreOffice, Collabora and ONLYOFFICE are mature, and Univer is a newer web option. None matches WPS on Chinese-market polish, compatibility with how Chinese users actually work, AI features and cloud integration. |
| Can a new open suite be built? | **Yes, if it starts narrow.** A credible **Writer (docx) MVP in Rust** is realistic in **3–6 months** with 2–3 engineers plus AI agents. A three-app suite (Writer + Sheets + Slides) that people can use day to day takes **12–24 months**. Matching WPS feature for feature is a **3–5+ year** effort. |
| Token / money cost? | AI tokens are **not the bottleneck**. Rough estimate on Claude Opus 5.5: **≈ $1k–5k** for a Writer MVP, **≈ $5k–20k** for a basic suite and **≈ $20k–100k** for a WPS-class product. People, testing against a corpus of real files, and review cost far more. |
| Is Rust the right language? | **Yes for the core engine:** document model, file formats, layout, formulas and rendering. It gives memory safety, speed, WASM output (one engine for desktop, web and mobile), and support for the platforms China's domestic IT program (信创) requires, including LoongArch. The UI layer should be chosen for practical reasons and doesn't have to be pure Rust. |
| The hardest problem? | **Layout fidelity**: making a `.docx` from Word or WPS paginate the same way. CJK typography and font-metric compatibility (SimSun/宋体, Microsoft YaHei/微软雅黑, FangSong/仿宋) come second. Coding effort comes after both. |
| Recommendation | Build **a Rust document engine plus a web-tech UI (Tauri)**. Start with Writer and Chinese official-document (公文) templates, and **reuse IronCalc for Sheets**. Track fidelity with an automated corpus of real files from day 1. |

---

## 2. Background: WPS and why it matters

- **Kingsoft WPS Office** had about **676 million monthly active devices** as of 30 June 2026, and about 678M at the end of 2025. Mid-2025 figures were roughly 305M on PC and 346M on mobile, and about 245M of the total are overseas. 2025 revenue was about **¥5.93 billion**. (Kingsoft filings and press; see Sources.)
- WPS has a free tier with ads, plus paid membership, the WPS 365 enterprise product and WPS AI. Users regularly complain about ads, bloat, storage nagging and lock-in to the cloud account.
- WPS is central to China's **信创 (domestic IT substitution)** program, running on Kylin and UOS Linux, LoongArch and Kunpeng CPUs, and HarmonyOS.
- **The opening for an open-source suite:** no ads, works fully offline, privacy by default, open file formats, auditable code, permissive license, and first-class Chinese typography. No current open-source product does all of these well.

---

## 3. Landscape of open-source office software (2026)

### 3.1 Mature suites

| Project | License | Language / stack | Size | Strengths | Weaknesses for a "WPS replacement" |
|---|---|---|---|---|---|
| **LibreOffice** (TDF) | MPL-2.0 | C++ (StarOffice legacy) | ~10M LOC | Broadest features, ODF reference implementation, good .doc/.xls/.ppt import | Dated UI, OOXML fidelity gaps, huge legacy codebase that is hard to modernize. **26.8** (Aug 2026) explicitly rejected AI features. In April 2026 TDF removed Collabora staff from its membership, which fractured the community. |
| **Collabora Office / Online** | MPL-2.0 | LibreOffice core + JS web client | — | Enterprise LibreOffice, web collaboration. Online 26.04 added an opt-in AI assistant; there is a Mac desktop app (Feb 2026). | Same core limits as LibreOffice. Commercial focus. |
| **ONLYOFFICE** | AGPL-3.0 (+ commercial) | JS (canvas editors) + C++ converters | — | **Best OOXML fidelity** of the open suites, modern UI, real-time co-editing. Workspace 12.8 shipped in May 2026. | AGPL. Branding and logo restrictions in its license terms. Ended its Nextcloud partnership over the "Euro-Office" fork (Feb 2026). |
| **Calligra** (KDE) | GPL/LGPL | C++/Qt | — | Native Qt | Small team, weak MS-format compatibility |
| **AbiWord / Gnumeric** | GPL | C/GTK | — | Lightweight | Barely maintained |
| **Apache OpenOffice** | Apache-2.0 | C++ | — | Permissive license | Effectively dormant |

### 3.2 Newer / web-native projects

| Project | License | Notes |
|---|---|---|
| **Univer** (DreamNum, a Chinese team) | Apache-2.0 | TypeScript office **SDK**: sheets, docs, slides; canvas rendering. Embedded more than used standalone. **Most relevant Chinese open-source effort.** |
| **IronCalc** | MIT / Apache-2.0 | **Rust spreadsheet engine** (formula evaluation, xlsx I/O, WASM). A strong base for Sheets. |
| **Typst** | Apache-2.0 | Rust typesetting system. Not WYSIWYG, but its crates (layout, `krilla` PDF, font handling) are useful reference and building blocks. |
| **WordCraft / GridCraft / DeckCraft** | open source (verify) | Reported in Oct 2026 as Rust Office clones built with AI help, with about 16k GitHub stars. **Single source; verify before relying on it.** If confirmed, it shows that AI-assisted Rust office suites are practical, and it may be worth contributing to instead of starting from zero. |
| **BetterOffice** (openooxml) | Apache-2.0 | Rust OOXML engines published as a crate, an npm package and a Python package, plus a web app. |
| **Hanthor Rust Office Suite** | open source | GTK4/libadwaita suite, v1.0 (June 2026). Its Writer handles Markdown, HTML and DOCX, and exports PDF through Typst. |
| **office-toolkit** | MIT | Pure-Rust OOXML read/write library. |

**Conclusion:** Rust building blocks now exist for almost every layer, and at least one AI-assisted Rust suite has reportedly appeared. A new project is no longer a moonshot. The gap that remains is **Chinese-first fidelity and polish**.

---

## 4. Why office suites are hard (where the effort goes)

1. **File-format fidelity.** The OOXML spec (ECMA-376) runs to more than 6,000 pages, and Word and WPS behave in ways the spec does not describe. Legacy binary formats (.doc/.xls/.ppt, MS-DOC/MS-XLS/MS-PPT) are documented but very large. Typical breakdown: reading files is about 30% of the work, getting *layout to match* is about 60%, and round-trip saving without losing data is the remainder.
2. **Layout engine.** This means incremental pagination, tables that split across pages, floating objects and text wrap, section and column breaks, headers and footers, footnotes, and widow/orphan control. It is the **single biggest risk**.
3. **Text stack.** Shaping (HarfBuzz-equivalent), bidirectional text, line breaking (UAX #14), font fallback, IME input and accessibility.
4. **Spreadsheet engine.** There are more than 500 functions, Excel-exact number formatting, date-system quirks, dynamic arrays, pivot tables, charts and conditional formatting.
5. **Presentation.** The masters/layouts/placeholders model, SmartArt, animations and transitions.
6. **Long tail.** Track changes, comments, fields, TOC, mail merge, equations (OMML), charts, macros (VBA or JS), printing, PDF export, digital signatures, encryption and collaboration.

### 4.1 China-specific requirements (the differentiator)

| Need | Detail |
|---|---|
| **Chinese typography** (W3C *clreq*) | Punctuation compression (标点挤压), line-start/line-end prohibition rules (避头尾), spacing between CJK and Latin text (中西文间距), grid layout (文档网格: chars per line / lines per page), vertical text (竖排), ruby/pinyin (拼音指南), enclosed characters (带圈字符), two-lines-in-one (双行合一), first-line indent in characters (首行缩进2字符) |
| **Official documents (公文)** | **GB/T 9704-2012** layout: red header (红头), document number, fixed 22 lines × 28 chars, FangSong (仿宋_GB2312) body, seal positioning. Shipping built-in templates is a strong selling point. |
| **OFD format** | **GB/T 33190-2016**, China's alternative to PDF, required in e-government and e-invoicing. Read, view and export support is a differentiator; open-source support is thin. |
| **Fonts** | Word/WPS documents use SimSun (宋体), SimHei (黑体), Microsoft YaHei (微软雅黑), FangSong_GB2312 (仿宋_GB2312), KaiTi (楷体) and similar. These fonts can't be redistributed, and **there are no free clones with matching metrics**, so layouts shift. Mitigation: map them to Source Han / Noto CJK, LXGW WenKai and Fandol fonts; use the system fonts when they are present; build a per-font metric-compensation table. |
| **信创 platforms** | Linux on x86_64, aarch64 and **loongarch64** (Rust support is tier 2 and works); Kylin and UOS packaging; **HarmonyOS NEXT** (Rust can build for OHOS targets, and the UI would need ArkUI or WebView). |
| **Input** | Robust IME support (Sogou, Microsoft Pinyin, Rime/fcitx5) in the editor canvas. Many custom-canvas editors get this wrong. |

---

## 5. Proposed architecture (Rust-first)

```
┌────────────────────────── UI shells ──────────────────────────┐
│ Desktop: Tauri 2 (WebView + Rust)  │  Web: same UI + WASM core │
│ Mobile/HarmonyOS: WebView shell    │  (later: native canvas)   │
└───────────────▲───────────────────────────────▲───────────────┘
                │ commands / events (typed IPC)  │
┌───────────────┴────────── Rust core (crates) ──┴───────────────┐
│ ow-model     document model (rich text, tables, sections, styles)│
│ ow-ooxml     docx/xlsx/pptx read+write  (quick-xml, zip)        │
│ ow-odf       odt/ods/odp                                         │
│ ow-legacy    .doc/.xls/.ppt import (CFB container: `cfb` crate)  │
│ ow-text      shaping/fonts/line-break (harfrust/rustybuzz,       │
│              skrifa/fontations, swash, icu4x, unicode-bidi)      │
│ ow-cjk       clreq rules: 标点挤压/避头尾/grid/vertical/ruby       │
│ ow-layout    incremental pagination & table/float layout         │
│ ow-render    vello / tiny-skia → canvas, GPU (wgpu), print       │
│ ow-pdf       PDF export (krilla / pdf-writer); ow-ofd OFD export │
│ ow-calc      spreadsheet = IronCalc (reuse/fork)                 │
│ ow-collab    CRDT (yrs / loro) for real-time co-editing           │
│ ow-script    macros: JS (boa / QuickJS) — WPS JSA compatible API  │
│ ow-ai        optional, pluggable LLM providers (local or cloud)  │
└──────────────────────────────────────────────────────────────────┘
```

**Key decisions:**

- **Rust for the engine, web technology for the UI.** Building the editor UI in pure Rust (egui, iced, Slint, GPUI) means rebuilding menus, ribbons, dialogs and IME handling from scratch. Tauri with a canvas-rendered document view, as ONLYOFFICE and Google Docs do, gets to a usable product much faster. The same core compiles to **WASM** for a browser version.
- **Canvas rendering, not DOM `contenteditable`.** This is the only way to get exact pagination and the same rendering on every platform.
- **Reuse, don't rewrite:** IronCalc (Sheets), Typst-ecosystem crates (fonts, PDF), the `cfb` crate (legacy containers), and yrs or loro (collaboration).
- **License:** **Apache-2.0 / MIT** for the widest adoption, including by Chinese enterprises and government integrators. Don't copy code from LibreOffice (MPL is file-level copyleft, so mixing is possible but adds friction) or ONLYOFFICE (AGPL). Using them as *behavior references and test oracles* is fine.
- **Fidelity test harness from day 1:** a corpus of thousands of real docx/xlsx/pptx files, including Chinese government and enterprise samples. Render each file with Wenshu and with reference renderers (LibreOffice headless, plus Word/WPS where licensed), compare page images (SSIM / pixel diff) and page counts, and track a **fidelity score in CI**. This loop is also what lets AI agents work autonomously: they need a measurable target.

---

## 6. Scope tiers

| Tier | Scope | Est. Rust LOC | Notes |
|---|---|---|---|
| **A. Writer MVP** | docx/odt open, edit, save; styles, lists, tables, images, headers/footers; CJK typography and 公文 templates; PDF export; desktop (Win/Linux/mac) | **80k–150k** | Achieves about 80–90% visual fidelity on typical documents |
| **B. Basic suite** | A + Sheets (IronCalc-based, ~300 functions, charts) + Slides (pptx view/edit/present) + .doc/.xls/.ppt import + OFD export + web build | **300k–500k** | Usable for most office workers |
| **C. WPS-class** | B + track changes/comments at full fidelity, pivot tables, macros (JSA), real-time collaboration, cloud sync, mobile/HarmonyOS, AI features, PDF editing, accessibility, i18n | **1.5M–3M** | For comparison, LibreOffice is about 10M LOC in C++ |

---

## 7. Token and cost estimate

### 7.1 Pricing basis (Anthropic API, cached as of 2026-10)

| Model | Input $/MTok | Output $/MTok | Cache read $/MTok |
|---|---|---|---|
| Claude Opus 5.5 | 4.00 | 20.00 | 0.20 |
| Claude Sonnet 5.5 | 2.00 | 10.00 | 0.20 |
| Claude Haiku 5.5 | 0.10 | 0.50 | — |
| Claude Fable 5.1 (top tier) | 10.00 | 50.00 | 0.25 |

In agentic coding, roughly **85–95% of tokens are cache reads**: the agent re-reads context and files on every turn. That makes the **blended cost about $1.0–1.8 per million tokens on Opus 5.5**, much lower than the list input price.

### 7.2 Tokens per line of shipped code

Calibration point: Anthropic's published experiment of building a ~100k-line Rust C compiler with a team of parallel Claude agents took about 2,000 sessions and about $20k of API cost at earlier, higher Opus pricing. That works out to about **$0.20 per final LOC**, or roughly **15–25k tokens per LOC**, for fully autonomous work with heavy testing. Human-steered work with good specs is typically **3–8k tokens per LOC**.

| Tier | LOC | Tokens (steered → autonomous) | Opus 5.5 cost (≈$1.4/MTok blended) |
|---|---|---|---|
| A. Writer MVP | ~120k | 0.5B – 3B | **≈ $700 – $4,500** |
| B. Basic suite | ~400k | 2B – 10B | **≈ $3k – $15k** |
| C. WPS-class | ~2M | 10B – 50B | **≈ $15k – $75k** |

Add **30–50%** for refactors, fidelity debugging and regressions. Using Sonnet 5.5 for routine work (tests, format mappings, function libraries) and Opus 5.5 or Fable 5.1 for architecture and layout can cut cost by roughly 30–40%.

**These are order-of-magnitude estimates, not quotes.** The real constraints are:
- **Human review and direction:** architecture, layout algorithms and accepting changes.
- **The test corpus and oracles:** agents can only work autonomously against measurable fidelity tests.
- **Tool/CI compute** for rendering thousands of documents per run. This cost is small but continuous.

### 7.3 People and calendar time (with AI agents)

| Tier | Team | Calendar time |
|---|---|---|
| A. Writer MVP | 2–3 engineers (1 typography/layout specialist) | **3–6 months** |
| B. Basic suite | 5–8 engineers + a QA/fidelity lead | **12–24 months** |
| C. WPS-class | 15–30 engineers + community | **3–5+ years** |

Without AI agents, multiply these by roughly 2–4×. The long tail of compatibility (Section 4) does not compress well: it is discovered through real user documents, not written up front.

---

## 8. Roadmap (suggested)

**Phase 0 — Foundations (month 0–1)**
- Repository, crate layout, CI, license (Apache-2.0/MIT).
- Build the **fidelity corpus and diff harness** (LibreOffice headless as the first oracle).
- Evaluate WordCraft, BetterOffice and IronCalc: **fork, contribute or build?**

**Phase 1 — Writer engine (months 1–4)**
- docx → model → layout → canvas render → docx/PDF save.
- CJK typography rules; mapping of 宋体/仿宋/黑体 to open fonts plus metric compensation.
- Built-in GB/T 9704 公文 templates.
- Tauri desktop shell, IME support, basic UI (ribbon or a simple toolbar).

**Phase 2 — Writer beta (months 4–6)**
- Track changes (read), comments, TOC, fields, equations (view).
- Windows/Linux (incl. LoongArch) packages; public beta; fidelity score published.

**Phase 3 — Sheets and Slides (months 6–15)**
- Sheets on IronCalc: UI grid, xlsx fidelity, charts, conditional formats.
- Slides: pptx render/edit/present.
- Web build (WASM); OFD export.

**Phase 4 — Ecosystem (15+ months)**
- Collaboration (CRDT), JS macros, plugin API, optional AI assistant (local models first), HarmonyOS/mobile.

---

## 9. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Layout fidelity plateaus at ~85% | Users abandon the product after the first broken document | Corpus-driven CI, a dedicated layout lead, and an honest "compatibility mode" |
| Font licensing (Chinese system fonts) | Metric drift | Use system fonts when present; metric tables; partner with open-font foundries |
| Scope creep toward "everything WPS has" | The project never ships | Tiered scope; Writer first; say no often |
| AI-generated code quality and maintainability | Hidden bugs, unreadable code | Strict crate boundaries, property and fuzz tests (cargo-fuzz on parsers), human review of core algorithms |
| Format security (malicious docs) | CVEs | Rust memory safety, fuzzing, no macro execution by default |
| Open-source sustainability | Burnout | Foundation or company backing, an enterprise-support model, Chinese-community engagement (Gitee + GitHub) |
| Existing Rust suites (e.g. WordCraft) move faster | Duplicate effort | Evaluate first; contributing or forking is legitimate. Differentiate on Chinese typography, 公文 and OFD. |
| Regulatory / 信创 certification | Enterprise and government adoption blocked | Plan for compatibility certification with Kylin/UOS early |

---

## 10. Recommendation

1. **Do it, but narrowly:** "**Wenshu Writer**" is an ad-free, offline, Apache-licensed Rust word processor whose selling points are **Chinese typography, 公文 templates and OFD support**. That niche is unserved and achievable.
2. **Spend the first month on evaluation and the test harness, not features.** Check whether WordCraft, BetterOffice or IronCalc can be the base; it could save months.
3. **Budget:** about **$5k–10k of AI tokens** and 2–3 engineers for six months to reach a public Writer beta. Reassess before committing to the full suite.
4. **Measure everything** with a public fidelity scoreboard. It drives both the AI agents and community trust.

### Next steps (proposed)
- [ ] Verify the WordCraft / BetterOffice claims directly (repo, license, code quality, CJK support).
- [ ] Collect a 1,000-document Chinese docx corpus (government notices, contracts, theses, invoices).
- [ ] Prototype: docx → layout → PNG for 20 sample files in Rust, and measure the fidelity baseline.
- [ ] Write the architecture decision record (UI shell, license, crate boundaries).

---

## Sources

- Kingsoft Office 2025 results / WPS device counts: [itiger.com](https://www.itiger.com/news/1138470106), [itiger.com H1 2025](https://www.itiger.com/news/1112971266), [HKEX 2026 interim filing](https://www1.hkexnews.hk/listedco/listconews/sehk/2026/0819/2026081901069.pdf), [itiger.com user feedback](https://www.itiger.com/news/1103563886)
- LibreOffice 26.8 and the TDF/Collabora split: [It's FOSS](https://itsfoss.com/news/libreoffice-26-8-release.md), [BornCity](https://borncity.com/news/libreoffice-26-8-document-foundation-setzt-bewusst-auf-ki-verzicht/), [The Register](https://www.theregister.com/a/5293301)
- Collabora Online 26.04: [BornCity](https://borncity.com/news/collabora-online-26-04-zehn-jahre-mit-ki-assistent-und-barrierefreiheit/)
- Office suite comparison 2026: [youngju.dev](https://www.youngju.dev/blog/culture/2026-05-16-office-suite-alternatives-2026-libreoffice-onlyoffice-collabora-wps-office-google-workspace-microsoft-365-hancom-ichitaro-deep-dive), [alternativeto.net](https://www.alternativeto.net/news/tag/office-suite/)
- Rust office projects: [WordCraft report (it-connect.fr)](https://www.it-connect.fr/wordcraft-gridcraft-deckcraft-suite-office-rust-claude/), [BetterOffice](https://github.com/openooxml/betteroffice), [Hanthor GTK suite](https://github.com/tuna-os/gtk-office-suite/releases/tag/v1.0), [office-toolkit](https://docs.rs/crate/office-toolkit/latest/source/README.md), [openalternative.co (Rust editors)](https://openalternative.co/categories/document-editors/using/rust)
- Standards: ECMA-376 (OOXML), ISO/IEC 26300 (ODF), GB/T 9704-2012 (公文格式), GB/T 33190-2016 (OFD), W3C *Requirements for Chinese Text Layout* (clreq)
- Pricing: Anthropic API model table (cached 2026-10-06). The C-compiler calibration figures are approximate, from Anthropic's engineering blog.

*Some 2026 news items come from single secondary sources and should be verified against primary announcements.*
