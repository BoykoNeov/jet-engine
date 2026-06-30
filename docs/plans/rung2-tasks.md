# Rung 2 — Task Checklist (Real Components & Dual-Gas)

Mirrors `docs/rung2-spec.md`. Decisions (with the user): isentropic efficiency now
(polytropic noted for later), dual constant `(cp, γ, R)` per section, anchored to a
textbook case. All boxes below land green.

## Design decisions (settled)
- [x] Efficiency knob = **isentropic** `η_c, η_t` (polytropic = later sub-rung).
- [x] Gas = **dual** cold/hot `(γ, cp, R)`; `R = (γ−1)/γ·cp` per section.
- [x] External anchor = **Mattingly Example 7.1** (verified, re-derived) — under-
      expanded (`P0/P9=0.5`), so rung 2 takes a *specified* exit pressure + the
      pressure-thrust term. A *choked* convergent nozzle stays deferred.
- [x] Two thermal efficiencies: keep `eta_brayton` (rung-1 hand-check) + add the
      real KE-based `eta_thermal` (restores the `η_o = η_thermal·η_p` cascade).

## Derivation (derive before code)
- [x] `docs/rung2-spec.md` — station equations with η/π terms + justifications.
- [x] `docs/plans/rung2-anchor-mattingly.md` — verified Example 7.1 inputs/outputs
      + the exact polytropic→isentropic conversion.

## Physics / code
- [x] Dual-section `Gas` (`g_c`/`g_t`, `unified()` collapse). `Gas()` == rung-1.
- [x] Inlet `pi_d` (+ `ram_recovery(M0)` helper); pure `apply(state, gas)`.
- [x] Compressor `eta_c` via ideal substate `Tt3s`; substate-isentropic + entropy
      inequality asserts.
- [x] Burner dual-cp `f` with `eta_b`, `pi_b`; pt4 == pi_b·pt3 asserted.
- [x] Turbine `eta_t` via ideal substate `Tt5s` (hot section).
- [x] Nozzle `pi_n` + specified exit `p9` (default `p0`); hot section.
- [x] Engine: dual-cp + `eta_m` shaft balance + closure assert; pressure-thrust
      term; `eta_brayton` + KE `eta_thermal` + cascade-closure assert.

## Verification
- [x] Reduce-to-ideal: `unified()` + all η/π=1 + `p9=p0` reproduces rung-1 to the
      digit (`tests/test_rung2.py::test_unify_reduces_to_rung1`; rung-1 suite too).
- [x] Directional: losses lower specific thrust, raise TSFC, lower `eta_thermal`.
- [x] Mattingly Example 7.1 anchor — matches to **< 0.02%** on every headline number.
- [x] Full suite green (`pytest`: 11 passed).

## Artifacts
- [x] `main.py` prints ideal vs real tables + the loss summary.
- [x] T–s diagram overlays ideal (vertical legs) vs real (legs tilt right).
- [x] `NOTES.md` — rung-2 section: the two efficiency kinds, why real gas rides in
      with real components, the dual-cp shaft balance, the thermal-efficiency fix
      (and the `eta_brayton` wrong-way surprise), the anchor.

## Deferred (seams kept)
- [ ] Polytropic efficiency as a first-class knob.
- [ ] Variable `cp(T)` (thermally perfect → reacting gas).
- [ ] Choked convergent nozzle (choke detection + pressure-thrust branch).
- [ ] Off-design / component maps; afterburner.
