# ADR 0002: Hooks upstream, Chinese data in Wenshu

*Accepted · 2026-10-30*

## Context
Chinese layout needs tables: document-font aliases (仿宋_GB2312, 方正小标宋简体…), punctuation
classes, line-start and line-end prohibitions, compression rules. Putting them inside the engine
would mean a large patch upstream may not want, and a fork that diverges.

## Decision
Upstream gets small generic extension points. Example: `wordcraft_fonts::word::set_substitute_provider`
(patch 0001), a plain `fn(&str) -> Option<&'static [&'static str]>`. Wenshu supplies the data from
`wenshu-cjk`, a dependency-free crate any engine (WordCraft, DeckCraft, BetterOffice) can use.

## Consequences
- Patches stay at tens of lines and are easy to review upstream.
- The same Chinese data serves the Writer, Slides and the web build.
- A provider set once per process: `wenshu_core::init()` must run before the first layout, which
  `Doc::open` guarantees.
