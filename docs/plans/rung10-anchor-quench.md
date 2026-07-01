# Rung-10 external anchors — the finite-rate quench, the RQL hazard quantified

Verified numbers + a machine-checked worked example that anchor the **finite-rate quench**
(rung 10): the secondary-zone Zeldovich in the cooling, mixing gas that turns rung 9's *ideal*
(infinitely-fast) quench into a real one — and shows *the* RQL failure mode, **NO spiking as the
gas dwells at stoichiometric while the quench air mixes in**. Every number was re-derived
**before any production code** (project discipline); the worked example runs on the *existing*
rung-6/7/8/9 primitives (`_equilibrium_composition`, `_h_molar_A`, `_mixed_out_T`, `_thermal_no`,
`_k_zeldovich`) plus the one new thing — a clamp-free NO integrator over a dilution trajectory —
in `M:\claud_projects\temp\rung10\proto_quench.py` (+ `proto_clamp.py`, `proto_maxa.py`).

> **Read `docs/rung9-spec.md` first.** Rung 10 is built *on* rung 9: it runs the **same**
> extended-Zeldovich rate on the **same** frozen-equilibrium pools — only now the mix-out is
> resolved in *time* instead of collapsed to an instant. The local mixture sweeps
> `far_p → f_stoich → far_overall` as air is added over a quench time `τ_q`, and NO integrates
> kinetically along that path. Nothing in the thermochemistry, the `Kp`/`_equil_solve` machinery,
> or the `a6`/`a7` substrate changes. NO stays a **trace, decoupled diagnostic**, so the cycle is
> still **bit-for-bit rung 6**.

**The lesson (it closes the RQL story rung 9 opened).** Rung 9 showed a *rich* primary is a
low-NOx regime — EI_NO collapses on the rich flank of the bell (φ_p = 1.5 → 0.0013 g/kg, ~1800×
below the stoich peak) — and framed that as *why* RQL burns rich. But rung 9's mix-out was the
**ideal, infinitely-fast quench**: NO frozen at the primary value. That is a fiction. In a real
combustor the quench air mixes in over a finite time, and while it does the **local mixture
passes through stoichiometric** — the exact peak of the NO bell. So a rich primary's temperature
**rises through the stoich peak on its way down** (φ_p=1.5: 2110 → 2453 K → Tt4), and NO is
*re-made* during the quench. A **slow** quench dwells at stoich and re-makes the NO the rich
primary avoided; only a **fast** quench escapes past the peak. The "quick" in quick-quench is the
whole game — rung 10 quantifies it.

---

## 1. Sourced numbers (numbers before code)

### 1a. The quench time `τ_q` (an un-anchored knob, like `τ` and `α`)
The quench/dilution zone must mix the rich primary products with the dilution air **faster than
NO forms at the stoichiometric crossing** the mixing passes through. Practical RQL quench-zone
mixing times are **order sub-millisecond to a few milliseconds**; the design intent is to make
the quench as fast as mechanically achievable (jets-in-crossflow, high-momentum dilution) so the
gas spends as little time as possible near stoich. Like the primary residence `τ` (rung 7) and
the air split `α` (rung 8), `τ_q` is **swept to reveal the mechanism**, not derived from a
specific combustor's mixing schedule. The complementary lesson: at `τ_q → 0` the model **must**
reduce to rung 9's frozen NO (§3); as `τ_q` grows, the NO climbs — the penalty for a slow quench.
*(Lefebvre & Ballal, *Gas Turbine Combustion*, low-emissions/RQL chapter — the "quick" quench
requirement and jet-mixing time scales; Turns, *An Introduction to Combustion* — the NO-vs-φ bell
the quench trajectory traverses.)*

### 1b. The mixing schedule (linear air addition — an un-anchored altitude choice)
Air is added **linearly in time** over `τ_q` (fraction of dilution air `β·(1−α)` present at time
`t = β·τ_q`), so the local air per mol fuel grows monotonically and `far_local(β) = far/(α +
β(1−α))` sweeps `far_p → far`. The *shape* of the schedule (linear vs. an S-curve vs. a
jet-mixing model) is a modelling knob at the same altitude as `α`; linear is the honest minimal
choice and is stated as such. The **majors are fast chemistry** (instantaneous equilibrium at each
`β`, exactly rung 8's re-equilibrating mix-out), so `T(β)` and the pool are a function of `β`
alone; NO is the one *slow* variable integrated over the path.

### 1c. Carried over from rungs 7–9 (unchanged)
- **Extended-Zeldovich rate constants** (Hanson–Salimian) + the thermo-kinetic **`K`-check**
  (asserted every run — now along the whole trajectory, T ∈ ~1520–2453 K, still in band, §4).
- The **reverse-rate one-equation form** `d[NO]/dt = 2R1(1−a²)/(1+βa)`, `a=[NO]/[NO]_e` — the
  *same* form as rung 7's `_thermal_no`, **but the hard `cNO ≤ cNOe` cap is dropped** (§1d).
- The EI_NO **ICAO band** (18–64 g/kg take-off, rung-8 anchor §1b) — an order-of-magnitude
  landing zone, not a book digit; the absolute Zeldovich rate is un-pinned (rung 7).

### 1d. The equilibrium-clamp trap (why the quench needs a *separate*, clamp-free integrator)
Rung 7's `_thermal_no` clamps `cNO → cNOe` and asserts `cNO ≤ cNOe` (gas.py). That is valid when
NO builds toward a **fixed** equilibrium *from below* at one temperature. In a **cooling**
trajectory the gas can drop below the inherited NO's equilibrium, so NO is legitimately
**super-equilibrium and frozen** (Heywood): a hard cap would delete exactly that NO — a
plausible-but-wrong low number with every assert still green. So the quench integrator **drops the
cap** and relies on the `(1−a²)` form (which already goes *negative* when `a>1` and freezes as the
Arrhenius constants collapse) to self-limit. Two design consequences, both verified:

- **Do not touch `_thermal_no`.** Its rung-6/7/8/9 reduce gates depend on its *exact* RK4
  trajectory (and the cap catches numerical overshoot there). Rung 10 adds a **separate**
  clamp-free `_quench_no`; `_thermal_no` stays **byte-identical**.
- **At the main.py lean design point the cap is *dormant*** (a clean surprise, stated plainly):
  because the overall mixture is very lean (φ_overall ≈ 0.40), the cold mixed-out state is
  O₂-rich and its equilibrium NO stays **high** — above the frozen NO — so NO lags *below*
  equilibrium the whole way (the rung-7 lesson persists). Measured **`max(a) = 0.677` across the
  entire in-scope φ_p×τ_q sweep** (`proto_maxa.py`), and clamp-on ≡ clamp-off to 4 sig figs
  (`proto_clamp.py`). We drop the cap anyway — it is physically wrong for a cooling path and this
  is a teaching artifact whose whole point is the cooling trajectory; a dormant-but-wrong clamp is
  exactly the hidden assumption the project exists to expose. Super-equilibrium freeze *does* bite
  in near-stoich exhaust/expansion cooling (the still-open rung-6 nozzle seam), not in this lean
  mixed-out state.

---

## 2. The finite-quench worked example

**Model (rung-9 flow, now with a finite quench):** split the air (α to the rich primary with all
the fuel), burn the primary adiabatically from Tt3 (rich equilibrium products, CO/H₂ major), run
the rung-7 `_thermal_no` there → the **inherited** primary NO. Then, over `τ_q`, add the rest of
the air **linearly**; at each step re-equilibrate the majors (fast chemistry → `T(β)`, the pool),
and integrate NO with the clamp-free reverse-rate Zeldovich on the *local* `[O],[N2],[H]` and
`[NO]_e`. Freeze at `β=1`. NO is extensive (moles per mol total air), so mixing conserves it and
only chemistry changes it.

**Worked example** (`main.py` design point: **Tt3 = 584 K, Tt4 = 1500 K, far = 0.0272,
p = 7.5 bar, τ = 3 ms**; φ_overall = 0.40):

**(a) The smoking gun — trajectory `T(β)` rises through the stoich peak for a rich primary:**

| primary φ_p | T_start (primary AFT) | T_peak @ β (φ) | T_end (mix) | shape |
|---|---|---|---|---|
| 0.80 | 2224.9 K | 2224.9 K @ 0.00 (φ 0.80) | 1518 K | monotone fall |
| 1.00 | 2441.9 K | 2441.9 K @ 0.00 (φ 1.00) | 1518 K | monotone fall |
| **1.50** | 2110.4 K | **2452.6 K @ β 0.16 (φ 1.046)** | 1518 K | **RISES through the stoich peak** |

A lean/stoich primary starts at or above the peak and only cools; a **rich** primary climbs
through the AFT maximum (slightly rich, φ≈1.05, per the rung-9 CEA anchor) *as it quenches*. That
up-through-the-peak excursion, plus the O-atom maximum there, is the NO spike. (If `T(β)` fell
monotonically for a rich primary, the model would be wrong — this shape is the load-bearing
check.)

**(b) The NO spike vs quench time (rich primary φ_p = 1.5):**

| `τ_q` | EI_NO (g/kg) | × rung-9 ideal |
|---|---|---|
| rung-9 ideal (frozen) | **0.00132** | 1× |
| 0.001 ms | 0.00242 | 1.8× |
| 0.01 ms | 0.0124 | 9.4× |
| 0.1 ms | 0.112 | 84× |
| 0.3 ms | 0.332 | 251× |
| 1 ms | 1.10 | 834× |
| 3 ms | 3.27 | 2480× |
| 10 ms | 10.6 | 8048× |

A rich primary that "avoided" NO (0.0013 g/kg) makes **3.3 g/kg at a 3 ms quench** — three orders
of magnitude re-made purely by quenching slowly. This is the RQL hazard, quantified.

**(c) The bell with a finite quench — the rich flank is *re-filled*:**

| φ_p | rung-9 ideal | quench 1 ms | quench 3 ms |
|---|---|---|---|
| 0.80 | 4.81 | 4.93 | 5.16 |
| 0.90 | 16.1 | 16.5 | 17.3 |
| 1.00 | 20.8 | 21.7 | 23.2 |
| 1.05 | 14.0 | 15.1 | 17.0 |
| 1.10 | 6.45 | 7.60 | 9.85 |
| 1.30 | 0.097 | 1.26 | 3.56 |
| 1.50 | 0.0013 | 1.10 | 3.27 |
| 1.80 | ~0 | 1.04 | 3.09 |

The ideal bell **collapses** on the rich flank (rung 9); a finite quench **fills it back in** to a
~3 g/kg floor (3 ms) that is nearly φ_p-independent — because *every* rich mixture passes through
the *same* stoich peak on the way down and dwells there about equally. So "a rich primary is
low-NOx" is only true **if the quench is fast**: the rich primary's benefit is *contingent on the
quench*, which is the entire RQL design tension. Near the peak (φ_p ≈ 1.0) the quench adds little
(the primary is already at the peak; nowhere hotter to pass through).

---

## 3. The reduce-to-rung-9 gate (exact by construction)

The production reduce is **not** an empirical `τ_q → 0` limit — it is a **short-circuit**: the
`zoned_nox` quench parameter defaults to `None` (ideal quench), which runs the *exact* rung-9 code
path (freeze NO at the primary value). Every existing `zoned_nox` call — every rung-1..9 test,
`main.py`, the whole suite — passes `None` and stays **bit-for-bit rung 9** (hence bit-for-bit
rung 6, cycle untouched). Only a call that *explicitly* passes a finite `τ_q` enters the new
integrator. So the reduce is provable, like rung 9's byte-identical lean seed.

As a *softer* numerical cross-check, `τ_q = 1e-7 s` recovers the rung-9 frozen EI to ~1e-6
relative at φ_p ≤ 1.2. At φ_p = 1.5 the numerical limit is ~8% off — a **tiny-denominator
artifact** (rung-9 EI there is 0.0013 g/kg, so the sliver of chemistry in even a 0.1 µs window
dominates), **not** a defect: the short-circuit default makes the production reduce exact
regardless. (Do not chase the 8%.)

---

## 4. The invariants (discriminating checks, not book digits)

- **`T(β)` rises through the stoich peak for a rich primary; falls monotonically for lean/stoich**
  (§2a). The *shape* is the anchor.
- **EI_NO rises monotonically with `τ_q`** and **`τ_q → 0` recovers rung 9** (§2b, §3): the
  slow-quench penalty and the ideal-quench limit are the two ends of the same curve.
- **The finite-quench bell re-fills the rich flank** to a ~φ_p-independent floor (§2c): the
  discriminating check that NO is re-made *at the stoich crossing*, not carried from the primary.
- **Clamp dormancy is quantified**: `max(a) = 0.677 < 1` across the whole in-scope sweep; the drop
  is correct-on-principle, dormant-on-numbers (§1d) — a stated surprise, not a swept-under one.
- **`K`-check binds along the trajectory**: 1.031–1.043 over T ∈ [1518, 2453] K (well inside the
  rung-7 assert band 0.90–1.15) — the transcribed rates stay tied to the `a6`/`a7` thermo at every
  temperature the quench visits.
- **Cycle bit-for-bit rung 6** (NO/N never enter `_equil_solve`; `zoned_nox` is a pure
  diagnostic; the finite quench is opt-in via `τ_q`).

---

## 5. What stays UN-anchored (state it plainly)

- **`τ_q` and the linear mixing schedule are knobs**, like `α`, `φ_p`, `τ` — swept to show the
  mechanism, not fitted to a combustor's airflow/mixing schedule. A real quench is a
  jets-in-crossflow mixing field, not a single τ_q with linear addition; we model the *time at
  stoich*, not the fluid mechanics that set it.
- **Super-equilibrium O / prompt (Fenimore) NO** (rung-7 seam) is still deferred — and matters
  *most* in the rich primary *and* at the stoich crossing (super-equilibrium radicals in the
  mixing shear layer). So even the finite quench is an **equilibrium-O lower bound** on the spike.
- **The soot bound (φ_p ≤ 2.0)** is carried from rung 9 unchanged — a scope guard, not a soot
  model.
- **Equilibrium-vs-frozen nozzle expansion** (the rung-6 seam) is where super-equilibrium NO
  *freeze* would actually bite (near-stoich exhaust cooling) — still open, and now explicitly the
  place the dropped clamp (§1d) earns its keep.
- The EI_NO **band** is an order-of-magnitude landing zone (rung 7); what is certified is the
  **shape** — the spike vs `τ_q`, the re-filled rich flank, the reduce to rung 9.

---

## Sources
- A. H. Lefebvre & D. R. Ballal, *Gas Turbine Combustion* — RQL (rich-burn / quick-quench /
  lean-burn) architecture; the "quick" quench requirement and dilution-jet mixing time scales.
- S. R. Turns, *An Introduction to Combustion* — the NO-vs-φ bell traversed by the quench path;
  thermal-NO / extended Zeldovich, EI_NO.
- J. B. Heywood, *Internal Combustion Engine Fundamentals* — NO freezing at super-equilibrium
  during cooling (why the cooling-path integrator must drop the fixed-equilibrium clamp).
- R. K. Hanson & S. Salimian, in *Combustion Chemistry* (Gardiner, ed.) — extended-Zeldovich
  rate constants (carried from rung 7).
- NASA TR 19730017094 — near-stoichiometric NO peak (carried from rungs 8–9):
  https://ntrs.nasa.gov/api/citations/19730017094/downloads/19730017094.pdf
