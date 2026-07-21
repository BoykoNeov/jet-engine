---
name: rung34-spool-transient
description: "SHIPPED rung 34 = the spool transient; N becomes a STATE (first DYNAMIC rung); finding is the two-timescale ratio tau_fuel/tau_spool NOT the tautological \"I-independent shape\"; compressor map runs FORWARD; added linear loading slope l to ComponentMap"
metadata: 
  node_type: memory
  type: project
  originSessionId: db3e7625-c4ab-4bfb-bb1c-27dcbde4f46c
  modified: 2026-07-21T11:46:36.262Z
---

SHIPPED rung 34 — **the spool transient** (`SpoolTransient` in `turbojet/engine.py`,
`docs/rung34-spec.md`, `docs/plans/rung34-anchor-spool-transient.md`, `tests/test_rung34.py`,
`main.py` `print_spool_transient_table`). The first **DYNAMIC** rung: `N` becomes a **state** under
the shaft-inertia ODE `I·ω·dω/dt = η_m·P_t − P_c`; rungs 31–33 *computed* `N`, rung 34 *evolves* it.

**The structural move:** the compressor map runs **FORWARD** (rungs 31–32 ran it backward via
`solve_n`). Given corrected speed `n(N,Tt2)` and trial flow `m`, the Euler speed line gives `τ_c`
directly; the **NGV choke closes `m` on EITHER branch with NO shaft balance** (`pt4=π_b·π_c·pt2`
doesn't involve the turbine — the key simplification). Turbine expansion: rung-31 `(★)` when choked,
nozzle-continuity when subsonic (rung-33 dispatch reused). Leftover power → `dν/ds` in nondim time
`s=t/τ_spool`, `τ_spool=I·ω_d²/P_ref`.

**THE FINDING is the advisor's correction of the obvious framing.** "Shape is `I`-independent, `I`
only sets the clock" is a **TAUTOLOGY** in a 1-state model (dimensional analysis — fails the
rung-29-gate-2 / rung-33-gate-4 anti-tautology bar). `I` is load-bearing only when a **second clock
competes**: ramp `Tt4` over finite `τ_fuel`; the peak accel excursion above the running line is
`E(r)`, `r=τ_fuel/τ_spool` — max at `r→0` (the constant-`N` displacement, an **algebraic map
property**, +5.4%), vanishing as `r→∞`, knee at `r≈1`. That's *why real engines schedule fuel ramps*.
The `E(r)` shape is `l`-independent (holds for any map); only the *direction sign* needed `l`.

**Sub-finding / a real map fix:** rung 32's loading law `ψ=1−σ(φ−1)²` PEAKS at design, so run
forward it gives the **wrong surge-side slope** (accel excursion came out −5.73%, `π_c` falling toward
low flow — non-physical). Added a **linear slope `l`** to `ComponentMap` (`ψ=…−l(φ−1)`, `dψ/dφ|_1=−l`,
**default 0 ⇒ rung 32 bit-for-bit**; `solve_n` now calls `psi()`, identical at l=0). Surge-realistic
shapes `surge_flow/pressure/tilted` turn it on. Gate 4 asserts the parabolic (l=0) map gives the WRONG
sign — so `l` is a *tested discovery*, not tuning.

**Reduce:** the equilibrium (`Φ=0`) reproduces `OffDesignMatcher.match` (flat, rung 31) /
`MapMatcher.match` (shaped, rung 32) via the **forward closure ONLY** (never calls the matchers ⇒
non-circular), machine-zero at design, ≤1e-8 on the sweep incl. a subsonic point. Spool-down crosses
choked→subsonic at M9≈1 toward rung-33 thrust-neutral idle (a too-fast fuel chop hits the flameout
boundary — integrator stops). Perf: added module-level `_illinois` (fast bracketed root) + a
`SpoolTransient._solve_turbine` Illinois override (`_instant`≈4ms, was 11).

Disclaimed: `I`+`ω_d` = one clock group (only `ν(s)`/`r` claimed); quasi-steady components (no
combustor volume-filling/heat-soak); `Tt4(t)` control; no surge line (rung-32 concession). Leaves
open: volume-filling/heat-soak clocks below `τ_spool`, `ṁ_fuel(t)` schedule, a surge line, two-spool.
Cross-links: [[rung33-subsonic-matching]] (the handshake), [[rung32-component-maps]] (the map + the l
fix), [[rung31-offdesign-matching]] (the reduce anchor), [[rung29-shifting-turbine]] (the disclaimed-knob
+ bound-first method precedent).
