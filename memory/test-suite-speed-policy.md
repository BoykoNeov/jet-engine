---
name: test-suite-speed-policy
description: "Test suite is fast-by-default: bare pytest = ~2.5min FAST subset (deselects slow FINDING gates but KEEPS the reduce spine); pytest --runslow = all 371 (~10-15min, the commit gate). Config in pytest.ini + conftest.py; NO test file edited."
metadata: 
  node_type: memory
  type: project
  originSessionId: 1c258a79-b4c7-4891-ab3d-022937f8d1a3
  modified: 2026-07-21T20:28:25.585Z
---

The full pytest suite was **49 min serial**; optimized to **~2.5 min routine / ~10–15 min full**
(2026-07-21). Achieved WITHOUT editing any test file — the rung gates stay pristine (the
derive/reduce spine is the project's whole point). All policy lives in `pytest.ini` + `conftest.py`.

**How it works:**
- `pytest.ini`: `addopts = -n auto --dist load --maxschedchunk=1` — PARALLEL (16-way). Requires
  `pytest-xdist` (added to requirements.txt).
- `conftest.py`:
  - **Fast/slow selection.** A test is `slow` if its LEARNED per-test call-duration ≥ 8 s (recorded
    to `.pytest_cache` every run — master-only write to dodge the xdist race; seeded from a baseline
    run so a cold checkout is already fast). Bare `pytest` DESELECTS slow; `--runslow` runs all;
    `-m slow` runs only slow. The slow set is the expensive FINDING / robustness sweeps (mixing-PDF
    per-pocket quench of rungs 16/20–24, transient marches).
  - **The SPINE override (`_is_spine`) — user's explicit choice.** The bit-for-bit REDUCE gates
    (`test_reduce_*`, `test_cycle_untouched_*`, `*_bit_for_bit`) are **NEVER slow-tagged** even when
    expensive, so bare `pytest` ALWAYS guards "each rung reduces to its predecessor, exactly and by
    test." This is why the fast default is ~2.5 min not ~1.2 min: rung 21's `test_reduce_super_eq_o_
    false_is_bit_for_bit` (120 s) and rung 23's `test_cycle_untouched` (117 s) are the fast-default
    floor. The user was offered fast-1.2min-spine-at-commit-only vs this, and chose spine-every-time.
  - **LPT scheduling.** Collection is reordered LONGEST-FIRST, **interleaved from both ends**
    ([longest, shortest, 2nd-longest, ...]) so xdist's initial 2-tests-per-worker seed can't stack
    the two longest poles (365 s + 273 s) on one core. Pure scheduling hint — tests are
    order-independent (parallel baseline proved it). Got the full run 13→10 min. The hard floor is
    rung 24's `test_ei_stays_monotone` (~365 s, an aggregate 5-point J sweep whose monotone assert
    needs all points — UNsplittable without coarsening its grid, a physics risk deliberately avoided).

**THE COMMIT GATE IS `pytest --runslow`, NOT bare `pytest`.** Bare pytest skips the expensive FINDING
gates. Updated [[session-end-routine]] + [[always-commit-and-push]] to require `--runslow` before any
green-commit. (`.pytest_cache/` is gitignored — the learned-durations file is NOT committed; the seed
in conftest is what a fresh clone relies on.)

**Advisor caught the load-bearing thing:** my duration-based slow-tag was silently sweeping up the
reduce SPINE gates, and my standing "always commit green" memory would then push on a spine-skipped
run. Fix = the spine override + the two memory edits. Don't tune the full-run scheduler further —
diminishing returns past the 365 s single-test floor; grid-coarsening is the only way lower and it's
a physics risk. See [[rung37-combustor-dynamics]] (the rung whose slow marches motivated this).
