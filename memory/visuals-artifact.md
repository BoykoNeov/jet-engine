---
name: visuals-artifact
description: "The interactive visuals page (docs/visuals/turbojet-visuals.html) WAS published as a Claude artifact — that URL is GONE as of 2026-09-07; ask before minting a new one"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 2d5da9c8-b78f-4515-a07b-560557fac552
  modified: 2026-09-07T09:15:22.742Z
---

The project's interactive visuals page (T–s diagram, NOx bell/quench, mixing-optimum
J-sweeps, rung-22 collapse, clamp ladder, rung map) is built at
`docs/visuals/turbojet-visuals.html`. It is the CHARTS page — a different artifact
from the animated engine cutaway in [[cutaway-artifact]].

**Status 2026-09-07: its artifact is GONE.** The URL it used to live at —
https://claude.ai/code/artifact/56cde230-f30a-44a4-be60-40b59e829180 — returns
"artifact not found", and it is absent from `action: "list"` on the user's own
artifacts (the cutaway, `968af1ea…`, is present, so this is not an auth or account
problem). Either it was deleted deliberately or it expired.

**Do not mint a replacement URL without asking.** The "update the same URL, don't
mint a new one" rule below presumes the artifact exists; it is not authorisation to
create a second one. A new URL leaves every link the user has already shared dead
while adding a URL the repo and this memory both have to track — and the user may
have deleted it on purpose. Report it and offer; let them decide.

If they say yes, the publish parameters that must stay stable (neither is
recoverable from the published page — the favicon is a publish-time param, not part
of the HTML):
- `favicon`: ✈️
- title: comes from `<title>` on line 1 of `template.html` — don't pass `title`.

Source of truth is `docs/visuals/` in the repo: `extract_data.py` (runs the model,
~10 min) → `data.json`, `build.py` splices it into `template.html` →
`turbojet-visuals.html`. Rebuild is only half the loop — republish too. The joints
between the pages and the model are gated by `tests/test_visuals_data.py`; see
[[visuals-model-binding]] for what is bound and what deliberately is not. Charts
read CSS tokens at build time and re-render on theme flip; every chart has a
data-table twin. Illustration grids are reduced (shape, not digits).
