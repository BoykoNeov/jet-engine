---
name: rung110-wip-handoff
description: "Rung 110 (turbofan flown) paused 2026-11-20 mid-model: bypass stream built INTO the two-shaft loop, compiles clean, UNCOMMITTED on main, untested — what is done, what is next"
metadata:
  node_type: memory
  type: project
  originSessionId: eb011473-7f0f-453f-88f5-c065f3698fd3
  modified: 2026-11-20T02:46:28.327Z
---

**Paused 2026-11-20 at the user's "wrap up for now".** Uncommitted on `main` (deliberately — unverified work is not
committed): `rust/src/two_spool.rs`, `bleed.rs`, `customer_bleed.rs`, `shaft_offtake.rs`, `stator.rs`,
`turbine_cooling.rs`, `two_spool_transient.rs`, `lib.rs`, and the new `rust/src/flown_turbofan.rs`.
`cargo check --release --lib` is CLEAN (no errors, no warnings). NO test has run.

**Built (the user's "one shared loop", see [[rung110-decisions]]):**
- `two_spool.rs`: `Bypass` (alpha, eta_f, pi_fn, eta_m_lp) + `FanHw`/`FanFace`/`FanSplit`/`FanOut`, `FAN_NOZZLE_DRY`.
  `TwoSpoolEngine.fan` + `with_fan` + `refuse_fan`; `run()` adds the bypass term to the LP shaft and scores two jets.
  `TwoSpoolCore.fan` captures A18 from the design flux; A8 at the ACTUAL exit when a fan's core nozzle is unchoked.
  `try_fan_split` (bisection on π_f; α = A18·flux/ṁ_core, ṁ_core from the outer loop's pt4).
  `try_cascade_map_at(…, face)` (`try_cascade_map` = it with `None`); `r39_body` passes the face, rebuilds with α,
  judges the unchoked/below-idle guard on TOTAL thrust (`guard_thrust`); fan + non-flat LP map is refused.
  `try_rebuild` gained a trailing `alpha` arg; `TwoSpoolResult.fan` (`fan: None` in the 4 other modules' literals).
- Refusals: `design_engine.refuse_fan(..)` before every `with_hooks` call (42, 90, 91, 92, 53, 40) and in
  `TwoSpoolMatcher::new`; `core.refuse_fan` in rung 38's `match_point` and `try_cascade`.
- `flown_turbofan.rs`: `FanComponents` (+ `.engine()`), `split_rung109` (D1 polytropic split), `FlownTurbofan`
  (holds a `TwoSpoolMapMatcher` on flat maps; π_f=1/α=0 → `lp_disabled`), `FlownPoint`. Shaft speeds = the matcher's
  own `n_lp_ratio`/`n_hp_ratio` — a DEVIATION from the anchor's D8 wording (√Δh_s), equal on a perfect gas: disclose
  it in the spec.

**Next, in order (advisor's staging):**
1. Run rung38/39/103/104 + `two_spool_oracle` + the 42/90/91/92 binaries: the no-fan path must be bit-unchanged.
2. `rust/tests/rung110.rs`: FIRST gate 3 (fly the design flight + throttle → α, π_f, π_HPC back to 1e-9) on a design
   whose core nozzle is UNCHOKED at design (only check of the A8/A18 capture). Then gates 1,2,4–8, then P1–P8.
   Gate 5 (Example 8-8, pin law) probe: `W:\temp\claude\rung110-flown-turbofan\probe\ex88.mjs`.
3. CLI panel `print_flown_turbofan_table` + rust-owned golden; `rust/sandbox-wasm/build.ps1` (lib.rs changed);
   the full gate; commit + push. Then the spec / CLAUDE.md row / notes / splice commit.
- Open risk: the outer (f, pt4) loop now carries α's lag — watch convergence; `try_fan_split`'s bracket start.
- Windows: the `cmd start /b /wait` wrapper can hang AFTER cargo finishes — read the log, not the tool's timeout.

Kept on purpose: `W:\temp\claude\rung110-flown-turbofan\` (probes, book OCR text, check scripts) — needed for gate 5.
