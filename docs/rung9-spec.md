# Rung 9 — Rich primary / RQL: the rich flank of the NOx bell

Rung 8 resolved a hot, near-stoichiometric primary and lifted EI_NO from the mixed-out ~zero
into the ICAO band — but it held the primary **lean-to-stoich** (φ_p ≤ 1). It could only climb
the *lean* side of the NO-vs-φ bell. Rung 9 lets the primary run **rich** (φ_p up to 2.0) and
shows the *other* side: **EI_NO peaks near stoichiometric and then collapses on the rich flank**.
That collapse is the whole reason a real low-NOx combustor exists — the **Rich-burn /
Quick-Quench / Lean-burn (RQL)** architecture deliberately burns its primary rich to stay off the
NO peak. It is the rung-6 *AFT-into-the-band* moment turned once more: the model now contains
both flanks of the bell, so it explains not just *that* zoning makes NO but *how you design the
zone to un-make it*.

> **Read `docs/rung8-spec.md` first**, and `docs/plans/rung9-anchor-rql.md` (numbers-before-
> code: the sourced primary-zone φ range, the CEA rich-methane AFT anchor, the soot bound, the
> machine-checked rich worked example, the reduce-to-rung-8 gate). This file states only what
> *changes*; the Zeldovich integrator, the two-zone `zoned_nox` scaffold, the `a6`/`a7`/`Kp`/
> `_equil_solve` substrate, and the "derive before you code" / conservation-assert contract all
> carry over **unchanged**.

---

## What rung 9 adds (and what it deliberately does not)

**Adds:**

- **Rich equilibrium** — the 8-species `_equil_solve` (3 element balances + 5 reactions) is
  *already complete* rich (CO/H₂ are unknowns; reactions 1+2 span the water-gas shift
  CO+H₂O⇌CO₂+H₂). The only thing that assumed lean was the **seed**. Rung 9 **branches the seed**
  on the O-balance sign: lean (`bO ≥ 2bC + bH/2`) keeps the byte-identical rung-6 expression;
  rich swaps in an O-limited seed (water first, all C→CO, upgrade CO→CO₂ with leftover O). No new
  species, reactions, or datum — the same integrator on a rich pool.
- **A rich primary in `zoned_nox`** — the φ_p guard lifts from ≤ 1 to **≤ 2.0** (the soot /
  no-C(s) bound). `_primary_aft`, `_mixed_out_T`, and the rung-7 `_thermal_no` are unchanged;
  they simply receive a rich (CO/H₂-major) equilibrium pool now.
- **The RQL payoff** — sweeping φ_p across the bell, EI_NO peaks near stoich (~20 g/kg in the
  ICAO band) and **falls steeply rich** (φ_p=1.4 → ~0.01 g/kg, ~1800× lower), even though the
  rich primary still burns *all* the fuel. The AFT rolls over (~φ 1.05) and the O-starved pool
  crashes [O]/[OH] — the two effects that starve the Zeldovich rate.
- **`main.py` panel + `NOTES.md` section + `tests/test_rung9.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; `zoned_nox` is a pure diagnostic. Every
  cycle station is **bit-for-bit rung 6** (the whole rung 1–8 suite stays green). The rich seed
  lives on a branch no rung-1..8 path ever takes (they are all lean), so the reduce is provable,
  not hopeful.
- **Model a finite-rate quench.** Mix-out is the **ideal (infinitely-fast) quench** — NO is
  frozen at the primary value. RQL's *defining hazard* — NO spiking as the gas **dwells at
  stoich** while the quench air mixes in — needs a secondary-zone Zeldovich (finite mixing rate).
  That is the next seam; rung 9 is "rich primary + ideal quench," not "RQL solved."
- **Add super-equilibrium O / prompt (Fenimore) NO** — still deferred (rung-7 seam). The rich
  primary is a rung-6 *equilibrium* pool.
- **Go above soot onset.** Held **φ_p ≤ 2.0** — the 5-species basis has no solid carbon or higher
  hydrocarbons, so it is valid only below soot (graphite onset is C/O=1 ⇒ φ=3 for CH₂; practical
  flame soot is φ~1.8–2). Above the bound the model is *silently wrong*, so it is a **hard
  assert**, not a note.

---

## The load-bearing result: it completes rung 7's inversion on the rich side

Rung 8 climbed the lean flank (φ_p 0.7 → 1.0, EI_NO rising). Rung 9 shows the **whole bell**:
peak at φ_p ≈ 0.95–1.0, then a steep rich collapse. This is the classic NO-vs-φ curve (Turns,
Heywood): thermal NO needs both high T **and** free O/O₂, and going rich kills both — the AFT
rolls over just past stoich, and the O-starved rich pool has orders-of-magnitude less atomic O.
So a **rich primary is a low-NOx regime** — which is *why* RQL burns rich, then quick-quenches
**past** the stoichiometric NO peak to a lean burnout. The peak flame temperature the cycle
designed to the blade limit is not where the NOx was made; and now the model shows you can move
*off* the NO peak by moving the primary φ, not just by lowering T.

---

## The datum / substrate reuse (no new convention)

No new energy datum, no new species, no new reactions. The rich composition comes from the
*same* `_equil_solve` (same `a6`/`a7`/`Kp` scale-A machinery); the primary AFT uses the *same*
scale-A `_h_molar_A`; the NO layer is the *same* rung-7 `_thermal_no`. The **only** change is a
seed branch inside `_equil_solve` — and the lean branch is byte-identical, so rung 5's one-datum
invariant and rung 8's reduce are both untouched. The only datum-free objects crossing between
steps are still **mole numbers**.

---

## The equations — a branched seed and a lifted guard, no station changes

Every cycle station is **bit-for-bit rung 6**. The equilibrium system is unchanged; only the
Newton **seed** branches (rich needs an O-limited start, or the damped Newton is fragile where
the lean seed's `O2 = (bO−2bC−bH/2)/2` goes negative):

```
lean  (bO ≥ 2bC + bH/2):   seed = rung-6 expression, byte-identical   (all rung-1..8 paths)
rich  (bO <  2bC + bH/2):   n_H2O = min(bH/2, bO);   o_left = bO − n_H2O
                            n_CO2 = min(bC, max(o_left − bC, 0));   n_CO = bC − n_CO2
                            n_H2  = bH/2 − n_H2O;     O2, radicals ~ tiny   (rich primary only)
```

`zoned_nox(far, Tt3, Tt4, p, φ_p ≤ 2, τ)` is otherwise the rung-8 flow: far_p = φ_p·f_stoich,
α = far/far_p (now < 1, more air *outside* the rich primary), primary AFT from Tt3 (scale A,
rich equilibrium products), rung-7 NO on that pool, re-equilibrating mix-out to T_mix ≈ Tt4,
frozen NO moles.

- **Standing asserts (rung-9 deltas):** (1) **φ_p ≤ 2.0** soot-bound guard (replaces the rung-8
  φ_p ≤ 1); (2) carry over rung 8's `T_mix ≈ Tt4` re-equilibration gate (now releasing CO/H₂
  oxidation energy too), rung 7's `K`-check (now also at the *lower* rich primary T, down to
  ~1715 K at φ_p=2), the `0 ≤ x_NO ≤ x_NO,e` clamp, and the trace guard (rich NO_eq is *lower*,
  so trace is safer).

---

## Verification gates (priority order)

1. **Reduce-to-rung-8 (load-bearing).** At φ_p ≤ 1 the primary far ≤ f_stoich, so `bO ≥ 2bC+bH/2`
   and `_equil_solve` takes the **lean branch — byte-identical** to rung 6/8. So the whole rung
   1–8 suite is bit-for-bit (verified: 58 green, untouched), and rung 8's exact same-`T_p`
   identity (`zoned` EI == `thermal_nox` at the primary AFT) still holds. Running `zoned_nox` at
   *any* φ_p (rich included) never touches the cycle far — bit-for-bit rung 6.
2. **Rich equilibrium is correct.** Methane-air φ=1.05 AFT in the **CEA band (~2231 K)**, the AFT
   peak sits *slightly rich* (φ ≈ 1.0–1.05), CO/H₂ are **major** and grow with φ, and the rich
   pool satisfies the **water-gas-shift identity** (CO+H₂O⇌CO₂+H₂, Kp from the same `g0`) — a
   thermodynamic self-check that the branched solve landed on the real equilibrium.
3. **The EI_NO bell falls on the rich flank (THE lesson).** EI_NO peaks near stoich (φ_p ≈
   0.95–1.0, in the ICAO band) and the rich flank collapses (EI(1.3) < 10 % of the peak; monotone
   falling past the peak). Why RQL burns rich.
4. **Rich mix-out returns to Tt4, split-independent.** `T_mix` is (near-)identical across the
   rich φ_p sweep and equals Tt4 within the η_b/datum gap — the re-equilibration now also releases
   the CO/H₂ *oxidation* energy, not just dissociation.
5. **Soot-bound guard.** φ_p ≤ 2.0 accepted; above rejected (the 5-species / no-C(s) basis limit).
6. **`K`-check + trace guard hold at the (lower) rich primary T** (~1715–2450 K, inside the
   rung-7 band).

## Conservation asserts (rung-9 deltas)
Carry over rung 6/7/8's, plus: the **φ_p ≤ 2.0** soot-bound scope guard (replacing φ_p ≤ 1); the
rich-branch seed selection (atom-conserving); the `T_mix ≈ Tt4` closure now spanning CO/H₂
oxidation.

## Done when
Reduce-to-rung-8 holds (lean branch byte-identical; rungs 1–8 green, untouched; cycle bit-for-bit
rung 6); the rich methane AFT anchors to CEA and the WGS self-check binds; the EI_NO bell peaks
near stoich and collapses on the rich flank; rich mix-out returns to Tt4 split-independent; the
soot-bound guard trips; the `K`-check/trace gates hold at the rich primary T. `main.py` gains a
rung-9 RQL panel (φ_p sweep across the bell: rich CO/H₂, the AFT rollover, EI_NO peaking then
collapsing, T_mix → Tt4); `NOTES.md` gains a rung-9 section (both flanks of the bell, why RQL
burns rich); `CLAUDE.md` scope + deferred-seams updated (rich primary / RQL done — with the
*finite-rate quench* honestly carved out as the next seam).

## The rung-10+ seam (keep it additive)
Rung 9 burns a rich primary and freezes NO through an *ideal* quench. Next seams, all still
additive on this substrate: (a) **finite-rate quench** — a secondary-zone Zeldovich in the
cooling, mixing gas so NO can *spike* as the gas dwells at stoich (RQL's real hazard, and the
knob that separates a good quench from a bad one); (b) **super-equilibrium O / prompt (Fenimore)
NO** — a richer radical pool + the prompt path, which matters *most* in the rich primary; (c) the
still-open **equilibrium-vs-frozen nozzle expansion** (rung-6 seam). Only *where* and *on what
pool* the chemistry runs changes.
