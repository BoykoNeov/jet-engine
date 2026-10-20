# Rung 104 anchor — THE LP TURBINE'S THROAT WAS NEVER CHOKED: a flow law replaces rung 38's pin on `A45` (pre-registered 2026-10-20 — §§ 2, 3, COST)

**Seam:** rung 38's concession, carried by every two-shaft rung since (rung 97 § Concessions, rung 103 § 1: "`A45` stays
assumed choked"). Rung 38 pinned the LP turbine's flow parameter `W45 = ṁ√Tt45/pt45` at its nozzle guide vane's choked
value `A45·MFP*(Tt45)`, so the HP turbine never heard the LP turbine.

**User decisions (2026-10-20, AskUserQuestion; memory `rung104-decisions`):**
* **The law is a KNOB** — Stodola's ellipse (the default), two rows (vane + rotor, an equal pressure ratio each at
  design: the guessed split), one row (the vane takes the whole drop); plus the old pin as its own setting, which runs
  the literal old code.
* **On everywhere** — the default for every two-shaft engine, not a page-only switch.
* **One rung** — the steady six, the transient family, the afterburner, the cooling, rung 97/103's branches.
* **The Python-held two-shaft oracles are RETIRED** at the end of this rung; during the build they run on the pin
  setting, the check that the pin is still the old model bit for bit.
* **Every older test that fails under the ellipse is SORTED** — (1) a number moved, the finding holds: the bar is
  updated; (2) the finding weakens or flips: a correction in that rung's spec; (3) untouched. "Recheck" means the
  finding in words, not its old numbers. **Every finding that flips under the ellipse is re-checked under two rows**, and
  the spec says whether it flips under the realistic law or only the extreme one (this leans on § 3's H2 ordering).

**Status of what was seen before this was written.** Scratch crates `probe` (G1–G4, against the repo at `33250a2`) and
`probe2` (P5–P9, against a throwaway worktree of `33250a2` carrying a PROTOTYPE of the device). Probed: rung 38's engine
(no map, perfect and equilibrium gas) and the Controls engine on map `tilted` — flights sea level +40 K Mach 0.85, sea
level Mach 0.30, the Controls design flight, rung 38's design flight, hot Mach 0.3 (§ 4). **Never probed: maps `steep`,
`flat-eta`, `flow/press`; § 3's flights; any variant but plain; the transient family.**

**A headline guessed, then contradicted.** Before measuring below choke the working guess was "the LP turbine expands
LESS, so `W45` falls, `pt45` rises, and the HP turbine expands less too: both shafts lose work". G4 already showed the LP
turbine's ratio moving the other way; P5 and P9 confirmed it: under a law the HP turbine expands less and the LP turbine
MORE than rung 97/103's branch said — work moves from the HP shaft to the LP shaft. The headline of § 3 is the measured
picture, not the guess.

---

## 0. GIVENS — measured before anything was written

**G1 — the vane is never choked** (`g1.txt`). On the choked running line of rung 38's engine (equilibrium gas, `π` 3/6,
`Tt4` 1500 K design, 250 K / 50 kPa / Mach 0.85) the LP turbine's total ratio `pt45/pt5` is 1.451–1.463. Even if the
vane took the WHOLE drop down to the turbine-exit static pressure at Mach 0.6 (with `γ` at `Tt45`), it would reach only
**0.990–0.996** of the critical ratio — design point included. (The HP vane, by the same bound, sits at 1.58–1.60× the
critical ratio — choked with margin unless reaction is large: a concession, not built.)

**G2 — perfect gas, nozzle choked: every law IS the pin** (`g2.txt`, P5, P9). With the nozzle choked, `A8` fixes the LP
turbine's ratio as a function of `x = pt5/pt45` alone, and on the perfect gas every law's `W45` is a function of `x`
alone too; both pass through the design point, so the LP turbine sits at `x_d` whatever the law — **≡ the pin to
1e−15** (G2), bit-identical in P5/P9's printed digits. The Controls, Blades and Cooling engines default to the perfect
gas: on the pages the law shows only below choke (or with the thermally perfect gas).

**G3 — equilibrium gas, nozzle choked: work moves HP → LP at low power, zero at design** (`g3.txt`, P5). Rung 38's
engine: `Δπ_HPT` vs the pin +0.72 % (ellipse) / +0.39 % (two rows) / +0.26 % (one row) at `Tt4` 700 K, falling to zero
at 1500 K (`γ(T)` makes `MFP*` and the ellipse disagree off design). `π_LPC` up ~1 %, `π_HPC` down ~1 %, **thrust
unchanged in the fifth digit** (no map: the cycle's total is set by `A4` and `A8`).

**G3a — the row family reaches the ellipse** (`g3a.txt`). `n` equal-pressure-ratio rows in series (the gas's own
convergent-nozzle flux, velocity dumped between rows) approach Stodola's ellipse as `n` grows: within ~0.5 % at
`n` = 8–16 over `x` 0.50–0.95. One and two rows are the same family's first members — the knob's three settings are one
family, the ellipse its many-row limit.

**G4 — the first below-choke probe** (`g4.txt`, fixed `pt4`, not re-closed). Moves of 1–4 % in `π_HPT`; at hot 650/700 K
no law solved — an artifact of holding `pt4` (P5's closed solve finds them).

**G5 — the prototype, the pin setting, the whole gate** (`gate_pin.log`). With the prototype's default set to the pin:
**2 929 of 2 930 passed**; the one failure is the page-currency check (the committed page's bytes, which embed the
compiled model, differ from a fresh build — every source edit does this). Its second half, the committed page's numbers
against the native model, passed with the SAME 142 992 of 311 971 bit-identical and the same worst differences, digit
for digit, as the last gate on `main` (the browser's own floating-point library): **the pin setting is the old model.**
This run was on the prototype BEFORE G7's repair and COST's search changes (both touch only the laws' paths; the pin's
is literal) — R1a repeats it on the shipped code.

**G6 — the compiler's census of `A45` readers.** Making the field private listed 16 reads in `src/` (rungs 38/39's
cascades, 42, 62–63's bleed transient, 90's two, 91's cooled HPT, 97's helper, 40's transient tail and powers, 100's
formula, 102's lit LP turbine, 103's dispatch) plus the tests. Every one now goes through the law module; outside it
`A45` is read only on the pin path.

**G7 — a solver artifact found and repaired in the prototype** (`p5_before_fix.txt` → `p5.txt`). The joint search's
first trial (HP ratio 0.2) starved the LP turbine and its "the nozzle cannot pass" refusal killed the whole search, so
every law refused points the pin solved. Repair: a trial whose LP turbine refuses with "the nozzle cannot pass"
(rung 97) or "the hot gas cannot get out" (rung 102's lit lean wall) passes NOTHING (every law's `W45` at `x → 1`); any
other failure is passed on; and the landed point is checked to be a ROOT (`|r| ≤ 1e−8·k`), else the LP's own refusal is
returned.

**G8 — independent checks below choke** (`p7.txt`).
* A hand-written perfect-gas solve (closed-form turbines, the ellipse formula, sonic `A4`, the subsonic nozzle, an outer
  bisection on `x` with the HP ratio from `A4 = A45` at each `x`) equals rung 38's matcher and rung 39's on flat maps to
  **≤ 2.6e−13** at 15 unchoked cells, `M9` 0.445–0.998.
* The shipped joint residual over the HP ratio, at the converged outer state, has **exactly one sign change** on
  [0.2, 0.9] at all 39 below-choke cells of P5's grid (three laws), the root at 0.43–0.63, residual there ≤ 7e−18.
* The hand system on rung 38's no-map engine at sea level Mach 0.30 has a SECOND root at 500–650 K: at 500 K two roots
  close together just above the lowest throttle that runs at all (gone at 450 K — a fold); at 550–650 K a degenerate
  one where the HP turbine barely expands (`π_HPT` ≈ 0.95, outside the search's [0.2, 0.9]). The shipped search lands
  on the working root (the hand's first). Below idle, no-map engine only: a concession, not a gate.

**G9 — what the law does below choke** (`p9.txt`; the Controls engine, map `tilted`, three flights). Against the old
model (the pin pushed past choke — rung 97's reference) and against rung 97/103's branch (the pin, the LP turbine free):
* rung 97's branch: the HP shaft's slowdown is **~1/70** of the LP shaft's (0.0075–0.017) — rung 97's "LP shaft ALONE";
* the ellipse: the HP shaft slows **0.52–0.65 ×** as much as the LP shaft at every probe cell; two rows ~0.45, one row
  ~0.32; the LP shaft's slowdown is about HALF rung 97's;
* the ordering pin < one row < two rows < ellipse holds in `Δπ_HPT`, `ΔN_H` and `ΔN_L` at every cell;
* at sea level Mach 0.30, 550 K, rung 97's branch refused ("the nozzle cannot pass") and every law solves; at 600 K the
  LP shaft runs 44 % faster than rung 97's branch said, the HP 12 % slower, thrust 16 % lower;
* thrust against the old model: the sign varies by flight (more at sea level +40 K Mach 0.85, less at Mach 0.30 above
  600 K) — rung 103 already bounded rung 97's "more thrust" to its own cells.

**G10 — cost** (`p6*.txt`, the Controls view's own entry). See COST below: the frozen triggers FIRED on the first
timing; the calls were counted and the searches started from design values; then within both limits.

---

## 1. THE DEVICE

**The knob.** `TurbineLaw { Pin, Ellipse, TwoRows, OneRow }` (default `Ellipse`) on the two-spool engine
(`with_turbine_law`), copied to the core in a `Cell` (a test or page may switch it). On the pages: a setting on the three
two-shaft views (Controls, Blades, Cooling), request key `turbine` (`ellipse` / `two` / `one` / `pin`), written to a
reply only off its default.

**The capture** (zero new constants, at the design run): `x_d = pt5/pt45`, `π_HPT,d`, `Tt45_d`,
`W45_d = A45·MFP*(Tt45_d)`; one row's area from the gas's own convergent flux to `x_d`; two rows' areas from an equal
pressure ratio `√x_d` each, the second row's inlet temperature through the LP efficiency.

**The laws.** Ellipse: `W45 = W45_d·√(1−x²)/√(1−x_d²)`. One row: the convergent flux (sonic-capped) times `√(Tt45/Tt45_d)`.
Two rows: the interstage pressure where both rows pass the same flow, then that flow. Pin: rung 38's choked vane.

**The solves.** Nozzle choked: the LP turbine's `x` where `W45(x)` meets `A8`'s choked relation; the HP turbine's ratio
where `A4·MFP*(Tt4)·√(Tt45/Tt4)/π_HPT = W45(x)` — ONE joint search (each trial runs the variant's own LP step), cooled
or not. Below choke: rung 97's helper with the supply moving with `π`. Every search starts at its design value (never a
previous call's root: a point does not depend on what ran before) and widens until the sign changes. **Pin: the literal
old code at every one of G6's sites.**

## 2. GATES (bars frozen here)

* **R1 — the pin is the old model**, two ways (the prototype's environment-variable default does NOT ship):
  * **R1a — the whole gate on the pin:** a cargo feature `pin-default` makes the pin the default law and changes nothing
    else; `rust\test-all.ps1 -Features pin-default` runs the whole gate with it. It passes (the page-currency check
    excepted, G5's reason) on the shipped device code BEFORE any older test is touched — the Python-held two-shaft
    oracles included. This is their last run: they are retired after it.
  * **R1b — the old model's record, permanent:** `rung104_pin.txt` (6 932 keys: the six steady variants on two maps
    and three flights, choked and below choke; rung 39's engine and rung 38's on the equilibrium gas; nine Controls runs
    — default, unchoked up and down, governed thrust, cooling, the HP generator, customer air, the afterburner lit
    twice), captured by `probe/src/pin_golden.rs` from the repo at `33250a2` before any rung-104 code. Rung 104's test
    computes the same keys with every engine on the pin and requires every value bit for bit and every refusal's
    words; it moves to `rust/oracle/rust_owned/`.
* **R2 — perfect gas, choked ⇒ the pin:** every law = the pin to ≤ 1e−12 (`π_HPT`, `π_LPT`, both speeds, thrust) at
  every choked cell of § 3's grids.
* **R3 — the design point:** at the design flight and `Tt4`, every law reproduces the design run's `π_HPT` and `x` to
  ≤ 1e−12, both gases.
* **R4 — a hand-written perfect-gas solve** (G8's): rung 38's and rung 39's flat-map matchers, below choke, ellipse, at
  three flights: `π_HPT`, `π_LPT`, `π_LPC`, `π_HPC` to ≤ 1e−10.
* **R5 — one root:** at every scored cell of § 3, the joint residual (outer state held) has exactly one sign change on a
  71-point grid over [0.2, 0.9], and the shipped root is ≥ 0.02 from both ends.
* **R6 — mass, read independently of the search:** at A45, the ellipse written out in the test — `W45_d` from the
  design run's station 45 and `A45` through the gas's own `MFP*`, `x` from the RESULT's stations 45 and 5 — times
  `pt45/√Tt45` equals the result's station-45 flow (≤ 1e−11 relative), never calling the module's law; below choke at
  the nozzle, `A8·ρ9·V9` from the rebuilt exit equals the station-5 flow (≤ 1e−11).
* **R7 — the family:** `n` = 16 equal rows within 1 % of the ellipse over `x` 0.50–0.95 (G3a's instrument, shipped).
* **R8 — never choked:** G1's bound ≤ 1 on rung 38's choked running line, 700–1500 K (shipped).
* **R9 — the red check:** with the joint search forced back to the pin's HP ratio (scratch edit), R2 still passes and R4
  fails.
* **R10 — the pages:** the setting reaches each view's engine (a reply differs from the pin's below choke); the pin
  setting reproduces the old reply bit for bit.

## 3. SCORED (bars frozen here; cells never probed)

**Cells.** The Controls engine (perfect gas), maps `steep`, `flat-eta`, `flow/press`; flights sea level +20 K Mach 0.60,
2 km standard Mach 0.50, sea level −10 K Mach 0.40; `Tt4` 575–825 K step 50. A cell is SCORED where the old model (the
pin past choke), rung 97/103's branch and the ellipse all solve there and the ellipse's nozzle is unchoked
(`M9 ≤ 0.99`). Variants: plain, bleed 0.08, customer share 0.03, cooling 0.02/0.04, generator 0.02 on the HP and on the
LP shaft (rung 103's H2 settings). The pre-flight (`preflight.txt`) printed only each cell's class, never a scored
quantity: **251 scored cells, 10–16 per variant and map, none vacuous.** Where the ellipse refuses (low throttles): the
generator cannot be carried (rung 92's words) or below idle, mostly where a reference refuses too; at 4 cells both
references solve and the ellipse's HP generator cannot be carried (the law moves work off the HP shaft — reported); one
"does not bracket" where both references refuse (now given in the LP turbine's own words).

**H1 — THE HEADLINE: below choke the back-pressure slows BOTH shafts.** Rung 97 found the LP shaft alone slows (the HP
turbine never read the LP turbine). With the vane unchoked the HP turbine's back-pressure follows the LP turbine's
expansion, so the HP turbine expands less and the slowdown is shared.
* **H1a (every variant):** against the old model, the ellipse's `N_H` AND `N_L` are both lower; against rung 97/103's
  branch, the ellipse's `N_H` is lower and its `N_L` higher (work moves from the HP shaft to the LP shaft).
* **H1b (plain, the three maps × three flights):** the share `s = ΔN_H/ΔN_L` (relative changes against the old model)
  is **≥ 0.25** under the ellipse at every scored cell, and **≥ 10 ×** the SIZE `|s|` of the same cell's share under
  rung 97/103's branch (whose HP change can be either sign and tiny — P9 has +0.003 % at one cell; the cells where it is
  positive are counted and reported).
* Cells with `|ΔN_L| < 1e−6` against the old model (at the boundary, where both vanish) are counted and excluded, not
  scored.

**H2 — the laws are ordered** (the premise of the "re-check under two rows" rule): at every H1b cell, the HP shaft's
slowdown against the old model is ordered `rung 97/103 < one row < two rows < ellipse`, and the LP shaft's the reverse.
And on the equilibrium gas, choked (rung 39's engine, flat and steep maps, design flight and 2 km Mach 0.80, `Tt4`
800–1400 K step 100): `|Δπ_HPT|` against the pin ordered `one row < two rows < ellipse`, all three > 0. Pre-flight
(`preflight_h2.txt`, class only): all 28 cells solve under all four settings, every one choked.

**Reported, not scored:** thrust against both references (its sign varies by flight — G9); the points rung 97/103's
branch refused that a law solves; the share under two rows and one row; on the equilibrium gas, choked, `Δπ_LPC`,
`Δπ_HPC` and thrust (G3's "work moves HP → LP, thrust still" re-measured with maps).

## 4. Probe cells (excluded from scoring)

Everything G1–G10 touched: rung 38's engine (both gases) at its design flight and hot Mach 0.3; the Controls engine on
map `tilted` at sea level +40 K Mach 0.85, sea level Mach 0.30 and its design flight; P6's seven Controls requests.

## 5. After scoring — reported, not scored

* **The sort** (the user's rule): the whole gate under the ellipse; every failing older test sorted (1)/(2)/(3); each
  flip re-checked under two rows; the list in the spec.
* **Rung 97's slam** (sea level +40 K Mach 0.85, 700 → 1400 K): its speeds, peak `Tt4` and thrust under each law,
  against its own published numbers.
* **The pages:** a seeded crash map of each two-shaft view's requests at low throttles, the ellipse against the pin,
  request by request; no panic allowed.

## COST — frozen 2026-10-20 BEFORE any timing was run (P6, the Controls view's own entry `sandbox_controls::controls`)

Each of P6's seven requests (the default 1000 → 1400 K move; sea level +40 K Mach 0.85, 700 → 1400 K and back; the
governed thrust; cooling 0.05/0.03; the HP generator; the customer share), timed on a quiet machine, each law against
the SAME request under the pin:

* **C1 — the default law (ellipse): ≤ 2× the pin** on every request.
* **C2 — two rows and one row: ≤ 10× the pin** on every request.
* **C3 — two rows on the equilibrium gas (frozen before P5's re-timed column was read):** a steady point of rung 38's
  matcher on the equilibrium gas under two rows ≤ 10× the same point under the ellipse, and ≤ 10 s — the sort's
  re-checks under two rows run there.
* These are TRIGGERS, not bars (rung 103's form): a request over its limit is not shipped as it is; the solver's calls
  are COUNTED (joint searches, LP solves, interstage solves per step) before anything is changed (rung 102 lessons 1, 5).

**What happened (recorded after the freeze).** First timing (`p6.txt`, single runs): ellipse 2–4×, two rows 18–38× —
both triggers FIRED. Counted (`p6b.txt`, ten runs each): ~13 joint trials per search from the wide bracket [0.2, 0.9],
~14 law calls per LP solve, ~11 interstage trials per two-row call; the cooled path bisected ~43 times; below choke
rung 97's helper halved ~45 times, each now a law call. Changed: every search starts at its design value and widens;
the cooled path uses the shared joint search; below choke a law uses false position (the pin keeps the halvings,
literally). Re-timed (`p6d.txt`): **ellipse 0.45–1.14×, two rows 1.5–6.4×, one row ≤ 1.6×.** P5 and P7 re-run on the
changed solver: P5 identical at its printed precision, P7's worst 2.1e−13.

**C3 FIRED** on its first reading (P5's re-timed column): two rows on rung 38's equilibrium-gas engine 2–151× the
ellipse, up to 15.7 s a point, irregular in `Tt4`. Counted (`p11.txt`): at 800–900 K the matcher's outer loop on
`(f, pt4)` runs to its cap of 200 passes (the ellipse: 8) — the two-row flow's nested searches stop at ~1e−13, the same
size as the outer loop's own tolerance, so it never settles. To be changed: the joint and interstage searches driven to
the last representable digit; then re-timed against C3, the result in the spec.
