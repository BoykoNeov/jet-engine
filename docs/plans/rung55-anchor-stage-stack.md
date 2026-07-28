# Rung 55 anchor — the STAGE STACK: predictions as written BEFORE measuring

The project's rule: the predictions below were written **before** the corresponding measurement,
and are scored honestly afterwards (rungs 44–54's discipline). Two probes ran *before* this file
and are recorded as such — they were the advisor's **sizing** and **non-tautology** gates, whose
whole purpose was to decide whether this is a rung at all.

---

## Pre-registration order (honest record)

| Probe | When | Purpose |
|---|---|---|
| Probe 1 — the SPREAD, read off rung 39's solved state | BEFORE this file | sizing: is the prize a few percent? |
| Probe 2 — marched `τ_c` vs lumped at fixed `(m,n)` | BEFORE this file | the advisor's **non-tautology gate**: is this content or a re-read? |
| P1 … P6 below | this file | the rung's actual claims |

---

## What ran before pre-registration, and what it said

### Probe 1 — the spread (sizing)

Read off rung-39's **already-solved** `(τ_c, π_c)` with no stack in the solver:

```
    φ_k = φ_1 · (θ_k/θ_k,d) / (ϖ_k/ϖ_k,d)          θ,ϖ = cumulative Tt/pt at stage k's INLET
```

Default shape `flow/press`, `K = 8`, equal-ΔTt split, per-stage `η` solved to reproduce the
live overall `π`:

| `Tt4` | spool | `φ_1` | `φ_K` | `φ_K/φ_1` |
|---|---|---|---|---|
| 1500 | lp | 1.0000 | 1.0000 | 1.0000 |
| 1500 | hp | 1.0000 | 1.0000 | 1.0000 |
| 1200 | lp | 0.8598 | 0.9779 | 1.1374 |
| 1000 | lp | 0.7731 | 0.9628 | 1.2454 |
| 800  | lp | 0.6996 | 0.9553 | 1.3656 |
| 800  | hp | 0.9320 | 1.3732 | 1.4733 |

Exactly 1.0000 at design on both spools, for every `K` — the ladder is design-consistent by
construction. **The textbook picture appears**: throttled back, the LP *front* stage takes the
whole excursion (0.70 against a 0.55 floor) while its rear sits near design; on the HP the *rear*
runs **37 % above** design `φ` — toward choke/negative incidence — while its front barely moves.

### Probe 2 — the NON-TAUTOLOGY GATE (the advisor's, and the reason this is a rung)

The spread above is a *re-read*: it is a function of the overall `(τ_c, π_c)` rung 39 already
computes, and it is essentially `K`-independent in shape. The advisor's blocking question was
whether the stack has **feedback** — whether a *marched* stack does different WORK than the lumped
law `ψ(φ_face)·n²` at the SAME `(m, n)`. If machine-zero → `docs/stage-stack-negative.md`.

Marched total `τ_c` minus lumped `τ_c`, same `(m, n)`, default shape:

| `Tt4` | spool | `K=1` | `K=2` | `K=4` | `K=8` | `K=8` as % of `τ−1` |
|---|---|---|---|---|---|---|
| 1500 | lp | `0.00e+00` | `−6.7e-16` | `−1.1e-15` | `−2.4e-15` | −0.00 % |
| 1200 | lp | `0.00e+00` | `−7.1e-03` | `−1.1e-02` | `−1.3e-02` | −3.98 % |
| 1000 | lp | `0.00e+00` | `−8.4e-03` | `−1.3e-02` | `−1.5e-02` | −5.67 % |
| 800  | lp | `0.00e+00` | `−8.3e-03` | `−1.3e-02` | `−1.5e-02` | −6.95 % |
| 800  | hp | `0.00e+00` | `−5.7e-02` | `−9.8e-02` | `−1.2e-01` | **−26.56 %** |

On `steep`: −35.7 % (HP, `Tt4` = 800). **The gate passes decisively.** `K = 1` is exactly
`0.00e+00` — the reduce is exact, not merely tight — and the gap grows monotonically with
throttle depth. The stack is not a re-read: it moves the running line.

**The sign, and why.** With `l > 0` the lumped law credits the WHOLE machine with the FRONT
stage's high loading (`ψ` rises as `φ` falls). In a stack the rear stages sit at higher `φ`, so
they do LESS work. The marched machine is therefore **weaker at the same `(m,n)`**.

---

## The predictions (written before measurement)

### P1 — the running-line shift, and its SIGN

The energy cascade pins `τ_c` and the choke pins `m`; `n` is what `solve_n` returns. A weaker
stack must therefore be run **faster** to make the same work.

> **P1: with the stack in the solver, the solved `n` RISES and `φ_1 = m/n` FALLS relative to
> rung 39 at the same throttle, monotonically deepening with throttle depth. At `Tt4` = 800,
> `K` = 8, LP: `φ_1` falls by 5–15 %.**

**Why it matters (the cross-rung consequence, also pre-registered):** rungs 36–53 read the surge
margin at the face, which *is* the front stage's `φ` — so those verdicts were reading the binding
stage all along (a **bounding** in rung 53's style). But P1 says the lumped solve places that
stage **optimistically**: the correction has a sign, and it is *against* margin. Rung 36's
"margin thin at low power" and rung 41's LP-eats-it should therefore be **SHARPENED**, not
overturned.

*Failure mode that would kill it:* `φ_1` rises, or the shift is non-monotone in throttle.

### P2 — the REDUCE

> **P2: `K_lp = K_hp = 1` reproduces rung 53/54 BIT-FOR-BIT on every matched field, at a moved
> stator and on both gases.** Probe 2 already shows the `τ` law reduces at exactly `0.00e+00`;
> the gate is the full field list, by dispatch.

### P3 — THE HEADLINE: rung 54's named seam discharged

Rung 53's `v*` schedule holds design incidence at `Tt4` = 1000 by moving **every** stage, and
pays `N_L` **+26 %**. Rung 54 refuted capacity as the escape and named stage rematching as the
real one. A real VSV moves the FRONT stages only.

> **P3: a FRONT-STAGE-ONLY stator holding the front stage's design incidence costs far less
> shaft speed than rung 53's all-stage lumped lever. Pre-registered as `≈ 1/K` of rung 53's
> penalty — at `K` = 8, `N_L` cost < 8 % against 26 %, and the ratio to the lumped penalty
> within a factor 2 of `1/K`.**

**This is a prediction, not a derivation** (the advisor's caution, recorded): as `n` rises less,
`φ_1 = m/n` falls less, so the incidence deficit the front stator must correct is itself
different — it is a coupled root, not a scaling. The measured number is the finding; `1/K` is
scored as a hit or a miss on the LEVEL, and the claim is the SIGN and the order of magnitude.

*Failure mode:* the penalty does not fall materially. Then P3 is dead, rung 54's named mechanism
is **refuted as well**, and P1/P4 carry the rung.

### P4 — the constraint SWAPS SPOOLS

> **P4: at the deepest throttle in the choked envelope, the smallest incidence margin in the
> machine is the LP's FRONT stage, while the largest excursion on the HP is its REAR stage
> running ABOVE design `φ` (toward choke). One machine, two opposite failures, neither
> expressible in a lumped block.**

### P5 — `K`-CONVERGENCE (the disclosed integer is a resolution, not a fitted knob)

> **P5: the running-line shift GROWS with `K` but SATURATES — successive increments shrink,
> `gap(8)−gap(4) < gap(4)−gap(2)` on both spools at every throttle.** The stack tends to a
> continuous march, so `K` is a resolution coordinate. (Probe 2 already suggests this; P5 pins it
> on the SOLVED shift, which probe 2 did not measure.)

### P6 — the WORK-SPLIT is disclosed, the verdict is robust to it

Equal `ΔTt` per stage / equal stage `τ` (`τ_c^{1/K}`) / equal `ψ` give different intermediate
`pt_k`, hence different `φ_k`. Rung 54's pattern applies: **shape derived, split disclosed,
verdict as a robustness claim.**

> **P6: P1's sign, P3's sign and P4's swap are unchanged across at least two work splits; only
> the magnitudes move (pre-registered band: < 25 % relative change in the P1 shift).**

---

## Scope, declared UP FRONT (the advisor's constraint)

Unlike rung 54's throat, the stack **enters the solver**, so there is no free invariance.

- **STEADY, TWO-SPOOL ONLY.** The rung-34/40/43 **transient** closures call `psi`/`phi_max`
  FORWARD; the stack must not reach them, or the blast radius is rungs 34–52. The transient
  plant stays on the lumped law, explicitly.
- **`π_c` is NOT re-derived by the stack.** The stack replaces the speed-line inversion
  (`(m, τ_c) → n`) and nothing else; `π_c` still comes from rung 39's overall-η island closure,
  untouched. The stack's internal ladder is placed on the LIVE `(τ_c, π_c)` inside the existing
  η fixed point, so it adds no constant: the per-stage `η` is the 1-D inversion that reproduces
  the live overall `π`, and at `K = 1` it IS the lumped `η` exactly.
- **No per-stage CAPACITY.** Rung 54's `capacity_margin` almost certainly lands on the REAR
  stage — a lovely unification — but it needs a `C` per row and doubles the rung. **Named as
  rung 55's seam, not built.**
- **One `K` per spool, a disclosed integer**, swept; all annuli sized so `φ_k = 1` at design.

---

# SCORING — measured against the predictions above

| | claim | verdict |
|---|---|---|
| **P1** | `n` rises, `φ_1` falls, monotone with throttle depth | **HIT** (all 5 shapes, both spools) |
| P1 | level: LP `d_φ` 5–15 % at `Tt4`=800, `K`=8 | **MISS** — measured **2.7 %** (2.7–4.2 % across shapes) |
| **P2** | reduce, bit-for-bit at `K`=1 | **HIT** — `0.000e+00`, 19 fields × 4 throttles × 4 stator settings, both gases |
| **P3** | front-only lever far cheaper (sign) | **HIT, decisively** — +2.30 % vs +66.73 % `N_L`, **29×** |
| P3 | level: ratio ≈ `1/K`, within 2× of 0.125 at `K`=8 | **MISS** — measured **0.0345**, ~3.6× below the band |
| **P4** | LP front is the machine's worst; HP rear runs above design `φ` | **HIT** (LP front `M_i`=0.349; HP rear `φ`=1.256, +25.6 %) |
| **P5** | shift grows with `K`, increments shrink | **HIT**, and stronger — increments **halve per doubling** (first order) |
| **P6** | verdicts survive the work split, < 25 % | **HIT** — `d_φ` agrees to **0.01 %**, `rear_excess` to 4 % |

## Why P3's level missed — the finding the prediction did not contain

Pre-registered as the positional leg alone. Measured, the cost **factorises**:

```
    dN_ratio = (1/K) × (v*_front / v*_lumped)        holds to 3 % over K = 2…16
```

| `K` | `v*_front` | `dN_L` | ratio | `v*` ratio | `1/K` | `(v*ratio)/K` | measured/predicted |
|---|---|---|---|---|---|---|---|
| 2 | 0.4801 | +12.38 % | 0.1855 | 0.3861 | 0.5000 | 0.1930 | 0.961 |
| 4 | 0.3868 | +5.02 % | 0.0752 | 0.3111 | 0.2500 | 0.0778 | 0.967 |
| 8 | 0.3536 | +2.30 % | 0.0345 | 0.2844 | 0.1250 | 0.0355 | 0.971 |
| 16 | 0.3392 | +1.11 % | 0.0166 | 0.2728 | 0.0625 | 0.0170 | 0.974 |

The second leg is that a front-only lever **does not fight its own speed rise**: the lumped lever
unloads everything, so `n` runs away, so `φ_op` collapses, so it needs *more* setting — a positive
feedback the positional lever breaks. `v*_front` **saturates** (→ ~0.33) while the penalty keeps
falling like `1/K`. This is exactly the advisor's pre-recorded caution ("a coupled root, not a
scaling") landing in my favour rather than against me.

## The defect an advisor check caught, and what resolved it

The row-count sweep first showed relief **reversing** at 6 rows. The advisor refused it: a curve
reading 0.5438 → 0.5788 → 0.5994 → 0.6004 → **0.3983** is a curve with a hole, and the two
candidates — a `_V_SCAN`=0.05 bracket landing on a different root (rung 54 P-C3's turning-point
hazard) versus real physics — could not be told apart from what was on the table.

**Resolved by measuring, not arguing.** (a) The residual `tan β₁(stage 0) − target` at `rows`=6 is
**smooth and single-rooted** in `v` over [0, 1.4], crossing once near 0.94. (b) Re-run at
`_V_SCAN`=0.01 with the missing `rows` = 5 and 7 the curve is continuous: relief 9.53 → 16.58 →
20.73 → **20.94** → 9.22 → −19.78 → −118.21 %. The reversal is **physics**, and its mechanism is
visible: the worst stage's identity **migrates rearward** (0 → 6 → 7) into the rows the stator does
not move. Had it been the bracket, the row sweep would still have carried the weaker claim
("relief saturates at 3–4 rows while cost keeps climbing"); the rung did not need the reversal.

## One number reconciled against a published rung

Rung 53 publishes `N_L` **+26 %** for its schedule at `Tt4`=1000; the same method rerun here gives
**+66.73 %**. Both correct, different denominators: rung 53 referenced to **design**
(`N_L`(v*) = 1.26006 ⇒ +26.01 %), rung 55 to **bare at the same throttle** (`N_L` = 0.75574).
Rung 55 uses bare-at-throttle throughout because every comparison is lever-vs-lever at fixed
throttle. Rung 43's currency-circularity lesson: name the denominator.

## One result found while gating, not predicted at all

The per-stage efficiency `e_d` — the 1-D inversion reproducing the shipped design `π_d` — comes out
**above** the lumped `η_d` (I wrote the gate asserting *below*, and it failed). That sign is the
**reheat effect**, and sweeping `K` shows `e_d` converging first-order on
`e_c = ln(π_d)/(kc·ln(τ_d))` = 0.9141074 — **rung 2b's polytropic efficiency**, which the stack was
never told about. So the construction interpolates **rung 2 (K=1, isentropic) → rung 2b (K→∞,
polytropic)**, and rung 2b's `η_c < e_c` ordering falls out rather than being imposed. Gated
(gate 2b) as a free consistency check on the whole thing.
