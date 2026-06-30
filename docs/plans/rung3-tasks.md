# Rung 3 — Task Checklist (Variable `cp(T)`, the Thermally-Perfect Gas)

Mirrors `docs/rung3-variable-cp.md`. Decisions (with the user): cp(T) model = NASA
7-coefficient, air-only (products as one representative cp(T), not composition
tracking); anchor = Çengel + Mattingly gas-table examples; process = spec-doc-first.
All boxes below land green.

## Design decisions (settled)
- [x] cp(T) model = **NASA 7-coeff**, air = mole-weighted N₂/O₂/Ar; hot = one
      fixed-composition lean-products mixture (CO₂/H₂O/O₂/N₂/Ar). No reacting gas.
- [x] The **CPG-branch trap**: a calorically-perfect section keeps the rung-1/2
      closed forms (γ-based) exactly; a thermally-perfect section integrates
      (`pr = exp(φ/R)`). Branch hides in `Gas`; only stations 0 and 9 (velocity↔
      enthalpy coupling) expose it and branch explicitly.
- [x] Enthalpy datum **h(0)=0** (bare antiderivative) — the burner is the one
      cross-section enthalpy use, so both sections must share it.
- [x] Anchor = **Çengel 9-89 + Mattingly Ex 2.7/2.8** (machinery, not turbojet —
      topology caveat). No variable-cp turbojet exists in the texts; products float.

## Derivation (derive before code)
- [x] `docs/rung3-variable-cp.md` — property interface, the trap, station equations
      in h/pr form, shaft balance in Δh, verification gates.
- [x] `docs/plans/rung3-anchor-cengel.md` — sourced + NASA-verified anchor data.

## Physics / code
- [x] `Gas` property interface: `h/pr/T_from_h/T_from_pr/gamma_*_at/cp_*_at` per
      section, CPG closed-form branch + TPG analytic-integral branch (piecewise at
      1000 K), safeguarded-Newton inverses, `Gas.thermally_perfect()` factory.
      `Gas()` defaults still == rung-1; `unified()` collapses TPG too.
- [x] Compressor/Burner/Turbine rewritten in h/pr form (reduce to CPG bit-for-bit).
- [x] `Turbine.apply` signature `delta_Tt` → `delta_h`; engine shaft balance in Δh.
- [x] Freestream (station 0) + Nozzle (station 9) branch CPG/TPG (the trap); the
      TPG nozzle energy-split assert is now exact (tight), loose only for CPG.

## Verification (`tests/test_variable_cp.py`)
- [x] Reduce-to-ideal — existing suites (`test_validation`, `test_rung2`,
      `test_polytropic`) stay **green and untouched**; a guard repeats it here.
- [x] Round-trip inverses (~1e-9) + monotonicity, cold & hot, across the 1000 K join.
- [x] Dual-section discriminating CPG-vs-integral check (~3e-4, distinct cold/hot
      flats — proves integral path + correct routing).
- [x] Air-table isentropic anchor (~574 K vs the CPG 579 K).
- [x] Çengel 9-89 (T2s, T4s, η_th) + Mattingly Ex 2.7/2.8 — machinery, to ≤0.15%.
- [x] Directional + the gas-table effect (TPG compression lands cooler than CPG).
- [x] Full suite green (`pytest`: 23 passed).

## Artifacts
- [x] `main.py` prints the rung-2-frozen-`cp` vs rung-3-`cp(T)` table (the honest
      baseline) + the gas-table effect (cooler Tt3, frozen-1239-vs-true-1131 fuel).
- [x] T–s diagram left alone (cp(T) is a numbers/curvature point, not a leg-tilt).
- [x] `NOTES.md` — rung-3 section: why constant cp worked, what h(T)/pr(T) buy, why
      the power law had to go, the closed-form-CPG-branch trap, the honest baseline.

## Deferred (seams kept)
- [ ] Reacting / variable-composition products (CO₂/H₂O/dissociation vs `f`) — the
      next gas rung; now just a new `Gas` behind the same four functions.
- [ ] Choked convergent nozzle (choke detection + pressure-thrust branch).
- [ ] Off-design / component maps; afterburner.
