"""Test-suite speed policy — ONE gate. Everything runs by default.

Policy (see the Commands section of CLAUDE.md):
  * `pytest`                 -> runs EVERYTHING. 1002 tests, 2:18 (PyPy, -n auto, idle box).
                                THIS IS THE GATE. Nothing is ever silently deselected.
  * `pytest -m "not slow"`    -> the iteration opt-out: skips the expensive FINDING sweeps, 1:31.
                                A convenience you TYPE, never a default you inherit.
  * `pytest -m slow`         -> only the expensive gates.

`slow` is therefore a LABEL, not a policy. It marks an inherently-expensive gate so you can opt
out of it while iterating; no code here reads it to decide what a plain `pytest` runs.

WHY THE THREE-GATE POLICY WAS RETIRED (2026-07-31, docs/plans/todo-pypy-switch.md slice 5).
Until the PyPy switch this file ran a tiered policy: a fast default, `--runslow` for everything,
and `--affected` (a git + AST symbol-diff + caller-closure selector, ~185 lines) as the per-rung
ship gate. All three existed for ONE reason — the full gate cost 17:27, so the only lever on it
was running fewer tests. PyPy took it to 2:47 and that reason went away.

The slice was held up by an argument that turned out to be about the wrong quantity. It ran: the
duration threshold accounts for only 30 of the 224 deselected tests, the other ~194 are explicit
`@pytest.mark.slow` decorators, so retiring the machinery would leave 87 % of the partition
standing. That is a COUNT argument, and MEASURED BY COST IT INVERTS:

    route                        nodeids      call-time   median    share of deferred cost
    the 194 hand-marked ones     87 %           196 s     0.30 s          28 %
    the 30 threshold-tagged      13 %           516 s        —            72 %

139 of the 194 hand-marked tests run in under a second. The hand-marked majority had stopped
costing anything; the automatic minority held all the money — AND those 30 are the project's
headline FINDING gates (rung 24's ⟨EI⟩-monotone negative, rung 46's relief split, the three heavy
golden kernels). The tiering had inverted its own purpose: it was deferring the gates that matter
most and running the cheap corroborating ones on every invocation.

So the default was INVERTED rather than the tier deleted. Everything runs; the 27 functions route
(2) used to tag automatically now carry an explicit `@pytest.mark.slow` (they are named in their
own test files, where the author can see them), and `-m "not slow"` reproduces the old fast subset
exactly. Three things fall out of the inversion, and they are the actual prize:

  (a) `CLAUDE.md`'s accepted risk — "a regression in an unreached non-spine gate can hide for
      <= 3 rungs" — is GONE. There is no unreached gate.
  (b) `_is_spine` is gone with it. The bit-for-bit reduce gates were force-run because a fast
      default could otherwise drop them; a default that runs everything protects them by
      construction, so the override has nothing left to override.
  (c) SLOW_SECONDS is gone, and with it a REAL nondeterminism hazard. PyPy attributes JIT warm-up
      to whichever test first touches a code path on a worker, so a trivial test can record
      seconds it does not cost (measured: rung 17's `test_identity_is_witnessed_not_a_test`,
      0.00 s on CPython -> 2.67 s here). Under the old policy a test near the threshold could flip
      SIDE between runs. The learned cache is still read below, but only to ORDER the run, so the
      same schedule-dependence can now cost wall clock and nothing else.

HONEST COST. The per-rung ship gate went from a narrowed ~1 min to the full 2:47, and `--affected`
was a real piece of work. It is deleted rather than kept-in-case: its whole justification was a
cost that no longer exists, and an unused selector that silently mis-selects is worse than none.

`--runslow` SURVIVES AS AN ACCEPTED NO-OP. It is not deprecation politeness: three test files
(`test_rung42/53/55`) pass it from their `__main__` blocks, and several specs record reproduction
commands that carry it (e.g. `docs/both-edges-limiter-negative.md`). Its meaning was "run
everything", which is now the default — so honouring it is CORRECT, not compatibility cruft.

Parallelism (`-n auto --dist load --maxschedchunk=1`, set in pytest.ini) is orthogonal and is
already at its floor: this box has 8 PHYSICAL cores behind 16 logical, and an LPT pack of the
measured durations onto 8 workers models 93 % of the observed wall clock, so there is no
scheduling slack left to reclaim. The hard floor is the single longest test, rung 24's per-pocket
⟨EI⟩ sweep (~46-53 s under load), so no selection takes a run below ~1 min.

HOW MUCH THE INTERLEAVE IS WORTH — measured by accident, 2026-07-31, and worth more than the
model said. A `-n0` diagnostic run wrote a duration of 0.00 s for rung 24's `test_ei_stays_monotone`
into the cache (single-worker, so it HIT the module memo described below). The next full gate
therefore ranked the suite's biggest pole as its CHEAPEST test and scheduled it LAST: it ran
52.74 s on the tail and the gate took 3:07. Re-recording the cache from that same run and
re-running gave 2:18 — SAME tests, same box, one variable. So the ordering below is worth ~26 %
of the full gate, and the cache that feeds it is spoiled by any `-n0` run. That is not a bug to
fix (it self-heals on the next parallel run) but it IS why a stray single-worker run can make the
next gate look slow.

MEASURED, and recorded because it will otherwise be re-derived: that ⟨EI⟩ sweep is memoised at
module level, but `--dist load` plus the interleave reliably split its two readers across workers,
so it is computed TWICE (48.14 + 46.40 s recorded; 34.82 s for the pair on one worker, with the
second reader at 0.00 s). Real, and NOT worth fixing: across 8 workers the duplicate is worth ~6 s
of wall clock, and grouping by file to recover it would break the pack that the paragraph above
prices at ~50 s.
"""
import os
import sys

import pytest

_CACHE_KEY = "durations/call"


def pytest_addoption(parser):
    parser.addoption("--runslow", action="store_true", default=False,
                     help="accepted and ignored — everything runs by default. Kept because the "
                          "test files' __main__ blocks and several specs' recorded reproduction "
                          "commands pass it, and 'run everything' is now what happens anyway.")


def _run_at_below_normal_priority():
    """Drop THIS process to below-normal scheduling priority. Called from `pytest_configure`,
    which runs in the controller AND in every xdist worker, so one call covers the whole fleet.

    WHY. `-n auto` packs one worker onto every physical core, so a gate saturates the machine and
    the box goes sluggish for whatever else is running on it. Priority is the right lever rather
    than surrendering a worker: a below-normal process is preempted only when something else
    actually wants the CPU, so an otherwise-idle machine still runs the gate at full speed — the
    cost is zero when nobody is competing, unlike `-n 6`, which pays unconditionally.

    HONEST SCOPE. This governs CPU scheduling only. Memory-bandwidth and L3 contention are NOT
    priority-governed, so a fully-packed run can still feel heavy on memory-bound work. If that
    bites, drop the worker count (`-n 6`) — that is the lever priority cannot pull.

    Opt out with `JET_TEST_NICE=0`. Never raises: a failure here must not fail a test run.
    Returns True if the priority actually moved — the caller reports it, because the Win32 call
    fails SILENTLY (see below) and a courtesy that quietly does nothing is worse than none.

    THE TRAP, measured: `kernel32.SetPriorityClass(kernel32.GetCurrentProcess(), ...)` returns
    **0** here. `GetCurrentProcess()` hands back the pseudo-handle `(HANDLE)-1`, and ctypes'
    default `restype` is `c_int`, which truncates it on 64-bit — so the call gets a bad handle,
    fails, and raises NOTHING. `restype = c_void_p` is load-bearing, not tidiness.
    """
    if os.environ.get("JET_TEST_NICE", "1") == "0":
        return False
    try:
        if sys.platform == "win32":
            import ctypes
            BELOW_NORMAL_PRIORITY_CLASS = 0x00004000
            k32 = ctypes.windll.kernel32
            k32.GetCurrentProcess.restype = ctypes.c_void_p      # see THE TRAP above
            handle = ctypes.c_void_p(k32.GetCurrentProcess())
            if not k32.SetPriorityClass(handle, BELOW_NORMAL_PRIORITY_CLASS):
                return False
            return k32.GetPriorityClass(handle) == BELOW_NORMAL_PRIORITY_CLASS
        # Read-first, because `os.nice` is a RELATIVE bump while `SetPriorityClass` is absolute:
        # a worker that already inherited nice 5 would otherwise drift to 10, and its children
        # further still. Keep the two platforms idempotent alike.
        target, current = 5, os.nice(0)
        if current < target:
            os.nice(target - current)
        return os.nice(0) >= target
    except Exception:
        return False            # best-effort courtesy, never a gate failure


def pytest_configure(config):
    config.addinivalue_line("markers", "slow: an inherently-expensive gate. A LABEL for the "
                                       "`-m \"not slow\"` iteration opt-out — it does NOT "
                                       "deselect anything from a plain `pytest`.")
    config._jet_niced = _run_at_below_normal_priority()


def pytest_collection_modifyitems(config, items):
    """Order the run LONGEST-FIRST-INTERLEAVED. This hook no longer SELECTS anything — selection
    is pytest's own `-m` and nothing else — so it can only change wall clock, never results.

    LPT scheduling: get every multi-minute pole started at t=0 so the makespan approaches the
    single longest test rather than a stacked tail. xdist hands items to workers in collection
    order AND seeds TWO tests per worker up front — so a naive longest-first order would pair the
    two longest poles on one worker (rung-24's 365 s scan + the 273 s one = 640 s on a single
    core). Instead interleave the cost-sorted list from both ends ([longest, shortest,
    2nd-longest, 2nd-shortest, ...]): every worker's initial pair is one long pole + one
    sub-second filler, and the long poles fan out across the workers. The tests are independent
    (the parallel baseline proved order-independence), so this is a pure scheduling hint.
    """
    cache = getattr(config, "cache", None)
    durations = cache.get(_CACHE_KEY, {}) if cache is not None else {}

    def _cost(item):
        rec = durations.get(item.nodeid)
        if rec is not None:
            return rec
        return 1e6 if item.get_closest_marker("slow") else 0.0   # cold cache: marked gates go early

    ranked = sorted(items, key=_cost, reverse=True)
    interleaved, lo, hi = [], 0, len(ranked) - 1
    while lo <= hi:
        interleaved.append(ranked[lo])
        lo += 1
        if lo <= hi:
            interleaved.append(ranked[hi])
            hi -= 1
    items[:] = interleaved


# --------------------------------------------------------------------------- duration learning
# Accumulate call-phase durations on the controller (in xdist the controller receives every
# worker's report), then persist once at the end. Writing only in sessionfinish — and only
# where there is no `workerinput` (i.e. the controller / a non-distributed run) — avoids the
# cache-clobber race that per-worker writes would cause. The cache feeds the LPT order above and
# NOTHING else, so a stale or missing entry costs wall clock, never coverage.
_RECORDED: dict = {}


def pytest_runtest_logreport(report):
    if report.when == "call":
        _RECORDED[report.nodeid] = report.duration


def pytest_report_header(config):
    """Report the priority drop rather than assume it: the Win32 call fails silently (see
    `_run_at_below_normal_priority`), and a courtesy that quietly no-ops is worse than none."""
    if not getattr(config, "_jet_niced", False) and os.environ.get("JET_TEST_NICE", "1") != "0":
        return ["gate: WARNING — could not drop to below-normal priority; the run will "
                "compete with your foreground apps"]
    return []


def pytest_sessionfinish(session, exitstatus):
    config = session.config
    cache = getattr(config, "cache", None)
    if cache is None:
        return
    if hasattr(config, "workerinput"):     # an xdist worker — the controller does the write
        return
    if _RECORDED:
        stored = cache.get(_CACHE_KEY, {})
        stored.update(_RECORDED)           # last-seen wins; keeps durations for tests not run this time
        cache.set(_CACHE_KEY, stored)
