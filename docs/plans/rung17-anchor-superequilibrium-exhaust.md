# Rung-17 external anchor — super-equilibrium (frozen) exhaust NO, and the mixing-fidelity ladder of the clamp

Verified numbers + worked checks that anchor rung 17: the **combustor-mixing-fidelity ladder** of the
dropped-clamp margin at the rich RQL primary. Rung 17 adds **no chemistry** — it composes rung-8/11/16
combustor NO with the rung-14 nozzle collapse — so the anchor is (a) the **published NO-freezing /
super-equilibrium exhaust** phenomenon (Heywood), (b) the inherited rung-14 collapse and rung-16
per-pocket gap, and (c) the **machine-checked ladder + scale sweep** re-derived before production code
in `M:\claud_projects\temp\rung17-probe\` (`derive.py`, `scale.py`, `smoke.py`).

> **Read `docs/plans/rung14-anchor-nozzle.md` (the collapse + the dropped clamp) and
> `docs/plans/rung16-anchor-pocket-quench.md` (the per-pocket gap) first.** Rung 17 inherits both.

The mechanism, in one line: **at a rich primary the mixed-out exhaust NO is deceptively low, so the
crude shortcut reads the clamp DORMANT — but the dilution re-making and the near-stoich β-PDF pockets
put NO back, and it freezes super-equilibrium through the nozzle.** Only the ladder *direction* is
robust; every magnitude rides on un-pinned mixing scales.

---

## 1. The physics anchor — NO freezes; the exhaust is super-equilibrium

The transferable, **signed** statement rung 17 rides on is the classic **NO-freezing** result: NO
kinetics are fast at flame temperature but quench as the gas cools and expands, so exhaust NO **freezes
well above** the local equilibrium value and leaves the engine **super-equilibrium** (Heywood, *Internal
Combustion Engine Fundamentals* § NOx formation and freezing; Turns, *An Introduction to Combustion* §
thermal NO and superequilibrium). This is the same physics rung 10 invoked to **drop** the `cNO ≤ cNOe`
clamp (super-eq NO on a cooling path must not be capped) and rung 14 used to **fire** it in the nozzle.

| statement | source | ours |
|---|---|---|
| NO freezes on cooling → exhaust NO is **super-equilibrium** | Heywood, Turns | `a = [NO]/[NO]_e(T9) > 1` for the resolved models |
| equilibrium NO **collapses** as `T` falls (`Kp_NO=exp(−ΔG°/RuT)`) | rung 6/14 | `x_no_e(Tt9)/x_no_e(T9) ≈ 120×` (design point) |
| a **rich** primary makes little NO but the dilution **re-makes** it through stoich | rung 9/10, RQL | `a_bulk > a_mixed` |
| NO **peaks at** stoich ⇒ segregation **raises** the mean | rung 13/16 | `a_pocket > a_bulk` |

There is **no bespoke published digit** for this exact `(CH₂)ₙ`/air, lean-overall/rich-primary,
`pt≈7.5 bar`, `Tt4=1500 K` ladder (stated plainly in §5); the binding ties are the inherited
CEA-anchored composition (rung 6), the inherited collapse (rung 14) and per-pocket gap (rung 16), the
signed NO-freezing trend, and the machine-checked reduce-to-components.

---

## 2. The numbers rung 17 turns on (the ladder)

Real-loss cycle (`π_d=0.97, η_c=0.88, η_b=0.99, π_b=0.96, η_t=0.90, η_m=0.99, π_n=0.98`), `π_c=10`,
`Tt4=1500 K`, `M0=0.85`, fully expanded (`p9=p0=50 kPa`); rich primary `φ_p=1.5`; jet `J=225`,
`H=0.10`, `U_c=75`, `C_e=0.20`; `PocketQuenchPDF(S=0.0625, C_opt=2.5, k_g=0.3, g_max=0.3, τ_res=2.5e-3,
b_u=3.0)`. Exhaust cools `Tt9=1263 K → T9=812 K`; equilibrium NO collapses `2.53e-4 → 2.12e-6` (**120×**).

| model | exhaust `x_no` | `a = x_no/x_no_e(T9)` | |
|---|---|---|---|
| MIXED-OUT (rung 8, `x_no_mix`) | 3.34e-8 | **0.016** | DORMANT — mixing-out **HIDES** the super-eq NO |
| BULK QUENCH (rung 11, `x_no_quenched`) | 7.09e-6 | **3.35** | FIRES — the quench re-making reveals it |
| PER-POCKET (rung 16, β-PDF mean) | 2.88e-5 | **13.6** | FIRES harder — segregation raises the mean |

(Grid `n_bell=32, n_quad=48`; the per-pocket **magnitude** is grid-dependent — rung-16's lesson — but
the **ladder direction** is not.) The identity `a_pocket/a_bulk = 4.06 ≡ ei_no_pocket_quench/
ei_no_quenched` (rung-16 station-4 gap) holds to machine precision **by construction** (§4).

---

## 3. The scale sweep — only the direction is robust (the honest scope, made a number)

`a_bulk`, `a_pocket`, **and the gap** ride on un-pinned mixing scales. Sweeping the entrainment
coefficient `C_e` (which sets the bulk-quench dwell `τ_mean = H/(C_e√J·U_c)`) at `J=225`:

| `C_e` | `a_bulk` | `a_pocket` | `gap = a_pocket/a_bulk` |
|---|---|---|---|
| 0.15 | 4.46 | 14.7 | **3.30** |
| 0.20 | 3.35 | 13.6 | **4.06** |

`a_mixed_out = 0.016` at every scale (no jet dependence). The gap moves **~23%** over `C_e=0.15→0.20`
(`gap = 1 + term2/term1`, and `C_e` moves the bulk floor `term1` while the per-pocket `term2` rides on
`τ_res`), and both fire margins move — but the **ladder direction `a_mixed<1<a_bulk<a_pocket` is
invariant**. That is the single certified claim; nothing about the firing *magnitude* is pinned.

---

## 4. The identity is algebra, not a check (state it, don't gate it)

`x_no_pocket = κ·⟨EI⟩_pocket` and `x_no_bulk = κ·EI_bulk` with the **same** `κ = x_no/EI` (a function of
the overall far only), and both `a` divide by the **same** `x_no_e(T9)`. So

```
a_pocket / a_bulk = x_no_pocket/x_no_bulk = ⟨EI⟩_pocket/EI_bulk = rung-16's station-4 gap
```

**by construction** — `κ` and `x_no_e(T9)` both cancel. No computation path (right or wrong physics,
right or wrong nozzle) can make this false, so it is a **witness** reported in the state
(`gap_pocket_over_bulk`), **not a discriminating gate**. The physical content it witnesses is real: the
**nozzle is a no-op** on the pocket/bulk ratio (the collapse denominator is common), so rung 17 is a
**synthesis** of rungs 11/16/14, not new physics. (The `κ` cross-check `x_no_bulk == κ·ei_no_quenched`
is exact for the same reason; `x_no_mix` uses a *different* EI so is fetched directly.)

---

## 5. The reduce gates (exact by construction)

- **Reduce-to-components:** every number `exhaust_no_clamp` uses is bit-identical to the underlying
  diagnostic — `x_no_bulk == zoned_nox(…, mixing).x_no_quenched`, `a_bulk == nozzle_flow(…,
  x_no_frozen=x_no_bulk).max_a`, `ei_no_pocket_quench ==` the rung-16 value. It **composes**, never
  recomputes.
- **Cycle untouched:** `exhaust_no_clamp` only reads state; the whole rung 1–16 suite stays green and
  the cycle is bit-for-bit rung 6.
- **The rung-14 contrast (not a collision):** the **same** `x_no_mix`-through-the-nozzle construction
  rung 14 uses **fires** at φ_p=1.0 (`a≈250`) and is **dormant** at φ_p=1.5 (`a=0.016`) — the rich
  primary hides the NO. Two faces of the one dropped-clamp lesson.

---

## 6. What stays UN-anchored / deferred (state it plainly)

- **No bespoke published digit** for this exact ladder — the anchor is the inherited composition/
  collapse/gap + the signed NO-freezing trend + the machine-checked reduce (§1, §5).
- **The firing magnitude is not pinned** — `a_bulk`, `a_pocket`, the gap all ride on un-pinned mixing
  scales (`C_e`, `τ_res`, `H`, `J`). Only the direction is certified (§3). Pinning a magnitude needs a
  transported PDF / dwell spectrum (rung-18 seam).
- **The clamp is dormant at station 4** (`max_a_quench < 1`) — the combustor NO is sub-equilibrium; the
  super-equilibrium is a **nozzle** phenomenon here. A pocket going super-eq *at the burner* (hotter
  `Tt4`, longer dwell) is deferred.
- **Frozen exhaust NO** — a finite-rate nozzle would relax the NO partway toward `x_no_e(T9)`, softening
  `a` (rung 14's frozen↔equilibrium bracket, un-resolved between the bounds).
- **Super-equilibrium O / prompt (Fenimore) NO** — every `a` is an equilibrium-O **lower bound**.

## Sources
- Heywood, J., *Internal Combustion Engine Fundamentals* — thermal NO formation and **freezing** on the
  expansion (exhaust NO super-equilibrium).
- Turns, S., *An Introduction to Combustion* — thermal NO, superequilibrium, and rate-freezing.
- Inherited: `docs/plans/rung14-anchor-nozzle.md` (the collapse + the dropped clamp),
  `docs/plans/rung16-anchor-pocket-quench.md` (the per-pocket β-PDF gap),
  `docs/plans/rung6-anchor-equilibrium.md` (the CEA-anchored equilibrium NO, `Kp_NO`).
