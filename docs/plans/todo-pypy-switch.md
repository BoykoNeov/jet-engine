# TODO — switch the project's interpreter to PyPy

**Status:** planned, not started. Not a rung. A build/infrastructure change. Raised 2026-07-31
after the measurement in § 0 refuted the scope of a claim this project had already recorded.

**Sliced into four units, of which one is already measured empty** — so the real remaining work is
**slices 1 + 2 (build the missing detector)** and **slice 4 (the switch, mostly config and docs)**.
Slice 5 is optional and deliberately deferred.

| slice | what | size |
|---|---|---|
| 0 | recon — the evidence base | **DONE** |
| 1 | the golden-fingerprint gate | **DONE** — `tests/test_numeric_fingerprint.py`, 6 381 pinned values |
| 2 | extend it across the rungs 3–30 kernels | the least certain; inventory grind |
| 3 | make the full gate green on PyPy | **MEASURED EMPTY — 973 passed, 0 failed** |
| 4 | the switch itself | small; config + docs + a durable install path |
| 5 | collapse the three-gate policy | optional, deferred by choice |

**The prize:** the full gate goes **13:17 → 2:37 (5.08×)** at the worker count `-n auto`
actually resolves to. That is the honest adoption delta — *not* the "28:18 → 1:55, ~14.8×" in
`todo-engine-size-and-speed.md`, which bundles the rung-30 algorithmic fix (already shipped, and
CPython keeps it) with an `-n 16` config that was explicitly declined.

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

**Residual work:** re-run once after slices 1–2 land (the new golden gate is itself 973+1 tests
that must be green on both), and keep the repair shape on file in case a future run does trip one
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

---

## § 5 — SLICE 5 (OPTIONAL, unlocked by the switch): collapse the three-gate policy

At 2:37–2:50 for *everything*, the three-tier policy (bare `pytest` / `--affected` / `--runslow`
13 min) has little reason to exist — the full gate would plausibly be **no slower than today's fast
subset**. Stated cautiously on purpose: `CLAUDE.md`'s `~5 min` fast-subset budget is flagged
**stale-HIGH since the rung-30 fix** in `todo-engine-size-and-speed.md`, so the comparison needs a
re-measurement on an idle box before it is quoted as a result. That would retire
`conftest.py`'s slow-tagging, its seed set, and `--affected`'s
AST symbol-diff + caller-closure machinery: a large amount of process complexity that exists
purely because the gate was expensive.

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
