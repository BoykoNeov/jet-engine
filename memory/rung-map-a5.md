---
name: rung-map-a5
description: "Roadmap A5 (2026-11-15, user chose the larger option) — the charts page's rung map is generated from CLAUDE.md + the notes; process lessons from building it"
metadata:
  node_type: memory
  type: project
  originSessionId: 64741af4-cab1-4c90-8f22-bf8db442c888
  modified: 2026-11-14T22:45:49.179Z
---

Roadmap A5 shipped 2026-11-15: `rust/src/rung_map.rs` builds the charts page's rung map at splice time from
CLAUDE.md's rung table / families line / read-only line / scope sentence and the notes' `In one sentence` lines
(placeholder `/*__RUNG_MAP__*/` in `docs/visuals/template.html`). Consequence: **every new rung (and any edit
to a rung row or a note) makes `tests/visuals.rs::visuals_html_is_the_current_splice` fail** until
`cargo run --release -- splice` — then republish the charts artifact ([[visuals-artifact]]). The test's
message says so; CLAUDE.md's Layout line says "Cycle, rung or notes change ⇒ rebuild and republish".

**Lessons.**
- A parser over hand-written prose needs a test that the WHOLE intended range came out plain, not spot pins:
  "rungs 25–108 all plain" is what caught rung 76's bullet written `**Rung 76** found why` (no full stop
  inside the bold), a form the other bullets did not use.
- Chrome's one-shot `--headless=new --screenshot=…` wrote NO file on this machine (with or without a debugging
  port, exit clean). Drive Chrome over CDP the way `rust/sandbox-wasm/browser.mjs` does instead.
- A Node script written through a bash heredoc lost its backslashes again (`W:\temp` → TAB + `emp`): write
  forward-slash paths in scripts ([[windows-tooling-file-hazards]]).
- Follow-up offered, not done: plain one-sentence notes for rungs 1–24 would replace their terse table hooks
  on the page with no code change.
