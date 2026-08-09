# Rung 76 anchor — THE FUEL-DEPENDENT CAP (rung 73 § 11's second seam)

Scored in `docs/rung76-spec.md` § 9. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

The seam, in rung 73 § 6's own words: *`∂required/∂mf ∈ {0,1}` IS A PROPERTY OF THIS LADDER'S
SOLVERS, not of limiters in general. Every cap here is a set-point solve, which is what
collapses "applied-referenced" from a continuum to three readings and makes the hook one line.
**A leg whose cap depended on the fuel it was asked about would be a fourth plant.***

Deferred by rungs 73, 74 **and** 75 (rung 75 § 6's last concession says so). This builds it.

§ 0 is everything measured **before this document existed**
(`M:\claud_projects\temp\rung76\probe_c.py`, `smoke.py`); none of it is scored. § 2A is
**derived on paper** from the inherited laws and is listed as derivation, not scored as
prediction, except where § 9 finds a derivation measured false (rung 72's D5 precedent).
§ 2B is scored. **No Jacobian has been read in any cell.**

---

## 0. WHAT WAS MEASURED BEFORE THIS DOCUMENT EXISTED

### 0.1 Where a fuel-dependent cap can live, and why it is rung 48's leg

Every cap in this family is a **set-point solve**: `_cap_gov` returns the `w` at which
`Tt4(w) = Tt4_max`, `_cap_fuel`'s φ branch the `w` at which `φ(w) = φ_lim`. A set-point solve
is a function of the **state alone** — the fuel it is asked about enters only the bracket — so
`∂cap/∂mf = 0` and rung 73's `{0,1}` follows.

**Rung 48's `Wf/pt3` leg is the one whose law is not a solve.** Its docstring states the law
as an inequality on the fuel:

    Wf  ≤  (1 + margin) · κ_ss(n_H) · pt3                                   [rung 48]

A real `Wf/pt3` limiter **evaluates** that right-hand side from the delivery pressure it senses
— the pt3 produced by the fuel actually burning — and caps. `_sched_fuel` instead solves the
implicit fixed point `w* = cap(w*)`, i.e. it asks *what fuel would be self-consistent with the
pt3 that fuel would itself produce*. **That is a modelling choice, not the schedule.** The two
readings are:

    solve    cap = w*  with  w* = (1+margin)·κ(n_H(w*))·pt3(w*)      — RUNG 48, shipped
    sensed   cap(w) = (1+margin)·κ(n_H(w))·pt3(w)   at  w = mf_app   — THE SCHEDULE AS WRITTEN

**`sensed` adds no constant.** `margin` is rung 48's own already-imposed scalar and κ's shape is
still derived from the plant's own equilibria.

**The φ leg and the governor have no sensed form and keep the solve in every cell** — a floor on
a state is not a formula for a fuel. That asymmetry is the knob's declared **domain**, stated
here and in the spec's § 0, not conceded later: `_cap_law` reaches the accel branch of
`_cap_fuel` and nothing else.

### 0.2 THE ACCEL LEG HAD NEVER BEEN ARMED IN THIS FAMILY, AND THAT RELOCATES EVERY COMPARISON

Rungs 72–75 arm `surge` (rung 49's φ leg) and never `accel`. The accel leg is a **supported**
argument of `integrate_fuel` / `_stator_march` all the way down, so this is a plant the ladder
already has and has never marched — but it means **every cross-rung number is at the wrong
settings**: rung 73 § 5's ledger, rung 74's bill and rung 75's `−160 K` and § 1.3 table were all
taken at `accel=None`. Rung 63's lesson (*check a quoted number was taken at THIS rung's
settings*) applies, so **nothing is differenced against a quoted rung-75 number.** The whole
`windup × cap_law` 2×2 is re-measured here, on one rig, and rung 75's headline is **reproduced**
rather than cited.

### 0.3 `c` IS MEASURED, NOT DERIVED — and it lands strictly inside (0, 1)

    c(w)  ≡  ∂ cap_sensed / ∂ w  =  (1+margin) · d[ κ(n_H(w)) · pt3(w) ] / dw

Central-differenced at `margin = 0.10`, `φ_lim = 0.76`, 18 points along the inherited march:

| | value |
|---|---|
| `c` range over the trajectory | **0.1859 … 0.2133** |
| sign | **positive at every point** |

`c > 0` because more fuel at fixed spool speeds means a hotter `Tt4`, less choked-NGV corrected
capacity, and therefore a **higher** `π_c` (rung 49's own docstring chain, read one station
further). **`c < 1` is a measurement and is not implied by the shipped solver working** — a
bracketing root-finder converges on a sign change whether or not `G = w − cap(w)` is monotone,
so "`_sched_fuel` brackets" buys "a root exists", not `G' > 0`.

**AND THE LINEARISATION IS EXACT TO THE DIGIT ALREADY.** `cap_sensed(w) − cap_solve =
c·(w − cap_solve)` predicts `1.02e−4` at `s = 0.8` where `1.0e−4` is measured. The sign of the
difference therefore **flips with the sign of `mf_app − cap_solve`**, which on this trajectory
crosses at `s ≈ 0.55` — just after the ramp ends.

### 0.4 THE ACCEL LEG BINDS, AND ALL FOUR CELLS MARCH

At `margin = 0.10` the accel cap is below the φ cap at **17 of 18** sampled points, so arming it
makes rung 48's leg the **binding** fuel-side leg and the φ leg stays armed beside it
(min-select one level down, rung 74's `_cap_fuel`). The four inherited cells on the armed plant:

| | `sched` | `applied` |
|---|---|---|
| `none` | marches, 341 pts | **ASSERT, residual 2.898e−03** |
| `track` (`τ_t = 0.05`) | marches, 341 pts | marches, 341 pts |

**The residual is `2.898e−03` — rung 74's own number, to four figures**, so arming the accel leg
leaves the joint IC sweep bit-unmoved and rung 75 § 2's derivation of that number carries here
untouched. Everything rung 75 needs its device for, it still needs it for.

---

## 1. THE PLANT, AND THE FIFTH DECLARED KNOB

`SensedCapTransient(AntiWindupTransient)`. Six states, four clocks, four loops, three actuators
— **every one of them rung 72/73/74/75's, unchanged.** No state, no loop, no actuator, no
reference and no clock is added, and **no constant is added at all** (rung 75 added `τ_t`).

    _cap_law = "solve"    rung 48's set-point solve  — RUNG 75, by the branch not being taken
    _cap_law = "sensed"   the schedule AS WRITTEN, evaluated at `mf_app`  — THIS RUNG

joining `_share_law` (72), `_ref_law` (73), `_lag_coord` (74) and `_windup_law` (75).

**`clip × sensed` is REFUSED, by assert and by name, at `integrate_fuel`.** The clip coordinate
dispatches out of this ladder before `_cap_fuel` is ever called, so a `clip × sensed` march
would silently be rung 73 and be reported as this rung — rung 75 § 0.1's refusal, for the same
structural reason one knob over.

**The sensed cap reads `mf_app = min(mf_sched, w_f, w_r)`** — the fuel actually burning, which is
what a delivery-pressure sensor is downstream of. It is threaded through the one hook
`_cap_fuel(..., mf_app=None)`; `None` is rung 75 exactly.

---

## 2A. DERIVED — worked out on paper. NOT SCORED.

Write `c = ∂cap_sensed/∂mf_app`, `τ_f` the fuel leg's clock, `τ_m` the masked leg's.

**D1 — the droop identity.** `cap_sensed(w) = cap_solve + c·(w − cap_solve) + O(·²)`, because
`cap_solve` is by construction the fixed point of `cap_sensed`. (§ 0.3 already sees this.)

**D2 — THE EQUILIBRIA COINCIDE WHERE THE FUEL LEG HOLDS.** Setting `dw_f/ds = 0` with the fuel
leg authoritative (`mf_app = w_f`) gives `w_f* = cap_sensed(w_f*) = cap_solve`, **exactly**, and
under `applied` identically because `_demand_reference` returns `cap` itself when
`mf_app == w_own`. So the knob is a **pure transient device on the leg that holds**: it changes
the path and not the destination.

**D3 — the authoritative fuel diagonal MOVES, and identically in both references.**
`∂mf_app/∂w_f = 1` there, so `∂target/∂w_f = c` and

    ∂RHS_f/∂w_f  =  ( c − 1 ) / τ_f          [BOTH references]

against rungs 73/74/75's `−1/τ_f`, each of which reports it **moved 0.0 relative**. At
`c ≈ 0.19`, `τ_f = 0.05`: `−20.0 → ≈ −16.2`.

**D4 — the masked fuel diagonal does NOT move.** `min()` is flat in the masked argument, so
`∂mf_app/∂w_masked = 0` and `∂cap/∂w_masked = 0`: rung 73's origin under `applied`, `−1/τ_m`
under `sched`, both unmoved.

**D5 — the masked row's cross moves.** `∂cap/∂w_auth = c`, so the masked row gains `+c/τ_m`
under `sched` (where rung 74 measures exactly `0.0`) and goes `−1/τ_m → (c−1)/τ_m` under
`applied`.

**D6 — `n_live ≤ 3`, a FIFTH time.** The masked **column** is untouched: the cap enters through
`mf_app`, and `min` is flat in what the masked leg holds. `mask_leak = 0.0` exactly.

**D7 — `det J(sensed)/det J(solve) = 1 − c`.** `det J` = masked diagonal × `det`(live 3×3)
(rung 75 § 1.3's own factorisation), the live block is rung 71's with only the authoritative
fuel diagonal moved, so the determinant must scale by exactly `1 − c` — **per point, against
that point's own `c`**, never pooled (rung 73 § 4's pooling failure).

**D8 — the governor's ROW is bit-identical.** `_cap_gov` has no sensed branch, so nothing in
that row can move in any cell.

**AND THE 2×2 WITH RUNG 75 IS THE POINT.** Rung 75's device sits in a leg's **law**, which
min-select masks, so it reaches only the **masked** leg. This one sits in the **plant both legs
read**, which min-select cannot mask because the plant is shared — so it reaches only the
**authoritative** one. `min`'s flatness in the masked state is neither a law nor a plant, and
that is why neither buys a rank.

---

## 2B. PREDICTED — scored in § 9. No Jacobian has been read.

**P1.** `c` at the Jacobian base states lies strictly in `(0,1)` on **both** stator arms
(`StatorLimiter` and rung 69's `StatorIncidenceLimiter`), positive at every point.

**P2.** The authoritative fuel diagonal is `(c−1)/τ_f` to `< 1e−9`, per point against that
point's own `c`, in **both** references.

**P3.** The move is **identical in the two references** to `< 1e−9` relative — a sharp asymmetry
against rung 75, whose masked diagonal is reference-*dependent* (`−1/τ_t` vs `−(1/τ+1/τ_t)`).

**P4.** The masked fuel diagonal is **moved 0.0 exactly** under `sensed`, both references.

**P5.** `mask_leak = 0.0` exactly ⇒ `n_live ≤ 3`, the fifth running.

**P6.** The masked row's cross is `+c/τ_m` under `sched` (rung 74's exact `0.0`) and `(c−1)/τ_m`
under `applied`, both to `< 1e−9`.

**P7.** `det J(sensed)/det J(solve) = 1 − c` per point to `< 1e−9`, in the three cells where
`det J(solve) ≠ 0` (`sched × none`, `sched × track`, `applied × track`).

**P8.** `zeros` is **unmoved in all four cells** — this rung does not revive `det J` and does not
kill it. The exact inverse of rung 75, which moved `zeros` `1 → 0`.

**P9.** The governor's row is bit-identical between `solve` and `sensed` (`< 1e−12`), and where
the **governor** is authoritative the fuel diagonal does not move either.

**P10.** On the marched arm the two trajectories **converge at the tail** (D2): `w_f` under
`sensed` and under `solve` agree to better than `1e−6` relative once the schedule stops and the
fuel leg holds, while differing by ≫ that during the ramp.

**P11 — THE BILL.** During the ramp `mf_app < cap_solve`, so `cap_sensed < cap_solve` and the
sensed leg cuts **harder**: `min φ_lp` **rises** and peak `Tt4` **falls** against `solve`. The
sign is the claim; the magnitude is reported with its `ds` band.

**P12.** `_cap_law = 'solve'` reproduces `AntiWindupTransient` **bit-for-bit** on the
accel-armed plant, in every live cell, by dispatch — and it is **not vacuous**: the same machine
under `sensed` must differ.

**P13.** The findings are not an artifact of `margin = 0.10`: P2/P4/P5/P7's structural entries
hold at three margins, and `c` varies with margin **only** through the operating point.

---

## 3. WHAT WOULD REFUTE THIS RUNG

* `c` outside `(0,1)` anywhere → the contraction story inverts there and D1/D7 are local only.
* The authoritative diagonal **not** moving → the cap is not being read at `mf_app`, i.e. the
  hook is threaded wrong (the failure mode rung 75 § 1.1 calls *a perfect refutation having
  measured nothing*, with the sign flipped).
* `mask_leak ≠ 0` → `n_live` would finally move and the headline is the opposite rung.
* `det J` ratio ≠ `1 − c` → the live 3×3 is **not** unmoved, and rung 71's block would have to
  be re-derived on this plant.
