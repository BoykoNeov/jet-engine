# Rung 97 anchor — THE UNCHOKED NOZZLE ON TWO SHAFTS: below the unchoke boundary the LP turbine's expansion is the unknown (pre-registered 2026-10-18 — § 3)

**Seam:** CLAUDE.md *Still OPEN*: *"The subsonic / unchoked LP branch in the two-spool solves (38 flags, unsolved) and
its transient."* Rung 38 § *Scope*: *"if the nozzle unchokes, `tau_LPT` stops being geometric and becomes the unknown
that matches the LPT-NGV-choked supply to the subsonic-nozzle demand — precisely rung 33's inversion, relocated one
throat upstream … a natural rung-33-shaped follow-on."*

**How it came up (2026-10-18).** While adding a Mach knob to the sandbox's Controls view (the user's choice for rung
96's seam, now rung 98), a probe found that the Controls engine runs points whose nozzle is UNCHOKED, and that the
two-shaft transient computes them on rung 38's two-choked-throat pin anyway: the LP turbine is solved as if the nozzle
throat `A8` were sonic, and the tail only FLAGS the point (`Branch::Subsonic`). The page ignores the flag and draws the
point. Rung 38's steady matcher refuses such a point; the transient never did. Offered three options (stop or refuse
in words; draw dashed and warn; build the branch), the user answered **"Fix it properly first"**.

**Scope (stated to the user):** the TRANSIENT family — every two-shaft tail and shaft-power hook the Controls engine
runs through (rung 40's, rung 62's bleed, rung 90's customer air, rung 91's cooling; rung 92's offtake reaches rung 62's).
The steady matchers' cascades (rungs 38/39/42/90/91/92) keep their behaviour: the CLAUDE.md status becomes "transient
BUILT, steady still OPEN".

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung97-probe`, archived in
`docs/plans/scratch-archive/rung97/`). It holds a PROTOTYPE of the subsonic root built from public pieces and a census
logger in the four transient tails (never committed). It printed root existence, monotonicity, the LPT expansion ratio
at the root and the relative gap between the pin's `π_LPT` and the root's, and hit counts. A last probe printed, for each of
H1's cells, the start end's branch and that gap (`h1_cells.txt`). **No shaft speed, thrust,
temperature or stall margin of a corrected run was printed** — no corrected run existed. § 1–§ 3 were written before any
rung-97 code existed in the crate.

---

## 0. GIVENS — measured before anything was written

The Controls engine is rung 95/96's (rung 43's rig, the perfect gas), flown since sandbox plan § 27 at the knobs' air
temperature and pressure, Mach 0.85.

**G1 — where the void points are** (`choke.txt`). On the default move (1000 → 1400 K) no point is unchoked at any flight
probed. From a 700 K start they appear: at the design flight's air 1–5 of 126 points for Mach 0.05–0.5 (none at 0.85);
at sea level 1–12 (Mach 0.85 → 0.05); at sea level +40 K 9–20. Every unchoked point is a low-power one: the start end
and the first part of the move. On the flat-LP map the 700 K start is off the map at sea level.

**G2 — who reaches the void path today** (census over the whole gate, `census.tsv`; a logger in the four transient
tails recording each test that met an unchoked nozzle — `subsonic` — or a nozzle that could not pass the flow at all —
`abort`). The gate passed apart from the page-currency check, which the logger's own bytes in the browser build
explain (its numbers agree with the committed page to 5e−12). **No oracle and no Python-held golden meets an
unchoked instant.** The tests that do (19, one record each):
* rung 95: the deep-slowdown search start and the governed run's fixed point;
* rung 96: six — the bit-for-bit re-run, H4's calibrated ends, the fixed point, the ram-drag decomposition, the
  true-thrust dispatch and the clean-table inertness;
* the sandbox views: `sandbox_controls` (2), `sandbox_hold` (2), `sandbox_governor`, `sandbox_inferred`,
  `sandbox_altitude` (its refusal test, which also holds the one `abort` — § 27's "the nozzle cannot push it out");
* `cli_golden`'s Rust-owned panels (a change detector) and the CLI it drives.

So the branch needs no switch to keep an oracle: every Python-held number is on choked instants. **Rung 96's
decomposition gate meets unchoked instants**: its identity (`F_true − F̂ = V0·Δṁ`) rests on G2 of its anchor, "the
pressure ratio reads gross thrust exactly", which needs a CHOKED nozzle. The census does not say whether the unchoked
instants are among the gate's RECORDED points or only inside its solves (a table row, an end search). If any recorded
point is unchoked, the identity cannot hold there on the real branch; the gate is then restricted to choked points and
the points it skips are counted (a correction of rung 96's scope, disclosed in both specs). Measured after the build.

**G3 — the subsonic root exists, once, in the bracket** (`g1_root.txt`, prototype). At every unchoked instant of 3
flights (sea level +40 K, sea level, the design flight) × 5 engines (plain; generator on the HP or the LP shaft, power
0.02; the bleed lever; the stator lever) × 4 maps × 3 moves (700 → 1400, 1400 → 700, 650 → 1000 K) — **1 667 instants** —
the residual `resid(π) = A45·pt45·MFP*(Tt45, f45)/√Tt45 − A8·ρ9·V9(π)` is monotone decreasing on `[π_pin, 0.9995]`
(400 samples each, no rise beyond 1e−12 of the supply), positive at `π_pin` and negative at 0.9995. So the bisection on
that bracket has one root and needs no march.

**G4 — the pin over-expands the LP turbine, by up to 7 %** (`g1_root.txt`). The root's `π_LPT` exceeds the pin's by
1.5e−9 to 6.9e−2 relative (less expansion, less LP work). The LPT expansion ratio `1/π_LPT` at the root is 1.36–1.45.
**Disclosed concession carried, not solved:** `A45` (the LPT nozzle guide vane) stays assumed choked, rung 38's
concession. With the turbine expanding by under 1.5 overall, a real vane row would be well short of sonic; the choked
`A45` is the model's, here and on the choked branch alike.

**G5 — the per-variant mass bookkeeping needs nothing new.** Every transient tail builds the nozzle inlet as
`pt5 = π_LPT·π_HPT·pt4` at `(Tt45, f45)` — rung 91's cooled tail with its own `Tt45`/`f45` from the cooled HPT — and the
pin equates the `A45` and `A8` flows with no mass added between them in any variant. The subsonic residual equates the
same two flows. So one helper taking `(Tt45, f45, pt45, p0)` serves every variant.

## 1. THE DEVICE — derived before coding

**The equation.** Below the unchoke boundary only `A45` stays choked; the convergent nozzle expands fully to ambient
(`p9 = p0`, `M9 < 1`) and passes `ṁ = A8·ρ9·V9` at the ACTUAL `pt9/p0`. Equating the `A45` supply to that demand:

```
resid(π_LPT) = A45·pt45·MFP*(Tt45, f45)/√Tt45 − A8·ρ9·V9 = 0,     pt9 = π_n·π_LPT·pt45,  Tt9 = Tt5(π_LPT; η_LPT)
```

rung 33's `(★★)`, one throat downstream. `π_LPT` is the unknown; `τ_LPT`, `Tt5` follow from the LP turbine's map
efficiency at the instant (unchanged).

**The helper** (`rust/src/unchoked_lp.rs`): `try_lp_turbine(core, gas, Tt45, f45, η_LPT, pt45, p0)`.
1. Solve the pin exactly as now (`try_solve_choked_turbine(…, A45, A8, π_n, η)`), bit for bit.
2. Run the convergent nozzle at the pin's exit. If it is choked by the tails' own test (`p9 > p0 + 1e−6`), return the
   pin untouched.
3. Otherwise bisect `resid` on `[π_pin, 0.9995]` (G3), `TOL` 1e−13 absolute and `MAX` 200 — rung 38's solve's own. A
   nozzle that cannot push the gas out at a trial `π` has zero demand (`resid` = supply > 0).
4. If `resid(0.9995) ≥ 0` (the LP turbine cannot pass the flow even nearly unexpanded), refuse in words (an `Abort`
   "rung-97: …"). Never clamp.

The nozzle now runs inside every shaft-power call, including the equilibrium Newton's trial points, where the tail is
never reached. Checked before wiring: `Nozzle::try_apply` has no panicking path on the perfect gas — every guard is a
`check` returning an `Abort`, and the sonic throat is the fallible `try_sonic_throat` — so a trial point the nozzle
rejects is an `Err` the helper reads as "unchoked", never a crash.

**Wired into every transient LP-turbine site**, the tails AND the shaft-power hooks (rung 40's `r40_powers` re-solves
the LP turbine itself; a tail that branched without its powers hook would make a steady end stop being a fixed point of
the march): `two_spool_transient.rs` (tail, powers), `bleed_transient.rs` (tail, powers), `customer_bleed.rs` and
`turbine_cooling.rs` (`turbines`, shared by their tails and powers). Rung 90's and 91's powers hooks gain `flight`
(`p0`) and every site passes `pt45 = π_HPT·pt4`.

**The switch** (G2: no oracle needs one): `TwoSpoolCore::lp_pinned`, a `Cell<bool>`, default OFF (the branch on). ON
forces rung 38's pin everywhere — "the old model". It exists for § 3's scoring and the red check only; the page and the
CLI never set it.

**On the page:** the Controls chart draws the unchoked stretches thin and dashed (as the *Fly it* slam does) and the
table says at how many points the nozzle was unchoked. The numbers there are now the model's own.

## 2. REDUCES and GATES (construction — no credit)

* **Choked ⇒ bit for bit** (dispatch at step 2). Every instant whose pin leaves the nozzle choked is unchanged, so
  every gate, oracle and golden that never meets an unchoked instant (G2) is unmoved. The G2 tests are each re-read:
  a moved number there is expected and is checked against this rung's equation, never re-captured blind.
* **`lp_pinned` ON ⇒ the pre-rung code, bit for bit**, on the G2 tests' own settings.
* **Continuity across the boundary, on every variant** (plain, the generator on either shaft, customer air, cooling,
  the bleed lever, the stator lever): bisect a steady end's throttle onto the unchoke boundary; the root's `π_LPT` meets
  the pin's there. **Bar frozen: ≤ 1e−9 relative.** Over it means investigate, not relax: a jump means that variant's
  bookkeeping is wrong. The residual has a kink at the boundary (continuous in value, not in slope), so the same gate
  checks that the equilibrium's Newton converges within its iteration cap (80) on both sides of the boundary, on every
  variant.
* **An independent check:** a hand-written perfect-gas solve of the subsonic root — closed-form isentropic nozzle and
  turbine, its own bisection, not calling the helper, `Nozzle` or `try_tau_of` — matches the helper's `π_LPT` to
  **≤ 1e−10**.
* **The fixed point:** a march started on an unchoked steady end stays there: ≤ 1e−9 in both speeds after 2 τ.
* **The red check:** with the helper forced back to the pin, at least one gate fails on a wrong VALUE, not a crash.
* **The step check** (0.02 against 0.005, max flow-coefficient difference) on unchoked runs (sea level +40 K, 700 K
  start, four maps, the four governor laws at their limits): reported against § 27's levels (≤ 1e−4 unclamped).

## 3. PREDICTIONS — frozen here, scored after the build

**"The old model"** is the same run with the pin forced (the switch, § 1). Cells use the 700 K throttle at sea level
+40 K and at sea level (Mach 0.85), each of the four maps. **Measured before freezing** (`h1_cells.txt`, the same kind of
print as G4: the end's branch and the pin-to-root `π` gap only): on flow/press, press/flow and tilted the 700 K start
end is unchoked at both flights (gap 1.9–2.0e−2 at +40 K, 1.6–3.1e−4 at ISA); on flat-LP the 700 K start has no steady
point at either flight (refused, off the map). So **6 live cells and 2 VACUOUS**.

**Resolution floor (registered):** a speed difference counts only if `|Δν| ≥ 1e−9` — 100× the speed error the
equilibrium's power tolerance (1e−12) allows. A cell or point under it is VACUOUS, never a HIT or a MISS.

**The derivation (D).** At a fixed instant `(ν_L, ν_H, Tt4)`, the compressor side and the HP NGV choke set `ṁ`, `pt4`
and the HP turbine without reading the LP turbine (rung 38's triangular cascade). So the change touches ONE thing: the LP
turbine expands less (G4), its work per unit gas `h(Tt45) − h(Tt5)` falls, and the LP shaft's power residual falls. The
HP shaft's residual is unchanged at that instant.

**H1 — at an unchoked steady end the LP shaft runs SLOWER than the old model said.** *Derivation:* (D) at the old
model's equilibrium leaves the LP shaft with a power deficit and the HP shaft balanced, so the LP shaft must slow until
its compressor's work falls to match. Slowing lowers `π_LPC`, hence `pt45` and `pt9/p0`, which unchokes the nozzle
further and takes more expansion off the LP turbine — the loop runs the same way. *Risk:* the HP shaft re-balances at
the new LP speed (its inlet is colder and at lower pressure), and its swallowing changes the LP compressor's flow; if
that cross-coupling were strong enough, the LP shaft could end faster. 4 maps × 2 flights = 8 cells (6 live, generator
off). *The named risk is measured even on a HIT:* the HP speed difference at each end, with its sign.

**H2 — through an acceleration that starts unchoked, the LP shaft is BEHIND the old model's at every recorded point of
the RAMP.** 700 → 1400 K over 0.5 τ (settling 2 τ, not scored): `ν_L,new(s) < ν_L,old(s)` at every recorded point with
`0 < s ≤ 0.5 τ` whose difference clears the floor; a cell needs at least one such point, else VACUOUS. *Why only the
ramp:* the run ends on the choked 1400 K end, identical in both models, so the gap decays toward rounding through the
settle and its late sign means nothing. *Derivation:* it starts slower (H1), and while the nozzle is unchoked (D) gives
it less turbine power at any state; once choked, both models share the dynamics from a state where it is behind.
*Risk:* the march is two-dimensional, so a lower LP speed with a different HP speed need not stay lower — the HP shaft
could drag the LP compressor's operating point to a better power balance. *Measured even on a HIT:* whether the two LP
trajectories ever cross over the whole run, settle included, above the floor. 8 cells (6 live).

**Reported, not scored** (no sign derived): the HP shaft's speed, the thrust, `Tt4`'s overshoot and both stall margins
at the unchoked ends and through the moves; how far the boundary sits from the default move; rung 96's pressure-ratio
reading (its G2 — the pressure ratio reads gross thrust exactly — needs a CHOKED nozzle, so it fails on the new branch:
measured and stated, not scored).

**Rungs 95 and 96's published numbers.** G2 puts rung 96's H4 calibrated ends, its fixed point, its decomposition test
and rung 95's governed fixed point on settings that meet unchoked instants. After the build every scored table of
both specs is re-measured on the G2 tests' settings. A moved number is restated with "corrected by rung 97", never
silently re-captured; a flipped verdict is reported as a finding in both specs. (Their default slams — 1000 ↔ 1400 K
at the design flight — never unchoke, G1, so their headline cells are expected not to move.)

**§ 27's "the nozzle cannot push it out" refusal** (the census's one `abort`, `sandbox_altitude`'s request: 600 K in
warm dense air with a large customer amount). On the pin the nozzle could not pass the flow at all. With the branch the
LP turbine expands less and `pt9` rises, so **expected: it now runs, or fails with the rung-97 words** (step 4) — its
test changes either way, and whichever happens is stated.

**Left on the pin, said so:** the Controls view's generator-refusal check asks rung 92's STEADY matcher
(`ShaftOfftakeMatcher::try_match_point`, a cascade this rung does not touch) whether the load removes a steady point.
At an unchoked throttle its answer comes from the pin; its words are checked against the transient's own failure at the
crash map's unchoked refusals, and any disagreement is reported.

## 4. What would change the plan

A MISS is reported as a finding. A refused instant (step 4) inside a run is a reported stop with its reason. The crash
map runs BEFORE the spec is written. **Cost:** an unchoked instant costs one extra nozzle evaluation per bisection step
(~44 steps); a choked instant one nozzle evaluation more than today in the powers hook — measured and reported.
