# Rung 29 — The shifting turbine: is the frozen turbine EARNED?

Every rung since 6 has **frozen** the station-4 equilibrium mixture through the turbine
and the nozzle. Rungs 14 and 25 then read that frozen pool at the nozzle entry, found it
**super-equilibrium**, and built the whole `(R−I)` entry-irreversibility gap on that
premise. Rungs 26–28 inherited it. Nobody ever asked whether the *freeze* was earned.

Rung 29 asks, the way rung 14 asked it of the nozzle: **bracket the turbine.** Frozen vs
fully-shifting, at the same shaft-set work. Zero knobs, no rate, no `τ_res`.

**The verdict: EARNED at the design point, BITES HOT** — and the reason the project's own
super-equilibrium language misled us about which is which is the rung.

---

## What is actually new: a WORK-LIMITED endpoint

The nozzle bracket (rung 14) is **pressure-limited** — both expansions run down to the
same back-pressure `p9`. A turbine cannot be posed that way, and this is the one
structural difference that makes rung 29 need its own solver.

The shaft sets the **enthalpy drop**:

```
delta_h = ( h_c(Tt3) − h_c(Tt2) ) / ( η_m·(1 + f) )        # engine-owned, unchanged
```

`delta_h` depends only on the compressor and `f`. **It does not depend on the turbine's
chemistry at all** — so a shifting turbine reopens *no* shaft fixed point. (This corrects
the framing that opened this rung, which claimed a shifting turbine "reopens the shaft
balance." It does not. It moves where the expansion *ends up* for a drop that was already
decided upstream, and those exit conditions then propagate to the nozzle and to the
25–28 brackets.)

So **both** bounds give up the *same* `delta_h`, and the unknown is the exit state.
Two unknowns `(T5, p5)`, two equations:

```
H_abs(comp(T5,p5), T5)      = H_abs(comp4, Tt4) − delta_h·m      # work-limited
S_mix(comp(T5,p5), T5, p5)  = S_mix(comp4, Tt4, pt4)             # reversible (the ideal bracket)
```

with `m` the recombination-invariant mass per mol air, and

- **(F) FROZEN** — `comp = comp4` throughout (the shipped model).
- **(S) SHIFTING** — `comp = _equilibrium_composition(far, T, p)` at every point: instant
  chemistry, the **maximum** shift any real turbine could ever show.

**Why absolute (scale-B) enthalpy.** The composition changes between entry and exit, so
formation enthalpy no longer cancels out of the work balance the way it does for a frozen
gas. Using sensible enthalpy here would silently discard exactly the recombination energy
the bracket exists to measure. (`_mix_h_abs_B`, available since fork B restored `a6`.)

`_work_limited_expand` solves it by outer bisection on `p5` (expanding to lower `p5`
extracts more work, so the enthalpy residual is monotone) with an inner isentropic
bisection on `T5`.

---

## Result

Design point `π_c = 10`, `M0 = 0.85`, real losses; the bound at four burner temperatures:

| `Tt4` [K] | `T5` frozen [K] | `T5` shifting [K] | `ΔT5` [K] | `ΔT5/T5` | `Δp5/p5` | earned? |
|-----|-----------|----------|-----|--------|--------|--------|
| 1500 | 1262.69 | 1262.83 | +0.13 | **+0.0107%** | +0.0051% | **YES** |
| 1800 | 1576.20 | 1577.41 | +1.21 | +0.0765% | +0.0282% | YES |
| 2100 | 1888.03 | 1894.98 | +6.95 | +0.3680% | +0.1121% | no |
| 2400 | 2200.06 | 2240.96 | +40.90 | **+1.8592%** | +0.4653% | no |

**At the design point the frozen turbine is earned outright.** The maximum conceivable
shift moves `Tt5` by 0.011% — an order below the cycle's own modelling error (`η_t`, `π_b`
are quoted to ~1%). No rate model, no `τ_res`, no knob can make a real turbine exceed
this, because it is the instant-chemistry reversible limit. That is the value of doing
the bound first: **the conclusion is rate-independent.**

**Hot, it stops being earned.** By `Tt4 = 2100 K` the bound is past the 0.1% threshold and
by 2400 K it is 1.9% in `Tt5` and 0.47% in `pt5` — a 174× growth across the band. Every
rung from 6 up quietly assumes the freeze; that assumption is a design-point fact, not a
structural one, and it should not be quoted outside the band where it was checked.

### `frozen_turbine_earned` is a DECLARED threshold

`_SHIFT_EARNED_TOL = 1e-3` is chosen, not derived — one order below the cycle's accepted
component-loss precision, so a shift beneath it cannot matter beside errors already
accepted. Stated so it is not mistaken for a physical boundary.

---

## The finding: RATIO ≠ ENERGY

The interesting part is not the numbers but **why we expected the opposite.**

Rungs 25–28 justify the super-equilibrium entry with a **ratio** — `x_frozen/x_eq`,
`[NO]/[NO]_e`, quoted at 10×, 100×, "3–9 orders." Read that ratio and you expect a
shifting turbine to have a lot to work with, especially at the lean design point where
the ratio is *largest*. The bound says the opposite, and the anti-correlation is exact:

| `Tt4` [K] | super-eq ratio (max) | radical inventory `x_O+x_H+x_OH` | `ΔT5/T5` |
|-----|----------|-----------|--------|
| 1500 | **109.4×** | 3.178e-05 | **+0.0107%** |
| 1800 | 17.7× | 3.106e-04 | +0.0765% |
| 2100 | 6.6× | 1.522e-03 | +0.3680% |
| 2400 | **3.3×** | 3.835e-03 | **+1.8592%** |

Across the band the ratio **falls 33×** while the inventory **rises 121×** and the shift
**rises 174×**. The inventory tracks the shift; the ratio anti-tracks it.

**The ratio is not wrong — it is the wrong currency.** `x_frozen/x_eq` correctly measures
*kinetic* super-equilibrium: how far the pool sits from where chemistry wants it, which is
exactly what a **rate** question needs, and it is doing honest work in rungs 25–28. But it
is **not a proxy for exploitable enthalpy**, which scales with the *absolute* radical
**inventory** `x·n`. A pool can be 109× super-equilibrium and carry no energy worth
recovering, because 109× of almost nothing is still almost nothing. At the lean design
point the radicals are ~3e-5 in mole fraction: complete recombination releases essentially
no heat.

So the ratio is **loudest exactly where the shift is most negligible** — the two are not
merely different, they are anti-correlated over the operating band. That is a cross-rung
correction, not a local one: every rung from 25 up quotes the ratio at the lean end, which
is precisely where it overstates what a shifting flow could take away.

### SHARPENED: `ENERGY = INVENTORY × COMPLETION`

The `π_c` check (`docs/rung29-pi-c-margin.md`) found that **the inventory is itself an incomplete
currency** — the same failure this section diagnoses for the ratio, one step milder. Along `π_c` at
fixed `Tt4`, the entry inventory **falls** 3.4× (higher `pt4` suppresses dissociation) while the shift
**rises**: they anti-correlate. What actually sets the shift is the **recombined** inventory —
`inventory × completion`, where completion is the fraction the expansion asks for, and it climbs
36.5% → 99.995% over `π_c` 2→80 because a larger `delta_h` runs deeper and colder. The two channels
are **comparable and opposed** on this axis, so their product turns over in the interior.

The `Tt4` claim above is **untouched**: there both channels move the same way and the inventory swings
two orders, so it dominates and reads correctly. The `π_c` axis is simply the one that can tell the two
apart. (The product law is quoted only at the cool design point — `x_O+x_H+x_OH` omits `CO→CO₂`, which
is why `ΔT5/Δinventory` is flat to ±4% at `Tt4`=1500 but varies 2× at 2100.)

### What is NOT a finding

**That a fully-shifted entry collapses rung-25's `(R−I)` gap to zero.** Measured
(`V_R − V_I` falls from 1.235 m/s to ~0 at `Tt4=2400`), but it is **structural, not a
result**: an entry pinned at equilibrium has no super-equilibrium left to relax
irreversibly, so `(R−I) → 0` is a tautology of the construction. The residual −0.003 m/s
is bisection tolerance, not physics.

What *is* worth carrying is the size of the move required to get there — 1.9% in `Tt5` —
and that the design point sits ~170× short of needing it. Downstream, the shifted entry
also halves the `(I−F)` bracket hot (25.57 → 12.88 m/s at `Tt4=2400`): half the
recombination benefit rung 14 attributes to the nozzle has, on that path, already been
banked in the turbine.

---

## Concessions

- **`η_t = 1` by nature.** A reversible bracket *is* isentropic; there is no way to thread
  turbine efficiency through it. A real `η_t < 1` adds a separate — and much larger —
  entropy source, but it acts on the *expansion*, not on the *chemistry*, so it does not
  touch what this brackets. Same concession rung 14 makes for the nozzle. The frozen bound
  is nevertheless bit-for-bit the shipped `Turbine.apply` at `η_t = 1` (gate 1).
- **No rate, therefore no location.** The bound says how far the turbine *could* shift, not
  how far it *does*. Deferred deliberately (below).
- **One design point.** `π_c = 10`, `M0 = 0.85`. The `π_c` axis is now **CHECKED**
  (`docs/rung29-pi-c-margin.md`): "earned at design" holds at every `π_c` from 2 to 80 with a
  **9.4×** margin, and the boundary `Tt4*` stays above **1846 K** everywhere. `π_c` is a **weak,
  non-monotone, double-edged** knob — *not* protective the way rung 28's `β` turned out to be. That
  check also **sharpens the finding below**: the inventory is an *incomplete* currency. The **`M0` /
  flight axis is now CHECKED too** (`docs/rung29-M0-margin.md`): "earned at design" holds at every
  runnable `M0` from 0.3 to 3.0 with an **8.8×** margin (worst case low-`M0` **takeoff**, not cruise),
  and — the clean **opposite** of `π_c` — the shift is **monotone-protective**, no turnover. The two
  axes **unify** on one `INVENTORY × COMPLETION` surface: the shift turns over only where the axis
  drives `delta_h` (hence completion) steeply, which `π_c` does through `τ_c` and `M0` does not through
  its datum shift — a **correction** to how that check framed the unification (headroom is not the
  discriminator; the `delta_h` swing is).
- **Cycle untouched.** A pure diagnostic beside the cycle: the production turbine still
  freezes, so the cycle remains **bit-for-bit rung 6** and the rungs-7–28 invariant holds
  (gate 3).

---

## Verification gates (`tests/test_rung29.py`)

1. **REDUCE TO PRIOR** (the spine) — the frozen bound **is** the shipped `Turbine.apply` at
   `η_t = 1`, **bit-for-bit** (`==`, not a tolerance). Structural: `Gas.shifting_turbine`
   takes those two lines verbatim rather than re-solving, so the bracket cannot drift from
   the cycle it brackets.
2. **THE SOLVER IS RIGHT** — the independent `_work_limited_expand(shifting=False)`
   bisection reproduces that closed form to 1e-6. Without this gate 1 is a tautology; the
   two are genuinely different code paths (mixture entropy + absolute enthalpy vs
   `h_t`/`pr_t`) onto the same physics.
3. **CYCLE UNTOUCHED** — calling the diagnostic does not perturb any station, `V9`, or the
   thrust (bit-for-bit rung 6).
4. **THE VERDICT** — `frozen_turbine_earned` at `Tt4=1500`, NOT at 2400; the bound grows
   >100× across the band.
5. **THE INVERSION** — the super-eq ratio is strictly **decreasing** in `Tt4` while the
   shift and the radical inventory are strictly **increasing**; ratio falls >10×, shift
   rises >100×. The anti-correlation is the rung.
6. **DIRECTION** — recombination reheats, so at equal work the shifting exit is warmer and
   at higher pressure (also asserted inside `shifting_turbine` on every call, contract #4).

---

## Deferred

- ~~**A finite-rate turbine march.**~~ — **INVESTIGATED, NEGATIVE, NOT a rung**
  (`docs/turbine-march-negative.md`). The natural next step: march the Damköhler flow *between* `F` and
  `S` the way rung 25 marched it between rung-14's nozzle bounds. It was built (probe) and returned a
  negative — **and the reason is structural, deeper than the un-anchored `τ_res` cited below**. Rung 25
  became a rung despite its un-anchored rate because its `Da→∞` **irreversible-fast ceiling (I)** is a
  rate-independent closed form sitting *strictly below* the reversible bound — the super-equilibrium
  nozzle entry makes a real flow relax irreversibly even at infinite rate. **That dodge cannot be
  repeated here:** the turbine entry is station 4, the *fresh burner exit*, which is **at equilibrium by
  construction** (`comp_entry = _equilibrium_composition(far, Tt4, pt4)`), so a `Da→∞` march stays on the
  equilibrium manifold and lands *exactly* on `S` — **`I_turb ≡ S`, no third state, no `(R−I)` sliver**
  (`dS(I_turb)` = machine-zero at every `Tt4` vs the nozzle's `+4.3e-4 → +4.1e-2`, from exact closed
  forms). The march is a **plain two-state interpolation** `F→S` whose only new degree of freedom (the
  interior landing) rides on **two** un-anchored knobs: `τ_res` (below) **and** — worse than the nozzle —
  an **ambiguous progress coordinate**, because the work-limited endpoint leaves `p5` unknown so there is
  no natural schedule to march on. The rung-26 clock at turbine conditions gives `τ_chem(4)` = 9.1e-4 s
  (`Tt4`=1500) to 5.7e-5 s (2400) ⇒ `Da_turb` = 0.05…8.8 (transitional, *not* fast despite the pressure —
  the residence is short too), but that span rides on the un-anchored `τ_res`; it is a **supporting
  sketch only**. **The positive by-products** (in the doc): rung 25's `(R−I)` gap is **manufactured by
  the freeze, not intrinsic to expansion** (localized to the one turbine→nozzle handoff); rung 29's `S`
  is **attainable**, the genuine `Da→∞` limit, not an unreachable ceiling like rung 25's `R`; and — the
  rate-independent statement worth carrying — since `F≈S` at design, the whole `[F,S]` interior is
  negligible **regardless of either knob**, so the march **cannot overturn "freeze earned at design."**
  A genuine rung would need a real turbine passage geometry to anchor *both* knobs at once — the blade-row
  count and the same choked-flow seam `docs/tau-res-negative.md` already named.
- ~~**The `π_c` axis**~~ — **CLOSED** (`docs/rung29-pi-c-margin.md`). Unlike rung 28's `β`, the worry
  did **not** invert: it **confirmed**, at 9.4× margin, with `π_c` weak and non-monotone (the
  boundary is bowl-shaped, worst case **interior** near `π_c`≈15; the not-earned band *widens* 2.7×
  with `π_c`). The substantive by-product is the **sharpening** above.
- ~~**The `M0` / flight axis**~~ — **CLOSED** (`docs/rung29-M0-margin.md`). The clean **opposite** of
  `π_c`: the shift is **monotone-protective** (the bracket's `β`-like axis, no turnover), earned at
  every runnable `M0` from 0.3 to 3.0 with an **8.8×** margin, worst case low-`M0` **takeoff**. Same
  `INVENTORY × COMPLETION` currency read where it is **lopsided** (completion near-saturated because
  `M0` drives `delta_h` only weakly — a datum shift, not a work climb). It **corrects** the `π_c` doc's
  unification: the turnover discriminator is the **`delta_h` swing**, not completion headroom (proven
  by the `π_c = 2` control, still monotone with headroom). And the flight axis is **double-edged**:
  protective per point, yet ram heating shrinks the earned **operating band** ×2.1 while the
  not-earned band widens ×1.7. Both `π_c`-family concessions are now closed; what remains is the
  finite-rate turbine march (above) and feeding station 5 back into the cycle (below).
- **Feeding the shifted station 5 into the production cycle** — that is a re-foundation
  (it re-anchors every rung's numbers), not a rung.
