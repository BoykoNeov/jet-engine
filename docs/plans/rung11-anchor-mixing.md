# Rung-11 external anchors — the physical mixing model: a jet-entrainment quench

Verified scalings + a machine-checked worked example that anchor the **mean-field jet-entrainment
quench** (rung 11): the model that **retires rung 10's two quench knobs** — the free quench time
`τ_q` and the *linear* mixing schedule — by deriving both from **jet-in-crossflow physics**. Every
number was re-derived **before any production code** (project discipline); the worked example runs
on the *existing* rung-6/7/8/9/10 primitives (`_equilibrium_composition`, `_primary_aft`,
`_mixed_out_T`, `_quench_trajectory`, `_thermal_no`, `_k_zeldovich`) plus the one new thing — a
**schedule-aware** quench integrator (β decoupled from time) — in
`M:\claud_projects\temp\rung11\proto_mixing.py`.

> **Read `docs/rung10-spec.md` first.** Rung 11 is built *on* rung 10: the τ_q-**independent**
> trajectory (`T`, `[O]`, `[N₂]`, `[H]`, `[NO]_e` as functions of the **dilution fraction β**) is
> reused **unchanged** — the fast chemistry doesn't know how fast mixing is. Rung 11 changes ONLY
> (a) where `τ_q` comes from (now **derived** from the jet momentum-flux ratio `J`) and (b) the
> **β↔t mapping** (now a decelerating **entrainment** schedule, not linear). Nothing in the
> thermochemistry, the `Kp`/`_equil_solve` machinery, the `a6`/`a7` substrate, or the clamp-free
> integrator's *form* changes. NO stays a **trace, decoupled diagnostic**, so the cycle is still
> **bit-for-bit rung 6**.

**The lesson (it grounds rung 10's abstract `τ_q` in real hardware).** Rung 10 showed a rich
primary's NO is *contingent on a fast quench* — but "fast" was a free knob (`τ_q`) with an
arbitrary *linear* mixing schedule. Rung 11 asks: **what sets the quench rate?** In a real
combustor the dilution air enters through **jets in crossflow**; the rate at which those jets
penetrate and entrain the hot core sets the mixing time, and it scales with the jet
**momentum-flux ratio** `J = ρ_j U_j² / (ρ_c U_c²)`. So "quick quench" becomes a concrete design
requirement: **a high-momentum jet**. A strong jet (high `J`) penetrates deep and mixes fast →
short `τ_q` → the gas escapes the stoichiometric NO peak quickly → low NO. A weak jet (low `J`)
under-penetrates and mixes slowly → the gas dwells at stoich → the NO spike rung 10 quantified.
The τ_q rung 10 *swept* is now **read off the jet design**.

---

## 1. Sourced scalings (numbers before code)

### 1a. The governing group — momentum-flux ratio `J` (the design knob)
Jets-in-crossflow penetration and mixing are governed by the **jet-to-crossflow momentum-flux
ratio** `J = q = (ρ_j U_j²) / (ρ_c U_c²)` — the single most important dilution-jet design
parameter (Lefebvre & Ballal; Holdeman). The jet **penetration depth scales as** `y/d ∝ √J`
(equivalently, jet trajectories collapse when the axial coordinate is normalized by `J·d`) — a
result robust across incompressible dilution jets through supersonic injection. Higher `J` ⇒
deeper penetration ⇒ faster cross-duct transport of the dilution air into the hot core.

### 1b. The derived quench time `τ_q` (retires rung 10's free knob)
Collapse jet penetration + shear-layer entrainment into **one** cross-stream mixing velocity
`U_mix = C_e·√J·U_c` (a higher-momentum jet penetrates *and* entrains faster; `U_mix ∝ √J` follows
1a and `U_j ∝ √J·U_c` from the definition of `J`). Complete the cross-duct mix over the duct
height `H`:

```
τ_q = H / U_mix = H / (C_e · √J · U_c)                         (monotone-DECREASING in J)
```

- `H` (dilution-zone duct height, ~0.1 m), `U_c` (bulk crossflow velocity, ~75 m/s) are
  dilution-zone **geometry / flow** — order-of-magnitude, un-anchored (a real design reads them off
  the cycle mass flow + liner area). **Note `U_c` cancels:** since `U_j = U_c·√(J·ρ_c/ρ_j)`, the
  group `√J·U_c ∝ U_j`, so `τ_q ∝ H/U_j` — **duct height over JET velocity, independent of the
  crossflow speed**. "Sweep `J` at fixed `U_c`" is physically "**sweep the jet velocity**" — it is
  the *jet's* momentum, not the crossflow's, that sets how fast the hot core is stirred out.
- `C_e` is an **entrainment constant** of order 0.1 (turbulent-jet entrainment coefficients run
  α_e ≈ 0.08–0.16; `C_e ≈ α_e·√(ρ_c/ρ_j)` folds in the hot/cold density ratio). **Un-anchored**,
  like the absolute Zeldovich rate (rung 7) and `τ`/`α` — the **absolute** `τ_q` is un-pinned; what
  is certified is the **√J scaling** and the **monotone direction**.
- For physical `J ∈ [4, 100]` this lands `τ_q ∈ 0.67–3.3 ms` — squarely in the RQL **sub-ms–few-ms
  quench band** (Lefebvre & Ballal), so the *derived* time is physically sensible (§2b).

### 1c. The entrainment schedule shape (retires rung 10's *linear* schedule)
Rung 10 added the dilution air **linearly** in time (β = t/τ_q). Leading-order turbulent-jet
entrainment is a **constant** entrainment rate (Morton–Taylor–Turner: `dṁ_e/ds ∝ const` for a
free jet) — so the linear schedule is secretly the **constant-entrainment limit**, not an arbitrary
choice. But as the jet bends over and its excess velocity decays, the entrainment rate **slows** as
the concentration gradient collapses → a **decelerating (concave)** β(t). Rung 11 models this with a
one-parameter finite-time family that **reaches β=1 exactly at t=τ_q** (no endpoint trap):

```
β(t) = 1 − (1 − t/τ_q)^n ,  t ∈ [0, τ_q]
   n = 1 : β = t/τ_q                 → LINEAR = constant entrainment = rung 10 (the reduce)
   n > 1 : concave / DECELERATING    → fast near the jet, slowing as the gradient collapses
```

`n` (the entrainment-driving exponent) is the **residual shape choice** — physically it must be
> 1 (decelerating), and NO is only **modestly** sensitive to it (a factor ~2 across n∈[1,3], §2c),
vs the **orders-of-magnitude** sensitivity to `τ_q`/`J`. Default **n=2**.

### 1d. Carried over from rungs 7–10 (unchanged)
- The τ_q-**independent trajectory** (`_quench_trajectory`): `T(β)`, `[O]`, `[N₂]`, `[H]`,
  `[NO]_e` — fast chemistry, a function of β alone. **Reused verbatim.**
- The **clamp-free** reverse-rate Zeldovich integrator form `d n_NO/dt = 2R1(1−a²)/(1+βa)·V`,
  `a=[NO]/[NO]_e` — extensive NO, **no equilibrium cap** (super-eq freeze on cooling; rung-10 §
  the clamp trap). The `K`-check + trace guards bind along the **whole** trajectory.
- The EI_NO **ICAO band** (18–64 g/kg take-off) — an order-of-magnitude landing zone (rung 8).

---

## 2. The jet-entrainment worked example

**Model (rung-10 flow, now with a jet-derived quench):** identical setup — split the air (α to
the rich primary with all the fuel), burn the primary from Tt3 (rich equilibrium, CO/H₂ major),
run rung-7 `_thermal_no` there → the inherited primary NO. Then quench, but now: `τ_q` is
**derived** from `J` (§1b) and the air is added on the **decelerating entrainment** schedule
(§1c) instead of linearly. NO integrates (clamp-free) along the *same* β-trajectory; only the
β↔t map and the total time change.

**Worked example** (`main.py` design point: **Tt3 = 583.5 K, Tt4 = 1500 K, far = 0.0272,
p = 7.47 bar**; φ_overall = 0.402; rich primary **φ_p = 1.5**, primary AFT = 2110 K, α = 0.268,
T_peak = 2452 K @ β=0.16 — the stoich crossing, *early/low-β*):

**(a) The reduce — n=1 (constant entrainment) + a given `τ_q` == rung 10, to machine precision:**

| `τ_q` | rung-10 `_quench_no` EI | n=1 scheduled EI | rel. diff |
|---|---|---|---|
| 0.10 ms | 0.11074 | 0.11074 | 3.4e-8 |
| 1.00 ms | 1.09209 | 1.09209 | 3.5e-8 |
| 3.00 ms | 3.25014 | 3.25014 | 3.4e-8 |

(The ~3e-8 residual is only the prototype's finer RK4 step; the production reduce is a
**short-circuit** — `mixing=None` runs the *exact* rung-10 code — hence **bit-for-bit**, §3.)

**(b) The monotone J-sweep — a stronger jet quenches faster and re-makes less NO** (n=2 shape,
H=0.10 m, U_c=75 m/s, C_e=0.20):

| `J` | derived `τ_q` | EI_NO (g/kg) |
|---|---|---|
| 4 (weak jet) | 3.33 ms | 2.062 |
| 9 | 2.22 ms | 1.378 |
| 16 | 1.67 ms | 1.035 |
| 25 | 1.33 ms | 0.829 |
| 49 | 0.95 ms | 0.593 |
| 100 (strong jet) | 0.67 ms | 0.416 |

**Monotone**: EI_NO falls smoothly as `J` rises (5× less NO from J=4→100). The τ_q rung 10 swept
as a free knob is now **read off the jet momentum**. This is the whole payoff — "quick quench" =
"high-momentum jet," quantified. (It is **monotone by construction** — a mean-field model has no
mixing *optimum*; see §5.)

**(c) The schedule-shape effect — a realistic decelerating entrainment clears the early stoich
crossing faster than linear** (fixed `τ_q` = 1 ms):

| shape `n` | kind | EI_NO (g/kg) | max_a |
|---|---|---|---|
| 0.5 | convex / accelerating | 1.695 | 0.040 |
| 1.0 | **LINEAR (rung 10)** | 1.092 | 0.026 |
| 2.0 | concave / decelerating | 0.622 | 0.015 |
| 3.0 | concave / decelerating | 0.434 | 0.010 |

Because the stoich crossing is at **low β (0.16)**, a decelerating schedule (high dβ/dt early)
sweeps *past* it fast → **less** NO. So **if** entrainment decelerates (as gradient-collapse
suggests), rung 10's linear schedule **over-predicted** the spike (by ~2× here). Be explicit that
this is **shape-contingent**: the sign flips for an *accelerating* schedule (n=0.5 → *more* NO than
linear). The shape sensitivity (~2× over n∈[1,3]) is real but **small** next to the τ_q/J
sensitivity (orders of magnitude, §2b) — so "conservative" is a contingent conclusion, not a bare
fact.

---

## 3. The reduce-to-rung-10 gate (exact by construction)

The production reduce is a **short-circuit**, not an empirical limit: `zoned_nox` gains a `mixing`
parameter (a `JetMixing` config) defaulting to `None`. With `mixing=None` the τ_q path is **exactly
rung 10** (linear schedule); with `tau_q=None` too, it is the rung-9 ideal quench. `mixing` and
`tau_q` are **mutually exclusive** (assert one-or-the-other — like the isentropic/polytropic knob).
So every existing call — every rung-1..10 test, `main.py`, the whole suite — is untouched and stays
**bit-for-bit rung 10** (hence rung 9, hence rung 6, cycle untouched). Only a call that explicitly
passes a `JetMixing` enters the new schedule-aware integrator. And a `JetMixing` with `shape_n=1`
(constant entrainment) is **bit-for-bit** the rung-10 linear path at the *derived* `τ_q` (same
`_quench_no`, same `nsteps`, identity schedule) — the reduce is provable at two levels.

---

## 4. The invariants (discriminating checks, not book digits)

- **Reduce is exact** (§3): `mixing=None` is byte-identical rung 10; `shape_n=1` matches the linear
  path at the derived `τ_q`.
- **EI_NO is monotone-DECREASING in `J`** (§2b): the load-bearing physical direction — a stronger
  jet quenches faster and re-makes less NO. A wrong sign fails it.
- **`τ_q ∝ 1/√J`** (§1b): halving requires 4× the momentum; the derived `τ_q` stays in the RQL
  sub-ms–few-ms band for physical `J`.
- **A decelerating schedule (n>1) makes LESS NO than linear at fixed τ_q** (§2c): the discriminating
  shape check — NO is re-made at the *early* stoich crossing, which a decelerating entrainment
  clears fast.
- **Clamp dormancy persists** (`max_a` ≈ 0.01–0.04 ≪ 1 across the J-sweep, §2c) — the dropped cap
  stays correct-on-principle, dormant-on-numbers at this lean point (carried from rung 10).
- **`K`-check + trace guards bind along the trajectory** (reused unchanged from rung 10).
- **Cycle bit-for-bit rung 6** (NO/N never enter `_equil_solve`; `zoned_nox` is a pure diagnostic;
  the jet-mixing quench is opt-in via `mixing`).

---

## 5. What stays UN-anchored / deferred (state it plainly)

- **This is a MEAN-FIELD model** — a single well-mixed core diluting on a mean β(t). It can only
  produce the **monotone** story (higher J → faster mix → less NO). It **cannot** produce a mixing
  **optimum**, because an optimum is a *variance* effect: an over-penetrating jet piles dilution air
  on the far wall and leaves an under-mixed hot near-stoich core. That spatial **unmixedness** — and
  with it the classic **Holdeman dilution-jet optimum** `C = (S/H)·√J ≈ 2.5` (Holdeman 1993, a
  *uniformity* criterion, **not** a mean rate) — is the **deferred rung-12 seam**. We deliberately do
  **not** dress this mean-field rung in Holdeman-optimum clothing; the optimum needs the variance
  model rung 12 will add.
- **We use "slow mean mixing" as a surrogate for a variance effect.** In a strict mean-field
  picture `J` governs cross-plane *uniformity* more than the mean addition rate; the real reason a
  weak jet is bad for NO is the un-mixed hot core (variance). `C_e` folds penetration + entrainment
  into one rate — **stated, not hidden**.
- **`τ_q`'s absolute value is un-pinned** (`C_e`, `H`, `U_c` are order-of-magnitude): what is
  certified is the **√J scaling** and the **monotone direction**, not a book τ_q.
- **The shape exponent `n`** is a residual modeling choice (must be >1; NO ~2× sensitive over
  n∈[1,3]) — physically decelerating, defaulted to 2, exposed as a knob.
- **Super-equilibrium O / prompt (Fenimore) NO** (rung-7 seam) still deferred — and matters *most*
  in the rich primary + the radical-rich mixing shear layer, so even this is an equilibrium-O
  **lower bound** on the spike (carried from rung 10).
- **The soot bound (φ_p ≤ 2.0)** carried from rung 9 unchanged.
- **Equilibrium-vs-frozen nozzle expansion** (the rung-6 seam) still open.

---

## Sources
- A. H. Lefebvre & D. R. Ballal, *Gas Turbine Combustion* — dilution-zone jets in crossflow, the
  momentum-flux ratio `J`, penetration/mixing time scales, the RQL "quick" quench requirement.
- J. D. Holdeman, "Mixing of multiple jets with a confined subsonic crossflow," *Prog. Energy
  Combust. Sci.* 19 (1993) — jets-in-crossflow mixing; the `(S/H)√J` optimum (cited as the
  **rung-12** variance seam, **not** used here).
- R. J. Margason, "Fifty years of jet in cross flow research" (AGARD) — the `y/d ∝ √J` penetration
  scaling, standard across the JICF literature.
- H. B. Fischer et al. / Morton–Taylor–Turner turbulent entrainment — the constant-entrainment
  free-jet limit (why the linear schedule is the leading-order shape).
- S. R. Turns, *An Introduction to Combustion*; J. B. Heywood, *ICE Fundamentals* — the NO-vs-φ bell
  and super-equilibrium NO freeze on cooling (carried from rungs 7–10).
- Jet-in-crossflow penetration ∝ √J, confirmed across configurations:
  https://www.sciencedirect.com/science/article/abs/pii/S0017931017317015 ,
  https://www.osti.gov/servlets/purl/1559030
