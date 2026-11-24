# A9 — the cutaway as a live sandbox view (plan, 2026-11-24)

Roadmap A9 (`docs/plans/roadmap.md`). The user's priority of 2026-11-20: "make stronger connections between
the computation and what the cutaway shows and make that connection permanent". The user's choice
(2026-11-24, "go with your recommendation"): rebuild the cutaway as a **live sandbox view** that follows the
knobs, rather than a separate page with computed constants.

## 1. What changes for the reader

A seventh sandbox view, **Engine**, draws the animated cutaway of the engine the *Design* view's knobs
describe. Change a knob in Design, open Engine: the flame, the stage count, the shaft's spin, the parcel
speeds and every station readout are the new design's. Each number on the view is either **computed** (read
from the model's reply) or listed under **"Drawn, not computed"** on the view itself.

## 2. What is computed, and from where (each verified in the code before relying on it)

| Drawn today | Becomes | Model source |
|---|---|---|
| Station Tt, pt, f, V0, V9, M9, T9, performance | the Design run's own reply | `sandbox::run` → `outcome_json` (already on the page) |
| Primary-zone flame `T_PRIMARY = 2150` K | the zoned burner's front-zone temperature at the Burner view's richness (default φ = 1) | `nox::zoned_nox` via `sandbox_burner` (its `t_primary`) — rungs 8–9, run on the equilibrium gas as the Burner view does |
| 8 compressor stages | the integer stage count `K` | rung 85 `blade_speed::size` on the single-spool compressor: face `Tt2`, work `h(Tt3)−h(Tt2)`, `γ, cp` at the face, map slope `l` of the Fly view's default map; blade knobs at rung 85's defaults |
| Shaft spin `.45` rev/s | the design shaft speed, shown in rpm; the animation runs it **slowed by a stated factor** | tip speed `U_tip` from the same sizing; tip radius from continuity at the face, `A = ṁ·√Tt2 / (pt2·MFP(M_abs))`, `r_tip = √(A/(π(1−h²)))`, `N = U_tip/r_tip` — **new arithmetic, disclosed** (rung 85 never names a radius) |
| Parcel speeds 150/60/95/260 px/s | physical speeds where the model has one: `V0` at the inlet, the front row's axial `Vx = Φ_d·U_m` at the compressor face, `V9` at the exit; one px-per-(m/s) display scale | sizing (`Φ_d`, `U_m`) and the run |
| Jet-pipe → throat `Tt5·0.87` | the throat's static temperature when the nozzle chokes (the model's sonic throat), else the exit's `T9` | `components::sonic_throat` on station 5's `Tt, pt, f` |

**Stays drawn, and is listed on the view as drawn:** all geometry and proportions; the turbine stage count (2;
no rung sizes a turbine stage); parcel speeds between the computed points (interpolated); every colour between
stations (interpolated); the animation's slow-down factor.

**Out of scope for this view (said on it, in plain words):** a turbofan (bypass ratio > 0) or a lit afterburner —
the cutaway draws a plain turbojet, and with either switched on the view says so instead of drawing a wrong
engine. Off-design (Fly) and the slam are not drawn either: the view draws the design point.

## 3. The model side

A new request `cutaway` (`rust/src/sandbox_cutaway.rs`, one module per view like the others): Design
settings + the Burner view's richness in, out the six computed items of § 2 (flame T, `K`, `U_m`, `U_tip`,
`Vx`, `r_tip`, rpm, throat static T, choked or not) plus a refusal in plain words for a fan, an afterburner, a
sizing error (`SizingError` — e.g. the front row chokes) or a burner refusal. Self-contained (re-runs the
design, as the Burner view does); no model code edited; nothing moves a cycle number.

## 4. The ideal / real toggle

The old page compared the default design with the ideal cycle. On the live view: **"Ideal"** = the same
design with every efficiency and pressure-recovery knob at 1, on the same gas — one extra `run`. (Rung 1's
ideal cycle is also a frozen perfect gas; the view says which it is.) The T–s diagram and the performance
table are already on the Design view, so the Engine view keeps the animated engine, the six station readouts
and the toggle, and points to Design for the diagram.

## 5. The old standalone page

`docs/visuals/turbojet-cutaway.html` (published at https://claude.ai/artifact/KbCcwpVMiJWJm1sN7kYdZm) and its
template are **retired** once the view ships: one drawing, not two that drift. `visuals.rs` keeps the charts
page; its cutaway splice, payload trim and `tests/visuals.rs`'s cutaway gates go, with the stale Python names
(`extract_data.py`, `test_visuals_data.py`, `main.py`). The published link keeps showing the old page until
the user decides what it should show (republish a pointer, or the sandbox page itself) — asked, not done.

## 6. "Permanent" — the gates

1. **Native** (`rust/tests/sandbox_cutaway.rs`): on a grid of designs, every number the `cutaway` reply
   carries equals the model called directly (`zoned_nox`, `size`, `sonic_throat`, the continuity radius by its
   own written-out formula), and the refusals fire where they should.
2. **The browser check** (`browser.mjs`): open Engine on three different designs (default; a high pressure
   ratio; a hot `Tt4`) and read back what the page DREW — the canvas exposes `data-` attributes for the flame
   temperature, stage count, rpm and the three anchor speeds, and the readout text. Each must equal that
   design's reply, and the three designs must differ: a constant that creeps back cannot follow all three.
3. **The template** (`tests/sandbox_page.rs`): the Engine view's script holds no `T_PRIMARY`, `0.87` or
   hard-coded stage loop; the "Drawn, not computed" list exists and names every drawn item of § 2.
4. A screenshot look at two designs (the slice-1 lesson: pictures catch what green checks miss).

## 7. Order of work

1. `sandbox_cutaway.rs` + `call` dispatch + native tests (red first: the reply before the module exists).
2. The view in `template.html`: port the cutaway's canvas code, feed it the reply, the "Drawn" list, the
   explain paragraph, the out-of-scope message.
3. `browser.mjs` checks; `build.ps1`; the gate; screenshots.
4. Retire the standalone cutaway (§ 5); docs (sandbox README, CLAUDE.md layout line, roadmap A9 DONE,
   memory); commit; ask the user about the old link.
