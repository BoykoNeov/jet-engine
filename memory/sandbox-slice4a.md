---
name: sandbox-slice4a
description: "Slice 4 (A) \"Slam the throttle\" built 2026-10-07 — process lessons: a stated cause never seen, a proxy classifier, replaying a march's dropped error, SANDBOX_SHOTS is a path"
metadata:
  node_type: memory
  type: project
  originSessionId: 72c63ec1-a167-4990-a47c-c49c8a076ddb
  modified: 2026-10-07T08:25:30.641Z
---

Slice 4 (A) — the throttle slam inside *Fly it* (`rust/src/sandbox_transient.rs`, plan § 12 / § 12.9),
built 2026-10-07 after the user answered § 12.8 (all four as recommended: both engines, slam first;
equilibrium refused; Controls perfect-gas-only with the limiter set + ONE airflow lever; time in τ).
Part (B), the two-shaft *Controls* view, is next. See [[sandbox-direction]], [[sandbox-slice3]].

Process lessons:
- **I wrote a stop's cause into the committed plan without ever seeing its message** ("the gas tables")
  — my probe had replayed only k1/k2 of the failing RK step. The advisor caught it; the full replay
  (k1–k4, then the next k1) showed the real cause (rung 35's `f_cap` = 0.05). Replay the WHOLE step
  before naming a cause.
- **A classifier built on a proxy hid a case.** "Burner fails because trial Tt3 > commanded Tt4" missed
  a trial 3 K BELOW the command (a near-zero rise the burner also cannot close), which then looked like
  a genuine burner failure. Replacing the proxy with an exact copy of the model's first trial
  (`low_wall_trial_fails`, held to `try_close_compressor`'s outcome on a grid with both answers) made
  the "unexplained" class vanish. When a classification compares a stand-in quantity, ask whether the
  model's real test is that comparison.
- **A marcher that `break`s drops its error** — the sandbox replays the failed step through the public
  instants, and a test holds the replay bit-equal to every recorded step, so the reason belongs to the
  step that actually failed.
- **`SANDBOX_SHOTS` is a FOLDER path**: `=1` wrote 15 PNGs into `W:\Claude_projects\jet engine\1\`.
  Use `SANDBOX_SHOTS=W:\temp\claude\<folder>`.
- A crash map run BEFORE a new pre-check yields drivers the pre-check now refuses (the stop examples had
  ramp 0): rerun the map after every rule change before mining it for test drivers.
