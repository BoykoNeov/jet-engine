---
name: rung29-shifting-turbine
description: "SHIPPED rung 29 = the shifting turbine; frozen turbine EARNED at design / bites hot, and the RATIO ≠ ENERGY cross-rung correction"
metadata: 
  node_type: memory
  type: project
  originSessionId: f64ea2a7-8077-438f-a7ce-c2a2402c676b
  modified: 2026-07-20T16:39:26.128Z
---

Rung 29 (shipped 2026-07-20) is **the shifting turbine** — `Gas.shifting_turbine(…)` /
`_work_limited_expand(…)` in `turbojet/gas.py`, spec `docs/rung29-spec.md`, anchor
`docs/plans/rung29-anchor-shifting-turbine.md`, tests `tests/test_rung29.py`.

It closes rung 26's deferred seam (c), which rungs 14/25/26 had all named.

**What it is.** The turbine analogue of rung 14's nozzle bracket: frozen vs fully-shifting,
zero knobs, no rate. The structural novelty is a **WORK-limited** endpoint — the shaft fixes
`delta_h` (compressor + `f` only), so both bounds give up the same enthalpy and the
chemistry moves where the flow *ends up*. Two unknowns `(T5,p5)`, two equations
(work-limited `H_abs` drop + reversible `S`), on **absolute** enthalpy.

**Verdict:** freezing the turbine is **EARNED at the design point** (`ΔT5/T5` = 0.011% at
`Tt4`=1500) and **NOT hot** (1.86% at 2400, a 174× growth) — **rate-independently**, since
the bound is the instant-chemistry reversible limit.

**The rung is RATIO ≠ ENERGY** — a cross-rung correction. Rungs 25–28 justify the
super-equilibrium entry with a *ratio* (`x_frozen/x_eq`, `[NO]/[NO]_e`). That is correct for
**kinetic** distance (what rate questions need) but is **not** a proxy for exploitable
**enthalpy**, which scales with the absolute radical **INVENTORY**. The two **anti-correlate**
across the band: ratio ÷33, inventory ×121, shift ×174 — so the ratio is loudest exactly
where the shift is most negligible. See [[rung25-finite-rate-nozzle]], [[rung28-coupled-no-march]].

**Two things deliberately NOT claimed** (both were tempting):
- `(R−I)→0` on a shifted entry is **STRUCTURAL, a tautology**, not a measurement — an entry
  pinned at equilibrium has no super-equilibrium left to relax irreversibly. Not gated.
- The **rate**. The rung-26 clock at turbine conditions gives `Da_turb`=0.05–8.8
  (transitional, and *not* fast despite high `p` — the residence is short too), but that
  rides on an **un-anchored** turbine `τ_res`. Leading with it would have made this a
  [[tau-res-negative]]-style negative rather than a rung. Kept as a supporting sketch only.

**Method note worth reusing:** the advisor's "do the zero-knob reversible bound FIRST" is what
made this cheap — the bound decided the scope (ship the bracket, defer the rate) before any
rate machinery was written, and it overturned the high-pressure intuition that both the
advisor and I started with.

**Reduce gate has two parts** (the pattern to copy): the frozen branch *delegates* to
`Turbine.apply` at `η_t=1` so bit-for-bit holds **by construction** — plus a **second** gate
that the independent work-limited solver reproduces that closed form, **without which the
reduce gate is a tautology**.

Open after 29: a finite-rate turbine march (needs real passage geometry) and feeding the shifted
station 5 into the production cycle (a re-foundation, not a rung). The `π_c` axis
([[rung29-pi-c-margin]]) and the `M0` / flight axis ([[rung29-M0-margin]]) are both CLOSED —
confirmations, not rungs.
