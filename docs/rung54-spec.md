# Rung 54 — the STATOR-ROW THROAT: what a CONSTRAINT costs, and in which coordinate

Rung 53 shipped the variable stator's swirl/incidence half and refused the other half twice —
once in its Concessions, once in its § The next seam:

> *"A real VSV row also changes the compressor's own flow CAPACITY (the stator throat) ... and
> the capacity channel needs a **new constant** (area per unit setting). **Refused, and named
> as this rung's seam.**"*

> *"It needs an anchored area-per-setting law, and with it the model would no longer have to
> buy incidence with overspeed."*

Two claims sit in there. This rung takes both, and neither survives intact.

---

## THE HEADLINE — rung 53's law, one level up

> **Rung 53: a MARGIN is a distance, so it is coordinate-dependent. Rung 54: so is a
> CONSTRAINT'S SEVERITY. The throat cuts the stator's SETTING by 30 % and its incidence
> MARGIN by 4 % — and where the benefit has already turned over, by nothing at all. A limit
> that looks severe in the lever's own units is nearly free in the protected variable.**

Measured on the default shape at `Tt4 = 1000`, `C = 0.90`: the row chokes at `v_ch = 1.754`
against a reachable `v_edge = 2.52` — a **30.4 %** cut in the coordinate — while `M_i` falls
0.9060 → 0.8697, **4.0 %**, retaining **90.5 %** of the whole swing from `v = 0`. Across all
five disclosed shapes and the throttle band, retention never drops below **78 %** while the
setting is cut by up to 30 %; the two never even come close to matching.

The mechanism is rung 53's own saturation concession, read the other way round. Rung 53 named
saturation as a *limitation* ("a stator has finite authority"). It is also what makes a hard
limit **cheap**: the tail the throat amputates was worth almost nothing.

---

## The instrument — shape DERIVED, level DISCLOSED, verdicts as THRESHOLDS

**This rung adds one constant.** Rung 53 added none, and that discipline is not weakened by
pretending otherwise — it is honoured by making the constant physical and by keeping the
claims free of it.

### Channel 3 — the THROAT. The area law needs no constant.

A cascade's throat is the minimum opening `o` between adjacent vanes; for pitch `s` and metal
exit angle `α₁` from axial, the standard cascade relation is `o/s = cos α₁` — the same relation
that fixes a blade row's exit angle from its throat. Rung 53's coordinate is `v ≡ tan α₁`, so

```
    A_th(v) / A_th(0) = cos α₁ = 1 / sqrt(1 + v²)                     [DERIVED, no constant]
```

**The rotation that buys incidence is the rotation that spends the throat.** One coordinate,
now three channels: `psi` (the work), `phi_surge_at` (the floor), `throat_ratio` (the throat).
The seam's stated blocker — an *area-per-setting* constant — **is dissolved**.

### What the constant actually is

The area law is not the capacity. Capacity is `ṁ_corr ∝ A_th · MFP(M_th)`, and evaluating
`MFP` requires knowing where on the MFP curve the DESIGN row sits. So the constant is not
removed, it is **replaced** — by one number with a physical reading:

```
    C ≡ MFP(M_th0)/MFP(1) ∈ (0,1)     the design row's fraction of choking capacity
    C = 0.70 ↔ M_th0 = 0.458   0.80 ↔ 0.553   0.90 ↔ 0.678        (γ = 1.4)
```

`ComponentMap.design_throat_mach()` inverts it, so the disclosure is in units an engineer can
judge rather than an abstract fraction. `C = 0` means **no throat model**, exactly as
`phi_surge = 0` means no surge line.

### The currency, and the escape from the constant

```
    X(v) = m · sqrt(1 + v²)      throat-referred corrected flow / design   [NO constant]
    c_min = 1/X                  the row chokes here IFF C ≥ c_min         [NO constant]
    M_c = 1 − C·X                the margin                                [needs C]
```

`X` uses the face corrected flow `m` **unchanged**. Annulus continuity gives `Vx = ṁ/(ρA)`
independent of `α₁` — the vane *turns* the flow, it does not squeeze the annulus — so the
throat never touches `φ = Vx/U`. It only sets where the Mach peaks. **That is the whole reason
this channel is diagnostic-only**, and it is P1.

Every load-bearing number below is reported as `c_min` / `C_edge` / `C*(Tt4)` — a **derived
threshold ON** the constant. The constant chooses a row; the model says where that row lands.

---

## P1 — BIND, NEVER RELIEVE (a theorem, and a stronger reduce than rung 53's)

Rung 53's P1 established that `v` enters the steady solve through `solve_n` **alone**. The
throat enters **no solver at all**: `X` is a post-hoc functional of the already-solved state.
Therefore an upstream throat **cannot change the map from setting to incidence** — it can only
**remove settings from the feasible set**. Rung 31's `(★)` pins the corrected flow at the first
choked throat *downstream* (`A4`); a compressor row that is not choked passes whatever the
turbine demands, and one that *is* choked does not relieve anything — it stops the engine
reaching that setting at all.

**THE REDUCE IS AN INVARIANCE OVER A PARAMETER**, stronger than rung 53's identity at a point:
every matched field is bit-identical for **every** `C`, at a **moved** stator, on both gases —
including a `C` at which the row is provably choked (gate 4). Rung 53 earned an identity at one
setting; rung 54 earns it over a whole axis.

**Consequence: the seam's second expectation is REFUTED.** Capacity does not buy back rung 53's
`+26 %` overspeed, and **no area law could** — the refutation is structural, not a matter of
magnitude. The overspeed is a consequence of the map-free ENERGY cascade plus a
single-stage-equivalent loading law. The real multistage mechanism is **stage rematching**,
which needs a stage stack, not an area constant. That is the new seam.

---

## The artifact is displaced — on every shape

Rung 53 conceded that its authority ceiling was `solve_n`'s speed-line bracket, *"a map-validity
edge"* — an artifact. Once the throat is modelled, **the artifact is never what stops the
stator**: `binds != "edge"` on **20/20** (five shapes × four throttles) at `C = 0.80`.

**Which physical ceiling does the stopping is not uniform, and the claim is careful about it.**
`v_ch < v_edge` also holds 20/20, but on `steep` the incidence PEAK is inside the throat
(`v_peak < v_ch < v_edge`), so there the *peak* is what binds and the throat is merely also
present. The load-bearing statement is the negative one — **the artifact never binds** —
carried by `binds`, not by `throat_before_edge`. The tightest cell is `steep` at the design
throttle (`v_ch` = 0.993 vs `v_edge` = 1.12, ~11 %); it is the first that would flip if the
scan resolution changed, so gate 6 pins `_V_STEP` alongside the claim.

The threshold itself is `C_edge = 1/X_edge`. Measured across the five shapes it runs
**0.628 … 0.779** (design throat Mach **0.40 … 0.53**) — it **fails** a ±0.05 robustness test,
and it was predicted to: it is a threshold on an *artifact*, and an artifact has no reason to be
shape-robust. **The LEVEL is disclaimed; the SIGN is the claim** — any row whose design throat
Mach exceeds 0.53 chokes before the bracket, on every disclosed shape.

---

## P-C2 — the TURNING POINT is reached: rung 53's concession CORRECTED

Rung 53 § Concessions:

> *"The incidence benefit SATURATES in `v` and does not turn back ... `solve_n`'s speed-line
> bracket is reached first. (The apparent turning point that this algebra suggests is **not**
> reached — see the anchor.)"*

Found by refusing a retention figure above 100 %, which is impossible unless `M_i` turns over.

| shape | interior peak? | `v_peak` / `v_edge` @1000 | max drop past the peak |
|---|---|---|---|
| flow/press | **no** — runs to the edge | 2.52 / 2.52 | 0 |
| flat-eta | **no** — runs to the edge | 2.48 / 2.48 | 0 |
| press/flow | yes | 2.296 / 2.48 | 0.0023 *(immaterial)* |
| tilted | yes | 1.966 / 2.40 | **0.0199** |
| steep | yes | 1.367 / 2.32 | **0.0761** |

**Interior peak on 3 of 5 shapes, material on 2 of 5.** Rung 53's concession is true on the
shape it was measured on and false on others: it generalised one shape. On `steep` the peak sits
at 55–67 % of the reachable range and `M_i` falls 11 % of its own peak value past it.

**And rung 53's P7 payoff object ceases to EXIST**: where the peak falls short of the design
incidence there is no schedule at all — `tilted` loses it by `Tt4 = 800`
(`min tan β₁` = 1.0574 > 1), `steep` by `Tt4 = 1000` (1.0722 > 1), both **inside** the choked
envelope (floor `Tt4 ≈ 650`).

**This is the rung-28 shape: the VERDICT survives, the REASON is corrected.** Rung 53 was right
that the stator has finite authority and cannot restore design incidence arbitrarily far off
design. But the ceiling is the incidence **PEAK** — an aerodynamic property of the loading law —
not a map-validity edge, and it is reached inside the envelope, not beyond it.

### P-C3 — the correction reaches rung 53's CODE

`incidence_schedule` brackets its root with a **doubling ladder**, justified in its own docstring
by *"the residual is monotone decreasing in `v`"*. Where the peak is interior that premise fails:
past the peak `tan β₁` turns back up, and the ladder steps **over** the root and out the far
side. Measured on `steep` at `Tt4 = 1200`: a root exists at `v* = 0.909`; rung 53's method walks
0.05 → … → 1.6 (residual `+1.64e-2`, already climbing) and asserts the schedule unreachable.

Rung 54's `_schedule_root` brackets off a scan and is immune; where the ladder succeeds the two
agree to ≤ 1e-9 (gated both ways). **Rung 53's published numbers are unaffected** — its table is
`flow/press`, where the premise holds — so its method is left algorithmically untouched and only
its docstring gains a pointer. The defect is latent, and only the shapes rung 53 did not tabulate
expose it.

---

## THE RACE — a constant-free region boundary

As power falls the schedule's demand `v*` **rises** while the flow `m` **falls**, so the
schedule's throat loading `X(v*)` is a race between them — and it has an **interior minimum**
(0.828 at `Tt4 ≈ 1200` on the default shape): the throttle wins above it, the schedule below.

| `Tt4` | 1200 | 1000 | 900 | **870** | **860** | 800 | 750 | 650 |
|---|---|---|---|---|---|---|---|---|
| `X(v*)` | 0.8281 | 0.8962 | 0.9720 | **0.9979** | **1.0067** | 1.0613 | 1.1074 | 1.1889 |
| `C*` | 1.208 | 1.116 | 1.029 | **1.002** | **0.993** | 0.942 | 0.903 | 0.841 |

> **Above `Tt4 ≈ 865` the schedule demands LESS throat than the DESIGN point itself, so it is
> feasible for EVERY conceivable row — any `C < 1` — with no disclosed constant involved.**
> Below it feasibility becomes `C`-dependent: throat-limited for `M_th0` above 0.87 (`Tt4`=850),
> 0.75 (800), 0.68 (750), 0.60 (650).

**Rung 53's entire published band (`Tt4 ∈ [1000, 1500]`) sits above the crossing.** The channel
it refused is provably inert over everything it published. The crossing is **shape-conditional**
and reported as such — of the five shapes only `flow/press` crosses inside the envelope.

## The exposure split, INHERITED

> **⚠ CORRECTED BY RESOLUTION — rung 56 P5.** Everything below is a **FACE** statement and stays
> true as one. Resolved onto rung 55's stage stack, the HP's *rear row* reads the opposite sign
> in throttle (the face `M_c` rises 0.216 → 0.365 over `Tt4` 1200 → 800 while the rear row's
> falls 0.216 → 0.164, `C*` = 0.913), so the machine's **capacity** exposure is the HP's even
> though its **schedule's** demand is not. Rung-28 shape: the reasoning survives at the fidelity
> it was made, the verdict does not survive resolution. See `docs/rung56-spec.md` § P5.

The capacity ceiling is a **pure-LP** object. The HP schedule's demand *falls* monotonically
(`X(v*)` 0.958 → 0.725 over `Tt4` 1400 → 800) and never approaches its throat at any throttle.
Nothing new: rung 53's P7 needs `v*_LP ≈ 6.7 × v*_HP`, and the throat cost goes as `√(1+v²)`, so
the LP eats it quadratically faster. *The spool that needs stator authority is the spool that
runs out of throat* — rungs 41/44/45/53's split, appearing in a third channel.

---

## Where it lives

All of it on rung 53's existing classes — no new class, because there is no new solve
(rung 36/41's precedent: a pure diagnostic bolts on as methods).

- `ComponentMap`: field `capacity` (default `0.0` = off) + `with_capacity`, `throat_ratio`,
  `throat_loading`, `capacity_margin`, `chokes`, `design_throat_mach`. `capacity` is **not**
  part of `is_flat()` — the `phi_surge` rule, since it never touches `psi`/`eta`/the running
  line. (Rung 53's `vsv` **is**, since it enters `psi`.)
- `VariableStatorMatcher`: `throat_margin`, `throat_sweep`, `authority_ceiling`,
  `schedule_throat`, with `_scan` / `_schedule_root` / `_interp` / `_cross`.

---

## Verification gates (`tests/test_rung54.py`)

1. **REDUCE — an INVARIANCE OVER `C`.** Every matched field bit-identical (`==`) for six
   values of `C` at four stator settings and four throttles, on CPG **and** the reacting gas;
   `C = 0` leaves every rung-53 expression bit-for-bit; the `is_flat` rule (capacity ignored
   like `phi_surge`, `vsv` still not).
2. **THE DERIVED AREA LAW.** `throat_ratio() == cos(atan v)` to 1e-15, **exactly even** in `v`,
   `== 1` at the design setting, and `X == m` there.
3. **THE ONE CONSTANT, DISCLOSED.** `with_capacity` rejects `C ≥ 1`; `design_throat_mach()`
   hits the tabulated Machs and is monotone; `c_min` is present with **no** constant attached
   and `X` is unchanged by `C`; `choked == (C ≥ c_min)`.
4. **P1 — BIND, NEVER RELIEVE.** At a setting that provably chokes the row (`M_c < 0`), every
   matched field is still bit-identical to the no-throat run.
5. **THE HEADLINE.** Across all five shapes and three throttles: `setting_cut > 10 %`,
   `retained > setting_cut`, `retained ≥ 78 %`; plus the default shape's numbers pinned.
   **5b, an EXACT ZERO:** on a flat-η island `X(+v) − X(−v) == 0.0` bit-for-bit (rung 53's P5
   pins `m`), while shaped islands are measurably uneven — so the asymmetry is provably the
   efficiency island's, and neither half is vacuous.
6. **THE ARTIFACT IS NEVER THE CEILING.** `v_ch < v_edge` and `binds != "edge"` on every shape
   × throttle at `C = 0.80`; `C_edge < 0.90` throughout.
7. **P-C2.** `flow/press` still runs to the edge (rung 53's concession stands where it was
   measured) **and** `tilted`/`steep` have interior peaks with a material drop — asserted as a
   contrast so neither half can be vacuous; plus `steep`'s schedule ceasing to exist at
   `Tt4 = 1000` for the right reason (`min tan β₁ >` design), while `flow/press` keeps it.
8. **P-C3.** Rung 54's root finds `steep`'s `v* = 0.909` and satisfies the incidence condition
   to 1e-9, at the same point where rung 53's ladder raises "does not bracket"; and the two
   agree to 1e-9 wherever the ladder succeeds.
9. **THE RACE.** The design-loading crossing bracketed between `Tt4` = 870 and 860; `c_min > 1`
   across rung 53's published band; the interior minimum; and the HP's monotone-falling demand
   with `v*_LP > 3 v*_HP`.
10. **CYCLE UNTOUCHED** — the default single-spool design path is bit-for-bit rung 6.

---

## Concessions

- **ONE NEW CONSTANT.** `C` is imposed, in the tradition of rungs 36/41's `φ_s0`. Its *level*
  is disclaimed; every verdict is stated as a threshold on it, and it is disclosed as a design
  throat Mach so a reader can judge whether a given row is inside or outside each claim.
- **THE THROAT PEAK COINCIDES WITH THE DESIGN SETTING ONLY BY INHERITANCE.** `A_th ∝ cos α₁`
  peaks at `α₁ = 0`, and that is the design setting *because rung 53 defined `v = 0` as zero
  swirl*. A real row designed with nominal swirl has its throat peak elsewhere, and the
  two-sided cost would not be centred on the design point. **This is a coordinate-origin
  inheritance, not a derivation**, and the two-sidedness rests on it.
- **The thin-vane cosine idealisation.** `o/s = cos α₁` assumes the pitch is unchanged by the
  rotation and the vane is thin enough that the throat tracks the exit angle. Real VSV throats
  also move with the pivot location.
- **No loss across the vane row** — throat totals are taken as face totals. Standard at this
  fidelity, and it makes `X` an *under*-estimate of the true throat loading.
- **One row at the compressor face.** A real VSV is an IGV plus several front stages; lumping
  them is rung 53's single-stage-equivalent concession, inherited.
- **The channel is DIAGNOSTIC-ONLY, by theorem.** Rung 54 reports where the throat would bind;
  it does not solve the engine that runs against it. Making the compressor throat the *binding*
  throat would invert rung 31's `(★)` (the flow set upstream of the burner) and restructure the
  whole matching cascade — a different rung, and a large one.
- **Steady only**, and the plant is rung 39's gas — rung 53's and rung 35's standing
  concessions, both inherited.
- `phi_max` is left as rung 53 generalised it and is still not exercised by a steady rung.

## The next seam

**STAGE REMATCHING — the stage stack.** This rung refutes the idea that flow capacity is what
spares a real engine rung 53's overspeed. What actually does it is the front stages rematching
against the rear ones as density changes through the stack — the classic "front stages stall,
rear stages choke at low speed" mechanism, which is *why* a real VSV schedule exists. It needs
the compressor to stop being one lumped block: `K` stage blocks in series sharing the existing
`τ_c`, linked by the thermodynamics already in the model, with the disclosed integer `K` as the
swept coordinate. That is the standing mixing-ceiling-sized item on this side of the project.

Then, unchanged from rung 53's list: a **stator schedule `v(n)` on the transient plant** (the
first lever that could move the wall *during* an accel), and **stator + bleed together**.

## Anchor

`docs/plans/rung54-anchor-throat-capacity.md` — the predictions as written before measuring
(three of them failed; P-B's failure is what forced out P-C2, this rung's largest result), the
probe transcripts, and one claim I made and withdrew.
