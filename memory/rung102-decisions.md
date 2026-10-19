---
name: rung102-decisions
description: "User scope decisions 2026-10-19 for rung 102 (the AFTERBURNER, picked from the post-101 list): ONE big rung (design point + fixed engine, nozzle fixed vs opened); duct loss DERIVED (flameholder drag + Rayleigh heat-addition loss from a duct-Mach knob); too-hot ⇒ refuse in words naming the max; page = Design view"
metadata:
  node_type: memory
  type: project
  originSessionId: c72a2043-730f-42b0-8a5e-3b17edf7d8be
  modified: 2026-10-19T05:57:20.657Z
---

User picked option 6 (afterburner) after `/clear`, 2026-10-19.

**Pre-question probe** (`docs/plans/scratch-archive/rung102/design.txt`, `design_eq.txt`; πc 10, Tt4 1500, real losses,
π_AB 0.95, η_AB 0.95, fully expanded, AB beside `build_turbojet` on station 5):
- Tt7 2000 K: thrust ×1.31 / TSFC ×1.57 (M0.2 take-off), ×1.39 / ×1.47 (M0.85), ×1.71 / ×1.42 (M2) — pays most fast.
- A8 opens ~38 %; `sqrt(Tt7/Tt5)/π_AB` short by 3–4 % (gas properties).
- Unlit duct at π_AB 0.95 costs ~1.5 % thrust.
- Equilibrium gas: ceiling ≈ 2250 K (2300 refused, balance open at stoich); 2200 K costs ~3 % more fuel than the
  reacting gas. "Lit, Tt7 = Tt5" is REFUSED on equilibrium — the frozen station-4 mixture recombines and warms itself
  (residual +4–5 J/mol) ⇒ OFF needs its own switch, not Tt7 = Tt5.
- Reacting gas PANICS at stoich (`gas.rs:264` "rich mixture"); equilibrium bracket is lean-only.

**User answers (AskUserQuestion):**
- Scope: **ONE BIG RUNG** — design point AND the fixed engine (nozzle held fixed vs opened). (I had recommended two.)
- Duct loss: **DERIVED** — fixed flameholder loss + heat-addition (Rayleigh) loss from a duct-speed knob.
- Too hot: **REFUSE IN WORDS**, naming the hottest reachable temperature.
- Page: **on the Design view** (switch + knobs; new stations in the table and T–s chart).

**Anchor committed:** `docs/plans/rung102-anchor-afterburner.md` (Parts A+B frozen, 341 cells; Part C addendum before its code).

Advisor traps to carry: main burner's dry-air guard stays; equilibrium gas allows ONE burn condition per Gas
(`gas.rs` EquilibriumSection guard); `try_score` reads f at station "4"; lean bracket; refusals are `Abort`s.
Related: [[sandbox-direction]], [[rung101-decisions]].

**Anchor found (2026-10-19):** Mattingly 2nd ed. (a downloaded copy, not kept; the anchor's G3 holds every number) § 8.4
REFERENCE afterburning turbojet, SLS: πc 20, e 0.9, Tt4 3200 °R, Tt7 3600 °R, π_AB 0.94, η_AB 0.95, cp_AB = cp_t 0.295,
γ 1.3 → book f 0.0358, f_AB 0.0195, f_o 0.0554, τt 0.79680, πt 0.33490, F/ṁ0 137.69 lbf/(lbm/s), S 1.4473, M9 1.85.
Probe (`anchor.txt`): Mattingly's own eqs reproduced by the model to 0.03 %; the book's printed F/ṁ0 sits 0.34 % BELOW
its own eqs (source unknown — P0/P9 moves it < 0.01 %; disclose). Wet/dry 1.194 vs book "about 20 %". § 8.4 states the
A8 control law (turbine exit unchanged) — the "opened" gate. Ex. 7.5 results are figure-only.

**Fixed-engine probe** (`fixed.txt`, Fly-it solver, Tt4 held 1500, A8 held ⇒ effective throat ×k,
k = π_AB·(MFP7/MFP5)·√(Tt5/Tt7)·(1+f5)/(1+f7)): Tt7 2000 → k 0.758, spool −26 %, πc and airflow −49 %, stall margin
0.395 → 0.020 (M2: 0.176 → 0.003). "Opened by 1/k ⇒ core identical" is the TEXTBOOK'S CLAIM + a continuity
argument, NOT measured — the probe's opened case ran `run(kd/kd)` = the dry engine again (advisor caught it). The real
gate must solve the lit engine through the new nozzle relation. Thrust under the held nozzle NOT yet measured.
**Two-spool probe** (`twospool.txt`) ran the STEADY map matcher (rung 39), NOT the Controls march: held, Tt7 1800 →
n_lp −47 %, n_hp +18 %, sm_lp 0.24 → 0.14, sm_hp UP; Tt7 2000 PANICS at `two_spool.rs:936` (LP choke bracket). HP is
NOT untouched (Tt25 falls ⇒ HPC pressure ratio 6 → 8.7) — a naive "HP between two choked throats" prediction misses.

**Second answers (AskUserQuestion):** fixed engine on **Fly it AND the Controls view (two shafts) too**; nozzle =
**a slider from held past the rule, plus a "match" setting**; throttle holds **turbine temperature** (no speed hold).
Slider unit (my default, stated to user): area relative to the dry throat; "match" = follows the rule at every point.

**Third answers:** Controls = **LIT AT A CHOSEN MOMENT** during the slam (light-off is an event in time); rungs 96–101's
thrust guesses (pressure ratio / fan speed / exhaust sensor) with the AB lit ⇒ **REFUSE IN WORDS** (direct thrust,
speed, Tt4 controls work). Staging (advisor): anchor → design + Design view → Fly it → Controls, each green + pushed.

**Progress (2026-10-19):** Part A shipped `7de9419` (261/261). Part B clarification committed before its code
(`f140dd1`: the two scored maps are flow + pressure, tilted reported; dry = the fitted engine UNLIT with throat
`A8_core/π_fh`). Part B built, 80/80 (H7 32, H8 32, H9 16); cost trigger fired first (lit held 6–12×), fixed by
measurement (see [[rung102-afterburner]]); slam refuses ANY fitted afterburner (my scope call, disclosed). Part C next:
probe the Controls march, write the anchor ADDENDUM with predictions, commit, then code.

**Part C (2026-10-19, user):** with the afterburner LIT on the Controls view, the "hold thrust" governor is REFUSED in words
(reason measured in the probe: lit at a fixed Tt7, more main fuel at frozen shaft speeds LOWERS thrust — less air, the
afterburner burns less — so the governor's root-or-clamp pushes the wrong way and bangs between the lean and rich walls).
Hold HP speed and plain Tt4 throttling run. This reverses the anchor's "direct thrust control works" line.
