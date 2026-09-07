---
name: cutaway-artifact
description: "The animated engine-cutaway page (docs/visuals/turbojet-cutaway.html) is a SECOND artifact beside the visuals page — republish the same URL, favicon ⚙️"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 1732215a-6025-4141-ac4b-1b208d090e7f
  modified: 2026-09-07T08:09:49.018Z
---

The animated 2-D engine cutaway lives at `docs/visuals/turbojet-cutaway.html`
(built by `docs/visuals/build_cutaway.py` from `cutaway-template.html` +
`data.json`). It is a separate artifact from the visuals page in
[[visuals-artifact]] — a new deliverable the user asked for on 2026-09-07
("engine diagram with moving components, air flow, burn - 2d"), not an update
to the old cutaway panel.

Publish parameters that must stay stable: favicon ⚙️; title from the
`<title>` tag (Turbojet Cutaway). URL (pass as `url` on a redeploy from another session):
https://claude.ai/code/artifact/968af1ea-4e24-436b-ac1e-16b299ec0304

Lessons from building it:
- Chrome's MCP tools refuse `file://` URLs; serve the folder with
  `python -m http.server` (record the PID, stop by PID — see the global
  kill-by-PID rule) to take the one design look.
- A bare local file needs its own `<meta charset="utf-8">`: the artifact
  wrapper supplies one, the local server does not, and the page's `·`, `×`,
  `π` mojibake'd until it was added.
- Rotor "spin" in a side cutaway is a projection, not a scroll: blade at
  angle θ spans y = r·cosθ, visible only on the far half (sinθ<0), shear from
  the stagger — this reads as real rotation with blades bunching at the rims.
