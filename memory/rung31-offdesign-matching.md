---
name: rung31-offdesign-matching
description: SHIPPED rung 31 = off-design matching; the FIRST STRUCTURAL rung (pi_c becomes an OUTPUT); tau_t DRIFTS on the real gas (CPG holds it constant); reduce-by-construction
metadata: 
  node_type: memory
  type: project
  originSessionId: 8be297fe-f263-410a-9736-b60ac12c28a8
  modified: 2026-07-21T03:39:38.518Z
---

SHIPPED rung 31 (`docs/rung31-spec.md`, `turbojet/engine.py OffDesignMatcher`,
`turbojet/components.py choked_mfp`, `docs/plans/rung31-anchor-offdesign.md`,
`tests/test_rung31.py`). The **first STRUCTURAL rung** — every rung 1–30 specified `π_c`+`Tt4`;
rung 31 makes `π_c` the **OUTPUT of a matching solve** against fixed hardware.

**Scope decision (user chose, advisor recommended): approach B — TPG matching on the project's
own machinery**, NOT CPG analytic referencing. B reduces to the design point bit-for-bit BY
CONSTRUCTION (the rung-29 delegation move); A (Mattingly closed-form CPG) could only reduce
against a separate CPG reference — the near-tautological gate the project distrusts.

**Physics**: turbine NGV + rung-30 convergent nozzle both CHOKED ⇒ two mass-flow-parameter
constraints (`ṁ√Tt/(A·pt)=MFP*`, computed EXACTLY from rung-30's `_sonic_throat`, `pt`-independent)
pin the turbine: `π_t/√τ_t = A4·MFP4/(A8·π_n·MFP9)`. **The inversion**: at design the shaft SETS
turbine work (`π_c` in); off-design `τ_t` is pinned by the choke and the shaft hands back the
COMPRESSOR (`τ_c-1 ∝ Tt4/(τ_r·T0)`, `π_c=[1+η_c(τ_c-1)]^(γc/(γc-1))` out). Zero new knobs, no rate.

**Verdict**: the choked hardware STRIPS the compressor of freedom — `π_c`,`ṁ`,thrust ride ONE
fixed **running line** (pumping characteristics WITHOUT a compressor map; `π_c` 10→4.0 as `Tt4`
1500→800). **The finding: `τ_t` DRIFTS** — textbook "choked turbine ⇒ τ_t exactly constant" is a
CPG statement; on the real gas the two sonic `MFP*` sit at different T so `τ_t` drifts ~2.8%/2:1-
throttle (0.16% on M0 axis); CPG holds it to **machine zero**. KILL-TESTED by a 3-gas ladder: CPG
0.000% → thermally_perfect (var cp(T), FROZEN comp) +2.30% → reacting +2.84%, so the **γ_t(T) CURVE
drives 81%** (composition minority ~19%) — clean because within a point both throats share the frozen
comp so R cancels in MFP4/MFP9. Same species as rung 30's "0.03% is the physics, not error". Choked
**envelope**: nozzle UNCHOKES near `Tt4`≈600
(`pt9/p0`<~1.85) — matcher FLAGS `nozzle_choked=False`, doesn't lie.

**Gates**: (1) reduce-to-design (`π_c`=10 to 5e-10, all stations/`ṁ`/thrust — design ref IS the
rung-30 choked-convergent point, specific thrust 745.7); (2) NON-tautological: CPG gas ⇒ Mattingly
Ch-8 closed-form referencing to machine precision (`τ_t`/`π_t` const, slaving factor const, `π_c`
1e-14); (3) cycle untouched (default run bit-for-bit rung 6, separate entry point); (4/6) running
line monotone + unchoke flagged; (5) `τ_t` drift finding (reacting≠const, CPG==const).

**Concessions**: `η_c/η_t` held at design (map curvature = rung 32, Cohen–Rogers–Saravanamuttoo);
NGV choke ASSUMED (not an NGV passage); isentropic knobs only; M0>1 folds in `ram_recovery`.
Equilibrium gas caches its frozen station-4 mixture pinned to ONE (Tt4,pt4), so the matcher
re-freezes a FRESH `reacting_equilibrium()` gas per trial condition (`_working_gas`); test is slow
(~100s) for this reason.

Refactor landed with it: performance scoring extracted from `Engine.run` into module-level
`_score(...)` so the off-design path scores identically.

**Next = rung 32**: component-map matching (real η/π maps, earns the η curvature rung 31 holds
constant). Also open: the subsonic-nozzle matching mode past unchoke; transient/spool dynamics.
Related: [[rung30-choked-nozzle]] (the fixed hardware this stands on), [[rung29-shifting-turbine]]
(the reduce-by-construction precedent).
