---
name: tau-res-negative
description: "The resolved-τ_res attack on rung 26's open seam (a) is a NEGATIVE on both counts; investigated, documented, NOT shipped and NOT a rung"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2e2c6196-a44c-4375-95f4-2d29d10ec11b
  modified: 2026-07-20T09:39:08.218Z
---

Rung 26's open item (a) — "a resolved `τ_res` from the nozzle area-schedule (retire `L`, pin the
location)" — was attacked on 2026-07-20 and returned a **negative on both counts**. Recorded in
`docs/tau-res-negative.md` (the [[mixing-scale-negative]] precedent: NOT shipped, NOT a rung, no
spec/code/tests/rung-table entry).

The conical `dt=dx/V` derivation does give a normalized shape `ĝ(s)∝|da/ds|/(√a·V)` in which `ṁ`
and `tanθ` genuinely cancel — but it is **moot**: the rung-26 march starts from *stagnation*, so
`ĝ∝s^{-7/4}` and the normalization does not converge without an entry Mach `M_e`. Deriving the
distribution therefore **ADDS** a geometric knob instead of retiring `L`, and the result is *more*
sensitive to `M_e` than rung 26 was to `L`.

Positive by-product: **rung 26 is CONFIRMED** (motion exists and rises with `Tt4`; lean still
frozen-from-entry), with its *already-disclaimed* magnitude refined ~3× smaller.

**Why:** so this is not re-derived from scratch, and so the confirmation of [[rung26-freeze-out]] is
not lost with the negative.

**How to apply:** do NOT re-run the reshape-on-the-`ln p`-frame construction. A new attempt is only
worthwhile with a **real `A(x)` geometry** (physical entry plane + throat area — which wants `ṁ`
threaded into the diagnostic and the still-deferred *choked* nozzle seam). Probes are in
`M:\claud_projects\temp\rung29-tau-res\` (outside git); the tracked doc is self-contained.
