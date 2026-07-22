# Rung-38 anchor — two-spool matching: the triangular cascade

Two-part anchor, mirroring rungs 31/33. Part A pins the cascade against an INDEPENDENT
closed-form CPG solve (the rigorous, non-tautological gate — no `_sonic_throat`, no
`Compressor`/`Turbine.apply`, no `TwoSpoolMatcher` internals called). Part B is the reacting-
gas running line that carries the triangularity finding.

## Part A — the independent CPG closed-form cascade (gate 2)

Self-consistent CPG dual gas (`R_t = (γ_t−1)/γ_t·cp_t`): `γ_c=1.4, cp_c=1004, R_c=286.9,
γ_t=1.3, cp_t=1239, hPR=42.8e6` — the same recipe rung 31's own gate 2 uses. Design
`π_LPC=3, π_HPC=6, Tt4=1500, M0=0.85`, real losses (`pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88,
eta_b=0.99, pi_b=0.96, eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98`).

For a CPG gas `choked_mfp` is provably **Tt-independent** (a pure `(γ, R)` constant — the
standard fact that a calorically-perfect choked throat's corrected mass flow depends on
composition alone), so `(★-HP)` and `(★-LP)` collapse to pure AREA-RATIO targets:

```
π_HPT / √τ_HPT  =  A4/A45                              (area ratio, Tt4-INDEPENDENT)
π_LPT / √τ_LPT  =  A45/(A8·π_n)                         (area ratio, Tt4-INDEPENDENT)
τ_t(π_t)        =  1 − η_t·(1 − π_t^((γ_t−1)/γ_t))      (CPG isentropic turbine, closed form)
```

A second solver, written entirely in bare Python floats — no `Gas`/`Component` calls, its own
bisection — solves each of these for `π_t`, then the two shaft balances (`ΔT` from `cp`-weighted
energy, `π_c` from `(ΔT_actual/ΔT_ideal)`-style closed forms) in the SAME triangular order the
shipped `TwoSpoolMatcher._cascade` uses, and the burner's one-shot CPG `f` (no `far`-dependence
in `h_t` for CPG, so the fixed point converges in two evaluations).

### Gate 2 result: machine-zero agreement, across a throttle sweep

| `Tt4` | shipped `π_LPC` | reference `π_LPC` | shipped `π_HPC` | reference `π_HPC` | shipped `τ_HPT` | reference `τ_HPT` |
|-------|------------|------------|------------|------------|-------------|-------------|
| 1500 | 3.000000 | 3.000000 | 6.000000 | 6.000000 | 0.837395115 | 0.837395115 |
| 1300 | 2.629005 | 2.629005 | 5.188399 | 5.188399 | 0.837395115 | 0.837395115 |
| 1100 | 2.294990 | 2.294990 | 4.405976 | 4.405976 | 0.837395115 | 0.837395115 |
| 1000 | 2.140956 | 2.140956 | 4.028025 | 4.028025 | 0.837395115 | 0.837395115 |

Agreement to the digits shown at every sampled `Tt4` (asserted in
`tests/test_rung38.py::test_cpg_independent_cascade` at `< 1e-8` relative on both `π`'s and
`< 1e-9` absolute on `τ_HPT, τ_LPT`). **The structural CPG check**:
`τ_HPT` and `τ_LPT` are themselves `Tt4`-INDEPENDENT (verified to `< 1e-9` across the sweep) —
the direct consequence of `choked_mfp` being `Tt`-independent for CPG, exactly parallel to
rung 31 gate 2's `τ_t`-constant result, now doubled across two chained throat-pairs.

**Why this is the load-bearing gate.** Gate 1 (the `lp_disabled` reduce) never enters the
two-spool cascade at all — it is a different code path entirely (delegation to
`OffDesignMatcher`). So gate 2 is the ONLY check tying the two-spool cascade's actual
NUMBERS to an independently-derived reference, exactly as rung 33's gate 4 was the only thing
tying its subsonic-branch numbers down (its own gate 1 similarly returns before the subsonic
dispatch).

## Part B — the reacting-gas running line + the triangularity finding

Design REFERENCE: `Gas.reacting_equilibrium()`, `π_LPC=3, π_HPC=6, Tt4=1500, M0=0.85`, same
real losses as Part A, `nozzle_convergent=True`. Three fixed throats captured from the design
run: `A4 = 7.493e-4, A45 = 1.579e-3, A8 = 2.247e-3` (m², per unit design mass flow).

### Running line — throttle sweep (M0 = 0.85, choked branch)

| `Tt4` | `π_LPC` (OUT) | `π_HPC` (OUT) | `τ_HPT` | `τ_LPT` | `ṁ/ṁ_R` | sp. thrust | TSFC |
|-------|--------|--------|----------|----------|---------|-----------|--------|
| 1500 (design) | 3.0000 | 6.0000 | 0.839235 | 0.924678 | 1.0000 | 729.5 | 3.298e-05 |
| 1400 | 2.8049 | 5.5882 | 0.836716 | 0.923367 | 0.9051 | 609.8 | 3.190e-05 |
| 1300 | 2.6210 | 5.1838 | 0.834012 | 0.921944 | 0.8176 | 503.1 | 3.087e-05 |
| 1200 | 2.4470 | 4.7892 | 0.831074 | 0.920362 | 0.7372 | 408.2 | 2.993e-05 |
| 1100 | 2.2818 | 4.4048 | 0.827808 | 0.918530 | 0.6632 | 324.2 | 2.912e-05 |
| 1000 | 2.1250 | 4.0292 | 0.824052 | 0.916549 | 0.5953 | 250.1 | 2.851e-05 |
|  900 | 1.9767 | 3.6618 | 0.819813 | 0.914539 | 0.5332 | 184.9 | 2.824e-05 |
|  800 | 1.8371 | 3.3044 | 0.815397 | 0.912612 | 0.4767 | 128.2 | 2.865e-05 |
|  700 | 1.7065 | 2.9587 | 0.811113 | 0.910853 | 0.4260 |  79.2 | 3.067e-05 |
|  650 | 1.6444 | 2.7909 | 0.809100 | 0.910055 | 0.4028 |  57.3 | 3.323e-05 |
| 600 | **nozzle unchokes — OUT OF SCOPE** (flagged, not solved; see spec "Scope") |

Both compressor ratios fall together as the engine is throttled back (both spools pump less),
monotonically, exactly the rung-31 running-line direction doubled across two spools. `τ_HPT`
and `τ_LPT` drift only weakly (~3.7% and ~1.6% respectively over the whole choked window) — the
reacting-gas `γ_t(T)`/composition effect rung 31 already identified, now appearing at TWO
throat-pairs instead of one.

### The triangularity finding — measured directly on `_cascade`, at fixed `(Tt2, Tt4, f)`

Perturbing each parameter in isolation, holding the OTHER four component parameters and
`(Tt2, Tt4, f)` fixed (so the outer `f`-loop's cross-talk, the ONE genuine channel connecting
the spools beyond the cascade, cannot confound the reading):

| Parameter perturbed | moves `π_LPC`? | moves `π_HPC`? | role |
|---|---|---|---|
| `η_HPC` (0.88 → 0.55) | **NO** (bit-for-bit) | yes | HP compressor's own pressure-inversion leaf |
| `η_LPC` (0.90 → 0.55) | yes | **NO** (bit-for-bit) | LP compressor's own pressure-inversion leaf |
| `η_HPT` (0.92 → 0.70) | yes | yes | energy-path (shapes shared `Tt45`) |
| `η_LPT` (0.90 → 0.70) | yes | yes | energy-path (shapes shared `Tt25` via `Tt5`) |

**The precise claim**: each compressor's OWN isentropic efficiency is a terminal leaf — it
converts an already energy-fixed `ΔT` into a pressure ratio for its OWN spool only, and
provably cannot reach the other spool's ratio (Step 3 never reads `η_HPC`; Step 4 never reads
`η_LPC` — confirmed by direct code inspection, not just numerically). Every OTHER parameter
(both turbine efficiencies, all three throat areas) sits upstream in the shared temperature
cascade `Tt4 → Tt45 → Tt5 → Tt25 → Tt3` and legitimately reaches both compressor ratios. This
is **narrower** than an initial "the LP spool solves independent of the HP spool" reading (that
framing is WRONG — `η_HPT` demonstrably moves `π_LPC`) and is the corrected, airtight version
of the finding (docs/rung38-spec.md § "The precise claim").

### Reduce check — `lp_disabled=True` is bit-for-bit `OffDesignMatcher`

A `TwoSpoolMatcher` built around a PLAIN single-spool `Engine` (from `build_turbojet`, no `"25"`
or `"5"` stations at all) with `lp_disabled=True` never enters the cascade — `__init__` builds a
bare `OffDesignMatcher` and every `.match()` call is forwarded to it directly. Verified
identical (`==`, not just close) on `π_c, mdot_air, thrust, τ_t` across a throttle sweep
(`Tt4 = 1500, 1300, 900`) against a plain `OffDesignMatcher` on the SAME design — literally the
same returned object's fields, not a converged limit.

## Cross-links

- **Mechanism anchor**: the same choked-throat mass-flow-compatibility fact rung 31 anchors to
  Mattingly Ch. 8 / Cohen-Rogers-Saravanamuttoo (a choked throat passes a pressure-independent
  corrected mass flow), applied to a second, genuinely new throat (the inter-turbine duct, `A45`)
  that does not exist in any single-spool rung. NOT attributed to a specific twin-spool textbook
  matching procedure — those are built around compressor MAPS and therefore iterate; this rung's
  triangular result is a property of the map-free idealization (see spec § "The finding is a
  no-map model artifact").
- **Hardware anchor**: the fixed convergent nozzle is rung 30's; `A4`/`A8` follow rung 31's
  capture method exactly; `A45` (the LP-turbine NGV / inter-turbine-duct throat) is the one
  genuinely new captured area this rung adds.
