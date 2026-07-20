---
name: visuals-artifact
description: "The interactive visuals page lives in docs/visuals/ and is published as a Claude artifact — update the same URL, don't mint a new one"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 2d5da9c8-b78f-4515-a07b-560557fac552
---

The project's interactive visuals page (engine cutaway animation, T–s diagram,
NOx bell/quench, mixing-optimum J-sweeps, rung-22 collapse, clamp ladder,
22-rung map) is published at:

https://claude.ai/code/artifact/56cde230-f30a-44a4-be60-40b59e829180

Publish parameters that must stay STABLE across redeploys (neither is
recoverable from the published page — the favicon is a publish-time param, not
part of the HTML):
- `favicon`: ✈️
- title: comes from `<title>` on line 1 of `template.html` — don't pass `title`.

Source of truth is `docs/visuals/` in the repo: `extract_data.py` (runs the
model, ~5 min) → `data.json`, `build.py` splices it into `template.html` →
`turbojet-visuals.html`. To update the artifact from another session, pass the
URL above as `url` to the Artifact tool so the link stays stable. Charts read
CSS tokens at build time and re-render on theme flip; every chart has a
data-table twin. Illustration grids are reduced (shape, not digits).
