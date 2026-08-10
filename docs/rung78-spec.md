# Rung 78 — THE RESIDUAL GAUGE

Rung 72–77's six states, four clocks, four loops and three actuators — **every one of them
unchanged**. This rung adds **one swept knob and no constant**, and it closes rung 77 § 9's
second seam by refuting its premise.

    accel (48)   G_a(w) = w − cap(w)                      G_a′ = 1 − c
    gauged       G_k(w) = w − [w0 + k·(cap(w) − w0)]      G_k′ = 1 − k·c      w0 := the k=1 root

Rung 77 § 3 lists two routes to a singular set-point solve (`dw*/dq = −G_q/G_w` with `G_w → 0`)
and calls the first — `c → 1` — *unreachable in this family*: `c ≤ 0.2234` over 24 cells. Rung 77
§ 9 then asks for a schedule that could reach it. **This rung reaches it in one line, at a fixed
set point, and finds that arriving costs nothing.**

> **HEADLINE — A RESIDUAL'S SLOPE IS A GAUGE; ITS ROOT'S UNIQUENESS IS NOT.** `cap_k(w0) = w0`
> identically, so the set point is invariant *by construction*, while `G_k′ = 1 − k·c` is a free
> dial: measured `+1.5 … −2.0`, through zero, agreeing with `1 − k·c` to `1.18e−08`. And
> `dw*/dq` **does not move** — `7.44e−08` over the whole sweep, against a `k = 1` value of
> `−2.6428e−04`. The `1 − k·c` that vanishes in the denominator vanishes in the numerator at the
> same `k`, because `∂cap/∂q = w0′·(1 − c)` forces `G_q` to scale in lockstep. **So rung 77 § 3's
> first route is a REMOVABLE singularity — `1/(1−c)` is a GAUGE, a number you may set to
> anything including infinity without the plant noticing.**

> **AND THE SINGULARITY LEAVES ITS TRACE WHERE NOBODY WAS LOOKING.** The gauge preserves the
> root (`|G_k(w0)| ≤ 1.08e−16` on the plant) and destroys its **UNIQUENESS**: a second root
> sweeps in from `0.574·w0`, collides with the true one exactly at `k·c = 1`, and departs to
> `2.51·w0` — root counts `{1, 2, 4}` over a band `k·c ∈ [0.9, 1.2]` that brackets the singular
> gauge. Inside it a solver converges **cleanly onto the wrong root** (`ok = True`, set point
> moved `61%`). **What `c → 1` costs is not sensitivity; it is WELL-POSEDNESS** — invisible to
> `dw*/dq`, which is why rung 77 could measure the limit correctly and still describe it wrongly.

> **AND RUNG 76 SURVIVES, FOR THE ONE REASON THIS RUNG ISOLATES.** `solve` → `sensed` moves the
> root by `2.83e−02 … 1.35e−01`. **A re-writing that MOVES the root is a device; one that
> PRESERVES it is a change of coordinates**, and rung 76 § 3's `1.228 … 1.246` is the former.
> That is the whole difference, and neither rung could see it alone.

Pre-registration: `docs/plans/rung78-anchor-residual-gauge.md`. Gates: `tests/test_rung78.py`.

---

## 0. WHAT MOVED, AND WHAT DID NOT

| | rung 76 | rung 77 | **rung 78** |
|---|---|---|---|
| the loops | four | the same four | **the same four** |
| states / actuators | 6 / 3 | 6 / 3 | **6 / 3** |
| what is added | a second READING of a cap | a READING of all three legs | **a one-parameter REWRITING of one leg's residual** |
| new constants | 0 | 0 | **0** (`k` is swept, `w0` is the plant's own root) |
| new knobs | `_cap_law` | none | **`_gauge_k`** |
| plant code touched | the plant both legs read | none at all | **one branch of `_cap_fuel`** |
| the reduce | by dispatch | by construction | **by dispatch** (`k == 1.0` → `super()`) |

### 0.1 It is a DIAGNOSTIC DEVICE and says so

`cap_k` needs the `k = 1` root as its anchor, so it is not a limiter anyone could build. It is
the one-parameter family that holds a set point fixed while sweeping its residual's slope —
exactly what is needed to tell a gauge from a device, and nothing here is proposed as hardware.

### 0.2 The settings are rung 77's, taken verbatim

`φ_lim = 0.80`, `margin = 0.10`, `Tt4_max = 1200 K`, all clocks `0.05`, `ds = 0.005`, read at the
`_riding4` points of the `clip | sched | none | solve` march, every 8th — so § 1's `k = 1` column
**is** rung 77 § 1 and can be differenced against it without rung 63's lesson biting.

### 0.3 `sensed × k ≠ 1` is REFUSED

The gauged branch bypasses `_sensed_cap`, so under `_cap_law = "sensed"` the cap would be
evaluated at the trial fuel instead of the applied one and the march would silently be rung
78-on-rung-75 while being reported as rung 78-on-rung-76. Refused by assertion — rung 76 § 0's
own refusal of `clip × sensed`, one knob over.

---

## 1. MEASURED — THE SLOPE IS A FREE DIAL, AND THE SET POINT DOES NOT CARE

10 riding points, `k` swept in multiples of each point's own `1/c` (`c ∈ 0.18566 … 0.19726`), so
the singular gauge sits at `k·c = 1` in every row. First point, `s = 0.110`, `c = 0.185659`,
`w*(1) = 1.085727129792e−02`:

| `k·c` | `k` | `G_w` | `1 − k·c` | `w` move | `dw*/dq` | gain move |
|---|---|---|---|---|---|---|
| −0.50 | −2.693 | `+1.50000` | `+1.50000` | `1.3e−15` | `−2.64278645e−04` | `3.3e−10` |
| 0.00 | 0.000 | `+1.00000` | `+1.00000` | `1.3e−15` | `−2.64278644e−04` | `1.3e−09` |
| 0.50 | 2.693 | `+0.50000` | `+0.50000` | `1.6e−16` | `−2.64278647e−04` | `8.9e−09` |
| 0.90 | 4.848 | `+0.10000` | `+0.10000` | `7.6e−14` | `−2.64278669e−04` | `9.3e−08` |
| 1.05 | 5.656 | `−0.05000` | `−0.05000` | `2.1e−14` | `−2.64278594e−04` | `1.9e−07` |
| 1.50 | 8.079 | `−0.50000` | `−0.50000` | `1.3e−14` | `−2.64278657e−04` | `4.7e−08` |
| 3.00 | 16.159 | `−2.00000` | `−2.00000` | `5.1e−15` | `−2.64278640e−04` | `1.8e−08` |

Over all 10 points and 10 gauges: **`G_w` spans `−2.0000 … +1.5000` and matches `1 − k·c` to
`1.18e−08`**; the set point moves `1.55e−14`; `dw*/dq` moves `7.44e−08`.

### 1.1 The `k = 1` column IS rung 76's `c`, and that is the non-vacuity anchor

`‖(1 − G_a′) − c‖ = 7.77e−10` against rung 76's own `_c_at`. Without it this sweep could be
measuring its own root finder: the shipped `_gauge_root` is a **damped** Newton (trust region +
backtracking on the plant's bracket assertion) and does not share a convergence test with the
probe that first ran this construction, so agreement with rung 76 is what pins the instrument.

### 1.2 The exclusion is MEASURED, and the pre-registered one is REFUTED

The anchor registered the exclusion as `‖1 − k·c‖ < 1e−3`. **That is wrong by ~2.5 decades**, and
widening it to a round number that happens to cover the measurement would be a tuned pass wearing
a pre-registered threshold's clothes. So there is no width here at all: a point is excluded **iff
`G_k` has more than one root there**, counted by `_root_count` at that point and that gauge. § 1
and § 3 interlock, and the band is an *output*.

30 of 100 readings are excluded, at `k·c ∈ {0.9, 1.05, 1.1}` — and the worst set-point move among
them is **`6.13e−01`**, so they are not a formality. `1.05` and `1.1` are in the sweep on purpose:
the anchor's window says nothing is wrong there.

---

## 2. WHY — THE TWO HALVES SCALE TOGETHER

Write `A = ∂cap/∂q` and `c = ∂cap/∂w`. At `k = 1`, `w0′ = A/(1−c)`. For general `k` the anchor
moves with `q` too, so `∂G_k/∂q = −(1−k)·w0′ − k·A`, and

    dw*/dq = [(1−k)·w0′ + k·A] / (1 − k·c) = w0′·(1 − k·c) / (1 − k·c) = w0′.

The vanishing factor is in both halves. **A residual's slope is not a sensitivity** — it is half
of one, and the other half carries the same factor. Rung 77's headline (*a forcing over a slope*)
survives intact; what dies is reading either half alone.

---

## 3. MEASURED — THE ROOT SURVIVES, ITS UNIQUENESS DOES NOT

`|G_k(w0)| ≤ 1.08e−16` over every point and gauge — the construction is exact on the plant, not
merely close, and it is **checked** because it is the one thing that would make §§ 1–2
meaningless. The residual is then walked across `[0.2, 3.0]·w0` and its sign changes counted.

First point, `w0 = 1.0857271298e−02`, `1/c = 5.3862`:

| `k·c` | roots | locations (`w/w0`) |
|---|---|---|
| 0.50 | 1 | `1.000000` |
| 0.90 | 2 | `0.574473`  `1.000000` |
| 0.99 | 2 | `0.956243`  `1.000000` |
| **1.01** | **4** | `1.000000`  `1.043995`  `1.119705`  `1.350931` |
| 1.05 | 2 | `1.000000`  `1.619566` |
| 1.20 | 2 | `1.000000`  `2.513583` |
| 1.50 | 1 | `1.000000` |
| 3.00 | 1 | `1.000000` |

Root counts over the sweep are `{1, 2, 4}`; the multi-root band is `k·c ∈ [0.9, 1.2]` and it
**brackets** the singular gauge. The closest a spurious root gets to the true one is `4.3e−02` in
`w/w0` — it collides at `k·c = 1` and the collision is what the fourth root at `1.01` is a
snapshot of.

### 3.1 `ok` IS NOT A CORRECTNESS GUARD, and this is why

The first version of this rung asserted that the gauged solve *converged*. Inside the band it
converges perfectly and returns the **other** root: the sweep's `k = 1.1/c` reading came back
`ok = True` with the set point moved 45%, at exactly the location § 3 independently puts the
spurious root (`1.448192·w0` against a measured stray answer of `1.448`). **Silent wrong-root
convergence is worse than divergence, because nothing flags it.** Every caller here checks
`|w − w0| / w0` instead — *found the root*, not *converged*.

---

## 4. MEASURED — A GAUGE AGAINST A DEVICE, AND THE OTHER SINGULAR ROUTE

### 4.1 P6 — rung 76 measured a DEVICE

| | root move, `solve` → `sensed` |
|---|---|
| min over the riding points | `2.8318e−02` |
| max | `1.3539e−01` |

Two to three orders above the `1e−3` threshold. Rung 76's re-writing **moves the root**, so its
`1/(1−c)` is not a gauge artifact and rung 76 § 3 stands. This rung BOUNDS it: the factor is real
*because* the two laws disagree about where the set point is, and that is a property of the pair,
not of the algebra.

### 4.2 P5 — rung 77 § 3's second route is not divergence either, and not `0/0`

| | measured |
|---|---|
| φ leg `G_w`, valve OPEN (frozen at `q`) | `9.97226e+00` |
| φ leg `G_w`, valve CLOSED (riding) | `2.04615e−08` (ratio `2.02e−09`) |
| `∂φ/∂q`, valve OPEN | `7.05860e−01` |
| `φ_lp` spread over ±10% fuel, CLOSED | `1.78e−15` |

**The anchor predicted both partials would die together, giving `0/0`. REFUTED** — `∂φ/∂q` is
`0.706`, nowhere near zero. The reason is structural, not numerical: **`dw*/dq` is an OPEN-loop
object**, and it exists only because `_b_state = q` makes `q` an input. The pinning rung 77 § 3
measures happens at `_b_state = None`, where the valve is a *dependent* variable solved from the
state — there is no `q` there to differentiate against, so `G_q` is not small, it does not exist.

So in the reading where `dw*/dq` is defined, `G_w = 9.97` is finite and nothing is singular; in
the reading where `G_w` dies, `dw*/dq` is not defined. **Rung 77 § 3 put two different derivatives
on one axis.** Its measurements are all correct; its sentence *"`dw*/dq` diverges"* is not, on
either of its two routes — the first because the singularity is removable, the second because the
quantity is not there to diverge.

---

## 5. THE MARCH — AND IT IS **BLOCKED**, BY RUNG 72's OWN LAW

§§ 1–4 read set points at frozen states. § 5 was to run the **plant**, with `_cap_fuel`'s accel
branch gauged at every RK4 stage. The measured result:

| `k·c₀` | `k` | branch ran | **value won the `min`** | steps | worst trajectory move |
|---|---|---|---|---|---|
| 0.00 | 0.000 | 1366 | **0** | 341 | `0.000e+00` |
| 0.50 | 2.472 | 1366 | **0** | 341 | `0.000e+00` |
| 2.00 | 9.888 | 1366 | **0** | 341 | `0.000e+00` |
| 3.00 | 14.831 | 1366 | **0** | 341 | `0.000e+00` |

The trajectory is **bit-identical** under every gauge — and **that is not evidence for anything**,
because `binds = 0`: the gauged cap is computed 1366 times a march and **discarded 1366 times**.
The φ leg is the lower cap at every riding point of a demand-coordinate march, at `margin = 0.05`
and `0.10` alike (checked directly: accel wins 0 of 15 sampled points at both).

**So the accel leg is MASKED in the only coordinate that consults the cap, and this section is
rung 72's law arriving from the other side.** Rung 76's headline — *a device in a leg's LAW
reaches only the MASKED leg* — assumed the masked leg is where such a change lands. A gauge is
**not** a device, and here it reaches neither: `min` throws the value away before the plant sees
it. `n_live ≤ 3` a **sixth** time, and again for a new reason.

### 5.1 Three vacuity traps in one section, and the third is the one that matters

This section was written wrong twice before it was written honestly, and each failure looked
exactly like a pass:

1. **The coordinate.** The first version marched in rung 77's `clip`, where rung 76 § 0's own
   refusal says the ladder dispatches out *before* `_cap_fuel` is reached. Four gauges, four
   bit-identical trajectories, branch never executed.
2. **The counter.** The fix was a `_gauge_hits` counter — written `self._gauge_hits += 1`, which
   creates an *instance* attribute and leaves the class attribute the reader queries at zero
   forever. **The instrument built to detect a vacuous section was itself vacuous.**
3. **The masking.** With both fixed, `hits = 1366` — and `binds = 0`. *The branch ran* and *its
   value reached the plant* are different claims, and only the second is the one § 5 needed.

Rung 77 § 8 records a closure that outlived its state block and returned a perfect `1.000e+00`.
This is the same failure three times in one section, and the only defence that worked was
**counting**, not inspecting.

### 5.2 What survives, and it is not nothing

The schedule is gated `k`-invariant (`0.000e+00`): `_shared_rig` carries `_gauge_k`, so had
`accel_for` tracked the gauge this section would have compared two schedules and called the
difference a trajectory. The swept `k·c` range is recorded per run and checked clear of § 3's
band. Both are real checks; neither substitutes for `binds > 0`.

**The headline does not rest on § 5.** §§ 1–3 establish gauge-invariance of `dw*/dq` and the
uniqueness collision on the *set-point solve itself*, which is where the claim lives. § 5 would
have shown the same statement one level up, on the plant, and it is **NOT ESTABLISHED** — recorded
here as a failure to measure, not as a result.

### 5.3 A FOURTH VACUITY, FOUND FROM OUTSIDE — THE PLANT WAS STANDING STILL

Added by `docs/rungs72-77-march-audit.md` after this rung shipped. § 5's march runs at
`(demand, φ_lim = 0.80)`, which is **rung 74 § 2.2's disclosed arrest cell**: at the inherited
floor a leg that tracks its cap pins `φ` on the floor and the acceleration never starts. Measured
over the same 341 steps — `nu_lp` spread **0.0**, `Tt4` **1000 → 1000**, `min φ_lp` **0.800000**,
all exact. § 5 was reading a **stationary operating point 341 times**.

This **strengthens** § 5's own verdict rather than weakening it. `binds = 0` stays true and stays
honest, but it is a **one-point** statement — *the accel leg is not the lower cap at the initial
condition* — not the trajectory statement § 5 wanted. And `hits = 1366` could never have
discriminated: **on a frozen plant every liveness counter reports the initial condition at FULL
count**, so the instrument § 5.1 built to escape trap 2 scores green on trap 4 by construction.
Three traps became four, and the fourth is the one that was invisible from inside the section.
The correction is to the **attribution** in § 9's first seam, not to any number here.

---

## 6. CONCESSIONS (in addition to every one rungs 62–77 list, all inherited)

* **The gauge is not realisable** (§ 0.1). It needs its own answer as an anchor. Everything here
  is a statement about *residuals*, not about limiters that could be built.
* **`_cap_free` cannot be reused past `k = 1/c`.** It walks the bracket upward and stops at
  `G > 0`, which assumes `G` increases in `w`; beyond the singular gauge it decreases. The damped
  Newton that replaces it is a solver of this rung's own, and § 1.1 is what keeps it honest.
* **The multi-root band is measured on this plant at these settings**, over `k·c ∈ [0.5, 3.0]`
  sampled at ten values. `[0.9, 1.2]` is the band *these samples* resolve; a finer sweep would
  place its edges more precisely and could only widen it.
* **UNIQUENESS IS ESTABLISHED ONLY INSIDE A WINDOW.** `_root_count` walks `[0.2, 3.0]·w0`, so a
  `1` in § 3's table means *one root in that window*, not *unique on the line*. The positive half
  of the headline is unaffected — a second root demonstrably appears, collides at `k·c = 1` and
  departs — but the implied "and is unique elsewhere" is a statement about `[0.2, 3.0]·w0` and
  nothing wider. The bound is not idle: by `k·c = 1.2` the spurious root is already at `2.51·w0`
  and climbing, so it **leaves the window before the band closes**, and the `1` at `k·c = 1.5` is
  partly the window's doing.
* **Only the accel leg has a gauge.** `Tt4_max` and `φ_lim` are constants, so the governor and the
  φ leg have no cap to anchor — rung 77 § 1's reason read one step on. A gauge needs a formula,
  and a floor on a STATE is not one.
* **`c` is read at one point per state** and treated as locally constant when `k·c` is formed.
  Where `cap` has curvature this makes `k·c = 1` a nominal marker rather than an exact one, which
  is precisely why § 3 finds a *band* and not a point.

## 7. THE REDUCE

`_gauge_k = 1.0` dispatches `_cap_fuel` to `super()` on an exact comparison, so not one float in
this family moves and the identity is structural rather than numerical. Gated in **both**
directions on rung 73's discipline: at `k ≠ 1` the residual's SLOPE must differ (else the knob is
dead), and outside the multi-root band the TRAJECTORY must NOT (else the gauge is not a gauge).
Either half alone passes something broken — the first passes a knob that does nothing, the second
passes a knob that is not wired.

## 8. THE ANCHOR, SCORED

| | prediction | verdict |
|---|---|---|
| P1 | set point `k`-invariant to `< 1e−9` outside `‖1−k·c‖ < 1e−3` | **REFUTED AS WORDED, HELD ON THE TRUE ROOT.** `1.55e−14` outside the *measured* multi-root band; the `1e−3` window is wrong by ~2.5 decades (§ 1.2) |
| P2 | `G_w = 1 − k·c` to `< 1e−6`, spanning both signs | **HELD** — `1.18e−08`, `−2.0 … +1.5` |
| P3 | `dw*/dq` invariant to `< 1e−6`, including `G_w < 0` | **HELD** — `7.44e−08` (same exclusion as P1) |
| P4 | non-convergence confined to `‖1−k·c‖ < 1e−3`, else P1–P3 refuted | **REFUTED, AND THE FAILURE MODE WAS THE WRONG ONE.** There is no non-convergence at all: `ok = True` everywhere, including where the answer is wrong (§ 3.1). The registered threshold *and* the registered symptom both missed |
| P5 | φ route is `0/0`: both partials `< 1e−6` of open-loop | **REFUTED** — `∂φ/∂q` open is `0.706`. The replacement is structural and stronger (§ 4.2) |
| P6 | `solve` vs `sensed` roots differ by `> 1e−3` | **HELD** — `2.83e−02 … 1.35e−01` |
| D2 | the march is `k`-invariant (derivation, § 1 of the anchor) | **NOT ESTABLISHED.** The trajectory is bit-identical, but `binds = 0` — the gauged leg is MASKED in the only coordinate that consults the cap (§ 5) |

**Three of six refuted, and all three refutations are the rung's content.** P4 is the one worth
naming: it was registered against *divergence*, and the actual failure is *silent convergence to a
different root*. A gate written from the anchor would have watched `ok` and seen nothing.

## 9. NEXT SEAMS

* **§ 5, ON A PLANT WHERE THE ACCEL LEG BINDS — AND WHERE THE PLANT MOVES AT ALL.** The
  trajectory claim is untested, not refuted. It needs an operating point where the accel cap is
  the lower one in a coordinate that calls `_cap_fuel` — a lower `φ_lim`, a hotter band, or a leg
  pair that is not this one. **Sharpened by § 5.3:** it needs *two* things, and this rung had
  neither. The second is cheap — rungs 72/73/77 march at **this very wall** in `clip` and move
  (`nu_lp` by ~11%), so the arrest is the coordinate, not the setting.
* **THE BAND'S EDGES, RESOLVED.** `[0.9, 1.2]` is what ten samples resolve (§ 6). Where does the
  second root enter — is it born at a fold at finite distance, or does it come in from the
  bracket's edge? § 3's `0.574 → 0.956` approach says fold; it has not been measured.
* **A GAUGE ON A LEG THAT IS MASKED.** Every reading here is on the authoritative accel leg.
  Rung 76's law says a device in a leg's LAW reaches only the masked leg — a gauge is not a
  device, so it should reach neither, and that is a testable difference between the two.
* **THE SAME QUESTION ON THE φ LEG'S OWN COORDINATE.** § 4.2 shows the φ leg has no gauge because
  its set point is a constant. Rung 69's INCIDENCE reference gave a leg a different coordinate
  without changing what it watches — a coordinate change is a gauge candidate on the *state* side
  rather than the *law* side, and nothing in 69–77 asked whether it preserves roots.
* **WHETHER ANY SHIPPED READING IN 46–77 IS GAUGE-DEPENDENT.** This rung proves one number
  (`1/(1−c)`) is a coordinate. It does not audit the rest, and the ledger of rung 77 § 1 is
  written in residual slopes throughout.
* Everything rungs 72–77 § 8–11 leave, unchanged by this rung.
