---
name: rung107-decisions
description: "User scope decisions 2026-11-10 for rung 107 (rung 106's seam: the governor beside rung 52's realistic release): rung 52 AS BUILT — fast-attack/slow-release lag on the accel schedule + stall floor, redline and metal cap instant on top; ALL governor holds/laws"
metadata:
  node_type: memory
  type: project
  originSessionId: 7858969d-53fe-4610-b043-797889a92dd6
  modified: 2026-11-10T08:26:51.491Z
---

User asked "Rung 52's more realistic release: the fuel limiter letting go slowly instead of all at once - work on
this" (2026-11-10) = rung 106's first named seam (the page refuses governor + realistic release,
`sandbox_controls.rs` precheck "The governor runs with the instant limiters only…").

A side note flagged before the question: rung 106's sag came from the TEMPERATURE limiter's lag; rung 52's lag is on the
accel schedule + stall floor, the redline instant on top — so the sag may not appear. Asked (AskUserQuestion):

- **Which limiter: rung 52 AS BUILT** (recommended) — schedule + floor carry the asymmetric lag, redline and rung 95's
  metal leg instant; the page's "Realistic release" switch stops being refused beside the governor. NOT the
  two-speed temperature limiter (that stays a possible later seam).
- **Holds: ALL** of rungs 95/96 (as 106), each speed law's step edge measured.

Stated, not asked: beside 106's temperature lag / 88's slow valve / 101's lagged sensor stays refused (two lags);
106's minimum-fuel hold carries over on the governor path; hot-start / floor-above-start rule waits for the probe.
Advisor discriminator: count recrossings of `required − g` (rung 52's self-pinned trigger rested on n_recross = 1).
Related: [[rung106-decisions]], [[rung106-governor-lag]].

**Probe (`W:\temp\claude\rung107-probe`, reach.txt — 560 governed runs, 7 laws × 5 cases × 4 maps × 4 lag pairs):**
prototype open path ≡ page rung-52 route bit for bit (64/64, check52.txt); pre-engagement ≡ rung 95 (560/560); cost ≈
rung 95's; every re-attack (4) and every lean-wall hit (24 of 25) is a flat-LP floor run whose START is below the
floor (asked for a cut at s = 0); the stops are the floor out of reach (rung 95 stops there too, earlier).

**Floor above the start (asked 2026-11-10): REFUSE IN WORDS** — governor + release with the floor's cut already asked
for at s = 0 (rung 106's hot-start principle); the governor-off route keeps showing it; lean hold a safety net.

**Knob floors beside the governor (asked 2026-11-10, after the fine step sweep `edge_*.txt`): LIMIT + NUDGE** — attack
≥ 0.2, release ≥ 0.1 (every cell clean < 1e-4 over 3 cases × 3 flights × 7 laws at rung 106's law edges; faster is
rough 1–3e-4, scattered); refused in words beside the governor; switching governor / release moves an out-of-range knob
to the limit with a note (rung 106's pattern); typed values meet the refusal. Governor off: rung 52's one-step floor.

**BUILD STATUS (session paused by the user's "wrap up", 2026-11-10) — UNCOMMITTED in the working tree, on purpose**
(the template changed but the built page is not rebuilt, and the full gate has not run, so not on `main`):
- Anchor COMMITTED + pushed (7b15c6b). Built since: `rust/src/governor_release.rs` (the frozen device), `lib.rs` entry,
  `governor_lag.rs` (`Demand::try_fuel` → `pub(crate)`, nothing else), `sandbox_controls.rs` (`Route::GovernorRelease`,
  `release_lag`/`gov_rel_der`/`gov_rel_step`, march/replay/stop/holders/route string, the old refusal LIFTED; new
  refusals: attack < 0.2 / release < 0.1 (`GOV_REL_*`), speed laws past rung 106's edges, beside the exhaust sensor,
  the floor asked for at s = 0; 4 check requests), `tests/sandbox_controls.rs` (route arm), `docs/sandbox/template.html`
  (attack slider max 0.2 → 0.5, `ctl-relnudge` note, `relNudge()` + `nudges()`).
- `rust/tests/rung107.rs`: 8 construction gates ALL PASS (`--test rung107`): open ramp ≡ rung 52 (64 runs), pre-
  engagement ≡ rung 95, pinned trigger, ledger (worst 9.5e-4 schedule / 5.4e-3 floor — = ledger.txt), τ→0 (order ~0.96),
  re-run, refusals. ⚠ `the_governed_lean_hold_holds_at_the_wall` is VACUOUS (0 holds of 126) — find a holding case or drop it.
- NEXT: score H1–H6 (a `score.rs` in the probe crate on the SHIPPED `gr::march`; archive it), pin in rung107.rs; crash
  map 1 000 (rung 106's crash.rs); panel `print_governor_release_table` + golden + cli_golden/claude_md_reference;
  browser check for the nudge in `rust/sandbox-wasm/browser.mjs`; `build.ps1`; spec, CLAUDE.md row (≤350 B) + scope/
  families lines, sandbox-plan, sandbox README (line ~147 governor paragraph); process-lesson memory; gate; commit+push.
- `W:\temp\claude\rung107-probe` (56 MB) KEPT: the scoring step needs the crate; its scripts/outputs are already archived
  in `docs/plans/scratch-archive/rung107/`. Delete it when the rung ships.
