# ADR 0003: Generate and check 公文 at the OOXML level

*Accepted · 2026-10-30*

## Context
GB/T 9704 compliance is mostly page geometry, the document grid and fonts, which are all stored
in `word/document.xml` and `word/styles.xml`. Going through the engine's command layer would tie
the 公文 tooling to engine internals and to a full layout pass.

## Decision
`wenshu-gongwen` writes and reads DOCX directly (zip + quick-xml) with the constants in `spec.rs`.
The engine is used only to render and export what the generator wrote.

## Consequences
- The generator and checker are fast, have no engine dependency, and can run in a web service or CI.
- The checker doesn't resolve style inheritance yet: a body font set only through a paragraph style
  reports the document default. Follow-up: resolve `w:pStyle`/`w:rStyle` chains.
- Visual checks (red rule position, organ-mark distance) need the rendered page. They belong in
  the fidelity harness, not the checker.
