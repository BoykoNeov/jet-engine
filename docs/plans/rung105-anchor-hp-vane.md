# Rung 105 anchor — THE HP TURBINE'S VANE IS NOT CHOKED AT A NORMAL REACTION: a flow law beside rungs 31/38's pin on `A4` (pre-registered 2026-10-20 — §§ 2, 3, COST)

**Seam:** rung 104's (`docs/rung104-spec.md` § The next seam, § Concessions: "The HP vane `A4` stays choked — by G1's
bound 1.58–1.60× critical, with margin unless the stage's reaction is large"). Rung 31 pinned the one-shaft turbine's
flow parameter `W4 = ṁ√Tt4/pt4` at its nozzle guide vane's choked value `A4·MFP*(Tt4)`; rung 38 kept it on two shafts.
The pin is where every off-design solver takes the engine's airflow from.

**User decisions (2026-10-20, AskUserQuestion; memory `rung105-decisions`):**
* Picked over the afterburner re-check (rung 104's other seam), because this answer decides whether that re-check is
  done twice.
* **The law: a REACTION knob** — vane and rotor as two convergent rows in series, the design split set by the stage's
  reaction (the vane takes `1 − R` of the isentropic drop); the choke EMERGES at a low enough reaction. Plus the old pin
  as its own setting. **Reaction 0.3** when switched on.
* **Both engines, every off-design page** (one shaft: rungs 31–37, the stage stack, Fly it and its slam; two shafts:
  every steady and transient solver, Controls, Blades, Cooling).
* **DEFAULT OFF** — re-asked once the prototype's cost and effect were measured (G3, G5): the old choked vane stays the
  default everywhere; the law is a setting. Nothing old moves, so no oracle value is retired and no older test is
  sorted. (First answer, superseded: on everywhere.)
* **Rung 33's double root (G4) is recorded, not fixed here** — its own small repair after this rung.

**Status of what was seen before this was written.** Scratch crates `probe` (G1, against the repo at `4ba69ee`) and
`probe2` (P1–P7, against a throwaway worktree of `4ba69ee` carrying a PROTOTYPE whose default was the law), archived in
`docs/plans/scratch-archive/rung105/`. Probed: the Fly-it engine (`π_c` 10, `Tt4` 1500 K at 250 K / 50 kPa / Mach 0.85;
rung 31/33's matcher on the perfect and thermally perfect gases; the Fly-it page on the thermally perfect gas) at its
design flight and 318 K / sea level / Mach 0.30; the two-shaft rig (`π` 3/6, `Tt4` 1500 K, rung 39's map matcher, maps
`flat` and a hand "mixed" pair, three gases) at the same two flights; the Controls engine on map `tilted` at rung 104's
three probe flights (P3). **Never probed: maps `steep`, `flat-eta`, `flow/press`; § 3's flights; any two-shaft variant
but plain; the one-shaft engine on the equilibrium gas below choke.**

**A knob defined wrong, then corrected (P8).** The prototype split the design drop on the ISENTROPIC drop
(`(1 − R)·Δh_s`); the published range and G1 put reaction on the stage's ACTUAL static drop (`1 − R = (c1²/2)/Δh0`,
`Δh0 = η·Δh_s`), so the prototype's 0.3 was a textbook ~0.24 and its vane choked at 0.26–0.27 instead of G1's
0.18–0.20 (advisor-reviewed). Corrected before this was frozen: the capture's choke flip now lands on G1's `R*` to the
fourth digit on both engines and two gases (P8). P1/P3/P7 were re-run on the corrected knob (`p1b`, `p3b`, `p7b`);
G3's numbers are those.

**A number given to the user, then corrected.** P1's first column was the first law that solved, labelled `pin`; at
one-shaft hot Mach 0.3 the pin refuses, so the "+2–4 % `π_c` below choke on one shaft" first reported compared reaction
0.3/0.5 with 0.15, not with the pin. Where the pin runs below choke (thermally perfect gas, design flight, 600 K) the
one-shaft change is +0.07 % (0.3) / +0.28 % (0.5). Corrected to the user before this was written.

---

## 0. GIVENS — measured before anything was written

**G1 — the reaction below which the vane chokes** (`g1.txt`, against the shipped code). The vane is choked iff its
exit kinetic energy reaches the sonic one, `c1²/2 ≥ c*²/2` (`c*` from the gas's own sonic throat at `(Tt4, pt4)`; the
condition is loss-independent — total enthalpy is conserved in the vane and `M = 1` is a static-temperature
condition). With a repeating stage, `c1²/2 = KE4 + (1 − R)·Δh0`, so it chokes iff `R ≤ R* = 1 − (c*²/2 − KE4)/Δh0`.
One stage:
* one shaft (Fly it, `π_t` 2.33–2.40): `R*` 0.16–0.19 with the nozzle choked (`KE4` = 0; up to 0.21 with the burner
  exit at Mach 0.15), 0.04–0.17 below choke;
* two shafts (rig, `π_HPT` 2.30–2.35): `R*` 0.17–0.20 choked; below choke at low power, under rung 104's ellipse,
  `π_HPT` falls to 1.35 and `R* < 0` — the vane is unchoked at ANY reaction.

Published single-stage reaction: an engine patent's high-work gas-generator stage 0.15–0.4, 0.25–0.33 preferred, 0.3 its
example (US 11 566 562); a Cambridge exam compares 30 % and 50 % designs (4A3 2023). **At a normal reaction (0.25–0.5)
the vane is unchoked everywhere these engines run**; only an impulse-like stage chokes it. (Real engines with choked
HP vanes run 3–4 per stage; these are lightly loaded.) Two equal stages: `R*` ≤ −0.6 — unchoked whatever.

**G2 — perfect gas, nozzle choked: the law IS the pin** (P1, P7). With both throats' flows functions of the turbine's
expansion alone and both captured at the design point, the design expansion solves every equation whatever the vane
does, so `W4 = W4_d` and every number is the pin's. P7 (shortcut off): the law's own search against the pin ≤ 1.2e−14
on both engines. With the shortcut (§ 1) the points are the pin's, bit for bit (P1: every choked perfect-gas cell 0).

**G3 — the effect** (`p1b`, `p3b` — the corrected knob; reaction 0.3 against the pin unless stated).
* **Nozzle choked, realistic gases — the consequences were nearly right.** One shaft (thermally perfect): `π_c`
  +0.155 % at 700 K, +0.08 % at 1000 K, 0 at design; thrust ≤ 0.034 %. Two shafts (equilibrium, the larger of the two
  realistic gases): `π_HPC` ≤ +0.22 %, `N_H` ≤ +0.11 %, thrust ≤ 0.10 %, `π_LPC` and `N_L` down — **work moves LP → HP**
  (rung 104's LP law moved it the other way), zero at design; reaction 0.5 about 1.7× as much. Reaction 0.10 (choked at
  design) = the pin at every choked cell.
* **Below choke.** One shaft, where the pin runs (thermally perfect, design flight, 600 K): `π_c` +0.17 % (0.3) /
  +0.30 % (0.5). Two shafts (rig, flat maps, hot Mach 0.3): `π_HPC` +3.3…+4.9 % at 600 K and +0.7…+1.2 % at 700 K,
  `N_H` +2.6…+4.0 % / +0.4…+0.7 %, `N_L` −0.7…−1.3 %, thrust −2.4…−4.4 % at 600 K (cells of ~20 N per kg/s, near idle) —
  reaction 0.10 moves too there (`π_HPC` +1.3…+2.5 % at 600 K: its vane unchokes); on the "mixed" pair at 600 K the
  engine is within ~10 N of idle and the changes jump past +10 % — not used.
* **Rung 104's headline under the HP law** (P3, rung 104's probe cells, Controls engine, map `tilted`): the HP shaft's
  share of the slowdown against the old model (both vanes pinned, pushed past choke) moves 0.52–0.71 → 0.50–0.64 (0.3)
  / 0.49–0.62 (0.5) — **the headline survives; the HP law hands back a few percent of it.**

**G4 — rung 33's double root, under the pin too** (P2, P4). On the Fly-it engine at 318 K / sea level / Mach 0.30,
650–900 K, the one-shaft subsonic residual over `π_t` has TWO sign changes on a 400-point grid — e.g. 700 K at 0.525
(the working point, `M9` 0.59) and 0.846 (the turbine barely expanding, `M9` 0.34) — perfect and thermally perfect
gas, the pin and the law alike. Rung 33's march from both ends then sees one sign and refuses "SUB-IDLE". **A solver
artefact of the old code**, recorded (user: fix later). Under the law the second root moves to 0.92–0.97 and the march
sometimes brackets the working one — so "the law solves where the pin refused" there is the SOLVER, not physics.

**G5 — cost of the prototype** (P5, P6; best of 5/3; the prototype's law had no cost work; timed before P8's knob
correction, which changes no call count):

| request | pin | law 0.3 |
|---|---|---|
| Fly it point (thermally perfect) | 2.5–8.8 ms | 194–414 ms |
| Fly-it slam (three requests) | 143–188 ms | 15.7–21.3 s |
| Controls slam (default; below choke) | 12.5; 17.7 ms | 316; 418 ms |

Census of one Fly-it point (P6): 151 searches for the turbine at a closure trial, 2 628 vane flows, 29 271 interstage
residuals. Under the pin the transient's closure solves no turbine at all; under any law the vane's flow depends on the
turbine's expansion, so every closure trial solves the turbine with the nozzle. The Controls engine is on the perfect
gas, where the law IS the pin choked — yet 25× slower: the prototype's shortcut ran the pin's turbine solves inside
every closure trial.

**G6 — the compiler's census of `A4` readers** (the field made private in a throwaway worktree): 36 reads in 13 source
files outside the two cores (turbine cooling 8, customer bleed 6, shaft offtake 4, rung 104's law 3, rung 34's spool 3,
the stage stack 2, rung 37's combustor 2, rung 62's bleed transient 2, rung 42's bleed 2; rung 40's transient, rung
32's map matcher, rung 43's fuel transient and the afterburner 1 each), ~19 lines inside `matcher.rs` / `two_spool.rs`,
and 13 reads in 10 test files.

**G7 — the old model's own design floor** (P7; `p7b` on the corrected knob — the same orders). At the design flight and `Tt4`, distance from the design run:
perfect gas ≤ 1.1e−14 (pin = law); one shaft thermally perfect / equilibrium, the pin 1.3e−12 / 2.6e−13 (the law ≤
2.2e−14); two shafts thermally perfect / equilibrium, the pin up to 6.7e−11 / 1.2e−10 (the law ≤ 1.9e−10).

---

## 1. THE DEVICE

**The knob.** `HpVane { Pin, Rows { reaction } }` (`crate::hp_vane`), **default `Pin`** (a cargo feature
`hp-law-default` makes `Rows { 0.3 }` the default and changes nothing else — § 2's R1b). Carried by the one-shaft
matcher (`OffDesignMatcher::set_hp_vane`, inherited by rungs 32/34/37 and the stage stack) and the two-shaft engine
(`TwoSpoolEngine::with_hp_vane`, copied into every core; `set_hp_vane` on a built core). On the pages: a setting on
Fly it (and its slam), Controls, Blades and Cooling — "HP turbine vanes: choked (the old model) / reaction law", the
reaction 0.05–0.6 — request keys written only off their defaults.

**The capture** (zero new constants but the reaction): at the design run, `y_d = pt_out/pt4` (`pt5/pt4` on one shaft,
`pt45/pt4` on two), `Tt4_d`, `η_t,d`, `W4_d = A4·MFP*(Tt4_d)`. The interstage pressure `p1_d` from the TEXTBOOK reaction — the vane
isentropic, its exit kinetic energy `(1 − R)` of the stage's actual drop: `h(Tt4) − h_s(p1) = (1 − R)·Δh0`,
`Δh0 = η_t,d·(h(Tt4) − h_s(y_d))` (a repeating stage) — so the vane is choked at design exactly when `R ≤ R*` of G1
(`KE4` = 0; P8). The vane's area from the gas's own convergent flux to `p1_d` (capped at its sonic point); the rotor's
from the flux from `(p1_d, t1_d)` to `y_d`, `t1_d` the temperature after an expansion to `p1_d` at `η_t` (rung 104's
two-row model: the velocity is dumped between the rows, so the rotor reads a state between the vane exit's static and
total one as its inlet total — **the reaction sets the design split exactly; off design the split is the dump model's**).
`R` from 0 (impulse — the rotor still has a finite drop, since the vane takes only `η·Δh_s`) up.

**The law.** `W4(y)`: the interstage pressure where the two rows pass the same flow, then the vane's flow,
`W4 = W4_d·a1·flux(1 → p1)·√(Tt4/Tt4_d)`. **Where a vane choked at design is choked here, the pin's own expression
`A4·MFP*(Tt4)` is returned** — so at a low enough reaction the law reproduces the pin exactly wherever the vane stays
choked. The vane's sonic point is found once per search, not per flux call.

**Where it enters — an effective `A4`.** Every reader's `A4` becomes the EFFECTIVE `A4 = W4(y)/MFP*(Tt4)`, which is
`A4` itself, bit for bit, on the pin and on the shortcut, so the old arithmetic runs unchanged there:
* one shaft, choked: (★) becomes `W4(π_t) = A8·π_n·MFP*(Tt5)·π_t/√τ_t`, searched from the design expansion; the
  airflow then at the root. Below choke (rung 33's (★★)): the vane's supply moves with the trial `π_t`, and the law's
  search STARTS AT THE DESIGN EXPANSION and widens to the first sign change, the landed point checked to be a root
  (rung 104's pattern) — not rung 33's two-ended march, which G4's double root defeats; the pin keeps that march
  literally. So below choke on one shaft the law can solve a point the pin refuses for the SOLVER's reason (G4) —
  every such cell is counted and reported as that, never as physics.
* two shafts: rung 104's joint search with the HP side `W4(π_HPT)·√(Tt45/Tt4)/π_HPT` (it was `A4·MFP*(Tt4)·…`); the
  pair reports the effective `A4` it used; rung 39's corrected-flow lines (†)/(‡) read it.
* the transients: the closure closes the compressor on the vane's flow, which now depends on the turbine; under the
  law each closure trial solves the turbine WITH the nozzle (one shaft: one root in `π_t`, choked or not; two shafts:
  the joint pair at the trial's `pt4` against the flight's `p0`), and the instant's tail reads that turbine. The flight's
  `p0` reaches the two-shaft closures through the shared inlet step; a closure reached under the law without it REFUSES
  in words (no silent pin).
* every variant (bleed, customer air, turbine cooling, the generator, the afterburner, the stage stack, the
  combustor's clocks): wired, or refused in words under the law.

**The shortcut** (from day one — rung 104's lesson 1): where the law provably IS the pin — the hot section calorically
perfect, the turbine(s) at design efficiency, no afterburner lit behind a held throat, and the nozzle choked — the pin's
own solve and `A4` itself. Cheap on the transients: the choke test reads the nozzle at the DESIGN expansion(s) (on the
perfect gas the pin's root is the design expansion), not a solve per trial. Off with `hp_shortcut` / rung 104's
`law_shortcut` (gate R2).

**Pin: the literal old code at every one of G6's sites.**

## 2. GATES (bars frozen here)

* **R1a — default off IS the old model:** the whole gate (`rust\test-all.ps1`) passes with no older test, oracle or
  golden changed — every reader's pin path is the old arithmetic; every page REPLY is today's byte for byte (the pages'
  own bytes change — a new control, and the rebuilt model the page embeds — and the page-currency check gates that).
* **R1b — the gate with the law on (census, reported, not a bar):** `test-all.ps1 -Features hp-law-default`; every
  failure classed (a number moved / a refusal moved / a panic). **No panic** may appear that the default gate does not
  have; refusals reached under the law are listed. Its runtime is measured first on ONE transient-family binary; if the
  whole census would exceed ~3 h, the transient family runs on a named subset and the spec says which.
* **R2 — perfect gas, choked ⇒ the pin:** the law's own search (shortcut off) = the pin to ≤ **1e−12** (`π_t` /
  `π_HPT`, `π_c` / both compressors, airflow, both speeds where two) at every choked cell of § 3's grids, both engines,
  reactions 0.15/0.3/0.5; with the shortcut on, bit for bit.
* **R3 — the design point:** at the design flight and `Tt4`, every reaction lands no further from the design run than
  **3× the pin's own distance (G7), or 1e−12, whichever is larger** — both engines, three gases.
* **R4 — a hand-written perfect-gas solve below choke:** the one-shaft (★★) and the two-shaft pair under reaction 0.3,
  written out in the test with the perfect gas's closed-form row fluxes and an outer bisection, at ≥ 6 unchoked cells
  of each engine: `π_t` / `π_HPT`, `π_LPT`, `π_c` / both compressors to ≤ **1e−10**.
* **R5 — one root:** at every scored cell, the shipped search's residual (outer state held) has exactly one sign change
  on a 101-point grid over its range, and the root is ≥ 0.02 from both ends — both engines' turbine searches and the
  interstage search.
* **R6 — mass, read independently:** the vane's flow recomputed in the test from the RESULT's stations with an
  independent row law (perfect gas: closed form; realistic: the gas's own `try_sonic_throat` and inversions, written
  out) equals the result's station-4 flow to ≤ **1e−10** relative.
* **R7 — the choke emerges:** the reaction at which the capture's vane flips from choked to unchoked equals the closed
  form `1 − (h(Tt4) − h*)/(h(Tt4) − h_s(y_d))` (`h*` the gas's own sonic enthalpy) to ≤ 1e−9, both engines, two gases;
  and below it the law returns `A4·MFP*(Tt4)` bit for bit wherever the vane is choked.
* **R8 — the pages:** the setting reaches each view's engine (a reply differs from the pin's where § 3 says it moves);
  off, every reply is today's byte for byte; the keys round-trip and are written only off their defaults.
* **R10 — the shortcut's edge** (rung 104's lesson 4: its shortcut made its own bug at the unchoke boundary): a 1 K
  `Tt4` sweep through the unchoke boundary of both engines on the perfect gas — the steady matchers (rungs 31/33, 39)
  and the transient closures (rung 34's instant and equilibrium, rung 40's instant), reaction 0.3 — shortcut on against
  shortcut off ≤ **1e−10** at every step, and the two classify the same steps choked.
* **R9 — the red check:** with the effective `A4` forced back to `A4` under the law (scratch edit), R2 still passes and
  R4 fails.

## 3. SCORED (bars frozen here; cells never probed)

**Cells.**
* **One shaft (S1):** the Fly-it engine through the Fly-it page's own solve (rung 34's equilibrium, map `flow`),
  thermally perfect and equilibrium gas; flights sea level +20 K Mach 0.60, 2 km standard Mach 0.50, 5 km standard Mach
  0.70; `Tt4` 900–1400 K step 100.
* **Two shafts choked (S2):** rung 39's engine on the equilibrium gas, maps `steep` and `flow/press`; flights 2 km
  standard Mach 0.80 and 5 km standard Mach 0.70; `Tt4` 800–1400 K step 100.
* **Two shafts below choke (S3):** the Controls engine (plain, perfect gas) on rung 104's scored grid — maps `steep`,
  `flat-eta`, `flow/press`; flights sea level +20 K Mach 0.60, 2 km Mach 0.50, sea level −10 K Mach 0.40; `Tt4` 575–825
  K step 50 — a cell is SCORED where the old model (both vanes pinned, pushed past choke), rung 104's model (HP pin,
  LP ellipse, below choke on) and the law (reaction 0.3, LP ellipse) all solve, the law's nozzle is unchoked
  (`M9 ≤ 0.99`) and its thrust ≥ 50 N per kg/s of design air (no near-idle cells — G3).
* A pre-flight prints each cell's CLASS only (solves / choked / near idle), never a scored quantity. S1's and S2's
  scored cells are the pre-flight's CHOKED class (under the pin and the law alike); an S1/S2 cell that unchokes is
  reported, not scored.
* **The boundary rule** (rung 104's H1b): an S3 cell where the law and rung 104's model differ in `N_H` by less than
  1e−7 relative (at the unchoke boundary, where both vanish) is counted and excluded from H3's first clause and H4.

**H1 — THE PREMISE: at a normal reaction the vane is never choked.** At every S1, S2 and S3 cell, under reaction 0.3,
the solved vane is unchoked (its interstage pressure above the sonic one). Under reaction 0.10 the vane is choked at
design (G1: `R*` ≥ 0.18 there) and at every choked S1/S2 cell with `Tt4` ≥ 900 K.

**H2 — THE CONSEQUENCES WERE NEARLY RIGHT, choked.** At every S1 and S2 cell (nozzle choked), reaction 0.3 against the
pin: one shaft `|Δπ_c|`, `|Δṁ|`, `|ΔN|`, `|ΔF|` ≤ **0.3 %**; two shafts `|Δπ_HPC|`, `|Δπ_LPC|`, `|ΔN_H|`, `|ΔN_L|`,
`|ΔF|` ≤ **0.5 %**. And on two shafts the work moves LP → HP: `Δπ_HPC > 0`, `Δπ_LPC < 0` at every S2 cell.

**H3 — BELOW CHOKE THE HP VANE HANDS BACK PART OF RUNG 104'S HP SLOWDOWN.** At every S3 cell, against rung 104's
model: `N_H` higher, `N_L` lower, `π_HPC` higher. And rung 104's headline survives: against the old model the HP shaft's
share `ΔN_H/ΔN_L` ≥ **0.25** (rung 104's own bar) at every S3 cell, both shafts slower than the old model.

**H4 — ordered in reaction.** At every S3 cell (the boundary rule applied), `ΔN_H` against rung 104's model ordered
`0.10 ≤ 0.3 < 0.5`, with 0.10's ≥ −1e−12; at every S2 cell, `|Δπ_HPC|` against the pin ordered `0.10 ≤ 0.3 < 0.5`,
with 0.10 equal to the pin to ≤ 1e−12 where its vane is choked at the solved point (its joint search may take a
different trial path and land a few ulps away).

**Reported, not scored:** thrust against both references below choke; one shaft below choke (the Fly-it engine at the
S1 flights and 600–850 K where it unchokes); the named risks of each H measured even on a hit (rung 96's lesson); G4's
double root's extent under each law.

## 4. Probe cells (excluded from scoring)

Everything G1–G7 touched: the Fly-it engine at 250 K / 50 kPa / Mach 0.85 and 318 K / sea level / Mach 0.30 (rung 31/33's
matcher, the page's own solve, the slams); the two-shaft rig at the same two flights on maps `flat` and the hand "mixed"
pair, three gases; the Controls engine on map `tilted` at sea level +40 K Mach 0.85, sea level Mach 0.30 and its design
flight; P5's seven requests.

**Recorded after the freeze (2026-10-20, before the pre-flight was read).** The hand "mixed" map pair of P1 (and of
G1, and of rung 104's own G1) IS shape `flow/press` (`blade_speed::shape_maps`, less its stall line): the header's
"never probed: … `flow/press`" is wrong — `flow/press` was probed, on the rig, at 250 K / 50 kPa / Mach 0.85 and 318 K /
sea level / Mach 0.30. S2's and S3's `flow/press` cells are at other flights, so they are still cells never probed.

## 5. After scoring — reported, not scored

* **The slams with the law on:** Fly it's default slam (both throttle modes) and the Controls view's default and
  below-choke slams — peak `Tt4`, end speeds and thrust against the pin.
* **The pages:** a seeded crash map of each view's requests with the law on at random reactions; no panic allowed.
* **Rung 33's double root:** written up as the next seam with G4's cells (the user: fix after this rung).

## COST — frozen 2026-10-20 (the prototype's timings G5 already seen; these are the SHIPPED code's triggers)

On the pages' own entries, best of 5 on a quiet machine, the law (0.3) against the SAME request on the pin:
* **C0 — off costs nothing:** with the default (the pin), every request of P5 ≤ **1.1×** today's `main`.
* **C1 — the two-shaft pages:** Controls (default, below choke), Blades and Cooling requests with the law on ≤ **3×**
  the pin.
* **C2 — Fly it:** a point with the law on ≤ **20×** the pin; a slam ≤ **5 s**.
* These are TRIGGERS, not bars (rung 103's form): a request over its limit is not shipped as it is; the calls are
  COUNTED (closure trials, turbine searches, vane flows, interstage residuals) before anything is changed.
