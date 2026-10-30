# Working with upstream engines

## Pinned engines (`upstream.lock`)

| Engine | Repo | Revision | Licence |
|---|---|---|---|
| WordCraft | storytold/wordcraft | `3b7f7ed` (2026-10-30, v0.12.0) | MIT OR Apache-2.0 |

## Workflow

```sh
scripts/sync-upstream.sh              # clone/reset upstream/<engine> at the pin, apply patches/<engine>/*.patch
# edit upstream/wordcraft/… , build and test through Wenshu
scripts/sync-upstream.sh --export     # write the diff to patches/<engine>/0001-wenshu.patch.new, then split by topic
```

Bumping a pin means updating `upstream.lock`, re-running the sync, re-basing any patch that no
longer applies, and running the fidelity corpus before and after.

## Patch rules

1. **One topic per patch**, named `NNNN-crate-topic.patch`, with a header line saying why it's
   needed and the upstream PR link once opened.
2. **Upstream-ready:** follows the engine's AGENTS.md: no panics, no `unsafe`, a test, docs comment,
   clean room, and no Chinese data tables (data stays in `wenshu-cjk`).
3. **Generic:** a hook other languages could use too. A Chinese-only rule goes in Wenshu.
4. **Sent upstream within a week**, and dropped here once merged. More than 10 open patches on one
   engine is a signal to stop and talk to the maintainers.

## Patch queue

| Patch | Crate | What | Upstream |
|---|---|---|---|
| 0001-fonts-substitute-provider-hook | wordcraft-fonts | `SubstituteProvider` + `set_substitute_provider()`: named substitutes from the embedding app, tried before the built-in lists | not yet opened |

## Issues to file upstream (from the M0 evaluation)

- Character grid (`linesAndChars`, linePitch 579, charSpace −849): body lines hold 27 characters,
  not 28, and 22 pt text on a snapped line is compressed to the 15.8 pt pitch, so glyphs overlap.
  Needs a WPS/Word reference render. Repro: `wenshu gongwen new examples/gongwen-sample.json x.docx`
  with `snapToGrid` removed.
- Curly quotes “” in East Asian runs are drawn with the Latin font (half-width).
- Kinsoku list misses ” ’ … — at line start.
- No punctuation compression (`w:characterSpacingControl` is read but not applied).
