# Rung 105 — THE HP TURBINE'S VANE IS NOT CHOKED AT A NORMAL REACTION: a reaction law beside rungs 31/38's pin on `A4`, a knob that defaults to the old pin

**Code:** `rust/src/hp_vane.rs` — `HpVane` (`Pin`, the default; `Rows { reaction }`), `HpCapacity` (the design capture;
`w4` / `w4_with` / `a4_at` / `a4_with`, `p1_at`), `Vane` (the vane's sonic point, found once per search, and the
interstage search's warm start), `no_flight`, the `census` counters; the cargo feature `hp-law-default`. One shaft:
`matcher.rs` — `OffDesignMatcher::{hp_cap, hp_shortcut, set_hp_vane, hp_is_pin, try_a4_choked, try_a4_free,
try_law_solve_turbine}`, the dispatch in `try_solve_turbine`, the law's own below-choke search `law_subsonic_root`;
`map.rs` (rung 32's airflow); `spool.rs` — rung 34's closure under the law (`a4_flow`, `HpTurb`, `HpAt`, `hp_last`) and
the tail that reads its turbine (`instant_out`); `combustor.rs` (rung 37's plenum wired, its metal-soak closure refuses).
Two shafts: `two_spool.rs` — `TwoSpoolEngine::with_hp_vane`, the core's `hp_cap` / `hp_p0` / `hp_warm` / `hp_last`,
`HpWarm`, `Cascade::a4`, `TwoSpoolMapCore::try_a4_trial`; `turbine_law.rs` — `try_turbine_pair_hp` (the pair and the
effective `A4` it used), `pair_shortcut`, `try_turbine_pair_flow`, `try_hp_from_flow`, `try_joint_hp`'s HP flow as a
function of the HP ratio; `turbine_cooling.rs` — `CooledHpt::a4`, `try_cooled_pair` under the law, `try_cooled_pair_flow`,
`cooled_a4_trial`; rungs 42 / 62 / 90 / 92 and rungs 40 / 43's closures and tails wired. **The pages:** an `hp_vane` /
`reaction` setting on Fly it (its slam with it), Controls, Blades and Cooling (`FlySettings`, `ControlsSettings`,
`BladeSettings`, `CoolingSettings`; written to a reply only off the default), and a select plus a reaction box on each.
**Gates:** `rust/tests/rung105.rs`. **Anchor:** `docs/plans/rung105-anchor-hp-vane.md`, committed as `865c338` before any
rung-105 code. **Scratch:** `docs/plans/scratch-archive/rung105/`.

The seam: rung 104's (its § Concessions and § The next seam). The user's decisions (memory `rung105-decisions`): the law
is a REACTION knob (vane and rotor in series, the split set by the stage's reaction; the old pin its own setting);
reaction 0.3 when on; both engines and every off-design page; and — re-asked once the prototype's cost and effect were
measured (anchor G3, G5) — **the old choked vane stays the DEFAULT**: nothing old moves, no oracle value is retired, no
older test is sorted.

> **THE HEADLINE — the old assumption's premise was wrong, its consequences nearly right.** The HP turbine's vane
> chokes only if the stage's reaction is at most 0.18–0.20 on these engines (single stages of total ratio 2.3–2.4);
> published single stages run 0.25–0.5, so at any usual reaction the vane is never choked, design point included. Yet
> with the nozzle choked the old pin was within 0.15 % on the one-shaft engine and 0.20 % on the two-shaft one (reaction
> 0.3, realistic gases; exactly right on the perfect gas, where the choked nozzle fixes the turbine's expansion whatever
> the vane does). Below choke the vane hands back part of the HP slowdown rung 104 found — the HP shaft keeps a little
> more speed, the HP compressor a little more pressure — and rung 104's headline survives (its HP share 0.52–0.57).

## 1. What it adds

**The equation.** `W4 = ṁ√Tt4/pt4` at the HP turbine's own `y = pt_out/pt4`. Two convergent rows in series — rung
104's two-row model, the gas's own isentropic flux capped at its sonic point, the velocity dumped between them — the
design split set by the TEXTBOOK reaction: the vane isentropic, its exit kinetic energy `(1 − R)` of the stage's actual
drop, `h(Tt4) − h_s(p1_d) = (1 − R)·η_t,d·(h(Tt4) − h_s(y_d))` (a repeating stage). So the vane is choked at design
exactly when `R ≤ R* = 1 − (h(Tt4) − h*)/Δh0` — anchor G1's choke condition (gate R7). Where a vane choked at design is
choked here, the pin's own expression `A4·MFP*(Tt4)` is returned. No new constant but the reaction.

**Where it enters — an effective `A4`.** Every reader's `A4` becomes `W4(y)/MFP*(Tt4)`, which is `A4` itself, bit for
bit, on the pin, on a choked-at-design vane still choked, and on the shortcut: the old arithmetic runs unchanged there.
On one shaft (★) becomes `W4(π_t) = A8·π_n·MFP*(Tt5)·π_t/√τ_t`; below choke (rung 33's (★★)) the vane's supply moves
with the trial `π_t`. On two shafts rung 104's joint search reads `W4(π_HPT)` on its HP side and reports the effective
`A4` it used; rung 39's (†)/(‡) read it as the A4 flow per pinned area. The transients' closures close the compressor on
the vane's flow, which now depends on the turbine (§ 1's cost notes).

**The shortcut** (from day one — rung 104's lesson 1): where the law provably IS the pin — the hot section calorically
perfect, the turbine(s) at design efficiency, no afterburner lit behind a held throat, the nozzle choked — the pin's own
solve and `A4` itself. On the transients the choke is read at the DESIGN expansion(s) (on the perfect gas the pin's root
is the design expansion), never solved per trial. A cooled HP turbine has no shortcut: the coolants' share moves with the
fuel, so its expansion leaves the design value even on the perfect gas, choked. Off with `hp_shortcut` (one shaft) /
rung 104's `law_shortcut` (two shafts) — gates R2, R10.

**Changed after the anchor froze — each disclosed:**

* **The law's own search below choke on one shaft** (anchor § 1, as frozen): from the design expansion outward to the
  first sign change, not rung 33's two-ended march, which G4's double root defeats. So the law solves throttles the pin
  refuses for the SOLVER's reason (§ 3's report).
* **COST — the closures restructured** (anchor § COST; all three triggers counted first). The prototype's closure solved
  "vane flow = nozzle flow" for the turbine at every compressor trial, each residual a vane flow, each vane flow a nested
  interstage search. Shipped: given a trial's compressor flow, the turbine's expansion follows from the NOZZLE alone (one
  shaft) or the LP side alone (`try_hp_from_flow`, two shafts) — searches with no vane in them — and the vane's law is
  read ONCE at the root; at the closure's root the compressor, the vane and the nozzle pass the same gas, so the answer
  is the same (R4's hand solves hold it to 1e−13). The tails and shaft powers, which know the flow, use the same form
  (`try_turbine_pair_flow`, `try_cooled_pair_flow`). Within one instant each search starts where the last trial landed
  (`hp_last`, `Vane`'s interstage share), cleared at every instant, so a point still never depends on what ran before
  it. A trial with no root in the range is answered at the end whose sign is right.
* **R10's reading near zero:** "1e−10" read as relative for quantities of order one, absolute below unit size — a
  shaft's power imbalance at its equilibrium speed is round-off (2e−15 against 8e−15 at rung 40's 805 K).
* **The anchor's "never probed: `flow/press`" was wrong**: the hand "mixed" pair IS `flow/press` (anchor addendum); its
  probed cells were at other flights than S2's and S3's.

## 2. Gates (anchor § 2, bars frozen at `865c338`)

| gate | result |
|---|---|
| R1a the whole gate on the default | FILLED AFTER THE GATE |
| R1b the gate built with `hp-law-default` (census) | FILLED AFTER THE CENSUS |
| R2 perfect gas, choked: the law's own search = the pin | **1.09e−14** at 138 cells × laws (bar 1e−12); shortcut on, bit for bit |
| R3 the design point | every reaction within 3× the pin's own distance on both engines, three gases |
| R4 a hand-written perfect-gas solve below choke | **1.38e−13** at 16 one-shaft and 12 two-shaft unchoked cells (bar 1e−10) |
| R5 one root | 79 turbine searches and 79 interstage searches, one sign change each on 101 points |
| R6 mass, read independently | **4.8e−11** at 61 points (bar 1e−10; the equilibrium gas's own inversions) |
| R7 the choke emerges | the capture flips at the closed form's `R*` (≤ 1e−9): one shaft 0.1854 / 0.1843, two shafts 0.1978 / 0.1926 (perfect / thermally perfect); below it, the pin's expression bit for bit |
| R8 the pages | the setting reaches Fly it, Controls, Blades and Cooling; round-trips; absent from every default reply; refused in words out of range and with Fly it's afterburner |
| R9 the red check | FILLED |
| R10 the shortcut's edge | 125 one-kelvin steps through the unchoke boundary (rungs 31/33, 34's instant and equilibrium, 39, 40's instant): shortcut on against off **8.3e−15**, every step classified alike |

## 3. Scored (anchor § 3) — every claim HIT

Pre-flight (class only, `preflight.txt`): S1 36 cells, S2 28, S3 33 of 54 — all choked where expected.

| claim | cells | HIT | measured |
|---|---|---|---|
| H1 at 0.3 the vane is never choked; at 0.10 it is | 36 + 28 + 33 | all | FILL: closest margin |
| H2 choked, the pin nearly right (0.3 %, 0.5 %) | 36 + 28 | all | largest change 0.150 % (one shaft), 0.198 % (two shafts); work moves LP → HP at all 28 |
| H3 below choke, the HP vane hands back part of rung 104's HP slowdown | 33 | 33 | none at the boundary; rung 104's share against the old model 0.516–0.569 (bar 0.25) |
| H4 ordered in reaction | 33 + 28 | all | 0.10 = the pin to ≤ 1e−12 where its vane is choked |

**Reported, not scored:** FILLED AFTER THE REPORT PROBE.

## 4. The pages

FILL.

## Concessions

* **The reaction is a design split; off design the split is the dump model's** (rung 104's two rows: the rotor reads a
  state between the vane exit's static and total one as its inlet total).
* **One stage.** With two equal HP stages the first vane's drop halves and `R*` falls below −0.6: the vane is unchoked
  whatever (anchor G1).
* **Fly it's afterburner and rung 37's metal-soak closure refuse the law in words** — each needs the turbine solved with
  a nozzle it does not have in its closure.
* **COST deviations** (§ COST below): with the law on, three two-shaft requests stay over their 3× trigger.

## COST (anchor § COST — triggers, counted before anything changed)

FILL.

## The next seam

* **Rung 33's double root** (anchor G4): the one-shaft subsonic residual has two roots at 318 K / sea level / Mach 0.30,
  650–900 K, the pin too; the two-ended march refuses "SUB-IDLE" there. Its own repair, after this rung (the user).
* The afterburner's held light-off re-scored on its own grid under rung 104's LP law (rung 104's other seam) — once,
  since the HP law is off by default.
