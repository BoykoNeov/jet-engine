# Turbojet Cycle Simulator — Rung 1 Spec & Thermodynamics Handout

> Drop this file into the repo root (e.g. as `SPEC.md` or fold it into `CLAUDE.md`) and start the session from it.

## Mission

Build a station-by-station model of an **ideal turbojet** Brayton cycle. The model takes flight + design conditions and produces the gas state at every station, the thrust, and the efficiencies — and draws the cycle on a temperature–entropy (T–s) diagram.

**The real deliverable is my understanding of the thermodynamics, not the tool.** The code is the medium that forces every assumption into the open. Optimize the work for *teaching me*, not for features or polish.

---

## How to work on this project (the contract)

Please follow these while building. They matter more than usual because the goal is learning.

1. **Derive before you code.** For each station, write the governing equation in a comment (or in chat) and say *why* it holds, before implementing it. A one-line physical justification per station.
2. **Show the work.** Print a full station table on every run (Tt, pt, and where relevant T, p, M, V at each station). I want to watch numbers propagate.
3. **Keep components pure.** Each component is a function `state_in -> state_out` with no hidden state. This is what lets us swap/extend later.
4. **Bake in conservation checks as assertions** (see the Conservation section). They should run every time, not as separate tests.
5. **Prefer clarity over cleverness.** Readable, named, commented over compact. No premature abstraction, no optimization.
6. **Stop and explain surprises.** If a number looks off or a result is counterintuitive, pause and reason about the physics with me rather than silently moving on.
7. **Stay in scope.** Build *only* rung 1 (ideal cycle). Do not add component efficiencies, pressure losses, variable cp, or off-design matching yet — those are later rungs. But *do* design the interfaces so they can be added without a rewrite.
8. **Language:** Python (clear, great plotting, matches the pyCycle ecosystem). Use only the standard library plus `matplotlib` for the plot. No heavy frameworks.

---

## Conventions

**Units — SI throughout.** Temperature in K, pressure in Pa, mass flow in kg/s, velocity in m/s, specific energy in J/kg. Pick these once and never mix. (The handout below quotes pressures in kPa for readability; convert internally.)

**Station numbering** (single-spool turbojet):

| Station | Location | Meaning |
|--------:|----------|---------|
| 0 | Freestream | Ambient / flight condition |
| 2 | Compressor face | After the inlet/diffuser (station 1 is folded into the ideal inlet) |
| 3 | Compressor exit | Burner inlet — peak pressure |
| 4 | Burner exit | Turbine inlet — peak temperature (Tt4) |
| 5 | Turbine exit | Nozzle inlet |
| 9 | Nozzle exit | Exhaust |

**Total vs static.** Cycle analysis works in **total (stagnation) quantities** — `Tt`, `pt` — which already fold in the kinetic energy of the flow. Compute the whole cycle 0→9 in totals. Only convert to **static** (`T`, `p`, `V`) at the **nozzle exit (9)**, because thrust needs the actual exhaust *velocity*. Relations:

```
Tt = T (1 + (γ-1)/2 · M²)
pt = p (1 + (γ-1)/2 · M²)^(γ/(γ-1))
```

**Sign of work.** Compressor *adds* energy to the flow (Tt rises); turbine *removes* it (Tt falls). The shaft couples them.

---

## Architecture

The engine is a list of components, each transforming a `FlowState`. Sketch (fill in the bodies):

```python
from dataclasses import dataclass

@dataclass
class FlowState:
    Tt: float       # total temperature, K
    pt: float       # total pressure, Pa
    mdot: float     # mass flow, kg/s
    far: float = 0.0  # fuel-air ratio carried downstream of the burner

@dataclass
class Gas:           # rung-1: cold-air-standard, constant properties
    gamma: float = 1.4
    cp: float = 1004.0     # J/(kg K)
    R: float = 287.0       # J/(kg K)
    hPR: float = 42.8e6    # fuel heating value, J/kg

class Component:
    def apply(self, s: FlowState, gas: Gas) -> FlowState: ...
```

Components to implement: `Inlet`, `Compressor(pi_c)`, `Burner(Tt4)`, `Turbine` (solves the shaft balance against the compressor), `Nozzle(p_ambient)`.

The turbine is special: it doesn't have a free pressure ratio — its work is *set* by the compressor it drives. So the engine builder must hand the turbine a reference to the compressor's work (or the required `ΔTt`). Keep that coupling explicit.

---

## Rung 1 scope — frozen assumptions

Everything ideal, so the numbers are deterministic and hand-checkable:

- **Calorically perfect gas:** constant `γ = 1.4`, `cp = 1004 J/(kg·K)`, `R = 287 J/(kg·K)`, everywhere (including after the burner). This is the "cold-air-standard" approximation.
- **Ideal inlet:** full pressure recovery, `pt2 = pt0`, `Tt2 = Tt0`.
- **Ideal compressor & turbine:** isentropic (efficiency = 1).
- **Ideal burner:** no total-pressure loss (`pt4 = pt3`), complete combustion.
- **Ideal nozzle:** isentropic and **fully expanded** (`p9 = p0`).
- **Mechanical efficiency = 1** on the shaft.
- Fuel mass *is* carried: work balance and thrust use `(1 + f)`.

**Out of scope for rung 1** (design the seams, don't build them): component efficiencies, pressure losses, variable `cp(T)`, off-design / component maps, afterburner, convergent-divergent nozzle CFD.

---

## Station equations (the handout)

Walk these in order. `g ≡ (γ-1)/γ = 0.2857`.

**0 — Freestream totals** from ambient `T0, p0` and flight Mach `M0`:
```
Tt0 = T0 (1 + (γ-1)/2 · M0²)
pt0 = p0 (1 + (γ-1)/2 · M0²)^(1/g)
V0  = M0 · sqrt(γ R T0)
```

**2 — Inlet (ideal):** `Tt2 = Tt0`, `pt2 = pt0`. (Ram compression already lives in the station-0 totals; an ideal inlet just preserves them.)

**3 — Compressor**, given pressure ratio `πc = pt3/pt2`:
```
pt3 = πc · pt2
Tt3 = Tt2 · πc^g            # isentropic
```

**4 — Burner**, given turbine inlet temp `Tt4`:
```
pt4 = pt3                  # ideal: no loss
f   = cp (Tt4 - Tt3) / (hPR - cp Tt4)    # energy balance → fuel-air ratio
```

**5 — Turbine** (the keystone — see next section). Shaft balance with `(1+f)` and mech eff = 1:
```
Tt4 - Tt5 = (Tt3 - Tt2) / (1 + f)
pt5 = pt4 · (Tt5/Tt4)^(1/g)   # isentropic expansion
```

**9 — Nozzle (ideal, fully expanded `p9 = p0`):**
```
Tt9 = Tt5,  pt9 = pt5         # isentropic: totals conserved
M9  = sqrt( ( (pt9/p9)^g - 1 ) / ((γ-1)/2) )
T9  = Tt9 / (1 + (γ-1)/2 · M9²)
V9  = M9 · sqrt(γ R T9)
```

**Performance:**
```
Specific thrust:  F/mdot = (1 + f) V9 - V0        # N·s/kg (pressure term = 0, fully expanded)
TSFC:             f / (F/mdot)                     # kg/(N·s)
Thermal eff:      η_th = 1 - Tt2/Tt3               # ideal Brayton; equals 1 - 1/πc^g
Propulsive eff:   η_p  = (F/mdot · V0) / ( ½[(1+f)V9² - V0²] )
Overall eff:      η_o  = (F/mdot · V0) / (f · hPR)
```

---

## The shaft balance (don't skim this)

This is the one constraint that makes a turbojet a *machine* and not just two unrelated processes. **The turbine extracts exactly the work the compressor needs — no more, no less.** Energy in = energy out across the shaft:

```
mdot_air · cp · (Tt3 - Tt2)  =  (mdot_air + mdot_fuel) · cp · (Tt4 - Tt5)
```

Divide through and that's where `Tt4 - Tt5 = (Tt3 - Tt2)/(1+f)` comes from. This sets Tt5, which sets how much energy is left for the nozzle, which sets V9, which sets thrust. Almost everything cascades from this one line. When implementing the `Turbine`, make this balance the *explicit* thing being solved, so I can see the coupling.

---

## Validation case (encode this as a test)

**Inputs:**

| Quantity | Value |
|---|---|
| Ambient temp `T0` | 250 K |
| Ambient pressure `p0` | 50 kPa |
| Flight Mach `M0` | 0.85 |
| Compressor pressure ratio `πc` | 10 |
| Turbine inlet temp `Tt4` | 1500 K |
| `γ`, `cp`, `R`, `hPR` | 1.4, 1004 J/kg·K, 287 J/kg·K, 42.8 MJ/kg |

**Expected outputs** (match to ~0.1%):

| Quantity | Expected |
|---|---|
| `Tt0` = `Tt2` | 286.1 K |
| `pt0` = `pt2` | 80.19 kPa |
| `Tt3` | 552.4 K |
| `pt3` = `pt4` | 801.9 kPa |
| fuel-air ratio `f` | 0.02304 |
| `Tt5` = `Tt9` | 1239.7 K |
| `pt5` = `pt9` | 411.5 kPa |
| exit Mach `M9` | 2.033 |
| exit static temp `T9` | 678.8 K |
| exit velocity `V9` | 1061.6 m/s |
| flight velocity `V0` | 269.4 m/s |
| specific thrust | 816.6 N·s/kg |
| TSFC | 2.821e-5 kg/(N·s)  ≈ 101.6 kg/(h·kN) |
| thermal efficiency `η_th` | 0.4821 |
| propulsive efficiency `η_p` | 0.4073 |
| overall efficiency `η_o` | 0.2231 |

**Primary hand-check (must match to the digit):** the thermal efficiency computed as `1 - Tt2/Tt3` must equal the closed-form ideal-Brayton result `1 - 1/πc^g`. Both give **0.4821**. If those two disagree, there is a bug in the compression leg — fix it before trusting anything else.

---

## Conservation checks (assertions on every run)

These catch most cycle bugs before any external comparison:

- **Mass:** `mdot_out == mdot_in · (1 + f)` across the burner; `mdot` unchanged across every other component.
- **Energy across the shaft:** compressor power and turbine power must be equal to within a tiny tolerance (that *is* the shaft balance — assert it after solving).
- **Burner energy balance:** `mdot_air·cp·Tt3 + mdot_fuel·hPR == (mdot_air+mdot_fuel)·cp·Tt4`.
- **Isentropic legs:** for compressor, inlet, turbine, nozzle, verify entropy change ≈ 0, i.e. `Tt_out/Tt_in == (pt_out/pt_in)^g` to tolerance. (In rung 1 these are exact; the assert becomes a *real* check once efficiencies arrive in rung 2.)

---

## Rung 1 deliverables checklist

- [ ] `FlowState`, `Gas`, and the five components, each with a one-line physical comment.
- [ ] An `Engine` that chains them and solves the shaft balance.
- [ ] A run that prints the full station table for the validation case.
- [ ] An automated test asserting the expected-outputs table + the primary hand-check.
- [ ] The conservation assertions wired into the components.
- [ ] A **T–s diagram** of the resulting cycle (stations marked 0,2,3,4,5,9; isentropic legs vertical, the two constant-pressure legs as curves). This is the payoff artifact.
- [ ] A short `NOTES.md` where you explain, in plain language, what each station does and what the shaft balance buys us — written for me to learn from.

---

## Future directions (for context only — do not build yet)

Rung 1 stays ideal. Everything below is a **branch**, not a step — the directions are largely independent, so pick by curiosity rather than order. They're cheap to add because of the architecture: a new capability is almost always **a new `Component` type, a new solver mode, or a new view over the same `FlowState` data** — rarely a rewrite. Each item notes how it attaches.

### A. Higher physical fidelity (the main ladder)
Make each process less ideal, one assumption at a time.
- **Real components.** Isentropic → polytropic efficiencies; inlet pressure recovery; burner pressure drop; combustion + mechanical efficiency. The T–s legs start tilting right; first real comparison against EngineSim's dry-turbojet mode. *Attaches as:* extra parameters on existing components.
- **Variable, then reacting gas.** `cp(T)` (thermally perfect) → equilibrium combustion products with high-temperature dissociation → composition tracked through the gas path. *Attaches as:* swap the `Gas` model behind a property interface; components are untouched. Closes most of the gap to pyCycle/NPSS.
- **Secondary flows.** Compressor bleed, turbine cooling air, leakage. *Attaches as:* mass/energy taps between components.

### B. Different engines from the same graph
The node/transform pattern is engine-agnostic — new architectures are new component lists.
- **Turbofan.** Add a fan, a bypass stream, and a splitter; separate or mixed exhaust; sweep bypass ratio. *Attaches as:* a second `FlowState` path plus a mixer.
- **Multi-spool.** Two- or three-shaft layouts (LP/IP/HP), each with its own shaft balance solved simultaneously.
- **Shaft-power engines.** Turboprop / turboshaft with a free power turbine and a propeller/shaft-power output. *Attaches as:* a power-extraction component in place of (or alongside) the thrust nozzle.
- **Afterburning.** A second burner between turbine and nozzle, with the nozzle re-sized.
- **Limit cases.** Ramjet/scramjet (no turbomachinery) at high Mach; recuperated/intercooled cycles for the ground-gas-turbine cousin. Useful for seeing *where the turbojet assumptions break*.

### C. Beyond the design point
Rung 1 is one operating point; real engines run an envelope.
- **Off-design + component matching.** Compressor/turbine maps and a nonlinear solver that finds where mass flow, work, and spool speed all balance. *The big one* — and the reason pyCycle sits on a gradient-based MDAO framework.
- **Performance decks.** Altitude–Mach–throttle sweeps producing carpet plots and thrust/TSFC tables.
- **Transient / dynamic.** Spool inertia, fuel schedules, acceleration, surge/stall margins — steady state becomes an ODE in time. (NASA's T-MATS toolbox is the reference.)
- **Installation effects.** Inlet spillage and nozzle/boat-tail drag → installed vs uninstalled thrust.

### D. Drill into a single component (multi-physics / CFD branch)
Pick one box on the station diagram and resolve its internals; feed *one better loss model* back into the cycle.
- **Quasi-1D nozzle** (the planned first step): choking, supersonic expansion, shocks; validated against isentropic-flow and normal-shock tables.
- **Meanline turbomachinery.** Velocity triangles + the Euler turbine equation, stages stacked; "the compressor" becomes N stages each with its own loss.
- **Deeper still:** 2D cascade / through-flow CFD, combustor chemistry & kinetics (incl. NOx), or blade heat transfer & cooling.

### E. Design & optimization (MDAO)
Shift from "what does this engine do" to "what engine should I build."
- **Parametric optimization.** Find the πc / Tt4 / bypass ratio that maximizes thrust or minimizes TSFC; trace the specific-thrust vs efficiency trade-off.
- **Inverse design.** Solve for the cycle that hits a target thrust *and* TSFC.
- **Trade studies & sensitivity.** Multi-objective Pareto fronts; which assumptions move the answer most.
- **Uncertainty quantification.** Monte-Carlo the component efficiencies to get confidence bands on performance.

### F. Fuels, sustainability & new propulsion
- **Alternative fuels.** Swap `hPR` and combustion products for hydrogen or sustainable aviation fuel; watch the cycle and emissions shift.
- **Emissions.** CO₂, NOx, and (stretch) contrail-relevant outputs.
- **Hybrid-electric.** A battery/motor feeding a spool, or electrified bleed — the "more-electric engine."

### G. Interface, teaching & interoperability
Since the point is *understanding*, the views may matter as much as the physics.
- **Interactive cycle explorer.** Live sliders on the T–s diagram with thrust/TSFC updating — the core learning instrument.
- **Auto-drawn station diagram** generated from the component list, plus **Sankey energy-flow** charts showing where the fuel energy actually goes.
- **"Explain this result" mode** that narrates the governing physics behind any on-screen number; guided exercises auto-graded against the model.
- **Interop.** A regression harness against EngineSim/pyCycle/textbook cases; import-export to couple with pyCycle or NPSS; calibration against published data for a real engine (e.g. CFM56-class).

> Design rung-1 interfaces so all of the above stay **additive** — a new component, mode, or view — never a rewrite of the solver.

---

## Cross-validation references

- **NASA EngineSim** (Tom Benson / NASA Glenn, public domain) — has a dry-turbojet configuration; drive identical inputs and diff outputs. Java and JavaScript/HTML versions exist.
- **pyCycle** (github.com/OpenMDAO/pyCycle) — actively maintained NASA cycle library built on OpenMDAO, same physics lineage as the industry-standard NPSS. Docs are thin; the `example_cycles` folder is where the usable models live. The serious oracle once you reach rungs 2–3.
- **Mattingly, *Elements of Propulsion*** — fully worked ideal/real turbojet examples with intermediate station values; ideal as additional unit tests. (Hill & Peterson and Cumpsty's *Jet Propulsion* are good companions.)
