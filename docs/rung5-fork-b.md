# Rung 5 — Fork B: Formation-Enthalpy Bookkeeping (Derived Heat Release)

Rung 4 made the product **composition** track the fuel/air ratio `f`, but kept
**Fork A**: a fixed heating value `hPR`, sensible-style enthalpies on the `h(0)=0`
datum. The burner *assumed* how much heat the fuel releases. Rung 5 switches to
**Fork B**: enthalpies carry each species' **formation** enthalpy (absolute scale —
production keeps rung 3's `0K` sensible datum and adds the `ΔH̄f°` formation sum; see
§ Adds for how this relates to the textbook "elements = 0 at 298.15 K" scale), and the
burner's heat release is **derived** from an
energy balance on those absolute enthalpies — `hPR` is no longer an input, it
**falls out**.

> **Read `docs/rung4-reacting-products.md` first.** This file states only what
> *changes*. The explicit `(CH₂)ₙ` stoichiometry, the `f`-parameterized composition,
> the `h`/`pr` property machinery, the dual cold/hot split, the two efficiency kinds,
> "derive before you code," and the conservation-assert contract all carry over.

> Numbers-before-code record: **`docs/plans/rung5-anchor-formation.md`** (sourced
> formation enthalpies, the derived-LHV = Mattingly-`hPR` anchor, the exact
> reduce-to-rung-4 equivalence proof, the AFT plausibility check).

---

## What rung 5 adds (and what it deliberately does not)

**Adds:**

- **The formation constant `a6`, restored.** Full NASA-7 is
  `H/RuT = a1 + a2 T/2 + … + a5 T⁴/5 + a6/T`; rungs 3/4 stored only `a1…a5` (the
  `cp/R` shape). Since rung 3's `_antideriv_h` is exactly the polynomial part *with no
  constant*, `Ru·a6` is precisely the additive formation term. We **derive** each
  species' `a6` from its standard ΔH̄f°(298.15) rather than transcribe a column
  (reuses the certified rung-3 polynomials, no fresh transcription risk):
  `a6 = ΔH̄f°/Ru − antideriv_h(A_low, 298.15)`.
- **Absolute enthalpies for the burner.** Production adds the **raw formation** sum to
  rung-3's sensible enthalpy: `h_abs(T) = h_sensible(T) + Σ ΔH̄f°/M` (air = 0, products
  carry `CO₂`/`H₂O`). `h_sensible` keeps rung 3's **0 K** datum, so this scale is
  *`0K-sensible + formation`* — a valid absolute scale, but its **elements are not zero
  at 298.15 K** (air sits at its 0 K-referenced sensible value). It equals the textbook
  "elements = 0 at 298.15 K" scale plus a per-species `h_sensible(298.15)` constant; that
  constant **cancels in the burner** because the Fork-A and Fork-B balances share the same
  sensible `h_c/h_t` (only the formation term survives, closed by the LHV identity). The
  `a6`-at-298.15 form (`h_sensible + Ru·a6/M`, elements = 0 at 298.15) appears **only in
  the test's formation self-check**, to verify the constants reproduce `ΔH̄f°`; it must not
  be mixed with production's scale (see the rung-6 seam). Fuel gets a formation enthalpy
  `h̄f°(CH₂)` (the single calibration input, replacing `hPR`).
- **The derived-heat-release burner.** The fuel balance becomes the adiabatic
  absolute-enthalpy balance `Σ N h̄(react) = Σ N h̄(prod)` (with `η_b` booking the
  loss); the lower heating value is the by-product `LHV = h̄f°(CH₂) − h̄f°(CO₂) −
  h̄f°(H₂O,g)` per unit fuel mass.
- **A `Gas.reacting_forkb(...)` factory** beside `Gas.reacting()` (Fork A), so the
  frozen and Fork-A paths are untouched and reduce-to-ideal stays free.

**Deliberately deferred (seam kept — now the whole of rung 6):**

- **High-temperature dissociation / equilibrium** (CO₂⇌CO+½O₂, H₂O⇌OH+½H₂, …). Needs
  a `T`-coupled `Kp(T) = exp(−ΔG°/RuT)` solve, which requires the **absolute entropy**
  constant `a7` (`S/R = a1 lnT + … + a7`) on top of Fork B's absolute enthalpy. **This
  is rung 6**, and Fork B is its prerequisite (you cannot compute which way
  `CO₂⇌CO+½O₂` goes from sensible-only enthalpies — the formation enthalpies *are* the
  driving force). Rich combustion (CO/H₂) rides the same equilibrium machinery.
- Off-design / component maps, the choked convergent nozzle, the afterburner — still out.

---

## The load-bearing result: Fork B reproduces rung 4 **exactly**

For **complete** combustion to **fixed** products, Fork B and rung-4 Fork A produce
**bit-identical** `f`, `Tt5`, thrust, and TSFC. This is not an approximation — it is a
theorem. The released chemical energy per kg air is *identically* `f·LHV`:
```
f·h̄f_fuel/M_fuel − (1+f)·h̄f_products(f)  ≡  f · LHV      (only CO₂,H₂O carry formation;
                                                          (1+f) = Σ nᵢMᵢ/M_air by mass conservation)
```
so the Fork-B balance `(1+f)·h_t0(Tt4,f) = h_c0(Tt3) + η_b·[derived release]` is
algebraically the rung-4 balance with `hPR := LHV`. Measured gap on `f`: **0.0**
(machine zero; `docs/plans/rung5-anchor-formation.md §3`).

**So what does Fork B buy, if the numbers don't move?** Structure, not digits:
1. **Heat release now varies with composition automatically.** Fork A's `hPR` is a
   frozen constant; Fork B's release is computed from *which products actually form*.
   Identical while products are fixed — but the moment rung 6 lets dissociation shift
   the products, the energy books follow with no recalibration.
2. **The absolute scale rung 6 needs.** `Kp` needs `ΔG° = ΔH° − TΔS°` on absolute
   `h̄` and `s̄`. Fork B installs the `h̄` half.
3. **`hPR` is explained, not assumed.** The number rung 4 typed in (42.8 MJ/kg) now
   *emerges* from `h̄f°(CO₂)`, `h̄f°(H₂O)`, and the one fuel calibration — the honest
   provenance of the heating value.

We do **not** oversell "derived, not assumed": the fuel enthalpy `h̄f°(CH₂)` carries
exactly the information `hPR` did (it is pinned to reproduce 42.8 MJ/kg). The win is
that the release is now *structural* and *composition-aware*.

---

## Station equations — what changed from rung 4

**Only the burner changes, and only in how it books enthalpy.** Every other station is
**bit-for-bit rung 4**: the turbine Δh, the nozzle `V9 = √(2(h(Tt9)−h(T9)))`, every
substate — all are enthalpy **differences**, and `a6` cancels in a difference. So the
turbine/nozzle keep the **sensible** enthalpies (no `a6`); only the burner uses
absolute ones. *(This is the surgical choice: "only the burner sees `a6`.")*

### 4 — Burner: derived-heat-release fuel balance (Fork B)
```
pt4 = π_b · pt3
f   : solve   (1+f)·h_t,abs(Tt4,f)  =  h_c,abs(Tt3)  +  f·h_fuel,abs  −  (1−η_b)·f·LHV
```
where `h_*,abs = h_*,sensible + formation`, air formation = 0 (elements), and the
`(1−η_b)·f·LHV` term books incomplete-combustion loss exactly as Fork A's `η_b·hPR`
did (the products we model are always complete — unburned CO/fuel is rung 6). Solved by
the same **fixed-point iteration** as rung 4 (it is the same contraction: substitute
the `f·LHV` identity and it *is* rung 4's `f = g(f)` with `hPR := LHV`). Reduces to the
rung-3 one-shot the instant `h_t` stops depending on `f`.

**Standing equivalence assert (the rung-5 gate):** the `f` from the absolute balance
must equal the `f` from the rung-4 datum-0 form with `hPR := LHV` to ~1e-9 — a live
check that the formation bookkeeping is a faithful re-derivation, not a drift.

The **datum invariant** (the trap to guard): reactants and products must sit on **one**
absolute reference. Consistent NASA data + the shared `a6` derivation give this for
free, but assert cold and hot use the same convention. Because product absolute
enthalpies are **negative** (formation dominates below ~1100 K), the burner's — and only
the burner's — `T_from_*` are never called on absolute `h` (the turbine/nozzle inverses
stay on sensible `h`, positive as before); the balance is solved directly in `f`.

---

## Verification gates (priority order)

1. **Reduce-to-ideal + reduce-to-rung-4 (load-bearing).** `Gas()`/dual-CPG and the
   frozen-TPG and reacting-**Fork-A** paths reproduce rungs 1–4 **to the digit**; every
   existing suite stays green **untouched** (Fork B is a separate factory). And the
   reacting-**Fork-B** cycle reproduces the reacting-**Fork-A** cycle's `f`/`Tt5`/thrust
   to ~1e-9 (the exact-equivalence theorem, measured machine-zero).
2. **Derived LHV = Mattingly `hPR`.** `h̄f°(CH₂) = −34.99 kJ/mol` ⇒ `LHV = 42.8000
   MJ/kg` (assert to 1e-6). The number rung 4 assumed, now derived.
3. **Formation self-check.** `H̄(298.15) = ΔH̄f°` per species (to ~1e-6 rel); elements
   land at `h = 0` at 298.15 K. NB: this gate exercises the **`a6`-at-298.15 form**
   (`Ru·(antideriv_h(T)+a6)`), the textbook absolute scale — *not* production's
   `0K-sensible + formation` scale. It certifies the formation constants; the two scales
   differ by a per-species `h_sensible(298.15)` constant and must not be mixed (rung-6 seam).
4. **Absolute-balance closure.** `Σ N h̄(react) = Σ N h̄(prod)` at the converged `f`
   (standing assert), and atom/mass conservation as in rung 4.
5. **AFT physical plausibility.** No-dissociation flame temps monotone in `f`, stoich
   ≈ 2375 K (deliberately high — real is ~2250 K *because of* dissociation → rung 6).

## Conservation asserts (rung-5 deltas)
Carry over rung 4's, plus:
- **Equivalence assert** (gate 1): Fork-B `f` == datum-0 `f` with `hPR:=LHV`.
- **Derived-LHV assert** (gate 2) at construction.
- **Absolute-balance closure** (gate 4) at the converged `f`.
- **One-datum guard:** cold and hot formation constants on the same 298.15-K reference.

## Done when
Reduce-to-ideal reproduces rungs 1–4 to the digit (existing suites untouched, green);
Fork B reproduces reacting Fork A to ~1e-9; the derived LHV, formation self-check, and
absolute-balance asserts hold. `main.py` gains a Fork-A-vs-Fork-B panel (identical `f`,
**derived** `hPR`, the absolute-enthalpy scale, the AFT diagnostic); `NOTES.md` gains a
rung-5 section (why the numbers don't move yet, what the absolute scale buys, why
Fork B must precede dissociation). `CLAUDE.md` scope + deferred-seams updated (Fork B
done; equilibrium/dissociation + `a7` = rung 6).

## The rung-6 seam (keep it additive)
Rung 5 installs absolute **enthalpy** (`a6`). Rung 6 adds absolute **entropy** (`a7`,
`S/R = a1 lnT + … + a7`) and builds `Kp(T) = exp(−ΔG°/RuT)` on `ΔG° = ΔH° − TΔS°`, then
solves equilibrium composition (element conservation + `Kp` per reaction) — replacing
rung 4's deterministic complete-combustion `_products_composition(f)` with a `T`-coupled
solve seeded from it. The burner's absolute balance is unchanged in *form*; only the
product mole numbers become `T`-dependent. Fork B wrote the enthalpy substrate once;
rung 6 adds entropy + equilibrium on top.

**One-convention rule for rung 6.** Production's absolute enthalpy is `0K-sensible +
formation` (`h_c_abs`/`h_t_abs` in `gas.py`); the `a6`-at-298.15 form lives only in the
Fork-B test's formation self-check. Rung 6's `Kp`/dissociation solve **must consume
production's `0K-sensible + formation` scale** (or migrate *both* paths to `a6`-at-298.15)
— it must not mix them. The two differ by a per-species `h_sensible(298.15)` constant that
cancels in the rung-5 burner (Fork-A and Fork-B share the same sensible `h`), but at the
dissociation join the products shift composition, so a mixed convention would leave a
silent `~cp·298 ≈ 0.3 MJ/kg`-scale seam exactly where Fork B exists to prevent one.
