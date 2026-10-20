# Rung 104 — THE LP TURBINE'S THROAT WAS NEVER CHOKED: a flow law replaces rung 38's pin on `A45`

**Code:** `rust/src/turbine_law.rs` — `TurbineLaw` (`Ellipse` the default, `TwoRows`, `OneRow`, `Pin`; its page key and
`pin-default`), `LpCapacity` (the design capture), `TwoSpoolCore::w45` (the law), `try_lp_choked`, `try_turbine_pair` and
`try_joint_hp` (the joint search), `law_is_pin` / `lit_held` (the shortcut, § 1). `two_spool.rs` — `TwoSpoolEngine::
with_turbine_law`; the core's `turbine_law`, `lp_capacity`, `law_shortcut`; `A45` private, read only through `a45_pin()`.
Every reader the compiler listed (anchor G6) goes through the module: rungs 38/39's cascades, 42's, 62–63's bleed
transient, 90's two, 91's `try_cooled_pair`, 92's offtake, 97's `try_lp_turbine` (a law's supply moves with `π`), 40's
transient tail and powers, 100's formula, 102's lit LP turbine, 103's dispatch. **The pages:** a `turbine` setting on the
Controls, Blades and Cooling views (`ControlsSettings`/`BladeSettings`/`CoolingSettings`, written to a reply only off its
default) and a select on each. **Gates:** `rust/tests/rung104.rs`; the old model's record `rust/oracle/rust_owned/
rung104_pin.txt`. **Anchor:** `docs/plans/rung104-anchor-turbine-law.md`, committed as `0ddab63` before any rung-104 code.
**Scratch:** `docs/plans/scratch-archive/rung104/`.

The seam: rung 38's concession, carried by every two-shaft rung since. The user's decisions (memory `rung104-decisions`):
the law is a **knob** (ellipse / two rows / one row, plus the old pin as its own setting); **on everywhere**; **one rung**;
the Python-held two-shaft oracles keep every value the law leaves alone and **retire only the moved ones, by name**
(§ 5); every older test that fails **sorted** (§ 5), each flip
re-checked under two rows.

> **THE HEADLINE — below choke the nozzle's back-pressure slows BOTH shafts.** The LP turbine's inlet vane never chokes
> (it reaches at most 0.998 of the critical pressure ratio), so the flow it passes follows the LP turbine's own expansion
> — and the HP turbine, whose back-pressure that vane sets, hears the LP turbine. Rung 97 found the back-pressure took
> work off the LP shaft ALONE (the HP shaft's slowdown 1/70 of the LP's). Under the ellipse the HP shaft slows **at least
> 0.535 ×** as much as the LP shaft at every one of 251 scored cells — **≥ 37 ×** rung 97's share — while the LP shaft
> slows about half as much as rung 97 said. Two rows ~0.45, one row ~0.32: the stiffer the law, the more is shared.
>
> With the nozzle choked on a perfect gas, the LP turbine at its design efficiency, every law IS the old pin (exactly —
> the choked nozzle fixes the LP turbine at its design expansion), so the pages' default engine moves only below choke
> (and where a turbine-efficiency map moves `η_LPT`); on a real gas the law moves work from the
> HP shaft to the LP shaft at low power, thrust still to the fifth digit.

## 1. What it adds

**The equation.** `W45 = ṁ√Tt45/pt45` at the LP turbine's own `x = pt5/pt45`, captured at the design run
(`W45(x_d, Tt45_d) = A45·MFP*(Tt45_d)`) — no new constant except two rows' split:

* **Ellipse** (Stodola, Cooke 1985): `W45 = W45_d·√(1−x²)/√(1−x_d²)` — the many-row limit (sixteen equal rows land within
  0.7 %, gate R7).
* **Two rows** — vane and rotor as two convergent rows in series, an equal pressure ratio each at design (the guessed
  split); **one row** — the vane takes the whole drop. The gas's own isentropic flux, capped at its sonic point.
* **Pin** — rung 38's choked vane, the literal old code at every site.

**The solves.** With the nozzle choked the LP turbine's `x` meets `A8`'s choked relation; the HP turbine's ratio meets
`A4·MFP*(Tt4)·√(Tt45/Tt4)/π_HPT = W45(x)` — one joint search, each trial running the variant's own LP step (cooled or
not). Below choke: rung 97's helper, the supply moving with `π`. Every search starts at its design value (never a previous
call's root) and widens to a sign change; a trial whose LP turbine cannot pass its gas ("the nozzle cannot pass",
rung 97; "the hot gas cannot get out", rung 102's lit lean wall) passes nothing, and the landed point is checked to be a
root, else the LP turbine's own refusal is returned.

**Changed after the anchor froze — each disclosed:**

* **The shortcut** (`law_is_pin`). Where a law provably IS the pin — the hot section calorically perfect, `η_LPT` equal to
  its design value, no afterburner lit behind a held throat, and the LP step landed on the pin's own choked root — the
  pin's own solve is used. The law's own search lands within ~1e−14 of that point (gate R2, the shortcut off); the
  shortcut lands ON it, so those points are the old model bit for bit. Why: rungs 48, 58, 66, 68, 69, 72–75 and 83 failed
  on the perfect-gas, choked plants where the two laws differ by ≤ 5e−15 a step (all choked, `p14`), because each reads a
  KINK — rung 58's identity is read at a minimum over marched steps that several steps tie on (rung 84's lesson), rung
  66's product halves when a central difference straddles `max(0, …)`. Exact conditions only; off with `law_shortcut`;
  never below choke or lit behind a held throat (advisor-reviewed, 2026-10-20).
* **COST.** C1/C2 fired on the first timing; the searches were started from design values and the cooled path given
  the shared joint search (anchor § COST). **C3 fired** (two rows on the equilibrium gas, up to 151× the ellipse):
  counted, the matcher's outer loop ran to its cap of 200 because the row laws' flux was rough at 1e−10 (the gas's
  inversion stops at 1e−11 of `T`); two more Newton steps on each inversion inside the module (smooth to ~2e−14) and the
  searches run to the last digit: **two rows 3–5.5× the ellipse, ≤ 0.21 s a point** (`p11c.txt`).
* **The joint search's range** [0.2, 0.9] → [0.01, 0.99], its start clamped into it: a Blades design with a high HP
  pressure ratio expands its HP turbine to 0.07 (`sandbox_blades` found it).
* **Just below the unchoke boundary** the LP turbine's subsonic root sits ON the choked one, and with the shortcut the
  residual there can read a hair under zero: that is the root (rung 97's halvings converge there too). Rung 103's R6
  found it on the cooled variant.

## 2. Gates (anchor § 2, bars frozen at `0ddab63`)

| gate | result |
|---|---|
| R1a the whole gate on the pin (`test-all.ps1 -Features pin-default`) | every cargo program passes but one test of rung 104's own (R10 compared the default with itself — fixed); the browser check's native side is built without the feature, so its grid compared the old page with the ellipse — rerun by hand with the feature: **142 992 of 311 971 bit-identical, the same worst differences, digit for digit, as `main`** |
| R1b the old model's record (6 932 keys, `33250a2`) | **0 differ** |
| R2 perfect gas, choked: every law = the pin | the law's own search ≤ **1.1e−14** at 216 cells; the shipped point bit for bit |
| R3 the design point | perfect gas ≤ 5.6e−16; **equilibrium gas MISS of the frozen 1e−12**: every law 4.7e−12 — and the pin itself 4.76e−12 (the matcher's own floor there, measured only after the freeze). Shipped: no law further from the design than the pin |
| R4 a hand-written perfect-gas solve below choke | **2.1e−13** at 12 cells (bar 1e−10) |
| R5 one root inside the search | 210 cells, one sign change each; the cooled variant's 48 cells not scanned (its residual mixes the coolants in) |
| R6 mass, read independently | A45 **7.4e−15**, nozzle **1.8e−13** (bar 1e−11) |
| R7 sixteen rows reach the ellipse | **0.71 %** (bar 1 %) |
| R8 the vane is never choked | worst **0.998** of critical over 48 points, two gases, four flights |
| R9 the red check (the joint search forced back to the pin's HP ratio, scratch) | R2 still passes (1.1e−14 — then a self-comparison, as predicted); **R4 fails at once** (550 K: `π_HPT` 0.4305 shipped against the hand's 0.5575) |
| R10 the pages | the setting reaches each view's engine; the default is the ellipse (the pin under `pin-default`) |

## 3. Scored (anchor § 3)

**H1 — HIT, 251 of 251 (H1a) and 43 of 43 (H1b).** Every variant (plain, bleed, customer air, cooling, the generator on
either shaft), three maps, three flights: against the old model both shafts slower, against rung 97's branch the HP
slower and the LP faster. H1b: the smallest share `ΔN_H/ΔN_L` **0.535** (bar 0.25), the smallest ratio to rung 97's
**37.5** (bar 10); rung 97's HP change has the other sign at 5 cells (counted, as the anchor asked); none excluded at the
boundary.

**H2 — HIT.** At all 43 perfect-gas cells below choke the HP shaft's slowdown is ordered rung 97 < one row < two rows <
ellipse and the LP's the reverse; at all 28 equilibrium-gas choked cells `|Δπ_HPT|` one row < two rows < ellipse.

**Reported:** thrust against the old model — more at sea level +40 K Mach 0.85 (+3.3 … +4.0 %), less at Mach 0.30 above
600 K; points rung 97's branch refused that a law solves (sea level Mach 0.30, 550 K). On the equilibrium gas, choked,
the law moves work from the HP shaft to the LP shaft (`π_LPC` up ~1 %, `π_HPC` down ~1 % at 700 K, zero at design)
while thrust holds to the fifth digit (`p5.txt`).

## 4. The pages

A **LP turbine's inlet vanes** select on Controls, Blades and Cooling (the fixed engine only — a redesigned engine runs at
its own design point): ellipse (the default), vane and rotor, the vane alone, choked as rungs 38–103 assumed. The reply
writes `turbine` only off its default, so every default reply keeps its shape. On the pages' perfect-gas engine with the
nozzle choked every choice gives the same run (the shortcut makes it the same bits); below choke the HP shaft slows too.

## 5. The sort — every older test that failed under the ellipse

(1) a number moved, the finding holds — the bar updated; (2) the finding weakens or flips — a correction in that rung's
spec; (3) untouched. Every flip re-checked under one and two rows.

**Numerical dust — (3), untouched, once the shortcut is in** (§ 1): rungs 48, 58, 66, 68, 69, 72, 73 (two), 74, 75, 83.

**Kept "as built" — the old model's own mechanics, on the `pin` setting** (each finding about rung 38's choked vane is
still true of it): rung 38's `(★)` cost (47 calls a solve); rung 97's hand-written root and its bit-for-bit pin; rung
103's hand-written point, held root and frozen headline cells; rung 102's instrument through the shipped pin (a second
instrument now checks the law, independently of the lit search).

**(2) Corrections — a finding flips, under EVERY law (one and two rows included, so not the extreme law's alone):**

* **Rung 97 — "the back-pressure takes work off the LP shaft ALONE"** flips. At its own 700 K start (three maps) against
  the old model: rung 97 LP −2.8 … −3.0 %, HP −0.04 %; one row −1.8 … −1.9 % / −0.56 … −0.62 %; two rows −1.6 … −1.7 % /
  −0.68 … −0.75 %; the ellipse −1.5 % / −0.74 … −0.82 %. Thrust still MORE (+3.3 … +4.0 %, rung 97 +3.7 … +4.8 %). The
  slam's peak `Tt4` runs a further 1.7 … 3.1 K hotter than rung 97's (`p20.txt`). Its test now asserts the old claim on the
  pin and the correction on the default; its panel's closing sentence says both.
* **Rung 102 Part C — "the HP shaft overspeeds 8–13 % after a held light-off"** flips: under every law the HP shaft SLOWS
  3–6 % (the back-pressure shared again); the LP shaft slows 10–16 %, not 35–41 %; the main burner still runs ~200 K
  hotter and thrust still rises (+22 … +27 %, more than the pin's +18 … +24 %) — the headline ("what the throttle holds
  decides the sign") survives (`p16.txt`). And **C12's three misses were the pin's**: the redline through a late held
  light-off stopped on the pin; under every law it holds 1480 K to the end (`p17.txt`).
* **Rung 103 — "below choke a better LP compressor LOWERS the HP pressure ratio"** flips: on rung 103's own instrument and
  grid, the pin 40 of 40 cells down, one row, two rows and the ellipse **50 of 50 up**. The arrow from the LP compressor to
  the HP spool survives below choke — the CLAUDE.md row's words — its direction reverses: a better LP compressor lets the
  LP turbine expand further, the vane passes more, and the HP turbine expands further too (`p15.txt`; gate `s103_*`).
* **Rung 100 — "deep below choke the table's engine has no working state"**: its case (sea level Mach 0.1, 600 K) runs
  under the law; lower down (Mach 0.05–0.2, 450–590 K) the engine itself or the speed table refuses first, so this
  refusal was not reached in that search. Kept on the pin.

**(1) A number moved, the finding holds:**

* Rung 99 H5 — the fan-speed formula still over-reads below choke, by about +2.7 % (was +5 %); bar 0.03 → 0.02.
* The low-start repair — the edge between below idle and running moved up ≤ 10 K: 470 K still below idle on every map,
  the throttle that runs on every map 490 K (was 480).
* The customer bleed's off-map endpoint — the crash map's case runs; a search of the same move found it lower down
  (`tilted`, a 0.08 share, to 530 K), the page's sentence unchanged.
* The altitude view's crash request still has no start, now in the generator's words (the law takes work off the HP shaft
  at low power); the old words kept on the pin.
* The Blades view's thermally-perfect point past the gas tables now runs below idle (a search of five designs found none
  past the tables); the old words kept on the pin.
* The lean-wall repair's "shafts too slow" first-step stop: its crawling start (LP at 15 %) runs the LP shaft at 37 %
  under the law, and a search (four maps, three flights, every customer mode, 400–490 K) found no such stop — kept on
  the pin (sandbox plan § 40 note).

**The CLI.** Six Python-era panels print different numbers and no different sentence — rungs 38, 39, 40, 41, 42, 46: their
Python segments are retired and each is held to its own capture in place (`panels::LAW_RECAPTURED`), still to the Python
segment under `pin-default`. Rung 38's panel says the table is the law's; rung 40's "cp(T) gas curve" line shrinks about
fivefold (4.3e−2 → 9.2e−3). Three Rust-owned panels recaptured (rungs 97, 100, 101); rung 97's closing sentence
corrected.

**Python-held two-shaft oracles — only the values the law moves are retired, by name** (the user's decision, re-asked
once the cost of retiring whole files was measured). Each arm of ten oracle programs has its list,
`rust/oracle/rung104_retired/<arm>.txt`, written once by the arm itself (`RUNG104_WRITE_RETIRED=1`) and committed; the
helper is `tests/common/rung104_retired.rs`. Every other key is still held to Python exactly as before, a listed key must
still be in the golden, and a retired key is kept out of a CPython arm's pinned worst case. The `pin-default` build reads
no list: there every key is held, as before rung 104. Slices L and the rung 38/39 oracle list the keys whose Rust value
differs law against pin (their abort reconciliation needs the list before the comparison); the rest list the keys that
differ from Python. **87 458 of 263 647 keys retired (33 %); 176 189 still held to Python.**

| arm | retired | of | arm | retired | of |
|---|---|---|---|---|---|
| slice L (PyPy / CPython) | 12 872 | 25 458 | rung 40/44 transient main | 2 | 6 853 |
| slice M PyPy | 3 297 | 10 950 | … equilibrium | 868 | 1 120 |
| slice M equilibrium | 480 | 840 | … CPython | 383 | 7 973 |
| slice M CPython | 2 299 | 10 950 | slice S smoke | 63 | 5 536 |
| slice N PyPy | 26 595 | 72 520 | slice R smoke | 32 | 1 182 |
| slice N equilibrium | 3 783 | 5 649 | topping (PyPy) | 82 | 1 729 |
| slice N CPython | 9 481 | 41 560 | topping CPython | 57 | 1 729 |
| slice O PyPy | 18 | 8 700 | fuel transient main | 0 | 4 671 |
| slice O equilibrium | 20 | 204 | fuel transient gas | 776 | 1 133 |
| rung 38/39 (PyPy / CPython) | 6 699 | 11 814 | fuel transient CPython | 96 | 5 804 |

**Where they sit:** every retired key is on the thermally-perfect, reacting, Fork B or equilibrium gas — except **264
keys at eight perfect-gas cells of rung 39's `turb` map** (`a_t = 0.02`, the turbine efficiency off its design value):
there the LP turbine's expansion leaves its design value even with the nozzle choked, so the vane leaves its design point
and the law differs from the pin for real (the shortcut's `η_LPT` condition) — by a few parts per million: `η_LPT` off
design by up to 4e−4, speeds and `π_HPC` 2–12e−6, the same on the pages' surge-realistic maps (`p22.txt`; not search
rounding, which is ~1e−14). No perfect-gas key at design efficiency moved. The views that carry the law (Controls,
Blades, Cooling) run maps with no turbine term, so § 4's "the same run, choked" holds there bit for bit.

**Counts the law moves** (not values, so not in a list) are held as the Rust-owned numbers they now are, Python's on the
pin build: slice M's bisection pass sets (31 joins the ladder's, 25 and 27 the root's); the fuel transient's Newton pass
total on the three TPG gases 157 → 67 (TPG 21 → 23; reacting and Fork B 68 → 22 each — the count probe 3 saw swing
16-fold between interpreters); slice N's uniform-profile binds (40 TPG half-rows bind at the rear, not the front),
`inc_worst`'s split (80 move rear → front), the schedule rows reached (120 → 118) and the design-throttle argmin ties that
differ from CPython (520 → 260). Its 13 interior readings are the same one population (HP, perfect gas, design). Slice
N's "every missed schedule row is a raise, not an exhausted scan" was read off Python's firing count; it is now also read
off the rows themselves (a miss whose last good setting is within a step of the scan's end) — **0 exhausted under the
law: the two new misses are raises** — and its pass identity `48 × looped solves` is read as "a multiple of 48" where
the aborts are the law's own.

## Concessions

* **The HP vane `A4` stays choked** — by G1's bound 1.58–1.60× critical, with margin unless the stage's reaction is large.
* **Two rows' equal split is a guess** (the user's): the middle setting brackets the realistic case between one row and
  the ellipse, and the H2 ordering held at every cell.
* **A second root** of the whole system on rung 38's no-map perfect-gas engine at sea level Mach 0.30, 500–650 K (a fold
  just above the lowest running throttle; a degenerate HP-turbine-idle branch outside the search) — below idle, never on
  the pages' engines (anchor G8).
* **R3's frozen bar was below the equilibrium matcher's own floor** — a miss of the bar, not of the law.

## The next seam

* The HP vane's own law (reaction, `A4` borderline at high power).
* The afterburner's held light-off and every lit claim of rung 102 re-scored on its own grid under the law (here: the
  failing tests' cells and a 24-cell probe).
