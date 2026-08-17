# Rung 55 — the STAGE STACK: a lever that moves ROWS, and what it pays for them

Rung 54 refuted flow capacity as the escape from rung 53's overspeed — structurally, by the
BIND-NEVER-RELIEVE theorem — and named the replacement in its § The next seam:

> *"Capacity does not buy back rung 53's `+26 %` overspeed, and **no area law could** … The real
> multistage mechanism is **stage rematching**, which needs a stage stack, not an area constant.
> That is the new seam."*

This rung builds the stack and takes the seam. The named mechanism is **confirmed** — and the
reason it works is not the one the seam implied.

---

## THE HEADLINE — a general law about POSITIONAL levers

> **A lever that acts on PART of a machine buys its relief from the part it does not act on,
> through whatever the parts share. So its cost collapses with position — and its benefit has
> an interior optimum in HOW MUCH of the machine it moves.**

Rung 53's stator moves one lumped block, i.e. *everything*, and holding the rotor's design
incidence at `Tt4` = 1000 costs `N_L` **+66.7 %**. Resolve the same compressor into `K` = 8 stages
and move only the FRONT ROW — what a real VSV actually is — and the same target costs **+2.3 %**:

```
    dN_ratio  =  (1/K)  ×  (v*_front / v*_lumped)          measured 0.961 … 0.974 of it, K = 2…16
                 └ positional ┘   └ the setting collapses too ┘
```

**29× cheaper, and the collapse FACTORISES** — one leg is positional (the row is `1/K` of the
machine), the other is that a front-only lever **does not fight its own speed rise**, so it needs
3.5× less setting as well. The second leg is what the pre-registered `≈1/K` prediction missed
(`docs/plans/rung55-anchor-stage-stack.md` P3, scored a miss on the level).

And the honest half, which is the same law read backwards: the shaft speed is the one thing every
stage shares, so relief taken at the front is **paid by the rows left behind**. Moving more rows
buys more — until it doesn't:

| rows moved | `v*` | `dN_L` | worst stage | `M_i` worst | relief | relief per `dN` |
|---|---|---|---|---|---|---|
| 0 (bare) | — | — | 0 | 0.4965 | — | — |
| 1 | 0.3536 | +2.30 % | 1 | 0.5438 | +9.53 % | **4.14** |
| 2 | 0.3951 | +5.29 % | 2 | 0.5788 | +16.58 % | 3.14 |
| 3 | 0.4515 | +9.31 % | 3 | 0.5994 | +20.73 % | 2.23 |
| 4 | 0.5332 | +15.09 % | 4 | 0.6004 | **+20.94 %** | 1.39 |
| 5 | 0.6653 | +24.32 % | 7 | 0.5422 | +9.22 % | 0.38 |
| 6 | 0.9396 | +43.09 % | 7 | 0.3983 | **−19.78 %** | −0.46 |
| 7 | 2.4300 | +140.95 % | 7 | −0.0904 | −118.21 % | −0.84 |

`K` = 8, `Tt4` = 1000, default shape, target = hold stage 0's design incidence. **The optimum is
a COUNT** — the first object in this project whose optimum is an integer — and it is *interior*:
relief peaks at 3–4 rows and then REVERSES, ending worse than bare. **Two currencies, two
different optima**: most relief at 4 rows, most relief-per-speed at 1. That is rung 53's law
appearing a third time.

**The reversal is physics, not a bracket.** The residual is smooth and single-rooted in `v` (a
scan at `_V_SCAN` = 0.01 with `rows` = 5 and 7 filled in), and the mechanism is visible: as `v`
rises with 6 rows moved, the identity of the worst stage **migrates rearward**, 0 → 6 → 7, into
the rows the stator does not move.

> **SHARPENED BY THE RUST PORT (slice N step 5) — the finer scan CONFIRMS the reversal, it does
> not RESCUE it.** The sentence above reads as though `_V_SCAN` = 0.01 were needed to tell physics
> from a bracket artifact. Measured at both steps on the shipped cell (`K` = 8, `Tt4` = 1000,
> `rows` = 1…6): the two scans agree on every relief and every cost to about the **11th decimal**
> — `v*` moves by ~1e-11, because the coarse scan only BRACKETS and the bisection then converges
> on the residual to `_INC_TOL` = 1e-12 either way. Every verdict on this curve is therefore
> **scan-step-invariant**, which is a stronger statement than the one this paragraph makes. (The
> override is still carried in the port, because the port runs the source's experiment rather than
> a neighbouring one that happens to agree — see `docs/plans/todo-rust-port.md` § 5.10 step 5.)

---

## The instrument — `StageStack`, and where it is allowed to bite

### The kinematics are DERIVED (no new constant)

All annuli sized so `φ_k` = 1 at design; `θ_k`, `ϖ_k` the cumulative `Tt`/`pt` ratio at stage
`k`'s inlet. From `φ = Vx/U`, `Vx = ṁ/(ρA)`, `ρ = pt/(R·Tt)`, `U ∝ N`:

```
    φ_k = φ_1 · (θ_k/θ_k,d) / (ϖ_k/ϖ_k,d)          n_k = n · sqrt(θ_k,d/θ_k)
```

**`φ_1 = m/n` EXACTLY** — the face flow coefficient every rung since 32 reads **is the front
stage's**. That is a cross-rung result before any measurement: rungs 36–53 were reading the
binding stage all along (§ P1 below).

### The ONE disclosed choice: the WORK SPLIT

> **⚠ REFRAMED — `docs/per-row-blading-negative.md` § 1.** The split is **not** free robustness
> furniture: it is a **BLADING law**. Rung 53's own inversion, per row, gives
> `Δh_k = U²(1 − t₂,k)` at design (`φ_k` = 1, `β₁` = 45° for every row), so a work split *is* a
> choice of blade metal angles, `l_k = (1+l)·Δh₀/Δh_k − 1`. Scored at this rung's own shipped
> design point: **`"dT"` ⇔ UNIFORM blades** (taper 0 — which is why a per-row `ψ_k` law read off
> the split has exactly zero content there), and **`"tau"` ⇔ taper 0.50 LP / 0.64 HP, with the
> HP at `w₂/w₁` = 0.7185 — outside de Haller's published 0.72.** Everything below stays true and
> every verdict here survives (each was asserted across both splits); what changes is that the
> split axis IS the blading axis, and its two shipped points are two hardware choices, one of
> them marginally non-physical.

How the design temperature rise divides between stages — `"dT"` (equal `ΔTt`, the default) or
`"tau"` (equal stage `τ_c^{1/K}`). At design every stage has `ψ` = 1, so *equal loading* is not a
third split, it **is** `"dT"`. Rung 54's pattern: **shape derived, split disclosed, verdict
robust to it** (§ P6).

### No new constant, twice over

The per-stage isentropic efficiency `e_d` is the 1-D inversion that makes the `K`-stage march
reproduce the **shipped** design `π_d` on the design ladder — below `η_d` by the
small-stage/reheat effect and **equal to it at `K` = 1**. Off design it is carried at the live
efficiency's ratio, `e = e_d·η_live/η_d`. So the stack does **not re-design the engine** (rung
42's valve-shut / rung 53's design-capture discipline): at design every `φ_k` = 1, every `n_k` = 1,
and the march returns `τ_d` **exactly**, for every `K` and every split.

**Disclosed CPG placement** (rung 41's `(★)` precedent): the stack's *internal* pressure ladder
uses the cold-section `γ` as a constant, `kc = γ_c/(γ_c−1)`. The **cycle's** own `π_c` is
untouched — still rung 39's, off the real gas. At `K` = 1 the ladder is never consulted, so the
reduce is exact whatever `kc` is.

### Where it bites — exactly one place

The stack replaces rung 32's **speed-line inversion** `(m, τ_c) → n` and *nothing else*. The
energy cascade (map-free, rung 38), the choke relations, the burner `f` fixed point, the
efficiency island, the rebuild-forward and every conservation assert are entered unchanged.
`π_c` is **not** re-derived by the stack.

### The reduce

**An IDENTITY at `K` = 1**, like rung 53's and for the same reason: no stack object is built when
both `K` are 1, both efficiency loops are the **inherited** ones, and there is no rung-55 code
path to skip. Measured: `0.000e+00` on all 13 matched fields × 4 throttles × 4 stator settings,
**at a moved stator**. `StageStack.solve_n` *also* dispatches to `ComponentMap.solve_n` at `K` = 1,
so even a hand-built one-stage stack is bit-for-bit. A one-sided stack (`K_lp` > 1, `K_hp` = 1)
leaves the other spool's loop literally rung 39's — so it is a controlled experiment.

---

## P1 — the RUNNING LINE MOVES, and rungs 36–53 are BOUNDED

The spread `φ_K/φ_1` alone would be a **re-read**: it is a functional of the `(τ_c, π_c)` rung 39
already solves. The rung is the **feedback**. With a per-stage `ψ(φ_k)` the work is no longer
`ψ(φ_face)·n²`, and with `l > 0` the lumped law credits the WHOLE machine with the FRONT stage's
high loading, so a marched stack is **weaker at the same `(m,n)`** — by up to **27 %** of `τ_c−1`
(HP, `K`=8, `Tt4`=800; 36 % on `steep`), **exactly `0.00e+00`** at `K` = 1. That was the
advisor's non-tautology gate, run before anything was pre-registered.

*(Two numbers, one object: the 27 % is measured at **rung 39's** solved `(m,n)` — the gap the
stack must close. `work_gap` in the shipped code reads it at the **stacked** solution's `(m,n)`,
where the running line has already absorbed most of it: −16.2 % HP / −6.6 % LP at `Tt4` = 800.
The first is the forcing, the second is the residue.)*

A weaker stack must be run faster to do the pinned work, so `n` RISES and `φ_1` FALLS:

| `Tt4` | `d_n` LP | `d_φ` LP | `d_n` HP | `d_φ` HP | `d_π` LP | `d`thrust |
|---|---|---|---|---|---|---|
| 1500 | 0.000 % | 0.000 % | 0.000 % | 0.000 % | 0.000 % | 0.000 % |
| 1200 | +1.436 % | −1.409 % | +2.255 % | −2.153 % | −0.047 % | +0.009 % |
| 1000 | +2.244 % | −2.137 % | +4.155 % | −3.848 % | −0.088 % | +0.086 % |
| 800  | +2.950 % | −2.697 % | +6.575 % | −5.905 % | −0.107 % | +0.323 % |

**Sign and monotonicity: pre-registered and HIT. Magnitude: a MISS** — predicted 5–15 %, measured
2.7 % LP (2.7–4.2 % across shapes). Scored as written.

**The cross-rung consequence.** Because the face `φ` IS the front stage's, this is a statement
about how the lumped solve placed the **binding** stage — and it placed it **optimistically**.
Rungs 36–53's `φ`-readings were of the right object; they were 2–4 % generous about it, and the
error grows with throttle depth. Rung 36's *"margin thin at low power"* and rung 41's
*LP-eats-the-excursion* are **SHARPENED, not overturned** — a BOUNDING in rung 53's style.

**And the correction lands harder on the HP** (−5.9 % vs −2.7 % at `Tt4`=800): the HP carries the
higher pressure ratio, hence the bigger density mismatch, hence the bigger spread. Rungs 41/44/45
put the *throttle* excursion on the LP; the *stack* correction goes the other way. Two different
exposures, and only a resolved compressor can tell them apart.

**Thrust and `π_c` barely move** (< 0.33 % and < 0.11 %); on a flat island `d_π` is **exactly
0.000 %**. Like rung 53's stator, the stack is paid in **shaft speed**, not performance.

---

## P4 — one machine, two opposite failures

Per-stage `φ_k` on the default shape, `K` = 8 (LP floor `φ_s0` = 0.55, `T_c` = 1.818):

```
Tt4=1500  LP  φ_k: 1.000 ×8                                          M_i: 0.818 ×8
Tt4= 800  LP  φ_k: 0.681 0.716 0.751 0.785 0.820 0.855 0.890 0.926   M_i(worst) = 0.349 @ stage 0
Tt4= 800  HP  φ_k: 0.877 0.926 0.975 1.026 1.079 1.134 1.193 1.256   M_i(worst) = 0.678 @ stage 0
```

**Pre-registered and HIT.** The smallest incidence margin in the whole machine is the **LP's
FRONT** stage (0.349); the largest excursion on the **HP** is its **REAR** stage, running **+25.6 %
ABOVE design `φ`** — toward choke and negative incidence — while its front barely moves.
`rear_excess` reaches **+36 %** (LP) and **+43 %** (HP). *Front stages stall, rear stages choke*
— the textbook mechanism, and a lumped block with one `φ` cannot represent either end of it.

---

## P5 — `K` is a RESOLUTION, not a knob

LP `d_φ` against `K`, default shape:

| `Tt4` | `K`=1 | `K`=2 | `K`=4 | `K`=8 | `K`=16 |
|---|---|---|---|---|---|
| 1200 | 0.0000 % | −0.8117 % | −1.2110 % | −1.4089 % | −1.5074 % |
| 1000 | 0.0000 % | −1.2250 % | −1.8334 % | −2.1367 % | −2.2880 % |
| 800  | 0.0000 % | −1.5359 % | −2.3090 % | −2.6968 % | −2.8911 % |

**Pre-registered and HIT, and better than the prediction asked.** The increments **halve as `K`
doubles** (0.812, 0.399, 0.198, 0.099 at `Tt4`=1200) — first-order convergence, so the stack has a
well-defined continuum limit and the disclosed integer is a **resolution coordinate**, not a
fitted parameter. Nothing here rides on a particular `K`.

---

## P6 — the disclosed split does not carry the verdict

`"dT"` vs `"tau"`, `K` = 8:

| `Tt4` | `d_φ` LP (dT) | `d_φ` LP (tau) | HP `rear_excess` (dT) | (tau) |
|---|---|---|---|---|
| 1200 | −1.4089 % | −1.4090 % | +14.3 % | +13.7 % |
| 1000 | −2.1367 % | −2.1355 % | +27.0 % | +25.9 % |
| 800  | −2.6968 % | −2.6935 % | +43.2 % | +41.4 % |

`d_φ` agrees to **0.01 %** relative, `rear_excess` to **4 %**, the worst stage is stage 0 in every
cell. **Pre-registered band was < 25 %; HIT with an order of magnitude to spare.**

---

## P3 — the headline measured, and rung 54's seam discharged

**Lead with the factorisation**, not the ratio (`Tt4` = 1000, default shape, target = stage 0's
design incidence, front-row-only lever):

| `K` | `v*_front` | `dN_L` | ratio to rung 53 | `v*` ratio | `1/K` | `(v*ratio)/K` | measured/predicted |
|---|---|---|---|---|---|---|---|
| 2 | 0.4801 | +12.38 % | 0.1855 | 0.3861 | 0.5000 | 0.1930 | 0.961 |
| 4 | 0.3868 | +5.02 % | 0.0752 | 0.3111 | 0.2500 | 0.0778 | 0.967 |
| 8 | 0.3536 | +2.30 % | 0.0345 | 0.2844 | 0.1250 | 0.0355 | 0.971 |
| 16 | 0.3392 | +1.11 % | 0.0166 | 0.2728 | 0.0625 | 0.0170 | 0.974 |

The product law holds to **3 %** across a 8× range in `K`, with a slow, monotone drift that is
itself honest content (the two legs are not perfectly independent). `v*_front` **saturates**
(0.480 → 0.339 → ~0.33) while the penalty falls like `1/K`: **the setting a front row needs
converges; the price it pays vanishes.**

**Shape robustness**, `K` = 8, `Tt4` = 1000:

| shape | `v*` (53) | `dN` (53) | `v*` front | `dN` front | ratio | machine relief |
|---|---|---|---|---|---|---|
| flow/press | 1.2436 | +66.73 % | 0.3536 | +2.30 % | 0.035 | +9.53 % |
| press/flow | 1.0499 | +61.55 % | 0.3305 | +2.44 % | 0.040 | +9.23 % |
| tilted | 1.4883 | +88.79 % | 0.3470 | +2.41 % | 0.027 | +9.53 % |
| flat-eta | 0.9620 | +53.68 % | 0.3331 | +2.22 % | 0.041 | +8.99 % |
| steep | *rung 53's schedule does not bracket* | | 0.3486 | +2.35 % | — | +9.4 % |

`steep` failing rung 53's own `incidence_schedule` is **inherited, not a gap** — rung 54 P-C3
documented that exact doubling-ladder failure and built the scan-bracketed root this rung uses.

**Scoring, as written.** P3's SIGN and order of magnitude: **HIT, decisively**. P3's LEVEL:
**MISS** — pre-registered `≈1/K` (band 0.0625–0.25 at `K`=8), measured **0.035**, ~3.6× below the
band. The reason for the miss is the finding: the setting leg was not predicted.

### The honest half — what the front row does NOT buy

It restores **the row it moves**, not the machine. Fixing stage 0 promotes **stage 1** to worst,
and the machine's worst incidence margin improves **+9.5 %** (0.4965 → 0.5438) rather than
returning to the design 0.818. Rung 53's lumped lever appears to restore the margin fully — but
the object it restores is a single-stage machine's, which the resolved compressor does not have.

### Rung 53's numbers, resolved

Move ALL `K` rows (rung 53's lever, resolved) and the schedule **ceases to exist** below
`Tt4` ≈ 1300: at 1100 and 1000 the scan runs into the speed-line bracket at `v` ≈ 2.1–2.4 and
`N_L` +128…+143 % without reaching the target. Rung 53 conceded its schedule numbers were
"model-bound"; resolved into stages, the all-rows schedule is **not merely expensive, it is
unreachable**. (Rung 54 found the same schedule ceases to exist under the throat, by a different
mechanism. Two independent ceilings on one object.)

### One currency reconciliation, stated up front

Rung 53 publishes **+26 %** for this schedule; this rung measures **+66.7 %** for *the same
method at the same point*. Both are right: rung 53 referenced `N_L` to the **design** point
(`N_L`(v*) = 1.26006, so +26.01 % against 1.0), this rung references to **bare at the same
throttle** (`N_L` = 0.75574). Rung 55 uses bare-at-throttle throughout, because every comparison
here is **lever versus lever at fixed throttle**. This is rung 43's currency-circularity lesson
again: a referenced excursion reads back its own denominator, so the denominator is named.

---

## Scope and concessions

- **STEADY, TWO-SPOOL ONLY.** Unlike rung 54's throat the stack **enters the solver**, so there
  is no free invariance. The rung-34/40/43 transient closures and the whole limiter family
  (46–52) run their own forward closures off `ComponentMap.psi`/`phi_max` and never construct a
  stack — **asserted by test**, not merely declared (gate 8).
- **`π_c` is not re-derived by the stack** (§ The instrument). The stack's internal ladder and
  the cycle's pressure ratio are different objects; they coincide at design and at `K` = 1.
- **All stages share one map.** Every stage reads the same `ψ`, the same island and the same
  `φ_surge`/`T_c`; a real stack's rows differ. The *positional* content is in where the stator
  sits and in the density march, not in per-row blading.
- **One `K` per spool, uniform annulus sizing** (`φ_k` = 1 at every stage at design).
- **No per-stage CAPACITY.** Rung 54's `capacity_margin` almost certainly lands on the REAR
  stage — the two rungs' objects meet there — but it needs a `C` per row. Named as the seam.
  **BUILT BY RUNG 56**, and the guess above is half right: the rear at part power, the FRONT
  near design (the design Mach profile fights the loading), and it needed only ONE constant —
  the ladder supplies the other `K−1`. `docs/rung56-spec.md`.
- **Inherited**: rung 39's gas and map closure, rung 38's choked-nozzle envelope, rungs 36/41's
  imposed `φ_s0` (still the incidence anchor), rung 32's disclosed map shapes.

## Verification gates

`tests/test_rung55.py` — 1 REDUCE (identity at `K`=1, every field, moved stator, both gases);
2 the derived kinematics (`φ_1` ≡ `m/n`; design ladder exact for every `K`/split); **2b the
rung-2b tie** (see below); 3 the
NON-TAUTOLOGY gate (marched ≠ lumped, exactly 0 at `K`=1, growing with throttle); 4 P1's sign and
monotonicity; 5 P4's front-stalls/rear-chokes; 6 P5's convergence (increments shrink); 7 P6's
split robustness; 8 SCOPE — the transient ladders are bit-for-bit unstacked; 9 P3's factorisation
and the row-count optimum; 10 CYCLE UNTOUCHED (the default design run is bit-for-bit rung 6).

> **THREE OF THESE GATES WERE MEASURED VACUOUS BY THE RUST PORT (slice N step 5), and the port's
> copies are STRONGER by exactly that much.** Each was found by injecting the defect the gate
> names and watching it stay green — not by reading it.
>
> * **Gate 1's `test_reduce_stack_object_dispatches_at_K1`** says its value equality shows *"the
>   same code and not merely the same algebra"*. It does not. Delete `StageStack.solve_n`'s
>   `K == 1` dispatch entirely and the assertion still passes **bit-for-bit**: the fall-through
>   bisects the same `[0.1, 2.0]` to the same `1e-14`, and its residual differs from
>   `ComponentMap.solve_n`'s by a POSITIVE affine factor — invisible to a bisection that reads
>   only SIGNS. The dispatch needs a structural witness (the port uses a pass/march census: both
>   exactly **0** when it dispatches, and the pass count measured **144** — 3 calls × 48 — when
>   it does not).
> * **Gate 7's `test_p6_verdicts_survive_the_work_split`** makes only UPPER bounds, every one
>   satisfied at `x == y` — so it cannot tell P6's claim (*the split is disclosed and no verdict
>   rides on it*) from *the split is dead code*. Collapse `_ladder_T`'s `"tau"` arm onto `"dT"`
>   and this gate stays green while **rung 56's gate 7 fails at once**, because its claim has the
>   opposite sign. The two rungs are two-sided only when read together, which is not how a suite
>   is read; the port adds a `!=` clause, as rung 56's own gate already does.
> * **Gate 9's `test_p3_row_count_has_an_interior_optimum`** asserts the cost curve is monotone
>   as `cost == dict(sorted(cost.items())) or [...] == sorted([...])`. The first disjunct compares
>   a dict to a re-ORDERED copy of itself, and `dict.__eq__` ignores order — so it is `True` for
>   ANY curve, the `or` short-circuits, and the monotonicity is never evaluated. The claim itself
>   HOLDS (0.0230, 0.0529, 0.0931, 0.1509, 0.2432, 0.4309, measured); only the gate was empty.
>
> The Python gates are left as they are — the port is a translation and repairing the source's
> suite is outside it (`docs/plans/todo-rust-port.md` § 8) — so this note is the record.

## Gate 2b — a free consistency check, found by writing a gate with the WRONG SIGN

I asserted that the derived per-stage efficiency would sit **below** the lumped one and the gate
failed. The correct sign is **above** — the **reheat effect** — and sweeping `K` shows `e_d`
converging first-order on

```
    e_c = ln(π_d) / (kc · ln(τ_d)) = 0.9141074            RUNG 2b's POLYTROPIC EFFICIENCY
    K = 1  0.900000   2  0.907160   4  0.910667   8  0.912396   16  0.913254   64  0.913895
```

Nothing here was told about polytropic efficiency: the stack is handed an **isentropic** design
point and a stage count. So the construction **interpolates rung 2 (`K`=1, isentropic) to rung 2b
(`K`→∞, polytropic)**, and rung 2b's shipped `η_c < e_c` ordering falls out rather than being
imposed. It costs nothing and it exercises the ladder, the split and the design capture at once.

## The next seam

**Per-row capacity — rung 54's channel, per stage. → TAKEN BY RUNG 56** (`docs/rung56-spec.md`):
right at part power, **wrong near design**, and it did **not** need a `C` per row — the design
ladder supplies the profile, leaving rung 54's single constant as the level. The two constraints
turn out to separate by SPOOL as well as by end.

`X(v) = m·sqrt(1+v²)` is a face quantity;
in a stack each row has its own throat and its own `X_k`, and P4 says the REAR rows are the ones
driven to high `φ`. Rung 54's `capacity_margin` should therefore **bind at the back** while its
incidence margin binds at the front — the two rungs' objects on the same machine, in opposite
places. It needs a `C` per row.

Then: the **stator schedule on the TRANSIENT plant** (still the first lever that could move the
wall *during* an accel — now with a row count as well as a setting), **stator + bleed together**,
and **per-row blading** (the stages here share one map, which is what keeps the positional claim
clean and also what bounds it).

## Anchor

`docs/plans/rung55-anchor-stage-stack.md` — the two pre-registration probes (the advisor's sizing
and non-tautology gates, run *before* the predictions), the six predictions as written, and their
honest scoring: **P1 sign HIT / level MISS, P2 HIT, P3 sign HIT / level MISS, P4 HIT, P5 HIT,
P6 HIT**, plus the one defect an advisor check caught and the measurement that resolved it.
