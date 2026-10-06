---
name: visuals-artifact
description: "The interactive CHARTS page (docs/visuals/turbojet-visuals.html) is published at https://claude.ai/artifact/G5uobeC9QAuJJ2CyLeCrKw (re-minted 2026-10-06 at the user's request; the old 56cde230… URL is gone) — republish to THAT url"
metadata:
  node_type: memory
  type: reference
  originSessionId: edd8c029-bea9-4b2c-99e0-c4ce222591f5
  modified: 2026-10-06T02:20:20.633Z
---

The project's interactive visuals page (T–s diagram, NOx bell/quench, mixing-optimum
J-sweeps, rung-22 collapse, clamp ladder, rung map) is built at
`docs/visuals/turbojet-visuals.html`. It is the CHARTS page — a different artifact
from the animated engine cutaway in [[cutaway-artifact]].

**URL (since 2026-10-06):** https://claude.ai/artifact/G5uobeC9QAuJJ2CyLeCrKw — icon
`chart`; title from `<title>` on line 1 of `template.html` (don't pass `title`). From
another session, pass this as `url` on the publish, or a THIRD artifact is minted.

**History.** It first lived at https://claude.ai/code/artifact/56cde230-f30a-44a4-be60-40b59e829180,
which was found GONE on 2026-09-07 (not in the user's own list; the cutaway was, so not
an auth problem). The rule then was "do not mint a replacement without asking"; on
2026-10-06 the user said "Publish a replacement link", and this URL is that replacement.
Neither page links to the other, so no cross-link needed updating.

**Before a republish, read the page for stale prose.** On 2026-10-06 the page still
said "28 cumulative rungs" / "29 rungs" and its footer named the deleted
`extract_data.py` — the splice/data gates cannot see prose. Fixed in `template.html`.

Source of truth is `docs/visuals/` in the repo, built by the Rust CLI:
`cargo run --release -- visuals` (runs the model, ~20 s) → `data.json`, spliced into
`template.html` (`-- splice` re-splices only) → `turbojet-visuals.html`. Rebuild is only
half the loop — republish too. The joints between the pages and the model are gated by
`rust/tests/visuals.rs`; see [[visuals-model-binding]] for what is bound and what
deliberately is not. `template.html` is CRLF — script edits must match `\r\n`.
