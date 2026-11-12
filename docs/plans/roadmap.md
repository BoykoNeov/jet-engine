# Roadmap — where the project stands and what to build next

Written 2026-11-12, after a whole-project review the user asked for ("improve the project — design and
concepts, current and future; fix bugs; write plans for what does not fit one session").

This file is the **entry point for future work**. Part 1 is the review. Part 2 is a list of ready-to-build
tasks, each written so that a session with no memory of this one can pick it up. Part 3 is the larger
directions, which need the user's choice before anyone starts.

Rules that apply to every task here (they are the project's own, from `CLAUDE.md` and
`docs/plans/sandbox-plan.md`):

- Measure before writing a number down. A sentence on the page or in a doc quotes only what a run showed.
- A new effect ships as **knobs with every result shown**, never as a pass/fail verdict.
- Anything that changes `rust/src/` or `docs/sandbox/template.html` needs
  `powershell -File rust\sandbox-wasm\build.ps1`, then the gate `powershell -File rust\test-all.ps1`,
  then a commit that includes the rebuilt page.
- A change must not move any existing number unless the task says so. The gate proves it.
- `CLAUDE.md` is within 20 bytes of its size budget. Do not add to it; a new rung's row is the only
  growth allowed, and that needs the budget raised in the same commit
  (`rust/tests/claude_md_reference.rs`).

---

## Part 1 — The review

### What is strong

- **The model is trustworthy.** 108 rungs, each reducing exactly to the one before, about 2 660 tests,
  results held bit-for-bit to recorded values. A linter pass over the whole crate (2026-11-12) found
  no defect: the two lines it rejects are deliberate and documented (a faithful copy of an old
  self-comparison in `applied_reference.rs`, and a not-a-number test written as `x != x` in
  `stator_transient.rs`).
- **The sandbox page works.** All 473 browser and bridge checks pass, and every view was driven by
  hand-written probes during this review without one uncaught page error.
- **The record is honest.** Misses are written down as misses; eight "this did not work" documents are
  kept beside the successes.

### What is weak

1. **The project was hard to enter.** There was no front page: the repository opened on `CLAUDE.md`,
   an index written for the build assistant. *Fixed this session:* `README.md`.
2. **The sandbox had no starting point.** Six views and about 150 knobs, with nothing saying what to
   try first. *Fixed this session, on paper:* `docs/sandbox/EXPERIMENTS.md` (19 experiments, every
   number read off the page). Moving them *into* the page is task A1.
3. **An engine could not be run standing still.** Mach 0 — the test-stand point every real engine is
   rated at — was refused. The cause was not physics: two sanity checks cannot pass at exactly zero
   speed. *Fixed this session for the Design view* (see "What changed" below). *Fly it*, *Controls*
   and the afterburner still refuse it: task A3.
4. **The plain-language notes stop at rung 22.** `NOTES.md` is the only place the physics is told in
   everyday words, and it covers a fifth of the model. Rungs 25–108 exist only as formal specs. For a
   project whose stated product is understanding, this is the largest gap: task A4.
5. **The charts page is frozen at rung 29.** `docs/visuals/turbojet-visuals.html` shows a "29-rung
   map" and plots rungs 1–23. Either retire it in favour of the sandbox or bring its map up to date:
   task A5.
6. **The efficiency split misleads with a convergent nozzle.** Switch the Design view to the fixed
   convergent nozzle and "thermal efficiency" reads 15 % and "propulsive" 115 %. The page explains why
   (the split counts jet speed only and misses the pressure push), but a learner should not have to be
   told to distrust two of the six headline numbers: task A6.
7. **Time has no seconds.** Every transient is in units of "the shaft's own response time", because
   the model has no rotor weight. That is honest, but "0.14 τ" means little to a newcomer: task B2.
8. **Rungs 64–84 drifted away from engines.** They are a careful study of how control loops combine,
   and of the project's own solver, and they are correct — but they read as control theory, and none
   of them can be seen on the page. Nothing needs undoing. The lesson for future rungs: prefer effects
   a user can see by moving a knob (as rungs 85–108 do).
9. **Small page roughness** (none of it wrong, all of it cheap): the six view buttons wrap into three
   lines each on a normal-width window; the long "What is computed" text shows every view's paragraph
   at once, whichever view is open; a setup cannot be saved or shared. Task A7.

### What changed this session (2026-11-12)

- `README.md` — the front page.
- `docs/sandbox/EXPERIMENTS.md` — 19 experiments with measured numbers.
- **The standing engine on the Design view.** `rust/src/engine.rs` gained `freestream_or_static` and
  `score_or_static`. At Mach 0 the air's total state is its static state (no ram), and propulsive and
  overall efficiency are exactly 0. The two existing functions (`try_freestream`, `try_score`) are
  untouched **on purpose**: rung 41's schedule methods rely on them refusing Mach 0 to skip that
  column of their grid, so changing them would move recorded results. Only the design run
  (`Engine::run`) and the sandbox's design pre-check call the new ones, and both failed at Mach 0
  before, so no earlier number can move. Test:
  `rust/tests/sandbox.rs::a_standing_engine_runs_and_is_the_limit_of_slow_flight` — every gas model
  and nozzle; the standing engine agrees with flight at Mach 0.000001 (scored by the old, checked
  path). The page's Mach slider now reaches 0. Sea-level static at the opening design: 18.86 kN.
- This file.

---

## Part 2 — Ready-to-build tasks

Ordered by value for effort. Each is independent unless it says otherwise.

### A1. Put the experiments into the page ("Try this") — DONE 2026-11-12

Shipped as written, with one shared menu under the view buttons (refilled per view) instead of one `<select>` per
view; 29 entries over five views, none on Size the blades (the guide measured none there). How it works and what
holds it: `docs/sandbox/README.md`, section "Try this". It also fixed the `window.sandbox.fly` timing gap that A2
names (`settled()` in the template).

**Why.** `docs/sandbox/EXPERIMENTS.md` is a document beside the page; most users will never open it.

**What to build.** On each view, a small **Try this** menu. Choosing an entry sets that view's knobs
and shows two or three sentences under the menu: what was set, what to look at, why it happens. A
**Back to the opening settings** entry undoes it.

**How.**
- All in `docs/sandbox/template.html`. The page already has one function per view that takes a partial
  set of knobs and runs it — see `window.sandbox = { set, fly, slam, controls, cooling, burner, blades }`
  near the end of the script. An experiment is therefore data: `{ view, title, patch, text }`.
- Add one `const EXPERIMENTS = [...]` table and one `<select>` per view, filled from it. On choose:
  call the view's function with the patch, write `text` into a `<p class="note">` below.
- Take the experiments, their knob values and their words from `docs/sandbox/EXPERIMENTS.md`. The
  knob names inside a patch are the keys of the page's settings objects (`S` design, `F` fly, `SL`
  slam, `C` controls, `BK` burner, `CL` cooling); print them with `JSON.stringify(S)` etc. in the
  browser console.
- Do not put numbers that can go stale into the page text. Say "thrust falls", not "to 14.91 kN" —
  the page shows the number itself. Use **pin & compare** (each view has a pin function) so the page
  shows the change: pin the opening result, then apply the patch.

**Tests (all required).**
- `rust/tests/sandbox_page.rs`: every experiment's `view` is a real view and every key in its `patch`
  is a real setting of that view (the file already checks knob names this way — follow it).
- `rust/sandbox-wasm/browser.mjs`: for each view, choose one experiment through the menu, wait for the
  run, and check the *direction* the text promises (for example, experiment 10: the fuel-metered peak
  is above the commanded 1500 K). Then choose "Back to the opening settings" and check the opening
  result returns.

**Done when** the gate passes and each view's menu works on a phone-width window (the browser check
can save screenshots: set the environment variable `SANDBOX_SHOTS` to a folder under `W:\temp\claude`
and look at them).

### A2. Hold the experiments guide to the page

**Why.** `EXPERIMENTS.md` quotes about 120 numbers read by hand on 2026-11-12. Nothing fails when the
model moves them.

**What to build.** A check in `rust/sandbox-wasm/browser.mjs` (or a Rust test calling
`turbojet::sandbox::call` with the same requests) that re-reads each table of `EXPERIMENTS.md` and
fails if a number differs from the page's by more than the rounding shown. Simplest form: keep the
experiments' inputs and expected outputs in one small data file both the document and the check read;
if that is too much, parse the Markdown tables directly.

**Careful.** After `window.sandbox.fly(...)` resolves, the result may not have landed yet: entering
the view queues a second run that supersedes the first (see `enterFly` → `change()` and the
`if (my !== gen) return` line in `runNow`). Wait until `window.sandbox.last.fly` matches the knobs you
set. This is a small flaw in the page's test interface, not in what a user sees. *Fixed 2026-11-12 with A1:*
`fly` and `design` now wait out the queued run (`settled()`) before their own, so they resolve on their result.

### A3. The standing engine on *Fly it*, *Controls* and with the afterburner

**Why.** Sea-level static is where engines are rated and where every flight starts. The Design view
now runs it; the other views say "not supported yet".

**What blocks it** (measured this session):
- `Engine::try_freestream` (`rust/src/engine.rs`) refuses Mach 0 on the two table-based gases: turning
  the air's enthalpy into a temperature and back lands a hair low, and the "stopping the flow must not
  cool it" check fails. On the perfect gas it passes.
- `engine::try_score` and its copy `afterburner::try_score_total` compare two efficiencies with a
  relative tolerance, which cannot pass when both are exactly zero.
- **Neither may simply be fixed in place**: rung 41 uses both refusals to skip the Mach 0 column of its
  grid, and its recorded results would move.

**How.** Follow this session's pattern: a twin that handles exactly-zero speed and otherwise calls the
original. `Engine::freestream_or_static` and `engine::score_or_static` exist. Needed next:
1. A `try_score_total` twin in `rust/src/afterburner.rs` for the lit afterburner at rest; then drop
   the "standing engine with the afterburner" refusal in `sandbox::precheck`.
2. *Fly it*: `rust/src/sandbox.rs` (`fly` and what it calls) reaches the freestream through the
   matchers (`rust/src/matcher.rs` lines near `fs_engine.freestream`). Give the matcher's freestream
   helper an `_or_static` route used **only** when the sandbox asks (a flag on the sandbox's call, or a
   sandbox-side wrapper), so rung 31–41 callers keep today's behaviour. Then replace the `score(...)`
   call near the end of `sandbox::fly` with `score_or_static(...)` and relax its `m0 <= 0` pre-check.
3. *Controls*: the same for the two-shaft engine (`rust/src/two_spool.rs`, the `try_freestream` calls)
   and `sandbox_controls.rs`'s `m0 <= 0` check. The thrust tables of rungs 96–101 are built over
   flight speed; check each reads sensibly at 0 before allowing it, and refuse in words where one
   does not.

**Tests.** For each view, the pattern of `a_standing_engine_runs_and_is_the_limit_of_slow_flight`
(`rust/tests/sandbox.rs`): the standing result agrees with Mach 0.000001 through the unchanged path.
Plus the gate, which proves rung 41 did not move.

**Stop and ask the user if** making *Fly it* work needs any change to a function rung 41 calls.

### A4. Plain-language notes for rungs 25–108

**Why.** See weakness 4. This is writing, not code, and it is the project's stated product.

**What to build.** New files `docs/notes/` — one per family of rungs, using the families listed at the
top of `CLAUDE.md`'s rung section (25–30 nozzle and turbine marches; 31–33 off-design; 34–37 one-shaft
transient; 38–45 two shafts; 46–52 fuel limiters; 53–56 and 61 airflow levers; 57–63 schedules; 64–68
valve and cascades; 69–84 control-loop structure; 85 blade speeds; 86–89 levers together; 90–94 loads;
95–101 governors and inferred thrust; 102 afterburner; 103–105 below choke; 106–108 governor beside
limiters). `NOTES.md` stays as it is and gains one line pointing at the folder.

**For each rung**, in the voice of `NOTES.md` (read its rung 10–13 sections first as the model):
the headline in one sentence a pilot or a student could repeat; the mechanism in a short paragraph
with no symbols that are not explained; one number from the rung's own spec that makes it concrete;
and, where the sandbox shows it, which knob to move (link the experiment in `EXPERIMENTS.md` if one
exists).

**Source of truth.** Each rung's `docs/rungN-spec.md`. Copy numbers from the spec exactly; never
compute or estimate a new one. If a spec's headline was later corrected by another rung (the rung
table in `CLAUDE.md` says "CORRECTS n"), tell the corrected story and say that it was corrected.

**Order.** Start with the families a sandbox user meets: 31–37, 46–52, 90–108. Leave 69–84 for last
and keep it short — one page for the whole family is enough.

**Done when** each family file exists, a reader who has not seen the code can follow it, and no
sentence uses a name from the code as if it were a word.

**Progress.** Started 2026-11-12: `docs/notes/README.md` is the index (which families are written).
Written: 31–33, 34–37, 46–52. Next: 90–108 (90–94, 95–101, 102, 103–105, 106–108). Practice that worked: before writing a family, grep
`CLAUDE.md`'s rung table and the later specs for "rung N" beside CORRECTS / REFUTES / BOUNDS / INVERTS, so
the corrected story is told; grep `docs/sandbox/template.html` for where the page shows each rung; label every
spec number with the case it belongs to (a spec's case is rarely the page's); before committing, scan the file
for Greek letters and code-style names and check every number appears in its spec.

### A5. Decide the fate of the charts page

**Why.** Weakness 5. `docs/visuals/turbojet-visuals.html` calls itself a map of 29 rungs.

**Two options — ask the user which:**
- *Small:* keep the page as "the first 29 rungs, in pictures", say so in its heading and in
  `docs/visuals/README.md`, and link it from the sandbox page's footer.
- *Larger:* replace its hand-kept rung map with one generated from `CLAUDE.md`'s rung table (the Rust
  command `cargo run --release -- visuals` already builds the page from data; add the table as data),
  so it can never fall behind again.

The cutaway (`turbojet-cutaway.html`) is fine as it is.

### A6. An efficiency split that survives a convergent nozzle

**Why.** Weakness 6.

**What to build.** On the page only — the engine's own scoring stays as it is (it is recorded bit for
bit). Add two rows to the performance table: thermal and propulsive efficiency computed from the
**equivalent jet speed** — the speed a fully expanded jet would need to give the same thrust:
`V_eq = (specific thrust + flight speed) / (1 + fuel-air ratio)`. Then
`thermal = ((1+f)·V_eq² − V0²) / (2·f·heating value)` and `propulsive = overall / thermal`. With a
fully expanded nozzle these equal the existing rows to rounding (check that in a test); with a convergent
nozzle they stay between 0 and 100 %.

**Derive before coding** (the project's first rule): write the three lines above with their reasons
into `docs/sandbox/README.md` first. Compute the rows in `rust/src/sandbox.rs` (`outcome_json`), not
in the page script, so the native tests can hold them. Tests in `rust/tests/sandbox.rs`: equal to the
existing split when fully expanded, on every gas; between 0 and 1 for the convergent nozzle over a
grid of pressure ratios and Mach numbers; their product equals the overall efficiency.

### A7. Page polish

Each is a small change to `docs/sandbox/template.html`; do them one commit at a time and look at
screenshots (`SANDBOX_SHOTS`) in light, dark and phone width after each.

1. **View buttons.** Six buttons in a row wrap to three lines each. Use shorter labels ("Design",
   "Fly", "Blades", "Controls", "Burner", "Cooling") or let the row wrap to two rows of three.
2. **Explanations per view.** The "What is computed" block shows every view's paragraph. Show the open
   view's paragraph first and fold the rest behind a "more" control. `rust/tests/sandbox_page.rs`
   checks element ids exist — keep the ids.
3. **Save and share a setup.** Put the open view and its changed knobs into the address after `#`
   (only the knobs that differ from the defaults, as JSON), read it back on load, and add a **Copy
   link** button. It must work from a double-clicked file. Browser test: set three knobs, copy the
   address, reload with it, check the same result appears.
4. **An emptied number box stays empty** after the cursor leaves it, while the model keeps the old
   value. On `blur`, write the setting's current value back into the box.

### A8. Tidy the two lines the linter rejects

`cargo clippy` stops on two deliberate lines. Replace `cell.s_eng != cell.s_eng` in
`rust/src/stator_transient.rs` with `cell.s_eng.is_nan()` (the same test, said plainly), and mark the
kept self-comparison in `rust/src/applied_reference.rs` with `#[allow(clippy::eq_op)]` and its existing
comment. Nothing may move; the gate proves it. The other 131 warnings are style and may be left.

---

## Part 3 — Larger directions (the user chooses)

Each of these is a new family of rungs or a new view. None should start without the user picking it
and without the project's usual first step: an anchor document with a published case to check against
and predictions written before the code.

### B1. A turbofan

The most natural next engine. Experiment 2 in the guide ends on it: a hot, fast jet wastes energy, so
move more air more slowly. What it adds: a fan, a bypass duct and a second nozzle; two new design
knobs (bypass ratio, fan pressure ratio). The two-shaft machinery (rungs 38–45) already carries a
low-pressure compressor on its own shaft; a fan is that compressor with most of its air going around
the core.

- *Smallest useful step:* the design point only, separate exhausts, on the Design view. Published
  check cases exist in the same textbooks the turbojet was anchored to (Mattingly, *Elements of
  Propulsion*, the turbofan chapters).
- *What a user would see:* fuel per thrust falling as bypass ratio rises, and the best fan pressure
  ratio moving with it.
- *Then:* flying it off design, and the fan's own stall line.

### B2. Real seconds

Give the shafts a weight. Rung 85 already sizes the blades (radii, speeds, stage counts); from those,
estimate each rotor's inertia (discs and blades as simple rings of the chosen material), and show a
second time axis in seconds beside τ. It ships as knobs: material density is already there; add a
"disc thickness" factor. Published spool-up times for small turbojets give the check (a few seconds
from idle to full thrust).

### B3. Starting and stopping

The model lives between idle and full power. Below idle — the starter motor, light-off, the windmilling
engine in flight, relight at altitude — is where much of real engine design effort goes, and the
project's solvers already stop there in words ("below idle"). This needs compressor behaviour at very
low speed, which the invented maps do not have; so it depends on B4.

### B4. A measured compressor map

The oldest open item: every stall margin on the page rests on an invented map and a stall line the
user types in. NASA published complete measured maps for research compressors (for example the Stage
35 and Stage 37 reports, and the Energy Efficient Engine compressor reports); digitise one as a plain
table file, read it through the existing map interface (`rust/src/map.rs`), and offer it as a fourth
"map shape" on *Fly it*. The stall margin then stops being "trend only". Honest limit to state: one
compressor's map, scaled to this engine's size.

### B5. A mission

Join the pieces: take-off (needs A3), climb, cruise, descent, flown by *Fly it*'s engine with a simple
aircraft (weight, drag). Readout: fuel burned and range. It makes "fuel per thrust" mean something —
and shows why the design that wins on a test stand is not the one that wins on a route.

### Standing items carried from `CLAUDE.md` ("Still OPEN")

They remain open and are unchanged by this review: a real spatial mixing model for the burner; a
pocket that freezes at the burner exit; detailed prompt-NO chemistry; fuel control on the reacting gas;
the two internal clocks of rung 37 on two shafts; feeding a chemically shifting gas back into the main
cycle. All are deep, none is visible to a sandbox user, and B1–B5 are better uses of the next sessions.

### A recommendation

A1 and A4 first: they turn work that already exists into something a person can learn from, at no
risk to the model. Then A3 and B1. B1 is the single change that would most widen what the project
teaches.
