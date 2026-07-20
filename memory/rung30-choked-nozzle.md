---
name: rung30-choked-nozzle
description: "SHIPPED rung 30 = the choked convergent nozzle; full expansion NOT earned for a convergent engine, pressure term rescues 87%; prerequisite for the rung-31 off-design build"
metadata: 
  node_type: memory
  type: project
  originSessionId: e128b926-9ffa-4dba-aa77-4b0d67fae88c
  modified: 2026-07-20T20:51:48.795Z
---

SHIPPED **rung 30 = the choked convergent nozzle** (`docs/rung30-spec.md`,
`Nozzle(convergent=True)` + module-level `_sonic_throat` in `components.py`,
`docs/plans/rung30-anchor-choked-nozzle.md`, `tests/test_rung30.py`, main.py panel
`print_choked_nozzle_table`). Chosen from the "off-design / maps" direction — but off-design
(Mattingly analytic route) **structurally needs a choked convergent nozzle**, which the model
lacked, so the ladder was split **two rungs: rung 30 = choked nozzle, rung 31 = off-design
matching built on it** (user picked the two-rung scoping).

**The question (parallels rung 29):** is FULL EXPANSION — assumed since rung 2 — earned? Every
thrust number expanded fully to `p0`, which at design means `M9`=**1.86 SUPERSONIC** — silently
a **converging-diverging** nozzle. A fixed **convergent** nozzle can reach only `M9`=1 and
**chokes**. Bracket the two like rung 29 bracketed the turbine; zero new knobs, no rate.

**The novelty:** a convergent nozzle lets the FLOW decide `p9` (not a told back-pressure) — a
choke test + branch. TPG sonic throat root-finds `h_t(Tt9)−h_t(T*)=½·γ_t(T*)·R_t·T*` (the
velocity↔enthalpy trap again); `p*=pt9·pr_t(T*)/pr_t(Tt9)`; CHOKED ⇔ `p*>p0`.

**Verdict: NOT earned at design** (for a convergent engine — a C-D nozzle approximately earns
it; keep the verdict CONDITIONAL). `pt9/p0`=6.29 (crit ~1.85) ⇒ chokes hard, `p*`=170.85 kPa
(underexpanded 3.4×), `V9` drops 38%, momentum thrust 51%. **The finding: net specific thrust
falls only 6.6% because the PRESSURE TERM rescues 87%** of the momentum deficit (+356 N·s/kg =
48% of the choked total) — the gap between "51% loss" and "6.6% loss" is why high-PR engines fit
C-D/variable nozzles. Sharper framing (advisor): the thrust number is **nozzle-geometry-
dependent** and the model silently picked the C-D geometry.

**Gates (`test_rung30.py`):** (1) reduce — subcritical convergent ⇒ full expansion bit-for-bit,
default `convergent=False` untouched; (2) **non-tautological** — `_sonic_throat` reproduces the
CPG closed-form critical ratio (`p*/pt`=0.5283@γ=1.4) to 1e-9 on a **self-consistent** gas (the
**rounded-R trap** is why it must be self-consistent — the Mattingly constants disagree at 1e-5,
solver is the faithful one); (3) verdict — chokes, `p*/p0`>3, thrust 5–8% down, TSFC up; (4)
cycle untouched (rung-6 exact); (5) direction — underexpanded, pressure term positive.

**Diagnostic beside the cycle** — production nozzle stays ideal ⇒ cycle bit-for-bit rung 6; the
rungs-7–30 invariant holds. Disclaimed: **fixed throat AREA deferred to rung 31** (rung 30 gives
the choke physics, not the `A9` that pins off-design); convergent-only.

**NEXT: rung 31 = off-design / component maps.** Analytic Mattingly *AEDsys* route: choked
turbine NGV + rung-30 choked nozzle both choked ⇒ turbine operating point pinned ⇒ compressor
runs a unique **running line** set by choked hardware, not map curvature (pumping characteristics
without a map). STRUCTURAL rung (operating point becomes an OUTPUT of a matching solve), unlike
the rung-7–30 diagnostics; needs the fixed throat area `A9` + choked flow function
`ṁ√Tt/(A·pt)=const`. Component-map matching (Cohen–Rogers–Saravanamuttoo, real maps) is the
natural rung-32 follow-on. See [[rung29-shifting-turbine]] for the bracket-an-idealization method
this rung reuses. Anchor family: Mattingly Example 7.1 [[rung29-shifting-turbine]] shares the
project's Mattingly lineage.
