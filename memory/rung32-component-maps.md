---
name: rung32-component-maps
description: "SHIPPED rung 32 = component-map matching; corrects rung 31's \"choked hardware IS the map\" over-claim (work map-free, but pi_c/mdot/N need the map); turbine pinned in corrected speed; representative-map closure"
metadata: 
  node_type: memory
  type: project
  originSessionId: 1b97ce92-593b-49f6-93de-9fc76303614e
  modified: 2026-07-21T07:04:50.993Z
---

SHIPPED rung 32 (`docs/rung32-spec.md`, `turbojet/engine.py` `MapMatcher` + `ComponentMap` +
`MapOffDesignResult`, `docs/plans/rung32-anchor-component-maps.md`, `tests/test_rung32.py`, a
`main.py` panel `print_component_map_table`). The **second STRUCTURAL rung** (subclasses rung 31's
`OffDesignMatcher`); the natural rung-31 follow-on.

**Scope decision (user chose, advisor recommended): representative analytic maps** — the rungs
12–24 closure methodology (parametric shapes DISCLOSED, load-bearing claims shape-robust across
3 shapes, magnitudes DISCLAIMED). Not transcribed hardware maps (no data-file dependency), not a
compressor-only droop (that would reduce the chosen scope). See [[rung31-offdesign-matching]].

**The finding is a CROSS-RUNG CORRECTION** (the rung-29 RATIO≠ENERGY / rung-28 β-repair move):
rung 31's "pumping characteristic WITHOUT a compressor map — the choked hardware IS the map"
**over-claimed by holding η at design**. Decomposed:
- The choke sets the compressor **WORK schedule `τ_c(Tt4)` MAP-FREE** — `τ_c` matches rung 31 to
  ~1e-4/1e-7 (this part of rung 31 SURVIVES). The genuine invariance.
- Converting the work into `π_c`, `ṁ` and a **shaft speed `N`** needs the real map. A peaked
  (peak-η-near-design) compressor map droops `η_c` off-design (throttle walks off the efficiency
  island) ⇒ `π_c`/`ṁ` fall **BELOW** rung 31's constant-η line, **SAME SIGN across 3 shapes**,
  gap growing with throttle (~−2% at `Tt4`=900; magnitude shape-dependent, DISCLAIMED).

**Structural novelty: `N` enters.** A compressor map is indexed by corrected speed; rung 31 traced
the whole running line WITHOUT `N`. Rung 32 derives `N` by inverting the **speed-line family**
`(τ_c−1)/(τ_c−1)_d = ψ(m/n)·n²` (Euler work `Δh_c=ψU²` + loading law `ψ(φ)=1−σ(φ−1)²`) at the
choke-pinned point. σ=0 collapses to the tautological `√(τ_c−1)`; σ≠0 makes `N` genuinely
map-dependent (spread ~few % over σ∈[0,1] — gated as bounded-but-nonzero). Absolute rpm DISCLAIMED
(needs blade geometry) — only `N/N_d` and corrected ratios.

**Sub-finding (SHARPER than "turbine maps are flat"): the turbine barely moves for a STRUCTURAL
reason.** On a single spool `nu_t=N/√Tt4` stays within ~1% of design (N and √Tt4 fall together),
so the turbine sits at a near-fixed map point ⇒ `|Δη_t|`~2e-5 **even for a 25×-steeper turbine
map**, vs `|Δη_c|`~1e-2. So rung 31's "hold η_t const" was nearly exact *because the turbine is
pinned in corrected speed*, not because turbine maps happen to be flat. **The compressor is where
the map bites.**

**Solver**: `η_c` feedback is POSITIVE (lower η_c→lower π_c→lower φ,n→lower η_c), so plain
substitution oscillates ⇒ a **SECANT** on `η_c` (`η_t` substituted alongside, nearly constant);
non-convergence assert guards the deep-throttle edge. Refactor: rung 31's `_solve_turbine` gained
an optional `eta_t=` (default = design ⇒ rung 31 untouched) so the map passes a per-trial η_t.

**Gates**: (1) reduce — FLAT map ⇒ `MapMatcher`==`OffDesignMatcher` **bit-for-bit** (machine-zero
at design, ≤1e-9 sweep, on REACTING gas; two code paths one operating point = non-tautological);
(2) cycle untouched (default run bit-for-bit rung 6); (3) π_c/mdot droop shape-robust; (4) τ_c
map-free ~1e-4; (5) turbine pinned in corrected speed; (6) N attaches/monotone/schedule-robust;
(7) direction + secant convergence. Finding gates run on fast `thermally_perfect` (gas-independent
physics); reduce/untouched on reacting.

**Disclaimed**: droop MAGNITUDE + absolute `N(Tt4)` (ride on the map shape); **no surge line ⇒ NO
surge-margin claim** (the CRS payoff deliberately not made — needs a real map's surge boundary);
`η_b/π_b/π_n` held at design; isentropic + choked-nozzle-branch only (inherited from rung 31).

**Next / open**: a real hardware/CFD map with a surge line (would earn the surge-margin claim); the
subsonic-nozzle matching branch past unchoke (rung 31's envelope boundary); feeding the matched
point into transient/spool-dynamics (`N` now exists). Related: [[rung31-offdesign-matching]]
(the choke-pinned line this map labels), [[rung30-choked-nozzle]] (the fixed hardware),
[[rung29-shifting-turbine]] (the reduce-by-construction / cross-rung-correction precedent).
