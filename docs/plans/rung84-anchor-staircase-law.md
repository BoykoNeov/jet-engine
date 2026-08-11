# Rung 84 anchor — THE MARCHED MINIMUM'S STAIRCASE

**Seam:** `docs/rung83-spec.md` § 8, first bullet — *"WHICH RAMPS HAVE ROOTS, AND AT WHICH `ds`.
§ 3.3 shows existence flipping between the two shipped steps at one ramp. The `(r, ds)` map — where
sign changes are crossings and where they are handovers — is unmeasured, and it decides whether rung
82's five-row table contains one discontinuity or several."*

**Plant:** `StaircaseLawTransient` (`turbojet/engine.py`) — **no state, no knob, no constant**, the
**fifth** reader-only rung (77, 81, 82, 83, 84). **Spec:** `docs/rung84-spec.md`. **Gates:**
`tests/test_rung84.py`.

**Rig:** `tests/test_rung83.py`'s verbatim — `FlightCondition(250.0, 50_000.0, 0.85)`,
`π_LPC/π_HPC/Tt4 = 3.0/6.0/1500.0`, the `REAL` loss set, `FLOOR = 0.55`, `Tt4 lo/hi/max =
1000/1400/1200`, `φ_fuel/φ_air = 0.75/0.77`, both `ComponentMap`s, and
`_lag_coord/_ref_law/_windup_law/_cap_law = "demand"/"sched"/"none"/"solve"`.

**A REFACTOR, DECLARED UP FRONT.** This rung needs the **identity** of the scored points, which
`_threshold_scan` reduces to counts. Rather than duplicate its fifteen lines, rung 82's method is
split into `_scan_cells` (the march + the riding set) and `_hats` (the point-wise threshold), which
it then calls. **The return value does not change, in any key, by any bit** — and that claim is not
asserted, it is *certified* by instruments that already exist: rung 83's bit-exact reduce gate and
the three `r82*` fingerprint arms, all of which carry `TOL = 0.0`.

---

## 0. THE PRE-CHECK — RUN BEFORE ANY OF §§ 1–2 WAS WRITTEN

Rungs 81, 82 and 83 all set the precedent: ask first whether the rung's framing survives contact
with the shipped plant, and **take no credit for anything the pre-check settles.** Six questions.
`M:\claud_projects\temp\rung84\{precheck,recon}.py`, 21 marches, 46 s.

### E1–E5 — the mechanism, named

The framing this rung started from was rung 83 § 3's: *"`F` is a `min`, so it is only piecewise
smooth, and JUMPS at every handover."* **That cannot be the mechanism, and one line of algebra says
so:** a minimum over a **fixed** finite set of functions each continuous in `τ` is itself continuous
in `τ` — it has a **kink** where the argmin changes hands, never a **jump**. A jump requires the
**set** to change.

Measured at rung 83 § 3.2's own two τ values (`r = 0.25`, `ds = 0.005`), and at § 3.4's crossing
(`r = 0.35`) as the control. The probe reproduces rung 83's published `g` exactly (`h/κ` with
`κ = 3`: `+1.6498e−3 → −2.4264e−3`), so this is that computation and not an adjacent one:

| | `r = 0.25` — rung 83's JUMP | `r = 0.35` — rung 83's CROSSING |
|---|---|---|
| scored set | **gains `s = 0.080`** (48 → 49) | unchanged (43 → 43) |
| `s_bind` | 0.085 → 0.080 | 0.125 → 0.125 |
| `Δh` total | `−1.223e−02` | `−2.515e−03` |
| `Δh` on the **common** points | `−7.065e−05` | `−2.515e−03` |
| **membership term** | **`−1.216e−02` (99.4 %)** | **exactly `0.0`** |
| sign change survives restriction to the common set? | **NO** (`+4.949e−3 → +4.879e−3`) | YES |
| binding point at the window's leading edge? | **YES, both sides** | **YES, both sides** |

**So the jump is the four-loop window OPENING ONE MARCH STEP EARLIER**, and the new leading point
immediately binds the minimum. The sign change at `r = 0.25` is *entirely* the membership term: on
the points both marches share, `h` does not change sign at all.

### E6 — the lattice, reconnaissance only (SIZING, never scored)

17 marches, `r = 0.25`, `ds = 0.005`, `τ ∈ [0.0190, 0.0206]` at `1e−4` spacing:

- **`s_bind == ride_s[0]` at 17 of 17.** The binding point is the window's **first** point, always.
- The edge takes the values `0.085` then `0.080` — **exact multiples of `ds`** — and moves **once**,
  by **one grid step**, in the window.
- On the branch, `g` falls smoothly at `dg/dτ ≈ −1.94`; the jump in `g` at the edge move is
  `≈ −4.0e−3`, against a drift of `≈ −3.1e−3` across the whole 1.6e−3-wide window.

This is what §§ 1–2 are sized against. **It is disclosed as reconnaissance and no prediction below
is scored on it.**

---

## 1. THE DERIVATION — zero new constants

### 1.1 The residual is a BOUNDARY reading, not a minimum

Rung 82 builds `F(τ) = min_{s ∈ S(τ)} ĥ(s; τ)/κ` over `S(τ)` = the interior points riding four
loops. E6 measures the argmin at the window's **leading** point at every τ sampled. If that holds
generally (**P1**), then

    F(τ) = ĥ( s_edge(τ); τ ) / κ ,        s_edge(τ) = the first MARCHED point inside the window

and the `min` is decorative: the object rungs 82 and 83 have been solving is an evaluation on a
**moving discrete boundary**.

### 1.2 The boundary is on a GRID, so the residual is a STAIRCASE

The four loops engage at some **continuous** `s*(τ)`; the march samples at spacing `ds`; so

    s_edge(τ) = ceil( s*(τ) / ds ) · ds

`F` is smooth in `τ` while `ceil(·)` holds, and **jumps each time `s*(τ)` crosses a grid line**. The
discontinuities are therefore a **LATTICE**, and two of its properties follow with no further
physics:

* **COUNT.** Over `[τ_a, τ_b]` with `s*` monotone, the number of jumps is exactly
  `( s_edge(τ_a) − s_edge(τ_b) ) / ds` — an **integer read from two marches**, not a ladder.
* **TREAD.** The τ-spacing between jumps is `Δτ = ds / |ds*/dτ|`, so it is **∝ `ds`**.

### 1.3 The staircase number, and why refinement may not help

The jump is `J = |ĥ(s_edge − ds) − ĥ(s_edge)|/κ ≈ |∂ĥ/∂s| · ds / κ` — **∝ `ds`**. The smooth drift
across one tread is `D = |dg/dτ| · Δτ = |dg/dτ| · ds / |ds*/dτ|` — **also ∝ `ds`**. Their ratio

    Λ  =  J / D                  the STAIRCASE NUMBER

therefore contains **no `ds` at all**. `Λ > 1` means `g` falls further at each lattice point than it
drifts between them: the residual is a **descending staircase** whose treads cover only `1/(1+Λ)` of
its own range, and a root exists only if the crossing happens to land on a tread.

**The consequence, and it is the rung's headline candidate: refining the march does NOT converge
root existence.** It halves the step and halves the tread together, relocating the crossing within a
finer lattice without changing the odds. Rung 83 § 3.3's *"refining the march step **creates** the
root"* would then be a **coin landing the other way up**, not a convergence — and § 3.3's own
observation that refining also *"opens a **new** handover at τ = 0.023"* is that same lattice
densifying.

### 1.4 What this says about the corrector — and about the criterion

A Newton or secant step reads a **value and a slope** and presumes a smooth root. On a staircase the
slope it reads is the tread's, and the root it aims at may be on a different tread or on none. This
is **why** rung 83's corrector failed at `r = 0.25` and worked at `r = 0.35` — and it is a statement
about the march, not about the plant.

---

## 2. THE PRE-REGISTERED PREDICTIONS

Written before the probes of § 4 were run. Scored in `docs/rung84-spec.md`, refuted or confirmed.

| # | prediction | bar |
|---|---|---|
| **P1** | the binding point is the window's **leading** point | `s_bind == ride_s[0]` at **≥ 95 %** of every march this rung runs, over all five ramps and both shipped steps |
| **P2** | the classifier is **EXACT**, needing no threshold | at every sign change of `g` between adjacent ladder points: sets equal ⇒ membership term **exactly `0.0`** and the sign change survives restriction to the common points; sets differ ⇒ membership term ≠ 0. **100 %**, both directions, no ties |
| **P3** | the lattice **COUNT** is `Δs*/ds` | at `r = 0.25` over one fixed τ window, `(edge(τ_a) − edge(τ_b))/ds` is an **integer** at `ds ∈ {0.005, 0.0025, 0.00125}` and **doubles ±25 %** at each halving |
| **P4** | rung 83's `argmin_moved` is a **PROXY**, and a counter-example exists | **≥ 1** adjacent pair somewhere on the map where the argmin moves and the set does **not** (an interior handover, `F` continuous). If **0**, P1 holds at 100 % and the two flags are coextensive **by construction** — which is the stronger reading and must be reported as such |
| **P5** | **`Λ` is `ds`-INVARIANT** | at `r = 0.25`, `Λ` measured at the same lattice event lies within **±30 %** across `ds ∈ {0.005, 0.0025, 0.00125}` |
| **P6** | root existence does **NOT** converge in `ds` | at `r = 0.25`, existence over `ds ∈ {0.005, 0.0025, 0.00125, 0.000625}` is **NOT** monotone: at least one **absence** appears after rung 83's `ds = 0.0025` presence |
| **P7** | the **MAP** (the seam's literal ask) | over the five shipped ramps × the two shipped steps, **≥ 1 and ≤ 4** of 10 cells have no root, **and the identity of a failing ramp changes with `ds`** |

**P4 is registered in the shape that can lose either way on purpose.** Rung 83's memory records two
bars that died of naming a direction rather than a point; this one names both outcomes and what each
would mean, so neither can be chosen after the fact.

**AND THE CROSS-RUNG CLAIM IS BOUNDED BEFORE IT IS MEASURED.** Rung 83's `argmin_moved` flag **fired
correctly** at its jump (0.085 → 0.080, E1). What § 1 corrects is the **cause**: an edge move forces
an argmin move, so the flag reports a **consequence**. This is rung 28's shape — **verdict
CONFIRMED, reason CORRECTED** — and it is *not* "rung 83 was wrong about the `min`". Rungs 63 and 71
are on record for exactly the over-claim this sentence exists to prevent.

---

## 3. THE VOIDS — a row that trips one is not reported, it is voided

* **V1** `window_open` — inherited from rung 82. A ramp with no four-loop point has no threshold to
  have a root of.
* **V2** `kappa_pure` — inherited (rung 82's V4). `κ` is READ, never imposed.
* **V3** **THE EDGE MUST BE ON THE GRID.** `edge/ds` an integer to within `1e-9`, else § 1.2's
  `ceil` picture is wrong and **no count may be quoted**.
* **V4** **THE EDGE MUST BE MONOTONE** over any window whose jumps are counted from its two
  endpoints. Certified by a ladder at the coarse step; a non-monotone edge voids the two-march
  count, which would otherwise silently under-count.
* **V5** `Λ` needs **both** terms at the **same** lattice event — a jump from one cell and a drift
  from another is not a ratio.
* **V6** a classification compares two marches differing **only** in `τ`.

---

## 4. THE PROBES

**Cost, as run: ~400 marches over eight probes, ~33 min wall on a contended box.**

| probe | scores | marches | wall |
|---|---|---|---|
| `precheck.py` | E1–E5 — the mechanism, named | 4 | 11 s |
| `recon.py` | E6 — the lattice, SIZING only | 17 | 35 s |
| `probe_edge.py` | P1 | 30 | 97 s |
| `probe_lattice.py` | P2, P3, P4, V3, V4 | 47 | 117 s |
| `probe_count.py` | P3's content, five `ds` levels | 10 | 159 s |
| `probe_profile.py` | why the rise is first order | 3 | 18 s |
| `probe_lambda.py` | P5, with the tread isolated | 57 | 324 s |
| `probe_root.py` | P5 (first cut), P6, P7 | 232 | 1212 s |

**Two probes exist because a first cut measured the wrong thing, and both are kept rather than
overwritten.** `probe_root.py`'s Λ isolated no lattice event — its bracket was the whole counting
window, so both terms were chords; `probe_lambda.py` tightens the bracket to `< 1e-6` and reads the
branch slope off a pair that straddles nothing. `probe_profile.py` did not exist until the rise
failed to halve, and it is what showed the profile is smooth at the boundary (exponent → 1).
