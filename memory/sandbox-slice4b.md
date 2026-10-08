---
name: sandbox-slice4b
description: "Slice 4 (B) Controls view built 2026-10-08 — process lessons: the plan's own trajectory classifier was a proxy; a near-touching tolerance hid a mixed category; a browser wait matched another view's result; perfect gas is not closed form"
metadata:
  node_type: memory
  type: project
  originSessionId: 2adb0ace-421a-41d2-870d-b49a5ddc7391
  modified: 2026-10-08T02:05:10.295Z
---

Slice 4 (B) — the two-shaft *Controls* view (`rust/src/sandbox_controls.rs`, plan § 12.10), built
2026-10-08 after the user said "Go" to the recommended next step. Slice 4 is complete; next in the
plan is slice 5 (the combustor). See [[sandbox-slice4a]], [[sandbox-direction]].

Process lessons:
- **My own plan said "classify stops from the trajectory" — a proxy.** Re-running the failed step
  through the public calls and recording WHICH call failed revealed a stop kind the plan never named
  (the march's every-step check at the full scheduled fuel fails while a limiter holds the fuel far
  below it). Slice 4A's lesson again: read the real failure before wording a cause.
- **A tolerance that separates two classes "by a hair" means one class is mixed.** "Which limiter
  holds" first measured holding ≤ 9.9e-8 vs not ≥ 1.0e-7; counting only limiters acting WITHOUT a lag
  on their route gave 4.8e-15 vs 3.2e-6. Look for the category being mixed in before nudging the bar.
- **A browser-test wait matched another view's result** (a Controls result also has `thrust`, as a
  list): identify a result by a field only it has (`stations`).
- **"Perfect gas" ≠ closed form**: a march of iterated solves differs browser vs native at the solver
  tolerance, so it needs its own bar; a difference of two near-equal numbers is compared absolutely.
- Running cargo in this repo from the Bash tool: the path's space breaks `cmd //c "... --manifest-path"`;
  use a PowerShell wrapper under `W:\temp\claude\jet-ctl\` launched with `start /belownormal`
  (PowerShell eats a bare `--`; the wrapper maps a stand-in token).
