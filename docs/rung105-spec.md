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
| R1a the whole gate on the default | **GATE GREEN** — 235 programs, 2 956 tests (re-run 2026-10-21 on the tree with R1b's two fixes: the first green run predated the committed `CLAUDE.md` row, which then put the file over its size guard; the budget is raised by this rung's row), every Python-held oracle and golden unchanged (the browser check included); the page's own bytes changed (the new control, the rebuilt model), every default reply did not |
| R1b the gate built with `hp-law-default` (census) | reported, not a bar: 235 programs, 2 757 passed, 199 failed, each classed (`r1b.txt`). **140** a number moved (Python oracles, goldens, hand values held on the pin); **10** a refusal moved (Controls' 1000 K start and 530 K end with customer bleed, Altitude's 600 K start, one slam no longer stopping where it did, rung 74's joint start, rung 104's nozzle back-pressure, rung 102's 2600 K and one fuel-closure stop); **27** the law refusing on a path not wired for it — 12 in its own words (direct inner calls, rung 37's soak), **15 swallowed by a trial-skipping search and re-worded "does not bracket"** (rungs 43–64's fuel rigs, rung 102's one-shaft afterburner match; measured, `noflight.txt`); **13** setup checks asserting the pin's path or hook; **1** the size guard (on the default too; fixed). **The no-panic bar MISSED:** rung 78's reader took the first point of an empty set (7 failures, 4 programs; 6 in `gauge_march`, 1 in rung 78's own test) — its stator's "riding" label is decided at round-off there (`v_cmd` −2e−15 on the pin, 0 under the law's 1e−15 move), so the law's 1e−15 move empties the set. Both sites now stop with words (still a panic, like the line above them, but no longer an index crash); the pin path unchanged |
| R2 perfect gas, choked: the law's own search = the pin | **1.09e−14** at 138 cells × laws (bar 1e−12); shortcut on, bit for bit |
| R3 the design point | every reaction within 3× the pin's own distance on both engines, three gases |
| R4 a hand-written perfect-gas solve below choke | **1.38e−13** at 16 one-shaft and 12 two-shaft unchoked cells (bar 1e−10) |
| R5 one root | 79 turbine searches and 79 interstage searches, one sign change each on 101 points |
| R6 mass, read independently | **4.8e−11** at 61 points (bar 1e−10; the equilibrium gas's own inversions) |
| R7 the choke emerges | the capture flips at the closed form's `R*` (≤ 1e−9): one shaft 0.1854 / 0.1843, two shafts 0.1978 / 0.1926 (perfect / thermally perfect); below it, the pin's expression bit for bit |
| R8 the pages | the setting reaches Fly it, Controls, Blades and Cooling; round-trips; absent from every default reply; refused in words out of range and with Fly it's afterburner |
| R9 the red check (the effective `A4` forced to `A4` under the law, scratch) | R2 still passes (a self-comparison, as predicted); **R4 fails at once** (one shaft 650 K: `π_t` 0.6217 shipped against the hand's 0.5719) and R6 fails |
| R10 the shortcut's edge | 125 one-kelvin steps through the unchoke boundary (rungs 31/33, 34's instant and equilibrium, 39, 40's instant): shortcut on against off **8.3e−15**, every step classified alike |

## 3. Scored (anchor § 3) — every claim HIT

Pre-flight (class only, `preflight.txt`): S1 36 cells, S2 28, S3 33 of 54 — all choked where expected.

| claim | cells | HIT | measured |
|---|---|---|---|
| H1 at 0.3 the vane is never choked; at 0.10 it is | 36 + 28 + 33 | all | closest: the vane's exit pressure 1.096× its sonic one (S1) |
| H2 choked, the pin nearly right (0.3 %, 0.5 %) | 36 + 28 | all | largest change 0.150 % (one shaft), 0.198 % (two shafts); work moves LP → HP at all 28 |
| H3 below choke, the HP vane hands back part of rung 104's HP slowdown | 33 | 33 | none at the boundary; rung 104's share against the old model 0.516–0.569 (bar 0.25) |
| H4 ordered in reaction | 33 + 28 | all | 0.10 = the pin to ≤ 1e−12 where its vane is choked |

**Reported, not scored** (`rep.txt`):
* **Thrust below choke (S3):** against the old model −5.3 … +1.6 %; against rung 104's model −0.29 … 0 % (the HP vane
  takes a little thrust back from rung 104's picture).
* **One shaft below choke (Fly it's own solve, thermally perfect, S1's flights, 600–850 K):** `π_c` +0.13 … +1.8 %,
  shaft speed +0.05 … +1.0 %, thrust −3.0 … +0.07 % (largest at 600 K, sea level +20 K Mach 0.6) — larger than rung
  31/33's matcher showed at the probe's cells (anchor G3: +0.17 % at 600 K), and fading to ~0.15 % by 800 K.
* **G4's double root:** at 318 K / sea level / Mach 0.30 rung 31/33's matcher refuses every throttle 600–950 K on the pin
  (the two-ended march); reaction 0.3 and 0.5 solve all of them, 0.10 from 650 K — the law's design-start search, not
  physics: the pin has the same two roots (anchor G4).
* **The slams (§ 5):** Fly it's default slam — peak `Tt4` unchanged (temperature control), 1756.40 → 1756.03 K (fuel),
  end speed within 1e−5; Controls' default slam unchanged (the shortcut); its below-choke slam peak `Tt4` 1857.10 →
  1856.92 K, speeds within 0.025 %.
* **The pages' crash map** (seed 105, 320 requests across the four views, the law on at random reactions 0.05–0.6,
  each against the same request on the pin): **no new panic**, none of the old model's own reached, 1 refused in words.

## 4. The pages

An **HP turbine's inlet vanes** select on Fly it (and so its slam), Controls, Size the blades and Cool the turbine (the
fixed engine only, beside rung 104's select): "choked, as rungs 31–104 assumed (the default)" or "vane and rotor in series,
split by the stage's reaction", with a reaction box (0.05–0.6, opening at 0.3). Keys `hp_vane` / `reaction`, written only
off the default (sandbox plan § 41). On the pages' perfect-gas engines with the nozzle choked the law gives the old run
bit for bit; Fly it's thermally perfect default moves about a tenth of a percent; a cooled engine moves a little even
choked.

## Concessions

* **The reaction is a design split; off design the split is the dump model's** (rung 104's two rows: the rotor reads a
  state between the vane exit's static and total one as its inlet total).
* **One stage.** With two equal HP stages the first vane's drop halves and `R*` falls below −0.6: the vane is unchoked
  whatever (anchor G1).
* **Fly it's afterburner and rung 37's metal-soak closure refuse the law in words** — each needs the turbine solved with
  a nozzle it does not have in its closure.
* **Paths the law is not wired on refuse it, sometimes in the wrong words** (R1b): rungs 43–64's two-shaft fuel rigs and
  rung 102's one-shaft afterburner match never give the law its flight's ambient pressure, so every trial refuses; their
  searches skip failing trials and report "does not bracket". No number is wrong (every trial refuses), and the pages
  never take these paths.
* **COST deviations** (§ COST below): with the law on, three two-shaft requests stay over their 3× trigger.

## COST (anchor § COST — triggers, counted before anything changed)

Three rounds, each COUNTED first (`hp_vane::census`: vane sonic points, vane flows, interstage residuals, closure
residuals): (1) the vane's sonic point found once per search, the cooled closure's fixed point stopped when it settles;
(2) warm starts within an instant; (3) the closures and tails restructured to read the vane's law once (§ 1). Final, the
pages' own entries, best of 5 (slams 3), the law at 0.3 against the same request on the pin:

| request | pin | law | trigger | |
|---|---|---|---|---|
| Fly it, four points | 3.1–8.1 ms | 24.7–57.3 ms (4.8–15.3×) | C2 ≤ 20× | met |
| Fly it's slam, three | 149–192 ms | 2.10–2.80 s | C2 ≤ 5 s | met |
| Controls, default | 17.6 ms | 24.2 ms (1.4×) | C1 ≤ 3× | met |
| Controls, below choke | 21.7 ms | 75.6 ms (3.5×) | C1 ≤ 3× | **DEVIATION** |
| Controls, turbine cooling | 14.1 ms | 879 ms (62×) | C1 ≤ 3× | **DEVIATION** |
| Blades, two points | 11.8–17.5 ms | 0.9–1.0× | C1 ≤ 3× | met |
| Cooling, a cooled point | 2.3 ms | 12.4 ms (5.3×) | C1 ≤ 3× | **DEVIATION** |

**Shipped above three triggers — a DEVIATION, disclosed (rung 102's precedent).** The floor is one vane flow per
compressor trial wherever the law is not the pin; a cooled HP turbine has no shortcut at all (its coolants' share moves
its expansion even on the perfect gas, choked). In absolute terms every request stays under a second, and only with the
law switched on. **C0 (off costs nothing, bar ≤ 1.1×): met, but not nothing** — measured head to head (`c0.txt`: the
same pin requests built against `4ba69ee` and today's code, interleaved twice, best of 7, slams 3): one shaft (Fly it and
its slams) 0.97–1.01×; two shafts 1.03–1.05× (Controls, Blades, Cooling), 1.08× on Cooling's 0.25 ms point. The cause is not measured (likely the pin's dispatch through the law's
plumbing on the two-shaft solves).

## The next seam

* **Wire the law's flight pressure through the paths R1b found unwired** (rungs 43–64's fuel rigs, rung 102's one-shaft
  afterburner match, rung 37's soak), and let those searches pass the law's refusal through in its own words.

* **Rung 33's double root** (anchor G4): the one-shaft subsonic residual has two roots at 318 K / sea level / Mach 0.30,
  650–900 K, the pin too; the two-ended march refuses "SUB-IDLE" there. Its own repair, after this rung (the user).
  **REPAIRED 2026-10-21** (`docs/rung33-spec.md` § The double root): wider than G4 saw, and its edges move with the
  flight — on the 45 °C day at Mach 0.3, 646 K to 975–1000 K; at other sea-level flights from 550–725 K up to
  775 K–past 1000 K (per flight in that spec); the pin's march now walks up from its low end where both ends share a sign.
  (Commit `bbfee58`'s message says "646 K to ~1000 K at every sea-level flight" — that overstates it.)
* The afterburner's held light-off re-scored on its own grid under rung 104's LP law (rung 104's other seam) — once,
  since the HP law is off by default.
