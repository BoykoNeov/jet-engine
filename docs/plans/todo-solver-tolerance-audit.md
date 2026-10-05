# TODO — audit the iterative solvers for absolute-tolerance-below-noise-floor

**Status:** CLOSED NEGATIVE for its own hypothesis — the six `_ETA_TOL` sites (2026-07-23)
and every Rust-era site added since (2026-10-05, last section). **One finding stays OPEN,
awaiting the user's decision: the rung-6 equilibrium Newton's `y >= -80` floor** (§ "Rust-era
re-audit" (c)). Not a rung. A code-health / correctness audit, raised by a real defect found in
rung 43.

## The observation that raised it

Rung 43's build tripped a failure in `TwoSpoolTransient.equilibrium` (rung 40) that
turned out to be **pre-existing and latent for three rungs**:

- `_EQ_TOL = 1e-12` is an **ABSOLUTE** test on the power residual `max(|Φ_L|, |Φ_H|)`.
- The residual's noise floor is **GAS-dependent**: ~1e-14 on CPG, but ~1e-10 on the
  **reacting** gas, because the equilibrium sub-solve inside `_close` cannot resolve
  `Φ` any finer.
- So on the reacting gas the 2-D Newton converged *physically* in ~5 iterations, then
  spun to the 80-iteration cap and **raised** — at `Tt4` = 1300/1400, while
  1500/1450/1200 squeaked under.

**The tell that it was a solver artifact and not physics: it was NON-MONOTONE in
`Tt4`.** Physics does not fail at 1300 and 1400 but succeed at 1200 and 1500.

Fixed in rung 43 by a **best-so-far acceptance AFTER the loop** (`turbojet/engine.py`,
`TwoSpoolTransient.equilibrium`), reached **only** by inputs that previously *raised* —
so rungs 40/41/42 are untouched by construction. The advisor explicitly blocked the
in-loop stagnation exit first reached for: that one can fire mid-descent on a
currently-passing case and change the returned iterate.

**Why it stayed hidden:** rung 40's own suite sampled around the hole.

## The hypothesis to test

The same pattern — an **absolute** residual test gating a `raise`, on a residual whose
floor is set by a *nested* solver whose precision depends on the gas — may exist
elsewhere. Rung 43's own `equilibrium_fuel` was audited and is **safe** (it refuses
equilibrium gases outright, so its floor is ~1e-14), but nothing else was.

## Concrete candidates (already located, not yet audited)

Most tolerances in the package are **RELATIVE** (`abs(f_new - f) <= self._TOL * f_new`)
and are not exposed to this. The exposed shape is *absolute residual + iteration cap +
raise*:

| Site | Constant | Residual | Notes |
|---|---|---|---|
| `engine.py` `TwoSpoolTransient.equilibrium` | `_EQ_TOL = 1e-12` | `max(\|Φ_L\|,\|Φ_H\|)` | **FIXED in rung 43** — the exemplar |
| `engine.py` `TwoSpoolFuelTransient.equilibrium_fuel` | `_EQ_TOL = 1e-12` | same | **AUDITED, safe** — asserts against equilibrium gases |
| `engine.py` `MapMatcher._operating_point` loop (~L1003) | `_ETA_TOL = 1e-11` | `\|η_c,tgt − η_c\|` and `\|η_t,tgt − η_t\|` | rung 32 — **unaudited** |
| `engine.py` `TwoSpoolMapMatcher._hp_eta_loop` (~L2739) | `_ETA_TOL = 1e-11` | `\|η_HPC,tgt − η_HPC\|` | rung 39 — **unaudited** |
| `engine.py` `TwoSpoolMapMatcher._lp_eta_loop` (~L2761) | `_ETA_TOL = 1e-11` | `\|η_LPC,tgt − η_LPC\|` | rung 39 — **unaudited** |
| `engine.py` turbine-η outer loop (~L2820) | `_ETA_TOL = 1e-11` | `\|η_t,tgt − η_t\|` | rung 39 — **unaudited** |
| `engine.py` `TwoSpoolBleedMatcher._lp_eta_loop_bleed` (~L3739, 3795) | `_ETA_TOL = 1e-11` | as above | rung 42 — **unaudited** |

`η` is O(1), so 1e-11 absolute ≈ 1e-11 relative there — looser than the `_EQ_TOL` case,
which is *why* these have not bitten. The question is whether the **reacting** gas's
equilibrium sub-solve propagates a floor above 1e-11 into `η` through `pr_c`/`T_from_h_c`.

The `_illinois(..., tol=…)` call sites are **bracket-width** tolerances, not residual
tests — bisection always shrinks the bracket, so they terminate and are not exposed to
this failure mode. They are out of scope for this audit.

## What the sweep should actually do

1. For each candidate, instrument the loop to record the residual **history**, and run it
   on the **reacting** gas across a `Tt4` sweep — the reacting gas is the one with the
   coarse floor. CPG will not reproduce the bug.
2. Look for the signature: residual **plateaus** well above the tolerance and stops
   descending, while the iterate stops moving. A plateau at 1e-10 against a 1e-12
   tolerance is the bug; a clean descent through the tolerance is fine.
3. Confirm any failure is **non-monotone in `Tt4`** before calling it a defect — that is
   the discriminator between a solver artifact and a genuine physical edge (the
   positive-feedback edge in rung 32's message, for instance, IS real).
4. Where a floor is found, apply the same fix shape: **best-so-far acceptance AFTER the
   loop**, gated so it is reachable only by inputs that previously raised. Never an
   in-loop stagnation exit — that changes returned iterates on currently-passing cases.
5. Re-run `pytest --runslow` and confirm the count is unchanged (the fix must be
   bit-for-bit invisible to every passing point).

## Deliberately NOT part of this

- Not a rung. No new physics, no new spec, no new panel.
- Not a re-tuning of any tolerance. Loosening `_EQ_TOL` globally would weaken the CPG
  paths, where 1e-12 is honest and reachable. The fix is an acceptance clause, not a
  looser bar.
- Not a change to any *relative* tolerance. Those are the correct form and are why the
  bulk of the package is unaffected.

## RESULT — CLOSED, NEGATIVE (2026-07-23). No code change.

**All six `_ETA_TOL` sites are safe. The hypothesis is refuted at every one of them, and
for a STRUCTURAL reason.** No fix was applied — an acceptance branch no input can reach
is untestable dead code, exactly the bar rung 43's own fix was gated against.

### The mechanism (this is the deliverable, not the sweep)

The `_EQ_TOL` bug lived on a residual with **no float64 zero**: `Phi = eta_m*P_t - P_c`
is a physical power imbalance built from equilibrium-noisy enthalpies (`h_t` of a working
gas whose composition re-solves to ~1e-10 each Newton evaluation). `Phi=0` requires that
noise to cancel, which it cannot, so it floored at ~1e-10 and spun to the cap.

Every `_ETA_TOL` residual has the opposite structure: `R = eta_map(eta) - eta` — a
**deterministic fixed-point-map minus the iterate**. For a fixed `eta`, `eta_map(eta)`
returns the *same* value every call (the equilibrium solve is deterministic in its
inputs), so `R(eta)` is a smooth deterministic function with an **exact float64 root**,
and the secant lands on it. Crucially the one reacting-dependent input to the two-spool
loops, `MFP4 = choked_mfp(wgas, Tt4, f)`, is computed **once outside** the loop — a
loop-CONSTANT — so the equilibrium noise enters as a fixed offset that *shifts* the root,
never as per-iteration jitter that *prevents* reaching it.

### The evidence (probes in `M:\claud_projects\temp\solver_audit\`)

Residual histories recorded with faithful high-cap copies that run PAST the natural exit
(the shipped tol exits at the first iterate `<= 1e-11`, so an early-exit measurement reads
the *threshold-crossing* residual, not the floor — a trap this audit fell into once and
corrected):

| Site | Loop | True floor `min|R|` | Notes |
|---|---|---|---|
| 2 | rung-39 `_hp_eta_loop` | **exactly 0.0** | shape-robust (tilted/flow/press/steep), gas-indep (tp≡re) |
| 3 | rung-39 `_lp_eta_loop` | **exactly 0.0** | same |
| 4 | rung-39 turbine outer loop | **~0 / <1e-16** | reaches `<=1e-11` by pass 3; `a_t>0` exercised |
| 5 | rung-42 `_lp_eta_loop_bleed` | **exactly 0.0** | measured on reacting+bleed; the `1/(1-b)` is another loop-constant so gas-indep is *argued* from site-3 structure, not measured |
| 6 | rung-42 turbine outer loop | **~0 (by identity)** | BYTE-IDENTICAL to site 4 (diff: only a comment + raise string); *argued*, not run |
| 1 | rung-32 `MapMatcher.match` secant | **~1e-14, usually 0** | see below |

**Site 1 is the only one with a non-trivial story, and it is still safe.** Its residual
is NOT the clean fixed-point form: `_operating_point` runs an inner `(f,pt4)` loop and
recomputes `choked_mfp`/`pr_c`/`T_from_h_c` (the cold `_solve`, `1e-11` *relative* tol)
each outer step, so `R(eta_c)` carries ~1e-12 jitter and cannot always hit exactly 0.
- **This floor is GAS-INDEPENDENT (`tp ~= re`)** — it is the cold-section `_solve`
  relative tolerance leaking through, **NOT** the equilibrium sub-solve the todo
  hypothesized. So the todo's specific mechanism is refuted here too; this is a distinct,
  milder finding.
- **The true floor is ~1e-14, three orders under the `1e-11` tol.** The secant converges
  superlinearly to machine zero: a typical tail is `7.3e-5, 5.1e-7, 1.0e-11, 1.1e-16,
  0.0` — the iterate *right after* a ~1e-11 value plunges to ~0, so it can never get
  *stuck* above the tol. One input (`surge_pr`, `Tt4=1220`) settles into a period-3 limit
  cycle, but at **~1e-14** — harmless against a `1e-11` bar.
- **Early "9.99e-12 floor" readings were exit-residual artifacts.** The shipped match
  exits at the first iterate `<=1e-11`; for a fast secant that iterate is often ~1e-11
  (one step above the ~0 it would reach next), so the exit residual *clusters just under
  the tol* across every shape — which reads as "thin margin" but is not one.
- **The real match NEVER raises**: `RAISES=0` across 173 fine `Tt4` points x 4 shipped
  shapes x 2 artificial steep/narrow shapes x 2 gases. If any input's floor exceeded
  `1e-11`, the real (tol=`1e-11`, cap=80) match would have raised there. None did.

The discriminator the audit was built to apply — *a raise that is non-monotone in `Tt4`*
— never triggers, because there is no raise. The `_ETA_TOL=1e-11` bar sits comfortably
above every site's floor. The fix shape (best-so-far acceptance after the loop, reachable
only by inputs that previously raised) stays **on file** for a future steeper-map/CFD
map that could genuinely floor a site above `1e-11`; it is deliberately NOT added now.

## Rust-era re-audit (2026-10-05) — the sites added after rung 43

The July sweep covered the six `_ETA_TOL` loops only. After the port, every `did not converge`
in `rust/src/` was censused (by message, then each site read). Probes ran on a scratch COPY of
the crate (`W:\temp\claude\solver_audit\rust`, instrumented there; the repo was never edited and
restored): `examples/probe_r55.rs`, `examples/probe_eq*.rs`, outputs `r55.tsv`, `eq*.tsv`.

### (a) Cleared by structure — no probe needed

| Site | Why it cannot floor-trip |
|---|---|
| Joint initial condition, rungs 66 / 68 / 70 / 72 / 74 (inline, undamped) | Exit `1e-12`, but the assert bar is **`1e-9`** — a 1000x built-in gap, so any floor between the two just runs the cap and passes. Inner solves are ~1e-11 or tighter. And each assert's own message declares a failure a FINDING (the degeneracy / min-select cycle): a best-so-far clause there would hide the signal, so it is out of bounds by design. |
| Joint initial condition, rung 67 (`joint_fixed_point`, damped `w = 1, 0.5, 0.25`) | Same `1e-12` / `1e-9` gap. A stall there IS a solver failure (its message says so), but no gas noise reaches it: the fuel path refuses equilibrium gases (`try_tt4_from_f`). |
| Burner fixed point / rung-31 / rung-38 off-design `f` | RELATIVE tolerances (`<= TOL * f_new`). The equilibrium burner is a bisection on a relative bracket width — it always terminates. The documented hot-throttle non-convergence (rung 31 § 5.4) is physical and stays closed. |
| `stator.rs` / `stator_bleed.rs` / `stage.rs` incidence and bleed solves, `spool.rs`, `state_coordinate.rs` | Bisections with a bracket-width fallback (`|| hi - lo <= 1e-14/1e-15`) or Illinois — out of scope as in July. `split_wall` / `shared_actuator` / `demand_coordinate` `*_TOL`s are comparisons, not solvers. |

### (b) Rung 55's two stacked-efficiency secants — MEASURED, SAFE

The one Rust-era site with July's exact shape: exit bar = raise bar = `ETA_TOL = 1e-11`, on the
two-spool map matcher, which runs on the reacting-equilibrium gas — and rung 55's own tests run
that gas only at `K = 1`, which bypasses these loops. New inside the loop: a 48-pass bisection
for `n` (`try_solve_n`, `N_TOL = 1e-14`).

Sweep: 2 gases x 5 shapes x `K` in {2,3,4,6} x stator {(0,0),(0.2,0.1)} x `Tt4` 800–1550 by 5 K;
each loop's history recorded to the shipped exit, then **60 secant steps PAST it** (July's trap:
the exit residual clusters just under the bar and reads as a thin margin).

| Gas | Loop solves | Raises / aborts / panics | True floor `min|R|` | Wander past exit (max of last 20) | Max iterations |
|---|---|---|---|---|---|
| CPG | 95 472 | 0 / 0 / 0 | 4.4e-16 | 8.9e-16 | 5 |
| reacting-equilibrium | 1 280 276 | 0 / 0 / 0 | 6.9e-15 | **4.1e-13** | 5 |

The reacting gas DOES inject jitter (wander 4e-13 vs 9e-16 on CPG; 389 solves above 1e-13, 11
above 3e-13, all HP spool) — but at its worst 25x under the bar, and the secant crosses the bar
by iteration 5 every time. No fix: an acceptance branch no input reaches is dead code.

### (c) OPEN — the rung-6 equilibrium Newton's floor makes the solve UNSOLVABLE below ~500–580 K

`gas.rs` `try_equil_solve`: Newton on `y = ln n`, converged iff the step `max|dy| * scale < 1e-13`,
200-step cap, then `Err("equilibrium Newton did not converge…")`. After each step every
`y_j` is floored at `-80` (`n >= ~1e-35`). **The step is measured BEFORE the floor is applied.**

Below ~460 K (any `f`), and up to ~580 K as `f -> 0` (the burner bisection's low end samples
`f` down to ~1e-14), the equilibrium `ln n_H` (and at tiny `f`, `ln n_CO`) lies **below -80**
(~-87 at 440 K). Newton asks for the same downward step every iteration, the floor undoes it,
the measured step sticks at ~7, and the solve spins to 200 and raises. Full history at
`(f, T, p) = (0.01, 440, 747 441)`: `dy_H` falls 66 -> 7 while `y_H` walks to -80, then holds at
`6.950` for 140 iterations. Every other species has converged; the residual is the H reaction
relation alone, and the species it concerns is ~1e-35 mol. **This is a solver artifact — the
equilibrium exists and the composition is converged — of a NEW kind: not a noise floor but a
REPRESENTATION floor.** It is monotone in `T` (a clean threshold), so July's non-monotonicity
discriminator would NOT have flagged it.

Measured: 138 of 690 `(f, T, p)` cases fail at `T <= 460` (f 0.002–0.03, p 1e5–2e6); 71 of
1 400 at `T <= 500` (f 1e-5–0.0675); 214 of 1 350 at `T <= 580` (f 1e-14–1e-6). Species at the
floor: H, plus CO at tiny `f`.

**Why it is not a quiet fix.** These raises are LOAD-BEARING (`todo-rust-port.md` § 5.4 (f)):
they move rung 33's low bracket on five cells and produce the rung-33 panel's 440 / 420 K rows
labelled "net thrust <= 0: below thrust-neutral idle" — rows that measured this Newton, not
idle (already booked there as an OPEN `main.py` honesty item for the user). Python raises at the
same points, so any fix deliberately breaks Rust ≡ Python parity at exactly those points and
changes published panel output. **Not yet attributed:** the recorded rung-33 raises at
`Tt4` = 600 / 650 K — above this probe's highest failing `T` (580 K); whether the march's trial
temperatures there sit lower than the cell's label was not checked.

**Candidate fix, NOT applied:** measure convergence on the step actually taken (post-floor), or
drop floored species pushing further down from the step test. On a currently-converging solve
the floor is inactive at exit, so the iterate sequence should be unchanged — to be proven by the
oracle gates, not assumed. Decision: the user's.

### (d) CORRECTION to (c), same day — measured after review (`examples/probe_eq4.rs`, `probe_r33*.rs`)

**(c)'s "every other species has converged" was NOT shown, and is false in general.** The
damping `scale = 1/step` is computed from the PRE-floor step too, so the pinned species throttles
EVERY species' update to ~`1/step` (~14 % of a Newton step at 440 K, ~2 % at 300 K). Re-measured
against the same solve with the floor lowered to -300 (the unfloored answer), on 848 failing
`(f, T, p)` cases (f 1e-14–0.0675, p 3e4–2e6, T 300–900):

| `f` band | Failing solves | Max relative error, any species above 1e-20 mol |
|---|---|---|
| `f >= 1e-3` (real mixtures) | 447 | **5e-4** (OH, a trace radical) — close, not converged |
| `f < 1e-3` (the burner bisection's low end) | 401 | **5.8e4** (H2O, itself a trace there) — far from converged |

So a fix must remove floored species pushing further down from the **damping scale** as well as
from the step test; changing the convergence test alone would exit on a still-damped composition.

**Passing solves never touch the floor: 0 of 1 787** successful solves on the same grid hit it at
any iteration. A fix gated on "a floored species still pushing down" is therefore invisible on
every currently-passing point of this grid — by measurement.

**Attribution through rung 33's REAL matcher** (reacting-equilibrium, `M0` in {0, 0.3, 0.5, 0.85,
1.2, 1.6}, `Tt4` 420–800):
- The panel's 440 / 420 K `SUB-IDLE` rows: the first failure IS this floor (H pinned, on the
  burner's FIRST bisection trial, an ordinary `f`), at every `M0` that reaches the burner.
- With the floor lowered to -300 those rows STILL fail — now at `inverse: root not bracketed`.
  **So a floor fix moves the failure, not the label:** the panel would print the same two rows.
- The 480 K row (floor failures swallowed inside the march at `M0` 0.5 / 0.85): lowered floor
  gives `pi_c 2.3471, F/mdot 27.32`; the published row is `2.347, 27.3` — **unchanged at printed
  precision** (bits not compared).
- **The recorded 600 / 650 K raises did NOT reproduce** on this grid — zero equilibrium failures
  at 600 / 650 at any `M0`. Their attribution stays open (the original census swept 3 gases and
  may have used other Machs).

**Net:** the floor is a real solver defect, but on the shipped panels it changes no printed number
found so far; its cost is a mislabelled failure and swallowed bracket trials. Decision still the
user's.
