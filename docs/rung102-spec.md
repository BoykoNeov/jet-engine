# Rung 102 — THE AFTERBURNER: a second burner behind the turbine, on the design, on the engine you fly, and lit during the slam

**Code:** `rust/src/afterburner.rs` — `Afterburner` (the knobs: `tt7`, `eta_ab`, `pi_fh`, `m6`), `AbMode` (`Unlit` /
`Lit`), `try_afterburn` (station 5 → 7), `fuel_frozen` / `fuel_equilibrium` (the fuel), `rayleigh` (the heated duct),
`try_score_total` (thrust on the total fuel), `build_afterburning_turbojet` / `AfterburningTurbojet` (Part A's builder,
continuing `build_turbojet`'s own run from station 5). `gas.rs`: the equilibrium section's burn-condition guard kept PER
fuel/air ratio, and `Gas::try_t_from_h_t`. **Gates:** `rust/tests/rung102.rs`. **Panel:** `print_afterburner_table` (born
in Rust, held to `rust/oracle/rust_owned/`; a change detector only). **Anchor:** `docs/plans/rung102-anchor-afterburner.md`,
committed as `379646c` before any rung-102 code. **Sandbox:** the Design view's *Afterburner* panel
(`docs/plans/sandbox-plan.md` § 34). **Scratch:** `docs/plans/scratch-archive/rung102/`.

CLAUDE.md's open item *"Afterburner"*, SPEC.md's *"a second burner between turbine and nozzle, with the nozzle re-sized"*.
The scope was set on measured probes, each answer the user's (memory `rung102-decisions`): ONE rung in three parts —
**(A)** the design point, **(B)** the one-shaft engine flown off design (*Fly it*), **(C)** the two-shaft slam
(*Controls*) with a light-off at a chosen moment; the duct loss DERIVED; a request past the oxygen REFUSED in words naming
the ceiling; on a built engine the nozzle a slider (throat over the dry throat) with a *match* setting; rungs 96–101's
thrust guesses refused with the afterburner lit.

**Status: Part A shipped** (this file's Parts B and C sections are filled as those parts ship; their predictions are
frozen in the anchor, Part C's in its addendum before its code).

> **THE HEADLINE (Part A) — the afterburner pays most flown fast, the nozzle must open MORE than the square-root rule,
> and dissociation sets the ceiling.** Lit to the same temperature, the same engine gains more thrust at higher flight
> speed for a smaller fuel-rate penalty (every one of 36 orderings across four flights; the project's design at
> 1900 K: ×1.35 thrust for ×1.39 fuel per thrust at Mach 0.85, ×1.64 for ×1.36 at Mach 2). The throat must open by
> `√(Tt7/Tt5)` over the heat-addition loss AND by another 1.1–5.9 %: the added fuel's mass and the hotter gas's lower
> choked flow per area. On the equilibrium gas burning all the oxygen left reaches only 2211–2244 K — 60–79 K short of
> the reacting gas, which keeps no dissociation — and near the ceiling dissociation costs 1.7–3.1 % more afterburner
> fuel. Mattingly's § 8.4 reference engine runs through the shipped model to his own equations at 1e−12.

## The device (anchor § 1)

Stations 5 → 6 → 7 → 9 (Mattingly Fig. 7.8); 5 → 6 a diffuser, isentropic, so 6 has no row. **The fuel:**
`(1+f5)·h_t(Tt5, f5) + η_AB·(f7 − f5)·h_PR = (1+f7)·h_t(Tt7, f7)`, `f7` the TOTAL per unit air — on the perfect gas
Mattingly's (7.29); on the reacting gas the fixed point `f7 = g(f7)` (increasing, a contraction, so iterates seeded at
`f5` rise onto the root and an iterate past the cap means the root is past it); on the equilibrium gas the scale-B
absolute balance with the station-4 FROZEN mixture at `Tt5` as reactant and the equilibrium products at `(f7, Tt7,
pt7)`, iterated with the duct's `pt7` (2–4 passes measured on Part A's grid, cap 50; the anchor's trigger was 10). **The
stoichiometric cap** `f_stoich·(1 − 1e−6)` on every gas. **The duct:** `π_AB = π_fh·π_R`, `π_R` the constant-area
heat-addition (Rayleigh) ratio from the three conservation laws (mass with the fuel added, momentum, energy), the exit
state parametrised by its kinetic energy; `M6 = 0` ⇒ `π_R = 1` by dispatch. **The nozzle** sees station 7; on the
equilibrium gas the afterburner freezes its own mixture. **Thrust, TSFC and efficiencies on `f7`** (Mattingly
7.30–7.32), through `try_score_total`, a deliberate copy of `engine::try_score` taking the fuel explicitly. **Defaults:**
`Tt7` 2000 K, `η_AB` 0.95, `π_fh` 0.97, `M6` 0.20 (anchor G5: 0.944 lit at a doubling, Mattingly Problem 7.9's 0.94).

**What the build changed from § 1 (disclosed):**
* The duct is solved by the general three-law solve on EVERY gas, the perfect gas included (§ 1 said "closed forms on
  CPG"): the closed form has no fuel-mass term, and one code path keeps the two gases comparable. The textbook closed
  form is the GATE's instrument instead (mass factor 1, below).
* The exit state is parametrised by `u = V7²/2`, not `T7`: the first crash map found ducts at Mach 0.001–0.002 failing
  the momentum closure — a bracket on `T7` resolves `Tt7 − T7` (a millionth of `T7`) only to 1e−8.
* The equilibrium fuel search never asks for the cap up front: the rung-6 Newton fails at isolated points hugging
  stoichiometric (φ 0.999999 at 1500 K, 3.2 bar; 1540–1575 K at 5.8–8.6 bar in the crash map) where no afterburner that
  runs ever probes. The bisection approaches the cap only when the root is up there; the cap test steps back to
  `1e−5`, `1e−4`, `1e−3` below stoichiometric only when the Newton fails at the edge (a ceiling within about a kelvin).
* The ceiling and the choke ceiling are named at the burning-zone entry pressure `π_fh·pt5` (the duct's few-percent drop
  moves dissociation by a few kelvin at most).

**Refusals, in words:** too hot (naming the ceiling); too cold (`Tt7 ≤ Tt5`; on the equilibrium gas below the frozen
mixture's own recombination heat, naming that floor — anchor G2); a choking duct (naming the hottest it takes); on the
Design view the knob ranges (`η_AB` 0.5–1, `π_fh` 0.85–1, `M6` 0–0.35).

## Givens (anchor § 0)

G1–G6 as frozen: the design point on four gases (G1), the equilibrium ceiling and the recombination self-heat (G2), the
Mattingly § 8.4 reference (G3: the book's printed thrust 0.34 % below its own equations, source not found), the held
nozzle on the built one-shaft engine (G4: thrust BELOW dry), the derived duct loss (G5), the steady two-shaft matcher
(G6, not the Controls plant). The probe slip the anchor disclosed (G4's "opened" line re-ran the dry engine) stands as
written: Part B's gate measures it.

## Gates (anchor § 2) — measured (`tests/rung102.rs`)

* **Mattingly's equations through the shipped model** — `f`, `f_AB`, `F/ṁ0`, `S` against (7.20)/(7.33) at G3's inputs
  and at Mach 2 / 11 km: **1e−12 relative** (`M6` 0, `π_fh` 0.94, dual perfect gas, `M0` 1e−3 both sides).
* **The duct solve is the textbook Rayleigh line** on a consistent perfect gas (`cp = γR/(γ−1)`), mass factor 1: `π_R`
  to 1e−10, `M7` to 1e−8, over `M6` 0.05–0.3 × heat ratios 1.1–2.5, and thermal choke exactly where the closed form
  chokes. (On `Gas::default()`, whose rounded constants disagree by 0.05 %, the gate would read that disagreement.)
* **`Unlit` with `π_fh` 1 is the shipped engine bit for bit** — station 9 and every performance number, four gases (so
  `try_score_total` at station 4's fuel IS `try_score`).
* **The per-`far` guard:** a second fuel/air ratio at its own burn condition is allowed; one ratio at two conditions
  still fires (rung 6's own gate unchanged). It moves no number: every oracle bit-exact (the gate).
* **Conservation every run:** energy (1e−9; the equilibrium absolute balance 1e−6), mass, the duct's momentum (1e−9).
* **Refusals:** too hot on every gas, the named ceiling itself running 2 K below; too cold on every gas, and on the
  equilibrium gas `Tt5 + 0.01 K` (the reacting gas runs it); a choking duct on every gas, its ceiling named; a
  Mach-0.001 duct closes.
* **The Design view:** the knobs inert while off (the reply, settings aside, identical); the default reply carries none
  of the new keys; the `defaults` op names all five and round-trips; a lit reply carries the block, station 7, the
  `ts_afterburner` curve.

## Scored, Part A (anchor § 3) — 261 HIT of 261 (`score_a.txt`; H1 in `tests/rung102.rs`)

| claim | cells | HIT | measured |
|---|---|---|---|
| H1 Mattingly § 8.4 printed | 1 | 1 | `f` 0.03581, `f_AB` 0.01955 (book 0.0358, 0.0195), `F/ṁ0` +0.31 %, `S` −0.30 %, `M9` −1.3 %, wet/dry 1.194 |
| H2 pays more flown fast | 36 | 36 | e.g. TPG `π_c` 15, 1400/2100: ×1.44 → 1.84 thrust, ×1.78 → 1.53 TSFC across the flights |
| H3 opens more than `√` | 144 | 144 | +1.14 … +5.93 % |
| H4 the ceiling | 16 | 16 | equilibrium 2211–2244 K; reacting 60–79 K higher |
| H5 dissociation's fuel | 16 | 16 | +1.72 … +3.09 % at 2100 K |
| H6 an unlit duct's cost | 48 | 48 | −1.30 … −0.59 % |

**The named risks, measured even on the hits** (rung 96's lesson):
* **H4's risk was WRONG.** It named 11 km Mach 1.6 (the lowest ambient pressure) as the lowest ceiling; it is
  2232–2241 K, mid-range. The lowest is 3 km Mach 0.5 (2211–2222 K): dissociation reads the burning zone's pressure, and
  Mach 1.6's ram rise holds that up. The highest ceilings are at 250 K Mach 1.2.
* H3's risks were right on both ends: the top (5.93 %) is the equilibrium gas at 2100 K, the bottom (1.14 %) a small rise
  (1700 K from `Tt4` 1600).
* H6's risk was right: sea level static (and 3 km) bite hardest, −1.30 %; the low-thrust 11 km flight least (−0.59 %).
* H5 landed in the lower half (the bar was [1.5, 8] %): at 2100 K the gas is still well below the ceiling.

## Census (anchor § 2) — what the defaults moved

* **Nothing.** Against a clean worktree of `884e60a`: all **335** earlier check requests (every view) byte-equal; a crash
  map of 1 500 random designs with the afterburner off (seed 102): 1 500 / 1 500 replies identical, including the 85 the
  old build panics on.
* **The afterburner's own crash maps** (seeds 103 and 202, 1 500 each, knobs randomized over their ranges): no new
  panic — every one of the 85 / 95 panics is the core engine's own, the same message on the old build with the
  afterburner removed. The first map found the three defects above (the slow duct, the Newton at the cap, an unlit
  readout at near-zero thrust refusing a lit design that runs — now the dry readout is left out); after the fixes every
  refusal is in words: too hot 297 / 292, too cold 225 / 229, a choking duct 20 / 20, the burner's precheck 206 / 195,
  and one efficiency-bookkeeping refusal in slice 1's words.
* **The check grid** gains 12 requests (appended last): a lit afterburner on every gas, two nozzles, `M6` 0, a hot Mach
  1.6 design, three refusals. `browser.mjs` lights it on the opening design (matches native; tiles, knobs and station 7
  shown; too hot refused; off restores the opening run bit for bit).

## Concessions

* The afterburner reads the hot section's properties (exact for Mattingly § 8.4, where `cp_AB = cp_t`; his Example 7.5
  uses a third set). Fuel enters with no enthalpy or axial momentum of its own (the main burner's convention).
* The flame holders are a fixed ratio (not a drag coefficient on the dynamic pressure); the diffuser 5 → 6 is lossless.
* The duct is constant-area with no friction; the afterburner burns completely to equilibrium (no finite-rate flame,
  no screech, no rich burning — the cap is stoichiometric).
* The chart's 5 → 7 jump mixes heat added with a change of reference mixture, as the main burner's does.
* Inherited: the frozen nozzle (rung 14 brackets it), the equilibrium Newton's isolated failures near stoichiometric.

## Part B — Fly it (pending; frozen as H7–H9 in the anchor)

## Part C — Controls (pending; its predictions in the anchor's addendum, frozen before its code)
