---
name: sandbox-slice5
description: "Slice 5 burner view built 2026-10-08..10 — process lessons: the error path had its own error (non-ASCII panic message → explain request trapped inside the trap handler → page hung, latent since slice 1); a hang left Chrome running; a plan's timing measured at ONE design"
metadata:
  node_type: memory
  type: project
  originSessionId: 7c463cda-f4fc-42dd-ab88-2a1d5d79ee23
  modified: 2026-10-10T07:08:42.946Z
---

Slice 5 — the *Burner* view (NOx rungs 7–24 as knobs; `rust/src/sandbox_burner.rs`, plan § 13, what
the build found § 13.9). Built 2026-10-08, finished and shipped 2026-10-10. Slices 1–5 cover the
plan's whole list; what comes next is the user's choice. See [[sandbox-direction]], [[sandbox-slice4b]].

Process lessons:
- **The error path had its own error.** The model's JSON reader takes ASCII only; the page's
  `JSON.stringify` does not escape. A panic message with β/Σ/— made the follow-up `explain` request
  trap INSIDE the trap handler, the worker never posted, and the page waited forever. Latent since
  slice 1 (the Design view's equilibrium burner-balance message has `Σ`), unseen because every
  browser check drove an ASCII message, and the native tests called `explain_*` DIRECTLY, never
  through the JSON `call`. **Test the failure path end to end with a REAL message, through the same
  transport the page uses.**
- **A hang is worse than a failure in a test driver.** The drive's `ev` had no ceiling: a hung page
  made Node drop out on an unsettled await, skipping the `finally` that closes Chrome — two headless
  Chromes from 8–9 Oct were still running on 10 Oct. Every await on an external process needs a
  ceiling that turns into a red check.
- **Diagnose a hang by peeking, not waiting**: Chrome's `DevToolsActivePort` file in the run's
  profile gives the port; one `Runtime.evaluate` read the page's state (`bnBusy 1`), and replaying the
  request against the `.wasm` in Node separated "model loops" from "page never hears back" in seconds.
- **A plan's timing taken at ONE design is not a bound.** § 13.2 told the user "at most ~10 s a point"
  for the per-pocket models; over the crash-map box they took a median of 4–24 s and up to ~2 min. Say
  where a timing was measured, or measure it over the box the sliders reach.
- **Coarse grids picked for speed must survive the crash map**: two L1 choices (quench 400 RK4 steps →
  NaN; quadrature 160 → the model's own mean check fails) went back to the model's defaults.
- Bash heredocs on this machine collapse `\\` → `\`: write JS with escapes via the Write tool. A
  `start /b /wait x.cmd` batch needs `exit` as its last line (see [[windows-tooling-file-hazards]]).
