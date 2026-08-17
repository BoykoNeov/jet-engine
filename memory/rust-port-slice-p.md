---
name: rust-port-slice-p
description: "Slice P (rungs 34/35/36, the port's first ODE) — a deferral note described a branch that does not exist, and two of five injected defects were invisible to 132 bit-exact values"
metadata: 
  node_type: memory
  type: project
  originSessionId: cc5d95b2-526a-4146-b929-a889adee8a25
  modified: 2026-08-17T18:45:11.428Z
---

Slice P ported `SpoolTransient` — the port's **first ODE**, where `N` becomes a state under an
RK4-marched shaft balance. `src/spool.rs` + `ComponentMap::phi_max` + `slice_p_smoke.rs` (132
values) + `rung34/35/36.rs` (25 tests, all 19 Python gates, none deferred) + `spool_oracle.rs`
(**7 300 keys bit-exact**). It consumes the `_solve_turbine` hook phase 5 shipped a phase early.

**I WROTE UP THE SLICE AS SHIPPED AT STEP 2 OF FOUR, AND THE ADVISOR CAUGHT IT.** Five of ten
registered predictions were unsettled and the prose did not say so — two needed a dump that did
not exist, one clause was true by construction, one was ungated, and **prediction 3 had never been
run**: I measured the injection against the SMOKE, not against the rung suites, which is the
comparison the whole discriminator argument is about. **A step's write-up is a claim about what is
gated, and it is only as good as an enumeration over the registered list.** Same failure the
slice-K audit found at the plan level, one level down. Three records also disagreed with each
other at that point (`rung41.rs`'s roster, `rung53.rs`'s ledger, the plan) — *a stale
cross-reference costs more than an uncounted one*, applied to the port's own paperwork.

**A DEFERRAL NOTE CAN BE PLACED PERFECTLY AND STILL BE WRONG ABOUT THE THING IT DESCRIBES.**
[[rust-port-slice-o]]'s lesson was *write the deferral where the next slice's compiler will hit
it* — and both records of `phi_max`'s deferral did exactly that. Both said its rung-53 repair is
an **early return at `vsv == 0.0`, "exactly as `psi` does"**. `psi` has one; **`phi_max` does
not** — it folds the swirl amplitude into three coefficients and branches on none of them. The
quoted assertion was the rung-34 form: correct exactly where slice P lives, wrong everywhere else,
and **unobservable here** (`vsv == 0.0` at all 16 508 measured calls). Placement was right, content
was wrong. Port from the source, never from the note that points at it.

**THE SMOKE DUMP PASSED FIRST TRY, SO EVERY GATE WAS WATCHED TO FAIL — AND TWO COULD NOT.** Five
defects injected into the shipped code, 132 values re-run: the wrong hook table failed 5 of 8
gates, dropping the Illinois down-weighting failed 6, changing the width test failed 6, and
**moving the convergence test ahead of `f(c)` failed 0, as did changing what the exhaustion arm
returns.** My own doc comment had asserted all three delicate details change the returned bits.
Two do not, and both are **COUNT properties**: the reorder returns the identical root and differs
only in whether a residual is evaluated; the exhaustion arm is never reached. Repaired rather than
documented — `illinois_evals`/`illinois_exhausted` counters, gated against PyPy's 227/403/1344/199,
which reproduced first run. Re-injecting the reorder now fails **exactly one** gate.
[[rust-port-slice-n-step5]]'s shape, arriving in the port's own scaffolding.

**A HOOK CAN BE LOAD-BEARING AND DEAD AT ONCE, DEPENDING ON THE BRANCH.** Of the eight value
gates, the wrong-hook injection moves five — but **not the subsonic one**, because `_instant_tail`
solves the choked geometry, dispatches on the nozzle, and on the subsonic branch **re-solves
`pi_t` from nozzle continuity**, discarding the hook's answer entirely. On a subsonic cell the
table could be wired to anything and a value oracle would report agreement. Reinforces
[[rust-port-ladder-architecture]]: the reason to gate a hook by FIRING COUNT is not only slice N's
unreachable-hook case.

**A LATENT ROUNDING DIVERGENCE CLOSED BY CONSTRUCTION.** `int(round(s_end/ds))` is Python's
half-to-EVEN; Rust's `f64::round` is half-away-from-zero. Ties need both operands dyadic
(`1.5625/0.125 = 12.5`), so no shipped grid reaches it — but it would give the Rust march one
extra step and read as a trajectory-LENGTH disagreement with no arithmetic explanation. Fixed with
`round_ties_even`, on `two_spool.rs::round6`'s precedent.

**PREDICTION 3, ONCE RUN, IS SHARPER THAN REGISTERED — ZERO OF NINETEEN.** With rung 31's
bisection swapped in for rung 34's Illinois (~9e-12 apart), **all 19 ported Python gates pass**;
only the bit oracle and the two gates the port ADDED see it. Every gate those suites ship is
written at 1e-6–1e-9, where their physics lives. The argument for an oracle is now a measurement
in this port rather than a principle inherited from it.

**AND THE CPython ARM'S 100 % AT PRE-FLIGHT WAS THE WARNING IT WAS RECORDED AS.**
[[rust-port-phase6-preflight]] got 100 % on a CPG gas and wrote down that it was NOT coverage. On
the thermally-perfect gas the same instrument is the sharpest since slice G: **22.6 % identical,
83.8 % of continuous keys moving.** The tiering inverts slice N's: **530 discrete OUTPUT keys —
branch labels, trajectory lengths, surge verdicts — ZERO differ**, while 22 of 60 census keys
(iteration counts) do. What it converged to is interpreter-invariant; how many passes it took is
not. Two structural-zero families named: `Phi`/`p_net_spec` at equilibrium, and **`E_surge` on a
peaked map, a `max(0, …)` sitting AT its clamp — exactly 0.0 on PyPy, 1.61e-11 on CPython, a
100 % relative difference out of a 1e-11 drift.** A max-accumulator at its floor converts arbitrary
drift into total relative disagreement.

**A FOURTH INSTRUMENT DEFECT, FOUND BY THE GATE IT WAS BUILT FOR.** The first dump came back
7 299 of 7 300 — the miss a COUNT, not a value. The Python wrapper tallied *after* `_illinois`
returned, so an Illinois whose residual raises mid-search (control flow here, not an error) skipped
the tally and discarded that call's evaluations; Rust's in-loop counter kept them. Aligned the
instrument to the port, not the reverse. [[rust-port-slice-m]]'s pattern, fourth instance.

**BOOKED FOR SLICE R, NOT LEFT TO ARRIVE MID-PORT.** `march` de-duplicates Python's two RK4
bodies, which is fine here (identical apart from the closure, and the source argues for no
separation). It stops being free at slice R: the two-spool march has a different signature,
`integrate_fuel` must be a HOOK (11 phase-7 overriders), and two more bodies add a third state.
Fusing them would put a hook's dispatch inside a body slice P's gates already cover — § 5.3's
`_solve_turbine` argument one level up.

**Two source claims measured, one confirmed and one 1.7× light.** The Illinois override finds the
*"same root as the inherited bisection to ~1e-11"*: true of `pi_t` (8.95e-12 over 14 002 paired
calls) and **exceeded by the derived `tau_t` (1.707e-11)**. And rung 34's gate-6 map inverse,
registered as EXACT rather than tight, came back inside **1e-14** against Python's `1e-9` bar —
the tight clause is asserted BESIDE Python's, not instead of it, so a future loosening shows in
the diff. Related: [[rust-port-phase6-preflight]], [[rust-port-inside-outside-exactness]].
