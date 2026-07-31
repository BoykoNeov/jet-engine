---
name: test-suite-speed-policy
description: "ONE gate since 2026-07-31: `pytest` runs EVERYTHING (1002 tests, 2:18). `pytest -m \"not slow\"` (1:31) is a typed iteration opt-out, never a default. The three-gate tiering (--affected, --runslow, SLOW_SECONDS) was deleted — its blocker was a COUNT argument that inverts when measured by COST."
metadata: 
  node_type: memory
  type: project
  originSessionId: 1c258a79-b4c7-4891-ab3d-022937f8d1a3
  modified: 2026-07-31T10:32:35.340Z
---

**ONE GATE (2026-07-31, slice 5 of the PyPy plan — user's call: "make all tests run by default").**
- `pytest` — **EVERYTHING**, 1002 tests, **2:18**. THE gate. Nothing is ever silently deselected.
- `pytest -m "not slow"` — **1:31**, 778 tests. The iteration opt-out. A convenience you **TYPE**,
  never a default you inherit. `pytest -m slow` = only the 224 expensive ones.
- `--runslow` survives as an **accepted no-op** (test files' `__main__` blocks and specs' recorded
  reproduction commands pass it; "run everything" is now the default, so honouring it is correct).

**WHEN to run it** (user, 2026-07-31): at **session end** if not run shortly before, and after a
**code** change. **NOT** at session start, **NOT** on a docs-only change, **NOT** "just to be sure".

**THE FINDING that unblocked it — a COUNT argument that inverts under COST.** The plan had recorded
a blocker: the `SLOW_SECONDS` threshold tags only 30 of the 224 deselected tests, the other ~194 are
hand-written `@pytest.mark.slow`, so deleting the machinery leaves 87 % of the partition standing.
Measured instead by **time**: the 194 hand-marked cost **196 s** total (median **0.30 s**; 139 of
them under 1 s, 28 % of the deferred cost) and the 30 automatic ones cost **516 s** (72 %) — *and
those 30 are the headline FINDING gates* (rung 24's ⟨EI⟩ negative, rung 46's relief split, the heavy
golden kernels). **The tiering had inverted its own purpose: deferring the gates that matter most,
running the cheap corroborating ones every time.** Generalise: *when a partition is defended by a
count, price it before you accept the defence.*

**What shipped: the DEFAULT was inverted, not the tier deleted.** The 27 functions the threshold
tagged now carry an explicit `@pytest.mark.slow`, so `-m "not slow"` reproduces the old fast subset
**exactly** (verified 1002 / 224 / 778, unchanged). Deleted: `SLOW_SECONDS`, `_SEED_SLOW`,
`_is_spine`, the deselection hook, and ~185 lines of `--affected`. `conftest.py` 589 → 217 lines.
Three consequences, and they are the prize:
1. The **ACCEPTED RISK is gone** — "a regression in an unreached non-spine gate can hide for ≤3
   rungs" has no referent when no gate is unreached. The every-3rd-rung cadence went with it.
2. **`_is_spine` deleted itself.** The reduce gates were force-run because a *fast default* could
   drop them; a default that runs everything protects them by construction.
3. A real **nondeterminism hazard** went with `SLOW_SECONDS` (PyPy's JIT-warm-up attribution let a
   test near the threshold flip SIDE between runs). The durations cache is still read — but only to
   ORDER the run, so that effect now costs wall clock and nothing else.

**⚠ ONE CONCERN TRANSFERRED RATHER THAN CLOSING (advisor caught this; I had written "evaporates").**
`SLOW_SECONDS` also MAINTAINED the marker set — a test that got expensive got tagged whether or not
its author noticed. What replaced it is a hand-kept **snapshot**. For `pytest` that is safe: an
unmarked expensive gate still RUNS, and the symptom is a visibly slower gate. **`-m "not slow"` has
no backstop at all**, and it is the documented iteration loop — a future unmarked sweep will inflate
it silently. If it drifts from 1:31, **mark the new offender**; do NOT reintroduce a threshold
(that trades the automation for hazard 3, on the command where being wrong is cheapest).
*Generalise: deleting a mechanism that served two purposes closes one and moves the other — name
which is which.*

**THE SCHEDULER IS WORTH ~26 %, measured by accident.** A `-n0` diagnostic wrote **0.00 s** for rung
24's `test_ei_stays_monotone` into the cache (on one worker it HITS the module memo). The next gate
therefore ranked the suite's **biggest pole as its cheapest test** and scheduled it **last** — it ran
52.7 s on the tail, gate 3:07. Re-recording and re-running: **2:18**. Same tests, same box, one
variable. So: the LPT interleave is worth ~50 s, and **a stray `-n0` run spoils the next gate's
schedule** (self-heals on the following parallel run). Previously this was justified only by "the
pack models 93 % of wall clock" — now it has a direct price.

**THE SCHEDULE HAS NO SLACK — but that was never the only lever (2026-07-30).** 8 PHYSICAL cores
behind 16 logical; `-n auto` counts PHYSICAL. The pack is tight, so nothing is left in *scheduling*.
What was WRONG was the conclusion *"the ONLY lever is running fewer tests"*: **less CPU per test**
(`_sonic_throat`, 28:18 → 13:17) and **a faster interpreter** (PyPy, 5–6×) both worked. See
[[perf-sonic-throat-and-pypy]], [[pypy-switch-shipped]].

**Lessons kept from the deleted machinery** (do not rebuild it without a cost reason):
- `--affected` diffed the **AST** (top-level def/class source text), not line ranges or coverage — a
  coverage map is stale exactly when it matters, since new rung code has no coverage rows, and a
  line-range map orphaned banner comments and escalated everything. If it is ever needed again,
  that is the design; but its whole justification was a 17-minute gate that no longer exists.
- **The xdist trap that cost a debug cycle:** under `-n auto` the **WORKERS collect**, so
  `pytest_collection_modifyitems` never runs in the controller — but only the controller may write
  the cache. Anything stashed on `config` during collection is invisible to `pytest_sessionfinish`.
  Decide controller-side facts in `sessionfinish` from the config alone. (`pytest_report_header`
  also runs BEFORE collection.) **Still live** — the duration recorder relies on it.
- **When a count you predicted and a count you measured disagree, the model is wrong** — find the
  second mechanism before shipping. (That is how the 122-decorator route was found at all.)

Remaining accepted risk: **`main.py` is covered by no test.** See [[always-commit-and-push]],
[[session-end-routine]], [[golden-fingerprint-gate]].
