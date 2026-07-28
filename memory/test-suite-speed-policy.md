---
name: test-suite-speed-policy
description: "Three gates: pytest = ~5min FAST subset (iteration), pytest --affected = ~6-16min per-rung SHIP gate (slow gates only where the git diff can reach), pytest --runslow = ~22min full, every 3rd rung. Reduce spine runs on ALL three. Config in pytest.ini + conftest.py; NO test file edited."
metadata: 
  node_type: memory
  type: project
  originSessionId: 1c258a79-b4c7-4891-ab3d-022937f8d1a3
  modified: 2026-07-28T13:16:26.009Z
---

The suite was **49 min serial** → **~5 min routine / ~22–25 min full** (2026-07-21), then the
per-rung cost was cut again with `--affected` (2026-07-28). All policy lives in `pytest.ini` +
`conftest.py`; **no test file is edited**, so the derive/reduce spine stays pristine.

**THE WALL-CLOCK FLOOR IS PHYSICAL — measured 2026-07-28.** This box has **8 PHYSICAL cores
behind 16 logical**. Total suite CPU is ~12.6 ks over 744 tests; an LPT pack onto 8 workers is
1581 s, and the observed full run is 1331 s. **The schedule has no slack left** — do NOT try to
tune scheduling further. The ONLY lever on gate cost is *running fewer tests*. (The single
longest test, rung 24's `test_ei_stays_monotone` at ~518 s, is a hard per-test floor.)

**The three gates** (see [[always-commit-and-push]] for which one gates a commit):
- `pytest` — FAST subset, ~5 min. Slow-tagged FINDING gates deselected. ITERATION ONLY.
- `pytest --affected` — ~6–16 min. **The per-rung ship gate.** Every fast test PLUS the slow
  gates of the modules the working diff can reach. Strict superset of `pytest`, strict subset
  of `--runslow`.
- `pytest --runslow` — ~22–25 min. Everything. **Every 3rd rung** (the report header nags), at
  session end, and whenever `--affected` escalates.

**How `--affected` decides** (the load-bearing design choice): rung commits are **~99 % ADDITIVE**
to `engine.py` (+404/−2 for rung 57), so `git diff` answers "what existing code moved?" directly
— which is why this is **NOT** a coverage/testmon map (a coverage map is stale exactly when it
matters, since new rung code has no coverage rows). It diffs the **AST** — top-level class/def
SOURCE TEXT, old vs new — *not* line ranges: a line-range map orphans the banner comments between
two newly-added classes and escalates the whole module (this was measured; v1 selected ~everything
and saved 0 %). Rules: a symbol NEW in this revision seeds nothing; one that CHANGED (or was
deleted) seeds a **caller-direction** closure to a fixpoint; a test module is affected if it names
anything in that closure. Result: `ComponentMap` correctly fans out to all of rungs 31–54, while
rung 55's purely-additive `StageStack` reaches only `test_rung55`.

**Two properties make the reduced gate safe:**
1. The **SPINE override** (`_is_spine`) — `test_reduce_*` / `test_cycle_untouched_*` /
   `*_bit_for_bit` are NEVER slow-tagged, so they run on EVERY invocation of all three gates.
   (User's explicit choice 2026-07-21; it is also the fast-default's ~250 s floor.)
2. The selector **ESCALATES to the full gate** whenever it cannot reason: `gas.py` /
   `components.py` / `__init__.py`, `conftest.py`, `pytest.ini`, module-level statements moved,
   an unparseable file, or no git. Every failure path returns "full". (`gas.py`/`components.py`
   have not been touched since rung 31, so this rarely fires.)

**BASELINE = the last PASSING full gate's sha** (cached), not HEAD — so `--affected` is
*cumulative* across the rungs gated since it, and a clean tree right after a rung commit still
selects that rung's changes instead of selecting nothing.

**The xdist trap that cost a debug cycle:** under `-n auto` the **WORKERS collect**, so
`pytest_collection_modifyitems` never runs in the controller process — but the controller is the
only process allowed to write the cache. Anything stashed on `config` during collection is
invisible to `pytest_sessionfinish`. Decide controller-side facts in `sessionfinish` from the
config alone. (`pytest_report_header` also runs BEFORE collection — compute lazily/memoised.)

ACCEPTED RISKS the user signed off on: a regression in an unreached non-spine FINDING gate can
hide for ≤3 rungs; and `main.py` is covered by **no test at all** (pre-existing, not introduced
by the selector — changes to it select nothing).

Advisor value here: it correctly talked me OUT of a coverage map and INTO the git-diff selector,
and told me to check where removals cluster (they cluster in shared machinery — `ComponentMap`,
the `_fuel_caps`/`integrate_fuel` chain — which is exactly what the closure must fan out from).
But its "~800 s of scheduling loss to reclaim" was **wrong** and my core-count measurement killed
it. See [[rung37-combustor-dynamics]] (the rung whose slow marches motivated the original policy).
