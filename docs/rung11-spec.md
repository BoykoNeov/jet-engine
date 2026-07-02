# Rung 11 — The physical mixing model: a jet-entrainment quench

Rung 10 resolved the quench in **time** and showed a rich primary's low-NOx benefit is
*contingent on a fast quench* — but "fast" was a **free knob** (`τ_q`) with an arbitrary **linear**
mixing schedule. Rung 11 asks the next question: **what physically sets the quench rate?** In a
real combustor the dilution air enters through **jets in crossflow**, and the rate at which those
jets penetrate and entrain the hot core is governed by the jet **momentum-flux ratio**
`J = ρ_j U_j²/(ρ_c U_c²)`. Rung 11 **retires both rung-10 quench knobs**: `τ_q` is **derived** from
`J` (the "quick" in quick-quench becomes "a high-momentum jet"), and the linear schedule is
replaced by a **decelerating entrainment** shape. The τ_q rung 10 swept is now **read off the jet
design**.

> **Read `docs/rung10-spec.md` first**, and `docs/plans/rung11-anchor-mixing.md` (numbers-before-
> code: the `√J` penetration scaling, the derived-`τ_q` law, the entrainment schedule, the
> machine-checked worked example — the exact reduce, the monotone J-sweep, the schedule-shape
> effect — and **the mean-field/monotone limitation** honestly carved out). This file states only
> what *changes*; the Zeldovich rate constants, the clamp-free integrator *form*, the
> τ_q-independent trajectory, the two-zone `zoned_nox` scaffold, the `a6`/`a7`/`Kp`/`_equil_solve`
> substrate, and the "derive before you code" / conservation-assert contract all carry over
> **unchanged**.

---

## What rung 11 adds (and what it deliberately does not)

**Adds:**

- **A jet-mixing config** (`JetMixing`) — the momentum-flux ratio `J` (the design knob) plus the
  dilution-zone geometry/flow (`H`, `U_c`), an entrainment constant `C_e`, and the entrainment
  shape exponent `shape_n`. Physical defaults, so the one knob a caller turns is `J`.
- **A derived quench time** — `τ_q = H/(C_e·√J·U_c)`, monotone-decreasing in `J` (a higher-momentum
  jet penetrates and entrains faster). Replaces rung 10's free `τ_q`. For physical `J` this lands in
  the RQL sub-ms–few-ms band; the **absolute** value is un-pinned (`C_e`/`H`/`U_c` are
  order-of-magnitude), but the **√J scaling** and the **monotone direction** are certified.
- **A decelerating entrainment schedule** — `β(t) = 1 − (1 − t/τ_q)^n`, reaching `β=1` exactly at
  `t=τ_q` (no endpoint trap). `n=1` is the linear/constant-entrainment limit (= rung 10); `n>1`
  (default 2) is the physical decelerating shape (fast near the jet, slowing as the concentration
  gradient collapses). Replaces rung 10's hard-coded linear schedule.
- **A schedule-aware quench integrator** — the **same** clamp-free reverse-rate Zeldovich as rung
  10, but with **β decoupled from time**: the trajectory is indexed on `β = schedule(t/τ_q)`, and
  the RK4 steps `dt` in **real time**. (Rung 10's `tf = t/τ_q` doubled as the β index *only* because
  its schedule was linear; conflating them under a nonlinear schedule is the bug that would silently
  reproduce rung-10 behaviour under a rung-11 label.)
- **The payoff** — a **monotone J-sweep**: EI_NO_quenched falls smoothly as `J` rises (a stronger
  jet quenches faster → escapes the stoich peak → less re-made NO). And a **schedule-shape** result:
  a realistic decelerating entrainment clears the *early/low-β* stoich crossing faster than linear,
  so rung 10's linear schedule was **conservative** (over-predicted the spike).
- **`main.py` panel + `NOTES.md` section + `tests/test_rung11.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the jet-mixing quench is opt-in via
  `mixing`. Every cycle station is **bit-for-bit rung 6** (the whole rung 1–10 suite stays green).
  The reduce is a **short-circuit**, not an empirical limit: `mixing=None` runs the exact rung-10
  path (and `tau_q=None` too → the rung-9 ideal quench).
- **Model spatial unmixedness / a mixing OPTIMUM.** This is a **MEAN-FIELD** model — a single
  well-mixed core diluting on a mean `β(t)`. It can only produce the **monotone** story (higher J →
  less NO); it **cannot** produce a mixing optimum, because an optimum is a *variance* effect (an
  over-penetrating jet leaves an under-mixed hot near-stoich core). The classic **Holdeman**
  dilution-jet optimum `C=(S/H)√J ≈ 2.5` is a *uniformity* criterion — it does **not** fit a
  mean-field model, and is the **deferred rung-12 seam**. We do **not** dress a mean-field rung in
  Holdeman-optimum clothing.
- **Anchor `τ_q` or the schedule.** `C_e`, `H`, `U_c`, `shape_n` are un-anchored / order-of-
  magnitude, like `α`, `φ_p`, `τ`, `τ_q` — the √J *scaling* and the monotone *direction* are the
  certified content, not a book τ_q.
- **Add super-equilibrium O / prompt (Fenimore) NO** — still deferred (rung-7 seam); the spike stays
  an equilibrium-O lower bound (carried from rung 10). Held **φ_p ≤ 2.0** (soot bound, rung 9).

---

## The mean-field limitation (stated loudly — it is the whole altitude choice)

Rung 11 collapses jet **penetration** + shear-layer **entrainment** into a single mean mixing rate
`U_mix = C_e·√J·U_c`, and models one well-mixed core. So it uses **"slow mean mixing" as a surrogate
for a variance effect**: in a strict mean-field picture `J` governs cross-plane *uniformity* more
than the *mean* addition rate; the real reason a weak jet is bad for NO is the un-mixed hot core
(spatial variance). Consequences, both deliberate:

- The J-sweep is **monotone by construction** — there is no mixing optimum here. That is not a bug;
  it is the ceiling of a mean-field model, and it is the honest minimal rung that **retires the
  knobs**. The optimum (EI_NO minimized at Holdeman `C≈2.5`, rising on *both* sides) needs the
  variance seam → **rung 12**.
- `C_e` is the collapse constant — **exposed and disclaimed**, not tuned. The absolute `τ_q` rides on
  it and is therefore un-pinned; the √J *direction* does not.

---

## The equations — a derived time + an entrainment schedule, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8/9/10 flow; rung 11 only
changes *where τ_q comes from* and *the β↔t map* when a `JetMixing` is passed:

```
IDEAL quench      (mixing=None, tau_q=None):  freeze NO at the primary value       → exact rung 9
LINEAR quench     (mixing=None, tau_q>0):     β = t/τ_q, rung-10 _quench_no          → exact rung 10
JET-MIXING quench (mixing=JetMixing(J,...)):  τ_q = H/(C_e·√J·U_c)                   → rung 11
   schedule:  β(t) = 1 − (1 − t/τ_q)^shape_n              (β=1 at t=τ_q; n=1 ⇒ linear)
   integrate (RK4, real time, clamp-free):  index the trajectory on β = schedule(t/τ_q)
                                            d n_NO/dt = 2 R1 (1 − a²)/(1 + β_arr·a) · V
   trajectory (T, [O],[N2],[H],[NO]_e vs β) is REUSED verbatim from rung 10 (τ_q-independent)
   freeze at t=τ_q (β=1);  EI = 1000·n_NO·M_NO/(n_fuel·M_CH2)
```

- **`mixing` and `tau_q` are mutually exclusive** — assert one-or-the-other (like the
  isentropic/polytropic knob). `shape_n=1` in a `JetMixing` reproduces the rung-10 linear path at
  the derived `τ_q`, **bit-for-bit**.
- **Standing asserts (rung-11 deltas):** the rung-7 **K-check** + **trace guard** still bind at
  every trajectory `T` (reused unchanged); `J > 0`, `H > 0`, `U_c > 0`, `C_e > 0`, `shape_n > 0`;
  the derived `τ_q > 0` (guaranteed by the positivity guards); `n_NO ≥ 0` (negatives only — **no**
  equilibrium cap, carried from rung 10).

---

## Verification gates (priority order)

1. **Reduce-to-rung-10 (load-bearing, exact by construction).** `mixing=None` short-circuits to the
   rung-10 path *before* any jet-mixing code — so every existing call is **bit-for-bit rung 10**
   (hence rung 9, hence rung 6; the whole rung 1–10 suite stays green, untouched). Second level: a
   `JetMixing(shape_n=1)` matches the rung-10 `_quench_no` at the derived `τ_q` bit-for-bit (identity
   schedule, same `nsteps`).
2. **The monotone J-sweep (THE lesson).** EI_NO_quenched **falls monotonically** as `J` rises (a
   stronger jet quenches faster and re-makes less NO). Load-bearing physical direction — a wrong
   sign fails it.
3. **`τ_q ∝ 1/√J`.** The derived time scales as the inverse square root of the momentum-flux ratio
   (4× the momentum halves `τ_q`); it stays in the RQL sub-ms–few-ms band for physical `J`.
4. **The schedule-shape discriminator.** At fixed `τ_q`, a decelerating schedule (`shape_n>1`) makes
   **less** NO than linear (`shape_n=1`), because NO is re-made at the *early/low-β* stoich crossing
   which a decelerating entrainment clears fast. (Also: `shape_n=1` == rung 10 at that τ_q.)
5. **Cycle untouched.** Re-running the cycle after a `JetMixing` `zoned_nox` call leaves station 4
   bit-for-bit (pure diagnostic).
6. **Clamp dormancy persists + K-check binds** along the trajectory (`max_a < 1`; reused from rung
   10).
7. **The mean-field ceiling is a documented invariant, not a hidden one** — the J-sweep is monotone
   (no optimum); a test asserting monotonicity *is* the statement that the optimum is out of scope.

## Conservation asserts (rung-11 deltas)
Carry over rung 6/7/8/9/10's, plus: `mixing`/`tau_q` mutual exclusivity; the `JetMixing` positivity
guards (`J,H,U_c,C_e,shape_n > 0`) ⇒ derived `τ_q > 0`; the schedule reaches `β=1` at `t=τ_q` (no
endpoint trap). The clamp-free integrator still guards **negatives only** (no equilibrium cap).

## Done when
Reduce-to-rung-10 holds exactly (short-circuit; rungs 1–10 green, untouched; cycle bit-for-bit rung
6); the J-sweep EI_NO falls monotonically and `τ_q ∝ 1/√J`; the decelerating schedule makes less NO
than linear at fixed τ_q; the mean-field/monotone limitation is asserted (no optimum) and documented
as the rung-12 seam; the K-check/clamp-dormancy gates hold. `main.py` gains a rung-11 jet-mixing
panel (J-sweep: derived τ_q, EI_NO falling; the schedule-shape contrast); `NOTES.md` gains a rung-11
section (what sets the quench rate; the mean-field ceiling); `CLAUDE.md` scope + rung table +
deferred seams updated (physical jet-entrainment mixing done — with the **unmixedness / Holdeman
optimum** as the explicit rung-12 seam, plus super-equilibrium O / prompt NO and the
frozen-vs-equilibrium nozzle still carved out).

## The rung-12+ seam (keep it additive)
Rung 11 derives the quench *rate* from jet momentum but on a **single well-mixed core** (mean-field).
Next seams, all still additive on this substrate: (a) **spatial unmixedness** — track at least a
two-stream split (a near-stoich core that dwells longer than the mean) so the real **Holdeman
dilution-jet optimum** `C=(S/H)√J ≈ 2.5` appears (EI_NO minimized at an optimal spacing/momentum,
rising on both sides) — the variance effect this mean-field rung defers; (b) **super-equilibrium O /
prompt (Fenimore) NO** — the richer radical pool in the mixing shear layer, above this
equilibrium-O lower bound; (c) the still-open **equilibrium-vs-frozen nozzle expansion** (rung-6
seam). Only *where*, *on what pool*, *how fast*, and *how uniformly* the chemistry runs changes.
