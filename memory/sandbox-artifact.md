---
name: sandbox-artifact
description: "The sandbox page is PUBLISHED at claude.ai/artifact/Qa5z3vGwo3E8GxuMThDrWD (user's choice 2026-11-25) — republish after every build.ps1; the old cutaway link and the charts page point to it"
metadata:
  node_type: memory
  type: reference
  originSessionId: 19af6582-8bb4-4a85-824c-d17edc6db008
  modified: 2026-11-25T09:44:02.262Z
---

`docs/sandbox/turbojet-sandbox.html` is published at https://claude.ai/artifact/Qa5z3vGwo3E8GxuMThDrWD (icon
`engine`, title from its `<title>`, "Turbojet Sandbox"). The user chose this on 2026-11-25 after a side note pointed
out that the two published notices ([[cutaway-artifact]], [[visuals-artifact]]) sent link-only readers to a file
they could not open.

**Rebuild is half the loop:** after `rust/sandbox-wasm/build.ps1` + commit, republish with that URL as `url` (from
another session; a publish without it mints a third page). Checked 2026-11-25 in the user's Chrome: the model
runs in the published frame (Worker + WebAssembly allowed), Design and Engine views both draw.

**On the published page:** only a plain `#name` hash survives the artifact link, so the page's JSON shared-setup
address (`#{"view":…}`) cannot be passed through a claude.ai link — the notices say "open it and choose Engine". The
page's relative link to `../visuals/turbojet-visuals.html` does not resolve online; the header carries the charts
page's published link beside it.
