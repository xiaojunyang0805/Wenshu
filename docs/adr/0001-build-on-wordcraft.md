# ADR 0001: Build the Writer on WordCraft through a thin patch queue

*Accepted · 2026-10-30*

## Context
We evaluated six kits on a Chinese corpus (`eval/EVALUATION_SUMMARY.md`). WordCraft (Rust,
MIT/Apache, egui, ~338k lines, 4.7k tests, no `unsafe`) rendered both Chinese DOCX files correctly
and round-tripped them losslessly. Its gaps were Chinese-specific: font classes, punctuation
compression, the 公文 grid. BetterOffice had weaker CJK support and a single maintainer. Writing
our own engine would cost months to reach where WordCraft already is.

## Decision
Use WordCraft as the Writer engine, pinned by revision. Extend it only through small, generic,
upstream-ready patches (hooks). Keep Chinese data and 公文 logic in Wenshu crates. Isolate engine
APIs behind `wenshu-core`.

## Consequences
- We get desktop, web and agent (MCP) support, plus docx/odt/rtf/pdf I/O, from day one.
- Upstream moves fast (3 weeks old, hundreds of PRs a week): pin bumps need the fidelity corpus as
  a regression gate.
- If upstream rejects hooks or stalls, the patch queue becomes a maintained fork. `wenshu-core`
  limits the blast radius.
- ArtCraft trademarks must not appear in Wenshu builds.
