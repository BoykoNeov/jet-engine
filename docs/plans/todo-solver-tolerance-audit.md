# TODO — audit the iterative solvers for absolute-tolerance-below-noise-floor

**Status:** open. Not a rung. A code-health / correctness audit, raised by a real
defect found in rung 43.

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
