# TODO — switch the project's interpreter to PyPy

**Status: SHIPPED 2026-07-31.** PyPy 3.11 is the project's interpreter. Not a rung — a
build/infrastructure change. Raised 2026-07-31 after the measurement in § 0 refuted the scope of a
claim this project had already recorded.

**Slices 0–4 are all DONE.** Slice 5 remains optional and deliberately deferred — and slice 4
found it is *bigger* than written (see its RESULT). The detector slices 1–2 exist to make the
switch safe, and they are built: **8 042 pinned CPython values — rungs 3–28 gated individually,
rungs 31–66 through the CPG ladder and the rung-66 cascade — green under both interpreters.**

| slice | what | size |
|---|---|---|
| 0 | recon — the evidence base | **DONE** |
| 1 | the golden-fingerprint gate | **DONE** — `tests/test_numeric_fingerprint.py`, 6 381 pinned values |
| 2 | extend it across the rungs 3–30 kernels | **DONE** — 18 arms by rung, 8 044 pinned values |
| 3 | make the full gate green on PyPy | **MEASURED EMPTY — 973 passed, 0 failed** |
| 4 | the switch itself | **DONE** — 1002 passed in 2:47; `SLOW_SECONDS` kept, and why |
| 5 | collapse the three-gate policy | optional, deferred by choice — and larger than written |

**The prize, as delivered:** the full gate goes **17:27 → 2:47 (6.2×)** at the worker count
`-n auto` actually resolves to, measured on the same 1002 tests on an idle box. The 13:17 → 2:37
(5.08×) predicted here was measured on a smaller suite at a different rung; the *ratio* held and
improved. Still *not* the "28:18 → 1:55, ~14.8×" in `todo-engine-size-and-speed.md`, which bundles
the rung-30 algorithmic fix (already shipped, and CPython kept it) with an `-n 16` config that was
explicitly declined — and which slice 4 found survives only because `psutil` is installed.

**The value count moved 8 044 → 8 042**: the two rung-28 `no_collapse_ratio!raises` keys left the
goldens when that dead property was deleted (its own commit, `0b2bdcd`, before the switch).

**The blocker this plan exists to remove:** the project has 973 tests and **not one of them pins
an absolute number**. Every tight gate compares two quantities computed in the *same* run. So a
green suite under PyPy proves PyPy agrees with itself — it cannot prove PyPy agrees with CPython.
Switching interpreters without building that detector first would be switching onto a road with
no speedometer.

---

## § 0 — Recon: DONE (2026-07-31). The evidence base.

Full data + scripts: `M:\claud_projects\temp\pypy-fingerprint\` (`FINDINGS.md`, `fingerprint.py`,
`cpg_probe.py`, `rung66_probe.py`, `libm_probe.py`, `compare.py`). CPython 3.14.3 vs
PyPy 3.11.15 (7.3.23) at `dfb99e8`.

**The finding: `todo-engine-size-and-speed.md`'s "trajectories are BIT-IDENTICAL across
interpreters" is TRUE but SCOPED, and the scope was never stated.** Its evidence was one
341-point rung-66 `_stator_march` — a CPG / `_sonic_throat` path. Measured elsewhere:

| kernel | values | differ | worst relative |
|---|---|---|---|
| **CPG structural ladder** (design run + 6 off-design matches) | **96** | **0 — bit-identical** | — |
| **rung-66 two-lag cascade march, BOTH clocks armed** (341 rows x 18 keys) | **6 140** floats + 12 discrete | **0 — bit-identical** | — |
| A equilibrium design point (`_equil_solve` / `_gauss_solve`) | 18 | 5 | 6.6e-12 |
| B mixing-PDF geometry (`_spatial_local_field`, rungs 22/24) | 32 | 31 | 1.0e-12 |
| C zoned NOx chemistry | 18 | 6 | 2.4e-15 |
| D shifting turbine (`_work_limited_expand`) | 10 | 9 | 1.0e-08 |
| E off-design matcher **on the equilibrium gas** | 42 | 33 | 6.6e-11 |
| F freeze-out nozzle march (rung 26) | 13 | 11 | **3.7e-06** |
| **total, equilibrium side** | **133** | **95 (71%)** | **3.7e-06** |

**The split is clean and lands the right way: rungs 31–66 — every transient, 72% of gate
minutes — are bit-identical; the rungs 3–30 diagnostics are not.**

**Mechanism, probed directly** (`libm_probe.py`): `exp`, `log`, `sqrt`, `**` and `math.fsum`
agree bit-for-bit. What differs is **`expm1`, `log1p`, `erf`** (1 ulp) and **naive `sum()` over a
generator** (19 ulp on a 10 000-term harmonic sum — reassociation). Those are exactly the
primitives the NASA-integral / Gaussian-plume / equilibrium kernels lean on, and exactly what the
CPG closed forms do not touch. The split is structural, not luck.

**The discrete-branch risk is CHECKED, not argued.** A ≤1e-8 perturbation could in principle flip
a min-select crossing or a limiter engage/release row — which is what rungs 46–66 measure. The
rung-66 cascade march reproduces every one exactly: `argmin phi_lp = 14`, `argmin phi_hp = 100`,
`edges.g = [1]`, `argmax Tt4 = 100`.

**Already cleared, no work needed:** `conftest._run_at_below_normal_priority()` works under PyPy
(`GetPriorityClass == 0x4000` verified on both — PyPy has its own ctypes, and the documented
silent-no-op trap does not re-open). Startup + `import turbojet.engine`: 93 ms CPython, 118 ms
PyPy. No 3.12+ syntax anywhere in the tree. PyPy env already carries matplotlib 3.11.1,
numpy 2.4.6, pytest 9.1.1, pytest-xdist 3.8.0.

---

## § 1 — SLICE 1: the golden-fingerprint gate

**This slice is worth landing even if the PyPy switch is later abandoned.** It closes a hole that
has nothing to do with interpreters: nothing in this repo detects an unintended numerical change
to the model, from any source — a refactor, a library update, a solver tweak. The rung-30 speed
fix moved every CPG result by 1–2 ulp and the suite could not have told you.

**Deliverable:** `tests/test_numeric_fingerprint.py` + a committed golden data file, asserting
absolute values against per-kernel tolerances.

**Scope:** promote the four § 0 probes into one test module, covering the CPG ladder, the rung-66
cascade trajectory, and kernels A–F.

**Decisions this slice must make (all real, none pre-answered):**

1. **Tolerance per kernel.** Principle: **one decade above the measured cross-interpreter drift**
   — tight enough to be a genuine regression detector, loose enough not to flake on known
   interpreter noise. Record the measured drift beside each tolerance *in the test*, so the
   constant is disclosed rather than magic (the project's standing habit). The CPG arms are the
   exception: they are measured **exact**, so they can assert bit-equality — but see risk R2.
2. **Where the goldens live.** A JSON/text data file under `tests/` beats inlining ~6 000 hex
   floats. Needs a documented, single-command regeneration path, and regeneration must be a
   deliberate act with a diff you read — never a `--update-goldens` reflex.
3. **How much of the rung-66 trajectory to pin.** All 6 152 floats is thorough but makes the
   diff unreadable when it trips. Likely: pin every discrete reading (argmin/argmax/edge rows) +
   a fixed sample of rows, and keep a full-trajectory hash beside them.
4. **Keeping it always-on.** `conftest._is_spine()` currently matches only `test_reduce*`,
   `test_cycle_untouched*`, `*bit_for_bit*`. A tolerance-based golden test cannot honestly be
   named `*_bit_for_bit`, so `_is_spine` needs a fourth pattern. Per `CLAUDE.md`, `conftest.py`
   is the sanctioned place for policy — no test file gets edited.
5. **Runtime budget.** Measured (under load, so upper bounds): the 6-kernel fingerprint 16.4 s,
   the rung-66 march 4.3 s, the CPG ladder 0.18 s. **16.4 s exceeds `SLOW_SECONDS = 8.0`**, so
   as one test it would be slow-tagged out of bare `pytest` — defeating its purpose. Split
   per-kernel so each lands under the threshold, and let `_is_spine` carry the rest.

**Done when:** the gate is green on CPython, runs in the bare `pytest` subset, and a deliberately
introduced 1e-7 perturbation to a kernel makes it fail.

### RESULT — DONE (2026-07-31)

**Shipped:** `tests/test_numeric_fingerprint.py` (8 kernel arms + 2 meta-guards) and
`tests/golden/numeric_fingerprint.json` — **6 381 pinned values (6 369 floats + 12 discrete),
384 KB**, generated on CPython 3.14.3 at `b145f98`.
Floats are stored as `.hex()`, which round-trips exactly *and* makes a last-bit change visible in
the committed diff; a failure names the first 5 differing keys with golden / actual / relative
error. All five decisions were taken:

1. **Tolerances** — one round decade above the § 0 measured drift, each recording the drift it
   was calibrated against, in the test. The two CPG arms assert **bit-equality** (0 of 96 and
   0 of 6 152 differed), which is R2 taken deliberately.
2. **Goldens live** in `tests/golden/numeric_fingerprint.json`, regenerated by
   `python tests/test_numeric_fingerprint.py --regenerate`, which **prints every changing value
   before writing**. The docstring carries the decision procedure (shape drift / regression /
   accepted ulp shift), with the rung-30 fix cited as the worked precedent for the third branch.
3. **The rung-66 trajectory is pinned in full** — all 6 138 floats, no hash. Readability is a
   property of the *message*, not of how much is stored, and a hash only says "something moved".
   The `keys` list is pinned too, so a moved trajectory key reads as a shape change.
   (**Precision fix while pinning it:** § 0 above reported "6 152 floats" for this march. The
   kernel does pin 6 152 values, but **12 of them are discrete** (argmin/argmax indices, edge-row
   lists, the `keys` list), so the float count is **6 140** — of which **6 138** are the
   trajectory proper (341 rows x 18 keys) and 2 are the located minima. No conclusion moves —
   every one of them was bit-identical — but "6 152 floats" was not what was measured.)
4. **`conftest._is_spine` gained a fourth pattern**, `test_golden_fingerprint*` — deliberately
   narrower than `test_golden*` so slice 2 cannot drag a rung-24-class kernel into the fast
   subset. It earns the override concretely: kernel E measures **7.7 s idle**, so under an
   8-worker load it records above `SLOW_SECONDS = 8.0` and would be silently tagged out.
5. **Runtime** — 12.2 s serial for the whole module; every arm is a separate test, so the
   parallel cost is one 7.7 s pole. Nothing needed splitting further.

**Two things the plan got wrong, both now recorded in the test itself:**

- **The done-when criterion was not uniformly achievable.** Sensitivity was therefore *measured*
  per arm (perturb the kernel's primary input, sweep ε, find the first decade that turns it red)
  rather than asserted:

  | arm | tol | smallest relative input change **detected** |
  |---|---|---|
  | cpg | exact | 1e-14 |
  | r66 | exact | 1e-14 |
  | A | 1e-10 | 1e-10 |
  | B | 1e-11 | 1e-11 |
  | C | 1e-13 | 1e-12 |
  | D | 1e-7 | **1e-13** — the turbine bisection amplifies ~6 decades past its own tolerance |
  | E | 1e-9 | 1e-9 |
  | F | 1e-4 | **1e-5** — the weakest arm |

  **The honest headline: this gate catches a relative change of 1e-5 anywhere, and 1e-10 or
  tighter on seven of its eight arms.** Seven beat the planned 1e-7; F cannot, and tightening its
  constant would buy flakiness rather than sensitivity — 3.7e-6 of genuine cross-interpreter drift
  lives underneath it. If rung 26's numbers ever need guarding below 1e-5, the fix is a
  better-conditioned probe (its worst values are differences-of-near-equals), not a smaller number.

- **A decision the plan did not anticipate, and it reaches into slice 4: the goldens are
  CPython's, permanently.** They must **not** be regenerated as part of the switch — doing so
  destroys the only cross-interpreter anchor and reduces the file to "PyPy agrees with itself",
  the exact hole this slice exists to close. So the per-kernel tolerances are load-bearing
  *forever*, not just during the transition. A dedicated guard
  (`test_golden_file_declares_its_provenance`) fails if the `meta` block ever stops naming CPython.

**GREEN ON BOTH — and this is the first real cross-interpreter validation the project has had.**

```
CPython 3.14.3   pytest --runslow           983 passed in 745.60s (12:25)   [973 + the 10 new]
PyPy 3.11.15     tests/test_numeric_fingerprint.py   10 passed in 3.86s
```

Every previous PyPy result — including § 3's 973-green full gate — could only show *PyPy agreeing
with itself*, because every tight assertion in the suite is a same-run relative identity. This is
the first time PyPy has been checked against **absolute numbers produced by CPython**. It agrees.

**How much of each tolerance PyPy actually consumed** (`headroom.py`, run under PyPy against the
CPython goldens) — a green gate says "inside the band"; this says whether the band is snug:

| arm | values | differ | worst rel | tol | **used** |
|---|---|---|---|---|---|
| cpg | 96 | **0** | 0 | exact | — |
| r66 | 6 152 | **0** | 0 | exact | — |
| A | 18 | 5 | 6.60e-12 | 1e-10 | 6.6% |
| B | 32 | 31 | 1.04e-12 | 1e-11 | 10.5% |
| C | 18 | 6 | 2.36e-15 | 1e-13 | 2.4% |
| D | 10 | 9 | 1.01e-08 | 1e-7 | 10.2% |
| E | 42 | 33 | 6.63e-11 | 1e-9 | 6.6% |
| F | 13 | 11 | 3.68e-06 | 1e-4 | 3.7% |

**No arm sits above 10.5% of its tolerance, and none below 2.4%** — the one-decade rule produced
bands that are snug without being on the edge, so the constants are neither flaky nor decorative.
The two exact arms reproduce § 0 exactly: zero differing values out of 96 and out of 6 152.

**Also in this commit:** `CLAUDE.md` § Layout names the new file, and its byte budget went
31 000 → 31 500. That bump is disclosed in `test_claude_md_reference.py`: the prior budget had
**33 bytes** left, so rung 67's table row trips it regardless — taking it here for a nameable
reason beats taking it later for a row that has none.

---

## § 2 — SLICE 2: extend coverage to the rungs 3–30 kernel inventory

**Why separate from slice 1:** slice 1 builds the *machine*; this fills it. Different work — this
one is an inventory-and-signature grind, and § 0 already hit signature friction twice
(`_spatial_local_field` returns a scalar, not a field; `freeze_out_nozzle` needs six positional
args). It is also the slice whose size is least certain, which is exactly why it should not be
welded to the one that must land cleanly.

**Deliverable:** the golden gate covers every kernel that carries quoted digits in a spec, not
just the six sampled.

**Scope — the uncovered list:** the mixing closures `JetMixing` → `SpatialLocalPDF` (rungs 11–24,
each a distinct integral), `exhaust_no_clamp` (17), `thermal_nox` (7), `zoned_nox` in its rich /
zoned / RQL configurations (8–10), and the remaining nozzle marches `finite_rate_` (25),
`no_freeze_out_` (27), `coupled_no_freeze_out_` (28). Fork A / Fork B / frozen-products gases
alongside the equilibrium one.

**Decisions:**
1. Coverage bar — one representative point per kernel, or the anchor point each spec actually
   quotes? The latter is more useful and more work.
2. Runtime. Some of these are minutes per point (rung 24's per-pocket quench). Sampling policy
   must be explicit, and anything dropped must be **logged as dropped** — a silent top-N reads as
   "covered everything" when it did not.

**Done when:** every rungs 3–30 kernel with quoted digits has at least one pinned value, green on
both interpreters.

### RESULT — DONE (2026-07-31)

**Shipped:** 18 new arms in `tests/test_numeric_fingerprint.py`, keyed **by rung** (`prop`, `r7`,
`r8`, `r10`–`r18`, `r22`–`r25`, `r27`, `r28`) rather than continuing the A–F letters — a failure
message has to name the rung that moved. The golden file goes **6 381 → 8 044 values (384 →
496 KB)**. **The eighteen arms reproduce the temp-dir probe bit-for-bit** (`verify_promoted.py`),
so the committed gate is the measured object, not a re-derivation of it.

```
CPython 3.14.3   pytest tests/test_numeric_fingerprint.py --runslow   29 passed in 66.6s
PyPy 3.11.15     (same, against the CPython goldens)                  28 passed in 11.6s
bare `pytest`    24 of 28 (the four heavy arms slow-tagged)            9.4s
FULL GATE        CPython 1001 in 13:00  /  PyPy 1002 in 2:00          both green (§ 3)
```

**Both § 2 decisions taken:**

1. **Coverage bar — the anchor CONFIGURATION at reduced quadrature**, not a cheap representative
   point and not the full-resolution anchor. S=0.0625, the J values, φ_p=1.5 and the design point
   are the ones the specs use; only node counts are cut (quench grid 24/200 vs 240/2000, PDF nodes
   24/48 vs 200/200, cross-plane 16² vs 48²). **Stated in the test, in the register the project
   uses for concessions: a green arm here does NOT guard the digit a spec quotes.**
2. **Sampling/runtime is explicit and nothing is silently dropped.** Every kernel in § 2's
   uncovered list is pinned. Total probe cost 209.8 s CPython (heaviest arm 60.5 s) against a full
   gate whose pole is rung 24's own `test_ei_stays_monotone` at ~518 s — so the arms pack in
   without moving the wall clock, and no top-N truncation was needed.

**R1 (“slice 2 is bigger than it looks”) did not materialise, and the reason is worth recording:**
the fear was that the rungs 7–24 kernels are too slow to cover meaningfully. They are, at their
**config-default** resolution. At reduced quadrature they are 0.01–60 s — and the reduction turned
out to cost nothing in fidelity (next paragraph). The expensive thing about those kernels was never
the arithmetic under test; it was the convergence resolution the FINDING gates need.

**The one thing that could have made this slice a lie, checked rather than assumed.** § 0's
mechanism for the equilibrium-side drift is naive `sum()` reassociation, whose error grows with
TERM COUNT — so a reduced-resolution arm could systematically under-report the drift of the
full-resolution path. Re-measured at 4× the terms:

| arm | reduced | 4× terms |
|---|---|---|
| r13 | 1.85e-15 | 1.85e-15 |
| r18 | 1.31e-15 | 1.31e-15 |
| r22 | 1.34e-15 | 1.31e-15 |
| r24 | 3.26e-14 | **2.49e-14** |

**It does not grow — it falls.** The drift on these kernels is not produced by the quadrature sums
at all; it is inherited from the fixed upstream layer (the design point, the Kp solve) and the
quadrature AVERAGES it down. (r22's worst key even moves under refinement, from `ei_no_spatial` to
the shared upstream `x_no_quenched`.) So the reduced tolerances are conservative, and the
full-resolution paths the specs quote are if anything tighter than what is gated.

**Per-arm drift and headroom.** Same one-round-decade rule; the full table is in
`M:\claud_projects\temp\pypy-fingerprint\SLICE2-FINDINGS.md`. Across all **26** committed arms,
PyPy consumes at worst **10.45%** of a tolerance and at least **1.31%** — the same snug band
slice 1 landed in. Worst drift anywhere on the diagnostic ladder: **1.98e-6** (`r25.dS_finite`),
against specs quoting 3–4 significant figures.

**THE FINDING THIS SLICE ADDS, beyond coverage: the drift is distributed by CONDITIONING, not by
rung.** Every arm at ~1e-15 reads a well-conditioned quantity. Every arm above 1e-10 reads a
**difference of near-equals** (`dS_finite`, `x_no_e_exit`), a **log-ratio** (`channel_ratio`), or
runs an **iterative inverse** (`prop.T_from_pr_t` — 78 412 ulp out of a 1-ulp property difference,
the widest drift in the whole property layer). Kernel F was slice 1's lone outlier and got a
one-off explanation; with r25 and r28 beside it, it is a **class**, and the class is predictable
from how a number is formed rather than from which rung forms it. The practical consequence: the
gate's floor is set by conditioning, so *tightening constants can never move it* — only a
better-conditioned probe can.

**A dedicated separability arm.** `prop` pins the gas property layer itself (cp/h/pr/γ and their
inverses, under all four factories, plus Fork B's absolute-enthalpy interface and the rung-6 Kp
solve) — 884 values, 0.01 s. It is not there for coverage: it is there so that `prop` red means
the PRIMITIVE layer moved, while a diagnostic red with `prop` green localises the change to that
diagnostic. Nothing in slices 0–1 could tell those two apart.

**Sensitivity, measured only where it is informative** (`sensitivity2.py`): the arms with a
tolerance ≥ 1e-7, since slice 1 established that tight arms simply detect AT their tolerance.
`r25` detects 1e-5, `r28` detects 1e-6. **The gate's floor is unchanged at 1e-5**, now reached by
two arms (F, r25) instead of one. The other sixteen were not swept and that is recorded as a skip.

**Policy: the four heavy arms are seeded slow in `conftest.py`,** not spine-overridden. The spine
prefix `test_golden_fingerprint_*` was given only to arms measured **≤ 2 s idle**, so none can end
up in kernel E's position (7.71 s against `SLOW_SECONDS = 8.0`, i.e. tagged or not depending on box
load). The other ten carry `test_golden_kernel_*` and are slow-tagged like any FINDING sweep —
which is weaker than the spine but not by much: `--affected` re-enables the slow gates of every
module the diff can reach, and this module imports `turbojet.gas` and `turbojet.engine`, so a rung
commit runs them on the **ship** gate. The seed entry exists because a cold cache has no learned
duration, so without it the first `pytest` in a fresh clone pays the full cost once.

**A guard the slice needed and slice 1 did not.** `test_every_kernel_has_a_disclosed_tolerance`
asserts `KERNELS == TOL == goldens`, which still fires if a tolerance is forgotten — but all three
sets can agree while a kernel has **no test function calling `_check` on it**: it would be
generated, pinned, counted in the 8 044, and never verified. At eight arms that was
eyeball-checkable; at twenty-six across two naming conventions it is not, and the failure is
silent in the worst direction (a green suite hiding an unchecked kernel). So
`test_every_kernel_is_actually_GATED` reads the gates' own source for `_check("…")` and asserts
every kernel is reached. Verified to fire by adding a synthetic ungated kernel.

**Side-finding — dead code in rung 28.** `CoupledNOFreezeOutState.no_collapse_ratio` reads
`self.x_no_e_entry`, a field copied from `NozzleFlowState` that this dataclass does not have; it
raises `AttributeError` unconditionally and nothing calls it (rungs 14/17 read the real one off
`NozzleFlowState`). **Not repaired here** — it is production code and outside this slice — but not
hidden either: `_floats_of` now pins the exception TYPE as a value, so the day it starts or stops
raising, the gate reports it as the shape change it is.

**What this slice did NOT do:** no arm runs at a spec's full quoted resolution (above), and
`main.py` is still covered by nothing (slice 4 item 4).

### ⚠ Two jobs are being conflated here — keep them apart when sequencing

- **Validating TODAY'S switch** — is PyPy's arithmetic close enough to CPython's that no quoted
  digit moves? **§ 0 already answered this**: 133 equilibrium-side values, 96 CPG ladder values,
  6 152 trajectory floats, worst drift 3.7e-6 against specs that quote 3–4 significant figures.
  Broadening that evidence needs only a bigger *script*, which is hours, not a committed gate.
- **Detecting FUTURE drift** — a permanent gate against a PyPy upgrade re-rolling `expm1` (R4), a
  library change, or a refactor moving a number nobody was watching. **That** is what slices 1–2
  build, and its value is interpreter-independent.

**Consequence for sequencing:** slice 4 does *not* strictly depend on the committed gate existing —
it depends on the validation being broad enough. If the golden gate turns out to be the slow slice,
switching on a broadened `fingerprint.py` while the gate lands on its own timeline is a legitimate
order. **The reverse is not:** switching with neither is what this plan exists to prevent.

---

## § 3 — SLICE 3: make the full gate green on PyPy — **MEASURED EMPTY (2026-07-31)**

This slice was planned as the unknown-size one. It is **already done, and it cost nothing.**

```
pypy3.11 -m pytest --runslow      ->  973 passed in 170.62s (0:02:50)
```

Cold cache (no learned durations, so no LPT ordering — 2:50 is a *pessimistic* wall clock;
the warm figure is 2:37). **Zero failures. Zero errors.**

**What was expected to break, and did not:** the suite carries **109 exact `==` float assertions**
(heaviest: rungs 54 and 53 at 33 each, rung 56 at 32, rung 55 at 30, rung 28 at 23). The
prediction was that some compare two paths reaching the same value by *different arithmetic* and
would trip on last bits — the precedent being the rung-30 fix exposing
`test_rung48.py::test_decel_never_fires_bit_for_bit_rung45` as a **~40% pre-existing flake**.
**Not one of the 109 tripped.** The § 0 split explains why: those gates live overwhelmingly on the
CPG ladder (rungs 41–66), which is bit-identical across interpreters, so both sides of every
comparison are not merely moving together — they are not moving at all.

**Read this result correctly — it is narrower than it looks.** 973 green proves *no exact-equality
gate trips*, which is exactly what this slice was sized for. It does **not** corroborate
CPython-vs-PyPy agreement; per the §-intro blocker, a green suite can only prove PyPy agrees with
itself. The agreement evidence is § 0's, and slices 1–2 make it permanent (see § 2's note on the
two jobs) — but they are not what makes today's switch safe. § 0 is.

**RE-RUN AFTER SLICES 1–2 — DONE (2026-07-31), and this is the result that actually licenses the
switch.** The gate grew 973 → 1 002 tests (slice 1's 10, slice 2's 19), and 26 of those now pin
absolute CPython numbers:

```
CPython 3.14.3   pytest --runslow    1001 passed in 780.79s (13:00)   [before the 1002nd, a test-only guard]
PyPy 3.11.15     pytest --runslow    1002 passed in 120.43s (2:00)    -> 6.5x
```

Run with `-o cache_dir=…` so PyPy's ~5× faster timings could not overwrite the learned durations
that drive CPython's fast/slow partition — the split-cache footgun § 4 names, avoided rather than
discovered. **Unlike § 3's original 973-green run, this one is NOT just "PyPy agrees with itself":
1 663 of those assertions are against absolute values CPython produced.**

**Residual work:** keep the repair shape on file in case a future run does trip one
— **by construction, not by tolerance**, the way rung 48's was fixed (comparison starts at row 1,
row 0 gated as the ulp artifact it is; bit-exactness kept everywhere the contract actually claims).

---

## § 4 — SLICE 4: the switch itself

**Deliverable:** PyPy is the interpreter; the CPython path is deleted, not maintained beside it.

1. **A durable install location.** PyPy currently lives at `M:\claud_projects\temp\pypy` — which
   by the standing convention is the *regenerable temp* area. A canonical interpreter cannot live
   in a folder that is safe to delete. Move it, and document the install so the repo is
   reproducible for anyone who clones it.
2. **Config:** `requirements.txt`, `pytest.ini`, `CLAUDE.md` § Commands + § Stack.
3. **Re-learn the durations cache — and re-tune `SLOW_SECONDS`.** This is not bookkeeping, it is a
   behaviour change: the threshold is `8.0 s` against timings that are about to get 5× faster, so
   the fast/slow partition silently re-cuts and most currently-slow tests join the bare `pytest`
   subset. Either re-tune to ~1.6 s to preserve today's partition, or accept the re-cut
   deliberately — but decide it, do not inherit it. (Related: `todo-engine-size-and-speed.md`
   already flags that below-normal priority makes learned durations depend on what else the box
   is doing. Re-learn on an idle machine.)
4. **Verify `python main.py`** — the design-point run, the T–s diagram, every rung panel. It is
   the actual deliverable and **no test covers it** (`CLAUDE.md`'s own accepted risk). matplotlib
   is installed under PyPy; that it *renders* is unverified.
5. **Retire the CPython-side stale numbers** in `todo-engine-size-and-speed.md` and add the § 0
   scope qualifier to its bit-identity claim.

**Done when:** a clean clone + documented install reproduces a green `--runslow` and a working
`main.py`, with no CPython dependency anywhere in the tree.

### RESULT — DONE (2026-07-31)

**Green: `--runslow` = 1002 passed in 167.83 s (2:47)**, against **1047.53 s (17:27)** for the
identical 1002 tests on CPython — **6.2× on wall clock**, 6.58× on summed call-time (per-test p10
3.38× / median 5.96× / p90 8.30×, non-uniform because the JIT amortises over long tests). Bare
`pytest` = **778 passed in 79.77 s (1:20)**. § 5 predicted "2:37–2:50 for everything" — measured
2:47, inside the predicted band.

1. **Install location — DONE.** `M:\claud_projects\tools\pypy3.11-v7.3.23-win64`, out of the
   regenerable temp area. The repo venv `.venv` is built from it and gitignored; nothing global
   changed, so other projects keep CPython. `requirements.txt` documents the two-line install.
2. **Config — DONE.** `requirements.txt` rewritten (and see the psutil finding below);
   `CLAUDE.md` § Commands + § Stack re-measured. `pytest.ini` needed **no change**.
3. **`SLOW_SECONDS` — DELIBERATELY UNCHANGED at 8.0.** See below; this is the substantive item.
4. **`main.py` — VERIFIED**, exit 0, 6m01s, and diffed against a CPython run. See below.
5. **Stale numbers — DONE** in `todo-engine-size-and-speed.md` (+ the § 0 scope qualifier) and in
   `conftest.py`, whose module docstring, `SLOW_SECONDS` comment, `_SEED_SLOW`, `_is_spine`
   docstring and § affected-set header were ALL carrying CPython measurements.

#### The threshold: a rescale would have been backwards

The plan proposed "re-tune to ~1.6 s to preserve today's partition, or accept the re-cut". Both
options were refused, on data:

- **The budget stopped binding.** At *every* candidate threshold bare `pytest` fits the ~5 min
  budget — the full gate itself is 2:47. So "preserve the partition" optimises a constraint that
  no longer exists.
- **Rescaling is not even a cost cut.** At 1.2 s (= 8.0/6.58) the duration route tags **167**
  tests; at 0.5 s, **249**. CPython's 8.0 tagged **159**. A rescale tags MORE, not fewer.
- **1–3 s is the noise band.** PyPy attributes JIT warm-up to whichever test first touches a code
  path on a worker, so trivial tests record seconds they do not cost: rung 17's
  `test_identity_is_witnessed_not_a_test` 0.00 s → **2.67 s**; rung 23's
  `test_correlation_concentrated_under_penetration` 0.35 s → **4.70 s**. A threshold there would
  flip tags run-to-run.
- **8.0 s is already the safe answer.** It tags 30, and against the CPython partition that is
  **0 newly slow / 107 newly fast** — every disagreement in the safe direction. 107 gates that
  `--runslow` used to own now run on *every* invocation, free.

**The constant survives, with its reason inverted: under CPython it bought TIME; under PyPy it
buys DETERMINISM.** The seed set WAS regenerated (61 → 27 pairs; 42 stale entries, of which 20
were spine-overridden and had always been inert), excluding by construction anything whose
CPython duration was < 1 s so a warm-up artefact cannot be frozen into the cold-cache path.

#### CORRECTION — the threshold governs a MINORITY of the partition

The plan's framing ("the fast/slow partition silently re-cuts") **over-stated what `SLOW_SECONDS`
controls**, and the error was only caught by the closing gate: bare `pytest` deselects **224**
tests, but the conftest duration route accounts for just **30**. The other ~194 come from
**122 explicit `@pytest.mark.slow` decorator sites across 18 test files**.

So the partition is **87 % hand-declared, 13 % measured** — the threshold is a *backstop for
gates nobody marked*, not the policy. This also corrects `CLAUDE.md`'s own "no test file is
edited", which was true of the *policy* but invited exactly the inference that got made here.
Both `conftest.py`'s docstring and `CLAUDE.md` now state the split. **Consequence for § 5:**
collapsing the three-gate policy is a bigger job than retiring `conftest.py`'s machinery — it
must also decide what to do with 122 author-declared markers, which are a statement of intent
that no timing overturns.

#### `main.py`: one line moved, and it is the informative one

Full-resolution output, CPython vs PyPy: **151,249 bytes both, one line differs.** In the rung-32
panel, `tau_c rel` reads `7.0e-10` (CPython) vs `6.9e-10` (PyPy). That line prints a **convergence
residual**, not a physical state — and it is the *only* such printed quantity. Print precision
(2–4 s.f.) protects a physical number because drift is ~1e-6, decades below the last printed
digit; it gives a residual no protection at all, because there the printed value **is** the noise.

So: **the ~1e-3 print-precision sensitivity bound holds for every physical number `main.py`
emits, and does not extend to printed residuals.** The claim that line supports ("the work
`tau_c` is choke-pinned, map-free to ~1e-6") survives by three decades under both interpreters.
This does **not** tighten § 0's bound — `main.py` prints far too coarsely to be a numeric gate;
it confirms the specs' quoted figures are safe. (`ts_diagram.png` changed 83,057 → 80,058 B: a
matplotlib build difference between the two environments, not physics.)

#### A THIRD friction, found the hard way: `psutil` is load-bearing

pytest-xdist counts **physical** cores via psutil and silently falls back to `os.cpu_count()` —
**logical** cores — without it. On this 8-physical/16-logical box that turns `-n auto` into
`-n 16`, which was measured (1.30×) and **declined** because it makes the machine sluggish.
CPython's environment happened to carry psutil, so the decline held *by accident*. Installing it
under PyPy is what makes `-n auto` mean the same thing after the switch (verified: `8/8 workers`).
It is pinned in `requirements.txt` with this reasoning, because nothing else in the tree would
reveal it.

#### A NEW property of the interpreter — disclosed, not fixed

The learned-duration cache is now **schedule-dependent** in a way it was not under CPython (the
JIT-warm-up attribution above). Both the fast subset and `--affected` read that cache, so a test
near *any* threshold can flip side between runs. At 8.0 s nothing sits near the line, so it does
not bite today — but it is a property of the interpreter, not a bug, and it is recorded in
`conftest.py`'s docstring rather than omitted for being currently harmless.

#### The one remaining CPython dependency — deliberate, and NOT a violation

"No CPython dependency anywhere in the tree" holds for **running** the project: model, plot, and
all three gates are PyPy-only. The exception is **regenerating** `tests/golden/numeric_fingerprint.json`,
which by slice 1's design is a committed **CPython anchor** — regenerating it under PyPy would
silently redefine the reference the gate exists to hold. That is not a dependency of the repo;
it is a property of the anchor, and the provenance guard (`meta.implementation`) enforces it.
The goldens were regenerated on CPython 3.14.3 **before** the switch, with the key delta
predicted first and verified after (0 moved / 2 removed / 0 added — the two rung-28
`no_collapse_ratio!raises` keys, from the dead-property deletion).

---

## § 5 — SLICE 5 (OPTIONAL, unlocked by the switch): collapse the three-gate policy

**MEASURED (slice 4, 2026-07-31) — the prediction held and the premise is now solid.** The full
gate is **167.8 s (2:47)**, inside the 2:37–2:50 predicted here; bare `pytest` is **79.8 s
(1:20)**. The caution above is discharged: the `~5 min` figure WAS stale-high, and so was the
`~13 min` full-gate figure — CPython's real full gate was **1047.5 s (17:27)**, corroborated
independently by its summed call-time ÷ 8 workers ≈ 1022 s. So the old numbers were stale, not
the runs contaminated. The full gate is now **~2× faster than the fast subset used to be**, which
is the condition this slice was waiting for.

Retiring the tiering would remove `conftest.py`'s slow-tagging, its seed set, and `--affected`'s
AST symbol-diff + caller-closure machinery: a large amount of process complexity that exists
purely because the gate was expensive.

**BUT — slice 4 found this slice is bigger than written.** The duration threshold accounts for
only **30 of the 224** deselected tests; the other ~194 come from **122 explicit
`@pytest.mark.slow` decorators across 18 test files**. Deleting `conftest.py`'s machinery would
therefore leave 87 % of the partition standing. This slice must decide what happens to those
author-declared markers — and they are a *declaration of intent* that no timing overturns, so
"they're fast now" is not by itself an argument for removing them.

**Deliberately optional, and deliberately last.** It is a real simplification and the biggest
second-order prize of the switch, but it deletes machinery that took work to build, and it should
be decided after living with the faster gate for a few rungs — not in the same motion.

---

## Risks — what could stall or reverse this

- **R1 — slice 2 is bigger than it looks.** The rungs 7–24 diagnostics are the most numerous and
  slowest kernels in the project. If coverage cannot be made both meaningful and affordable, the
  switch should be reconsidered rather than shipped on partial evidence.
- **R2 — asserting bit-equality on the CPG arms is itself a flake surface.** It is measured true
  today across two interpreters, which is strong. But it is the same class of claim that made the
  rung-48 gate a 40% coin flip. If it proves fragile, demote those arms to a tight tolerance and
  say so — do not quietly relax the rest.
- **R3 — the 3.11 language pin is permanent.** Nothing in the tree needs 3.12+ today (verified),
  so this costs nothing now. It is a door closed, not a bill paid.
- **R4 — a future PyPy upgrade re-rolls `expm1`/`log1p`/`erf`.** This is precisely what slices 1–2
  exist to catch. Without them the switch is unsafe; with them it is a test failure like any other.
- **R5 — PyPy is a smaller project than CPython.** Low exposure here: the model is stdlib +
  matplotlib and both work today. But a future dependency without PyPy wheels would bite.

## Deliberately NOT part of this

- **Not `-n 16`.** It is a 1.30× on top, and it was declined because the box goes sluggish. The
  below-normal-priority fix addresses CPU scheduling only, not memory bandwidth.
- **Not a Rust extension.** `todo-engine-size-and-speed.md` item 4 stays where it is; slice 1 of
  that ladder already took the algorithmic win, and a full rewrite is refused on the project's own
  terms ("the deliverable is understanding, not the tool").
- **Not a hybrid / opt-in second interpreter.** That was the first recommendation and it was
  wrong: dual environments and the split durations-cache footgun are costs *of the hybrid*, which
  a clean switch deletes outright.
- **Not a re-derivation of any anchored number.** If a spec's quoted digits ever have to change to
  make this pass, that is a finding to stop and investigate — not a step in this plan.
