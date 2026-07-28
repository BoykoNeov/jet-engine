# Rung 54 anchor — the STATOR-ROW THROAT: predictions, and how they scored

Rung 53 shipped the variable stator's swirl/incidence half and named this rung in its own
Concessions and its § The next seam:

> *"A real VSV row also changes the compressor's own flow CAPACITY (the stator throat) and
> rematches the stage stack against itself — the dominant effect in a real multistage machine,
> and the reason a real schedule does not need this model's overspeed. A lumped
> single-stage-equivalent map has neither, and the capacity channel needs a **new constant**
> (area per unit setting). **Refused, and named as this rung's seam.**"*

> *"**The stator-row FLOW CAPACITY channel** ... It needs an anchored area-per-setting law, and
> with it the model would no longer have to buy incidence with overspeed."*

Two claims are embedded there, and this rung takes both on: (i) that the channel needs a new
constant, and (ii) that it would relieve rung 53's overspeed.

---

## The instrument, fixed before any measurement

### Channel 3 — the THROAT. Shape DERIVED, level DISCLOSED.

A cascade's throat is the minimum passage opening `o` between adjacent vanes. For a vane row
of pitch `s` turning the flow to a metal exit angle `α₁` from axial, the standard cascade
relation is `o/s = cos α₁` — the same relation that fixes a blade row's exit angle from its
throat. Rung 53's coordinate is `v ≡ tan α₁`, so:

```
    A_th(v) / A_th(0) = cos α₁ = 1 / sqrt(1 + v²)                     [DERIVED — no constant]
```

**The rotation that buys incidence is the rotation that spends the throat.** The seam's stated
blocker — an *area-per-setting* constant — is dissolved: the area law rides on the SAME swept
coordinate rung 53 already carries.

But the area law is not the capacity. Capacity is `ṁ_corr ∝ A_th · MFP(M_th)`, and evaluating
`MFP` needs to know where on the MFP curve the DESIGN row sits. So the constant is not
removed, it is **replaced** — and the replacement is one number with a physical reading:

```
    C ≡ MFP(M_th0) / MFP(1)  ∈ (0,1)          [DISCLOSED — the design row's fraction of
                                               choking capacity, i.e. its design throat Mach]
```

`C = 0.70 ↔ M_th0 = 0.458`, `0.80 ↔ 0.553`, `0.90 ↔ 0.678` (γ = 1.4). One constant, read as a
Mach number, in the tradition of rungs 36/41's imposed `φ_s0`.

**The throat-referred corrected flow and its margin** (normalised so design = 1):

```
    X(v) ≡ m · sqrt(1 + v²)          M_c = 1 − C·X          choked ⟺ C·X ≥ 1
```

Note `X` uses the corrected flow `m` **unchanged**: annulus continuity gives `Vx = ṁ/(ρA)`
independent of `α₁` (the vane turns the flow, it does not squeeze the annulus), so the throat
does NOT touch `φ = Vx/U`. It only sets where the Mach peaks. That is why rung 53's `φ_op` is
untouched — and it is also the whole reason for P1 below.

### The escape from the constant

Every load-bearing claim is delivered as a **threshold ON** `C`, computed with zero new
constants: `C*(Tt4) = 1/X(v*)` (the schedule's demand), `C_edge = 1/X_edge` (the artifact's
location). The constant chooses a row; the model reports where each row lands.

## Config

Rung 53's, unchanged: CPG + reacting gas, `FLIGHT(250 K, 50 kPa, M0 = 0.85)`,
`π_LPC/π_HPC/Tt4 = 3/6/1500`, LP `(a=0.20, b=0.05, σ=0.1, l=0.7)`,
HP `(a=0.08, b=0.15, σ=0.1, l=1.0)`, `φ_s0 = 0.55`. Five disclosed shapes.
**Choked-envelope floor measured at `Tt4 ≈ 650`** (the nozzle unchokes by 600, rung 39's
OUT-OF-SCOPE assert), so every throttle quoted below is in scope.

---

# THE PREDICTIONS, and how they scored

## P1 — BIND, NEVER RELIEVE (a theorem, registered as one)

> The seam expects capacity to remove rung 53's `+26 %` overspeed. **It cannot, structurally.**
> Rung 53's P1 established that `v` enters the steady solve through `solve_n` **alone**. The
> throat enters no solver at all: `X` is a *post-hoc functional of the already-solved state*.
> An upstream throat therefore cannot change the map from setting to incidence — it can only
> **remove settings from the feasible set**. Rung 31's `(★)` pins the corrected flow at the
> first choked throat DOWNSTREAM (`A4`); a compressor row that is not choked passes whatever
> the turbine demands, and one that is choked does not relieve anything — it stops the engine
> reaching that setting at all.

**SCORED: HOLDS, and is stronger than predicted.** It makes the reduce an **invariance over a
parameter**, not an identity at a point: every matched field is bit-identical for *every* `C`,
not merely at `C = 0`. Gate 1 asserts exactly that.

**Consequence for the seam: its expectation (ii) is REFUTED.** Capacity does not buy back the
overspeed and no area law could. The `+26 %` is a consequence of the map-free ENERGY cascade
plus a single-stage-equivalent loading law; the real multistage mechanism is **stage
rematching**, which needs a stage stack, not an area constant. That is the new seam.

## P-A — is `C_edge` shape-robust? *(registered before the five-shape run)*

The advisor's test: does `C_edge` stay inside **±0.05 of 0.70** (LP)?

> **P-A1: I predict it does NOT.** `C_edge = 1/X_edge` is a threshold on `solve_n`'s
> speed-line bracket — an ARTIFACT (rung 53's own word), located by `psi(m/n)·n² = target`
> losing its bracket at the hard-coded `n = 2.0`. `l` runs 0.7 → 1.2 across the shapes. An
> artifact has no reason to be shape-robust. Expect scatter of order ±0.10.
>
> **P-A2 (the load-bearing form): `C_edge < 0.90` on all five** — i.e. the edge-equivalent
> design throat Mach stays below ~0.68. This is a SIGN claim and should survive where the
> level does not.

**P-A1 SCORED: HOLDS (it does fail the ±0.05 test).** Measured over five shapes × two
throttles: `C_edge ∈ [0.6278, 0.7787]`, width **0.1509**.

| shape | `l` | `σ` | `C_edge` @1500 | `C_edge` @1000 | `M_th0(edge)` |
|---|---|---|---|---|---|
| flow/press | 0.70 | 0.10 | 0.6890 | 0.6933 | 0.453 |
| press/flow | 1.00 | 0.10 | 0.7389 | 0.7104 | 0.467 |
| tilted | 0.85 | 0.20 | 0.6998 | 0.7063 | 0.464 |
| steep | 1.20 | 0.30 | 0.7690 | 0.7787 | 0.531 |
| flat-eta | 0.70 | 0.10 | 0.6278 | 0.6299 | 0.401 |

**P-A2 SCORED: HOLDS on all five** — max `C_edge` = 0.7787, so **any row with a design throat
Mach above 0.531 chokes before the artifact, on every disclosed shape.** The level is
disclaimed; the sign is the claim.

**And the stronger form, measured after the fact: the artifact is NEVER the binding ceiling.**
`v_ch < v_edge` on **20/20** (5 shapes × 4 throttles) at `C = 0.80`, and a fortiori at 0.90.
Rung 53's stated authority ceiling is displaced on every shape — by the throat, or (P-C2) by
the incidence peak, but never left standing.

## P-B — the severity inversion: retained swing at `C = 0.90`, `Tt4 = 1000`

> **I predict ≥ 85 % on all five**, and further that retention is **monotone rising in `l`**:
> `dM_i/dv = 1 − (1+l)/D_v`, so a steeper loading slope makes `φ` fall faster with `v`, which
> makes `1/φ` climb faster against `+v` — stronger saturation, so the truncated tail is worth
> less. "steep" (`l = 1.2`) should retain most, "flow/press" (`l = 0.7`) least.

**SCORED: the HEADLINE holds, the 85 % line MISSES by 0.4 pt, and the REASONING was wrong.**

- Against the artifact endpoint, three shapes retained **> 100 %** — impossible unless `M_i` is
  non-monotone. That is what sent me to P-C2, and it is this rung's biggest finding.
- Re-measured in the correct currency (retention against the **achievable peak**, not against
  an artifact): at `C = 0.90` retention is **90.5 / 96.4 / 98.8 / 92.7 / 84.6 %** across the
  five shapes at `Tt4 = 1000`; over the whole grid (5 shapes × 4 throttles × C ∈ {0.80, 0.90})
  it never falls below **54.9 %**, and below **76 %** only at the DESIGN throttle, where the
  schedule asks for `v* = 0` and no authority is wanted.
- **The monotone-in-`l` sub-prediction FAILS outright**: the two `l = 0.70` shapes give 90.5 %
  and 84.6 %. Retention is not a function of `l`.
- **The inversion itself is confirmed and is the rung:** at `Tt4 = 1000`, `C = 0.90` cuts the
  SETTING from `v_edge = 2.52` to `v_ch = 1.754` (**−30 %**) and the MARGIN from 0.9060 to
  0.8697 (**−4.0 %**, i.e. 90.5 % of the swing retained). The cut is severe in the lever's own
  coordinate and nearly free in the protected variable.
- **A claim I made and then withdrew.** The raw read `M_i(v_ch)` is non-monotone in `C` on
  `steep` (97.6 % at `C = 0.90` vs 93.7 % at 0.80, `Tt4 = 1500`), and I first wrote that as
  *"a tighter throat can be worth more than a looser one."* **That is wrong**, and it is an
  artifact of scoring past the peak: no operator would set the stator beyond `v_peak`, so the
  usable authority is `M_i(min(v_ch, v_peak))`. Under that (shipped) definition retention is
  monotone in `C` and the curiosity disappears. `authority_ceiling` returns both
  (`m_i_at_throat` raw, `m_i_usable` clipped) so the difference stays visible.

## P-C — the design-loading crossing `X(v*) = 1`

> **I predict 3 of 5 inside `Tt4 ∈ [800, 950]`, all five inside `[700, 1050]`.**

**SCORED: FAILS.** Only **flow/press** crosses in range (bracketed **870 / 860**:
`X = 0.99788 → 1.00667`). `press/flow` reaches only 0.989 and `flat-eta` 0.919 by `Tt4 = 700`;
`tilted` and `steep` have no schedule that far down at all (P-C2). So the crossing is a
**shape-conditional** object and is reported as one — on the default shape it is a genuine
constant-free boundary:

> **Above `Tt4 ≈ 865` the incidence schedule demands LESS throat than the design point itself,
> so it is feasible for EVERY conceivable row (any `C < 1`), with no disclosed constant
> involved. Below it, feasibility becomes `C`-dependent** — `C*` = 0.985 (850), 0.942 (800),
> 0.903 (750), 0.841 (650), i.e. throat-limited for `M_th0` above 0.869 / 0.752 / 0.683 / 0.599.

**Rung 53's entire published band (`Tt4 ∈ [1000, 1500]`) sits above the crossing on the default
shape.** The channel it refused is provably inert over everything it published.

### P-C2 — the finding P-B's overshoot forced out: THE TURNING POINT IS REACHED

Not predicted; found by refusing to accept a >100 % retention. Rung 53 § Concessions:

> *"The incidence benefit SATURATES in `v` and does not turn back ... `solve_n`'s speed-line
> bracket (a map-validity edge) is reached first. (The apparent turning point that this algebra
> suggests is **not** reached — see the anchor.)"*

**Measured — `v_peak` = argmax `M_i(v)`, against `v_edge`:**

| shape | @1500 | @1200 | @1000 | @800 | interior peak? | max drop past peak |
|---|---|---|---|---|---|---|
| flow/press | at edge | at edge | at edge | at edge | **no** | 0 |
| flat-eta | at edge | at edge | at edge | at edge | **no** | 0 |
| press/flow | 1.161 / 1.20 | 1.763 / 1.84 | 2.296 / 2.48 | 2.931 / 3.28 | yes | 0.0023 (immaterial) |
| tilted | 1.157 / 1.20 | 1.597 / 1.80 | 1.966 / 2.40 | 2.370 / 3.16 | yes | **0.0199** |
| steep | 0.750 / 1.12 | 1.077 / 1.76 | 1.367 / 2.32 | 1.690 / 3.08 | yes | **0.0761** |

**Interior peak on 3 of 5 shapes; MATERIAL on 2 of 5.** Rung 53's concession is true on the
shape it was measured on and false on others — it generalised one shape. On `steep` the peak
sits at 55–67 % of the reachable range and `M_i` falls 11 % of its own peak value past it.

**And the consequence for rung 53's P7 payoff object — the schedule ceases to EXIST:**

| shape | 1200 | 1000 | 800 |
|---|---|---|---|
| flow/press | `v*` = 0.577 | 1.244 | 2.197 |
| press/flow | 0.514 | 1.050 | 1.780 |
| flat-eta | 0.487 | 0.962 | 1.581 |
| tilted | 0.585 | 1.489 | **NONE** (min `tan β₁` = 1.0574 > 1) |
| steep | 0.910 | **NONE** (1.0722 > 1) | **NONE** (1.1467 > 1) |

### P-C3 — the correction reaches rung 53's CODE, not only its prose

Found while wiring `schedule_throat`: rung 53's `incidence_schedule` brackets its root with a
**doubling ladder**, justified in its own docstring by

> *"Because closing the stators lowers tan beta_1 monotonically (`dM_i/dv > 0`), the residual
> is monotone decreasing in `v` and a bracketed secant is safe."*

Where the incidence peak is interior that premise is false: past the peak `tan β₁` turns back
**up**, so the ladder can step OVER the root and out the far side. Measured on `steep` at
`Tt4 = 1200`: a root exists at `v* = 0.909`, and rung 53's method instead walks 0.05 → … → 1.6
(residual `+1.64e-2`, already past the peak and climbing) and **asserts the schedule is
unreachable**. Rung 54's `_schedule_root` brackets off the scan instead and is immune;
where rung 53's ladder succeeds the two roots agree (gated, ≤ 1e-9).

**Rung 53's published P7 numbers are NOT affected** — its table is the `flow/press` shape,
where the residual really is monotone. The defect is latent, and only the shapes rung 53 did
not tabulate expose it. Rung 53's method is left algorithmically untouched (its numbers are
its own); a pointer to this finding is added to its docstring.

**The rung-28 shape: rung 53's VERDICT survives, its REASON is corrected.** Rung 53 disclosed
that the stator has finite authority and cannot restore design incidence arbitrarily far off
design — correct. But the ceiling is the incidence **PEAK**, an aerodynamic property of the
loading law, not `solve_n`'s bracket; and on two disclosed shapes it is reached **inside the
envelope**, not beyond it.

## P-D — the evenness control *(mine, registered with P-A/B/C)*

> On the **flat-η** shape (`a = b = c = 0`) rung 53's P5 exact zero says `m` cannot move with
> `v` at all. So `X(v) = m·√(1+v²)` should be **EXACTLY even in `v`**, to machine precision,
> while every shaped island gives a measurably uneven cost.

**SCORED: HOLDS, exactly.** At `Tt4 = 1500`, LP:

| `|v|` | flat-η `X(+v) − X(−v)` | flow/press | steep |
|---|---|---|---|
| 0.2 | **0.00e+00** | 5.15e-04 | 5.91e-04 |
| 0.4 | **0.00e+00** | 3.88e-03 | 4.19e-03 |
| 0.6 | **0.00e+00** | 1.19e-02 | 1.17e-02 |

So the geometric capacity cost is **exactly two-sided by construction**, and the measured
asymmetry is **entirely the efficiency island's** — it vanishes bit-for-bit when the island is
flat. (`m(−v) − m(+v)` is likewise `0.00e+00` on flat-η.)

## P5 — the exposure split, inherited

**HOLDS.** The HP schedule's throat demand *falls* monotonically (`X(v*)` = 0.958 → 0.725 from
`Tt4` 1400 → 800) while the LP's turns back up. **The capacity ceiling is a pure-LP object**,
because the demand is: rung 53's P7 needs `v*_LP` ≈ 6.7 × `v*_HP`, and the throat cost goes as
`√(1+v²)`, so the LP eats it quadratically faster. Inherited from rungs 41/44/45/53's split,
not new.

---

## Scorecard

| # | prediction | verdict |
|---|---|---|
| P1 | bind-never-relieve; the channel cannot relieve the overspeed | **HOLDS**, stronger (invariance over `C`) |
| P-A1 | `C_edge` fails the ±0.05 shape-robustness test | **HOLDS** (width 0.151) |
| P-A2 | `C_edge < 0.90` on all five | **HOLDS** (max 0.779) |
| P-B | ≥ 85 % retention at `C = 0.90`, `Tt4 = 1000` | **MISSES** (84.6 %); inversion confirmed |
| P-B′ | retention monotone in `l` | **FAILS** |
| P-C | crossing inside `[800, 950]` on 3 of 5 | **FAILS** (1 of 5; 2 have no schedule) |
| P-C2 | *(unregistered — forced out by P-B's overshoot)* | rung 53's concession **CORRECTED** |
| P-D | flat-η evenness exact, shaped islands not | **HOLDS**, exactly |

Three of the registered predictions failed. P-B's failure is what produced P-C2, the rung's
largest result — the same pattern as rungs 42/46/49, where the probe inverted the author.

## Probe transcripts

`M:\claud_projects\temp\rung54\probe{1..7}_*.py` (out of tree, per the temp-file policy) and
`PREDICTIONS.md` there, written before the five-shape run.
