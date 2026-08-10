# Rung 78 anchor — THE RESIDUAL GAUGE (rung 77 § 9's second seam, expected REFUTED)

Scored in `docs/rung78-spec.md` § 8. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

The seam, in rung 77 § 9's own words: ***A CAP WHOSE `c` APPROACHES 1**, still open and now
bounded: § 4 shows `margin` cannot get there, so it needs a schedule referenced to a quantity the
fuel moves harder than `pt3`.*

**This rung expects to close that seam by REFUTING its premise.** Rung 77 § 3 lists `c → 1` as one
of two routes to a singular set-point solve and calls it *unreachable in this family*
(`c ≤ 0.2234` over 24 cells). This rung makes it reachable **by construction, at a fixed set
point**, and expects to find that arriving there costs nothing — because `1/(1−c)` is a property
of **how the residual is written**, not of the plant.

**Nothing below has been measured on the plant.** § 0 is algebra plus a TOY, run before this
document existed (`M:\claud_projects\temp\rung78\gauge_algebra.py`, `gauge_nonlinear.py`), on a
three-parameter linear/`tanh` cap with no engine in it. It is listed as **derivation, not
prediction, and is not scored** — a toy agreeing with its own algebra is not evidence about a
turbojet. § 2 is what is scored.

---

## 0. THE CONSTRUCTION, AND WHAT THE TOY SAYS — UNSCORED

### 0.1 The dial

Rung 76's sensed cap is `cap(w) = (1+margin)·κ(n_H(w))·pt3(w)`, and `c = dcap/dw` is a **plant**
property — rung 76 measures it, it is not a schedule parameter. So a dial on `c` cannot be a new
constant in the schedule; it has to be a re-writing of the leg's residual. Anchor the cap at its
own set point and scale its departure from it:

    cap_k(w) = w0 + k·(cap(w) − w0)        w0 := the k = 1 root, i.e. cap(w0) = w0

* **The root is invariant in `k`, exactly and for every `k`.** `cap_k(w0) = w0 + k·(cap(w0) − w0)
  = w0 + k·0 = w0`. No approximation, no tolerance: `w0` is a fixed point of `cap_k` identically.
* **The slope is a free dial.** `G_k(w) = w − cap_k(w)`, so `G_k′ = 1 − k·c`, and `k = 1/c`
  puts it at **zero** — rung 77 § 3's route one, reached deliberately.
* **`k = 1` is `cap_k ≡ cap`**, so it is rung 77 with the branch not taken.

`k` is a **swept knob, not a constant**: nothing is imposed at a fixed value, and `w0` is the
plant's own already-solved set point. **This rung adds no constant.**

### 0.2 What the algebra then forces, and it is the whole rung

Write `A = ∂cap/∂q` (`q` the valve position) and `c = ∂cap/∂w`. At `k = 1` the implicit function
theorem gives `w0′ = A/(1−c)`. For general `k`, the anchor `w0` moves with `q` too, so

    ∂G_k/∂q = −(1−k)·w0′ − k·A
    dw*/dq  = −G_q/G_w = [(1−k)·w0′ + k·A] / (1 − k·c)

and substituting `A = w0′·(1−c)` collapses the numerator to `w0′·(1 − k·c)`:

    dw*/dq = w0′ ,  for every k.

**`G_w` and `G_q` scale in LOCKSTEP, and their ratio does not move.** The `1 − k·c` that vanishes
in the denominator vanishes in the numerator at the same `k`. So the singularity is **removable**,
and `1/(1−c)` is a **gauge**: a number you can set to anything, including infinity, without the
plant noticing.

### 0.3 The toy, and the one thing it changed

Linear cap (`c = 0.19`): `w*` invariant to `1e-16` relative across `k ∈ {0, ½, 1, 2, 4, 5}`,
`G_w` falling `1.000 → 0.050`, `G_q` falling in step, `−G_q/G_w` pinned at the closed-form
`β/(1−α)` to eight figures, and a **direct** re-solve at `q ± dq` agreeing. At exactly `k = 1/c`
the bracket **failed** — but a linear cap makes `G_k ≡ 0` identically there, so *every* `w` is a
root. That is an artifact of the toy.

Re-run with curvature (`+0.06·tanh(w−1.2)`, `c = 0.24997`), Newton instead of bisection:

| `k` | `w*(k)` rel. move from `k=1` | `G_w` | `−G_q/G_w` | direct `dw*/dq` |
|---|---|---|---|---|
| 1 | `0` | `+7.50e−01` | `−4.133190e−02` | `−4.133190e−02` |
| 4 | `2.3e−13` | `+1.04e−04` | `−4.133350e−02` | `−4.133184e−02` |
| **4.000415 = 1/c** | `1.8e−07` | `−2.5e−09` | *(0/0 noise)* | *(noise)* |
| 5 | `9.1e−16` | `−2.50e−01` | `−4.133190e−02` | `−4.133190e−02` |
| 10 | `5.5e−16` | `−1.50e+00` | `−4.133190e−02` | `−4.133190e−02` |
| −3 | `0` | `+1.75e+00` | `−4.133190e−02` | `−4.133190e−02` |

**The set point walks straight through `G_w = 0` and out the far side**, where the residual's
slope is *negative*, without moving in the sixteenth digit. Only a `k`-window of width `~1e−3`
around `1/c` is disturbed, and there only the **arithmetic** is — `G_w ≈ 2e−9` makes the ratio
`0/0`. This is what § 2's P4 is registered against, and § 0.4 says why it must not be scored.

### 0.4 Why none of § 0 is scored

The toy has no compressor, no map, no six states and no `min`. It establishes that the algebra is
self-consistent — which was worth checking before writing plant code, and is exactly the check
rung 77 § 2's first version failed by building closures on the wrong plant. It says **nothing**
about whether `cap` on the real plant is smooth enough for `c` to be well defined at the riding
points, whether `_sched_fuel`'s bracket survives `G_w < 0`, or what `min` does to any of it.

---

## 1. WHAT IS DERIVED ON PAPER FROM THE INHERITED LAWS — DERIVATION, NOT PREDICTION

* **D1.** `k` reaches the ACCEL leg and nothing else. `Tt4_max` and `φ_lim` are constants, so the
  governor and the φ leg have no cap to anchor — rung 77's own reason, inherited unchanged.
* **D2.** Because the root is invariant at **every** state, the applied fuel `min(accel, φ, …)`
  is invariant, so the **whole march is `k`-invariant**. `k` is therefore inert on every
  trajectory quantity and on the Jacobian: `n_live ≤ 3` a SIXTH time, for a **new** reason —
  not `min`'s flatness (rung 76) but the leg's own root being untouched.
* **D3.** Rung 76's `solve` vs `sensed` is **not** this: those two laws have **different roots**
  (rung 76 § 2 measures the path moving), so their `1/(1−c)` factor is not a gauge. This rung
  BOUNDS rung 76 § 3, it does not refute it — see P6.
* **D4.** `k < 0` is admissible and is the cheapest check that `G_w`'s **sign** is not load
  bearing. It is included for that reason and for no physical one.

Listed as derivation. § 8 scores any of these that measurement finds false, on rung 72's D5
precedent.

---

## 2. THE PREDICTIONS — SCORED

Settings are rung 77's, taken verbatim: `φ_lim = 0.80`, `margin = 0.10`, `Tt4_max = 1200 K`, all
clocks `0.05`, `ds = 0.005`, the `clip | sched | none | solve`-armed march, read at `_riding4`
points every 8th. `k` swept over `{−3, 0, ½, 1, 2, 0.9/c, 1.1/c, 2/c, 3/c}` per point, with `c`
that point's own measured value.

* **P1 — THE SET POINT IS `k`-INVARIANT.** `‖w*(k) − w*(1)‖ / w*(1) < 1e−9` at every riding
  point and every swept `k` **outside** the excluded window `‖1 − k·c‖ < 1e−3`.
* **P2 — THE SLOPE IS A FREE DIAL, AND IT IS THE ONE THING THAT MOVES.** `G_w(k)` matches
  `1 − k·c` to `< 1e−6` relative, spans **both signs**, and passes within `1e−3` of zero.
* **P3 — THE RATIO DOES NOT MOVE.** `dw*/dq`, measured **directly** by re-solving at `q ± dq`
  (not via `−G_q/G_w`, which is the thing under test), matches its `k = 1` value to `< 1e−6`
  relative, **including at `k > 1/c` where `G_w < 0`**. This is the rung: rung 77 § 3's route one
  is **REMOVABLE**.
* **P4 — NON-CONVERGENCE, IF IT HAPPENS, IS A DEFECT OF THE SOLVE AND IS DECLARED SO NOW.**
  Registered *before* measurement, with its threshold: a failure to converge is scored as
  arithmetic **iff** it is confined to `‖1 − k·c‖ < 1e−3` **and** that window narrows when the
  solver tolerance is tightened by two decades. If `w*` moves outside that window, or the window
  does not narrow, then **P1–P3 are REFUTED and the singularity is real** — and this rung's
  headline is wrong.
* **P5 — THE φ LEG'S ROUTE IS A DIFFERENT ANIMAL, AND IT IS NOT DIVERGENCE EITHER.** Rung 77
  § 3 says `dw*/dq` **diverges** where a riding valve pins `φ_lp`. Predicted: it does **not**
  diverge, it becomes **UNDETERMINED** — a valve that pins `φ` against the fuel pins it against
  the valve too, so `∂φ/∂w → 0` **and** `∂φ/∂q → 0`, and `dw*/dq` is `0/0`. Quantitatively:
  both partials `< 1e−6` of their open-loop values at the same points. **Rung 77 § 3's wording
  is expected REFUTED on both of its two routes, for two different reasons.**
* **P6 — RUNG 76 SURVIVES, AND THIS RUNG BOUNDS IT.** `‖w*_sensed − w*_solve‖ / w*_solve > 1e−3`
  at every riding point: the two laws have genuinely different roots, so rung 76 § 3's measured
  `1.228 … 1.246` is **not** a gauge artifact. If this comes back below `1e−9`, rung 76 § 3 is
  the thing that was measuring a gauge and this rung's finding is much larger than stated.

## 3. THE REDUCE

`_gauge_k = 1.0` (the default) must be **rung 77 bit-for-bit**: `cap_k ≡ cap` algebraically, so
the branch is not taken and no float in this family moves. Gated on rung 73's discipline — the
same machine at `k ≠ 1` must **DIFFER in `G_w`** (else the knob is not wired), while the
**trajectory must NOT** (D2). Both halves are required; the first alone would pass a dead knob,
and the second alone would pass a knob that does nothing at all.

## 4. THE TRAP THIS RUNG IS EXPOSED TO

The carried-knob trap's **sixteenth** face: `_gauge_k` must travel through `at_lever`,
`_shared_rig` and `_cap_march`, or every reader below runs at `k = 1` on a rig that looks right.
Rungs 61–77 hit this fifteen times; it is written here **before** the code so that the gate for it
is written from the anchor rather than after a surprise.
