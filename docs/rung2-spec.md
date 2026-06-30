# Rung 2 — Real Components & Dual-Gas Spec / Thermodynamics Handout

Rung 1 built the *ideal* turbojet (everything isentropic, one constant `cp`).
Rung 2 makes the **components real** — each process now generates entropy — and,
because the canon couples "real components" with "real gas," it splits the gas
into a **cold** and a **hot** section. Read this before touching code; it is the
rung-2 contract and the derivation handout, mirroring `SPEC.md` for rung 1.

> **Read `SPEC.md` first.** This file only states what *changes*. The station
> order, the totals-vs-static convention, the "derive before you code" contract,
> and the conservation-assert habit all carry over unchanged.

---

## What rung 2 adds (and what it deliberately does not)

**Adds (all additive — a parameter or an extra term, never a solver rewrite):**

- **Dual gas.** Two calorically-perfect sections: *cold* `(γc, cpc, Rc)` upstream
  of the burner (stations 0→3) and *hot* `(γt, cpt, Rt)` downstream (4→9). The
  burner is the hand-off. This is why "real gas" rides in with "real components":
  once combustion products are hot and heavy, one `cp` no longer fits.
- **Component efficiencies** (entropy-generating): compressor `η_c`, turbine `η_t`,
  burner `η_b` (combustion completeness), mechanical `η_m` (shaft friction).
- **Specified total-pressure ratios** (loss, but *not* efficiency-driven): inlet
  `π_d`, burner `π_b`, nozzle `π_n`. These drop `pt` without an ideal substate —
  they are inputs, like `πc` is.
- **Inlet ram recovery.** `π_d = π_dmax · η_r(M0)` with the MIL-spec
  `η_r = 1 − 0.075·(M0−1)^1.35` for `1 ≤ M0 ≤ 5` (and `η_r = 1` for `M0 ≤ 1`).
- **Specified exit pressure `P9`** (default `P9 = P0`, fully expanded). When
  `P9 ≠ P0` the nozzle is under/over-expanded and specific thrust carries a
  **pressure-thrust term**. `P9` is an *input*, so this is straight-line — no
  choke detection.

**Deliberately deferred (still out of scope):**

- **Choked convergent nozzle** — choke *detection* + a branch between
  "exit = ambient" and "exit M=1, exit p > ambient." That is new control flow and
  a new failure surface; a *specified* `P9` is not. Later rung.
- **Polytropic efficiency** as the knob (we use *isentropic* `η_c, η_t`; see the
  conversion below), **variable `cp(T)`**, reacting/dissociating gas, off-design,
  afterburner.

---

## The two efficiency *kinds* — keep them straight

This is the rung-2 trap. Two different things both "lose pressure":

1. **Isentropic (adiabatic) efficiency** `η_c, η_t`. The real machine reaches the
   *same pressure* as the ideal one but at a *worse temperature*. Defined against
   an **ideal substate** (`Tt3s`, `Tt5s`) computed at the actual pressure ratio.
   The leg is no longer isentropic, so the rung-1 check `Tt/Tt = (pt/pt)^g` is
   reworked: assert the *ideal substate* satisfies it, plus an entropy-generation
   *inequality* on the actual temperature.

2. **Specified total-pressure ratio** `π_d, π_b, π_n`. A flat fractional `pt`
   drop, given as an input. No ideal substate, no temperature coupling. Asserted
   exactly (`pt_out == π · pt_in`), exactly like rung-1's `πc`.

Both collapse to rung 1 when set to 1.

---

## Gas model (dual section)

```
cold:  γc = 1.4,  cpc = 1004 J/(kg·K),  Rc = (γc−1)/γc · cpc = 286.9 J/(kg·K)
hot:   γt,        cpt,                  Rt = (γt−1)/γt · cpt
gc = (γc−1)/γc    (cold isentropic exponent)
gt = (γt−1)/γt    (hot  isentropic exponent)
```

`R` is **not** independent: `R = (γ−1)/γ · cp` per section (equivalently
`cp = γR/(γ−1)`). A shared `R` with split `cp` will *not* close the validation to
the digit — the speed of sound and the static drop both use the section `R`.

**`unify()`** collapses the hot triple onto the cold one (`γt←γc, cpt←cpc,
Rt←Rc`). This is the lever for the reduce-to-ideal gate: collapsing the *whole
triple* (not just `cp`) is what makes rung 2 reproduce the rung-1 table — if `γt`
stays 1.3, `gt ≠ gc` tilts the turbine and nozzle legs and the digits drift.

---

## Station equations (what changed from rung 1)

`g` below is the *section* exponent: `gc` cold, `gt` hot.

### 0 — Freestream (cold)
Unchanged in form; uses cold `γc, Rc`:
```
Tt0 = T0·(1 + (γc−1)/2·M0²),  pt0 = p0·(…)^(1/gc),  V0 = M0·√(γc·Rc·T0)
```

### 2 — Inlet (real diffuser): ram recovery, no isentropic substate
```
Tt2 = Tt0                                   # adiabatic duct: Tt conserved (general)
π_d = π_dmax · η_r(M0)                       # MIL-spec ram recovery
pt2 = π_d · pt0                              # specified pressure ratio (loss)
```
*Why:* `Tt2 = Tt0` still holds for any adiabatic, work-free duct. The change is
`pt2 < pt0`: a real inlet loses total pressure to friction and shocks. `π_d` is a
*specified ratio*, so there is **no ideal substate** here — drop the rung-1
isentropic equality, assert `pt2 == π_d·pt0` and `pt2 ≤ pt0`.

### 3 — Compressor (real): isentropic efficiency via an ideal substate
```
pt3  = πc · pt2                              # pressure ratio is still the knob
Tt3s = Tt2 · πc^gc                           # IDEAL exit temp at this pressure (substate)
Tt3  = Tt2 + (Tt3s − Tt2)/η_c                # actual exit: hotter than ideal
```
*Why:* the compressor reaches `pt3` either way; with `η_c < 1` it has to spend
*more* temperature rise to get there (`Tt3 > Tt3s`). The extra `ΔTt` is wasted
work that the turbine must still repay across the shaft — losses cost fuel.
Assert `Tt3s/Tt2 == (pt3/pt2)^gc` (the *substate* is isentropic) **and**
`Tt3 ≥ Tt3s` (entropy generated). At `η_c = 1`, `Tt3 = Tt3s` → rung 1.

### 4 — Burner (real): dual-cp energy balance, combustion + pressure loss
```
pt4 = π_b · pt3                              # specified combustor pressure loss
f   = (cpt·Tt4 − cpc·Tt3) / (η_b·hPR − cpt·Tt4)   # dual-cp energy balance
```
*Why:* the steady-flow energy balance now spans the cold→hot hand-off and books
incomplete combustion via `η_b`:
```
mdot_air·cpc·Tt3 + η_b·mdot_fuel·hPR = (mdot_air + mdot_fuel)·cpt·Tt4
```
Divide by `mdot_air`, set `f = mdot_fuel/mdot_air`, solve → the `f` above. Note
`cpt` on both the fuel-heating ceiling and the product enthalpy: the products are
hot-section gas. At `cpc=cpt=cp, η_b=1` this is rung-1's
`f = cp(Tt4−Tt3)/(hPR − cp·Tt4)`. Still **not** isentropic (heat added), so no
`(pt/pt)^g` check — assert `pt4 == π_b·pt3` and the mass/energy balances.

### 5 — Turbine (real): dual-cp shaft balance, then η_t via a substate
```
ΔTt = cpc·(Tt3 − Tt2) / (η_m·(1 + f)·cpt)    # SHAFT BALANCE (dual cp + η_m)
Tt5  = Tt4 − ΔTt
Tt5s = Tt4 − ΔTt/η_t                          # IDEAL drop for this work (substate)
pt5  = pt4 · (Tt5s/Tt4)^(1/gt)               # isentropic from the substate
```
*Why (shaft):* compressor and turbine still share one spool, but now the gas is
hot-section downstream and the shaft itself leaks work (`η_m`):
```
η_m · (mdot_air + mdot_fuel)·cpt·(Tt4 − Tt5) = mdot_air·cpc·(Tt3 − Tt2)
```
→ `ΔTt = cpc(Tt3−Tt2)/(η_m(1+f)cpt)`. Three rung-2 effects move the drop relative
to rung 1: `cpc/cpt < 1` *shrinks* it (hot products carry more enthalpy per kelvin,
so a smaller drop pays the same work), the `(1+f)` heavier stream *shrinks* it
(the turbine works more mass), and `η_m < 1` *grows* it (the turbine must
over-produce to cover shaft friction). Net for Example 7.1: `τt = Tt5/Tt4 = 0.8156`.
*Why (η_t):* with `η_t < 1` the turbine gets *less* pressure drop for the same
work — it must fall to a *lower* ideal temperature `Tt5s < Tt5` to reach `pt5`.
Assert `Tt5s/Tt4 == (pt5/pt4)^gt` (substate isentropic) **and** `Tt5 ≥ Tt5s`
(entropy generated) and `ΔTt > 0`. At `η_t=η_m=1, cpc=cpt`: rung-1's
`pt5 = pt4·(Tt5/Tt4)^(1/g)`.

### 9 — Nozzle (real): π_n loss, expand to a *specified* P9 (hot section)
```
Tt9 = Tt5                                     # adiabatic: Tt conserved
pt9 = π_n · pt5                               # specified nozzle pressure loss
P9  : given (default P9 = P0 → fully expanded)
M9  = √( ((pt9/P9)^gt − 1) / ((γt−1)/2) )
T9  = Tt9 / (1 + (γt−1)/2·M9²)
V9  = M9 · √(γt·Rt·T9)
```
*Why:* the nozzle adds no heat/work so `Tt9 = Tt5`, but a real one loses total
pressure (`π_n`). It then expands to whatever back-pressure it is *told* — `P9`.
When `P9 = P0` all of `pt9` is spent (fully expanded, the rung-1 case); when
`P9 > P0` (Example 7.1: `P9 = 2·P0`) the jet leaves still pressurized and a
pressure-thrust term appears below. `π_n` is a *specified ratio*: assert
`pt9 == π_n·pt5` exactly; the static drop `pt9/P9 == (Tt9/T9)^(1/gt)` stays exact.

---

## The shaft balance (dual-cp form)

Rung 1's keystone, now with two `cp`s and shaft friction:
```
η_m · (1 + f) · cpt · (Tt4 − Tt5)  =  cpc · (Tt3 − Tt2)
```
Left = useful turbine power delivered to the shaft (hot, heavy stream, minus
friction); right = compressor power demanded (cold stream). The closure assert
(engine-owned, it alone holds Tt2/Tt3) checks these two sides agree.

---

## Performance (with the pressure-thrust term and two thermal efficiencies)

**Specific thrust** — gains a pressure term when `P9 ≠ P0`:
```
F/mdot = (1+f)·V9 − V0 + (1+f)·Rt·T9·(1 − P0/P9)/V9         [SI, gc = 1]
```
The last term is the static-pressure imbalance `A9(P9−P0)/mdot` rewritten via the
ideal gas law and `mdot9 = ρ9·A9·V9`. It vanishes when `P9 = P0`, recovering
rung-1's `F/mdot = (1+f)V9 − V0`.

**TSFC:** `S = f / (F/mdot)`.

**Two thermal efficiencies — report both, named distinctly:**
```
eta_brayton  = 1 − Tt2/Tt3                                  # Brayton identity (cold)
eta_thermal  = [ (1+f)·V9² − V0² ] / ( 2 · f · hPR )         # KE/fuel (net work / heat)
```
- `eta_brayton` is the rung-1 number (`0.4821`) and the **primary hand-check**
  (`1 − 1/πc^gc`). It is a *cold-Brayton identity* that no longer equals the true
  thermal efficiency once legs tilt — we keep it only because it preserves the
  rung-1 table and the hand-check (so `tests/test_validation.py` stays green
  untouched).
- `eta_thermal` is the **real** thermal efficiency (kinetic energy added to the
  jet per unit fuel power). It anchors Mattingly's `η_T = 41.92%` and equals
  `0.5477` in the rung-1 ideal limit (not `0.4821` — different quantity).

**Propulsive & overall** — definition-consistent, *no* split needed:
```
eta_p = (F/mdot · V0) / ( ½·[ (1+f)·V9² − V0² ] )
eta_o = (F/mdot · V0) / ( f · hPR )
```
These reduce to the rung-1 values (`0.4073`, `0.2231`) **and** match Mattingly
(`74.39%`, `31.18%`) with one formula each.

**Cascade closure (free consistency assert + teaching payoff):** under the
KE-based `eta_thermal`, the textbook cascade now holds *exactly*:
```
eta_o == eta_thermal · eta_p     # 0.5477·0.4073 = 0.2231;  0.4192·0.7439 = 0.3118
```
Rung 1's NOTES had to explain why this *failed* under `eta_brayton`; switching to
the physically-correct thermal efficiency *restores* it. Assert it.

---

## Verification gates (in priority order)

1. **Reduce-to-ideal (the load-bearing gate).** Set every efficiency and pressure
   ratio to 1 (`η_c=η_t=η_b=η_m=1`, `π_d=π_b=π_n=1`), `unify()` the gas triple
   (`γt=γc, cpt=cpc, Rt=Rc`), and `P9 = P0`. The full rung-1 validation table must
   reproduce **to the digit** — station `Tt, pt`, `f`, `V9`, `M9`, `T9`, specific
   thrust, TSFC. (Efficiency *rows* are checked under their rung-1 definitions:
   `eta_brayton`→0.4821, `eta_p`→0.4073, `eta_o`→0.2231.) This is the rung-2
   analog of the primary hand-check — build for it from the first component.
2. **Directional checks.** With losses on, `η<1` must *lower* specific thrust,
   *raise* TSFC, and push endpoints to higher entropy: `Tt3 ≥ Tt3s`, `Tt5 ≥ Tt5s`.
3. **External anchor — Mattingly Example 7.1** (`docs/plans/rung2-anchor-mattingly.md`),
   to the digit. Inputs use **isentropic** `η_c=0.8641, η_t=0.9099` converted from
   the book's polytropic `e_c=e_t=0.9` (conversion below). Targets: `f=0.03567`,
   `V9≈1253.8`, `M9=2.253`, `T9≈833.4`, `F/mdot=806.9`, `S=4.421e-5`,
   `eta_thermal=0.4192`, `eta_p=0.7439`, `eta_o=0.3118`.

### Polytropic → isentropic conversion (exact for a perfect gas)
```
η_c = (πc^gc − 1) / (πc^(gc/e_c) − 1)
η_t = (1 − τt) / (1 − τt^(1/e_t)) = (1 − τt) / (1 − π_t^gt)   # τt from the shaft balance
```
The code stays isentropic (the chosen rung-2 knob); we convert the book's `e`
once to feed the anchor test. Polytropic efficiency as a *first-class* knob is a
later sub-rung.

---

## Conservation asserts (rung-2 deltas)

Carry over rung 1's mass / shaft-energy / burner-energy / isentropic-leg habit,
with these changes:
- **Isentropic legs → substate + inequality** for the compressor and turbine only:
  the *ideal substate* satisfies `(pt/pt)^g`, and the *actual* temperature shows
  entropy generation (`Tt3 ≥ Tt3s`, `Tt5 ≥ Tt5s`).
- **Specified ratios asserted exactly:** `pt2 == π_d·pt0`, `pt4 == π_b·pt3`,
  `pt9 == π_n·pt5`.
- **Shaft closure** uses the dual-cp + `η_m` balance above.
- **Cascade closure** `eta_o == eta_thermal·eta_p` as a performance-level assert.

---

## Done when

The reduce-to-ideal gate reproduces the rung-1 table to the digit, the directional
checks hold, and the Mattingly Example 7.1 anchor matches to ~0.1%. The T–s
diagram now shows the compressor and turbine legs **tilted right** (entropy
generated) instead of vertical, and `NOTES.md` explains, in plain language, what
each efficiency buys, why the dual-cp shaft balance couples differently, and why
switching the thermal-efficiency definition restores the cascade.
