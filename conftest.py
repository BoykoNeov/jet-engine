"""Test-suite speed policy — fast by default, full on demand.

The full suite is dominated by a handful of inherently-expensive tests (the mixing-PDF
per-pocket-quench sweeps of rungs 16/20-24, the transient marches of rungs 29/31/34/37).
Those are the load-bearing FINDING and robustness gates; they are correct to be thorough,
but they cost minutes and you do not want them on every quick iteration.

Policy (see the Commands section of CLAUDE.md):
  * `pytest`               -> runs the FAST subset only (slow gates deselected). Routine.
  * `pytest --runslow`     -> runs EVERYTHING (every gate). Use at commit / session-end / CI.
  * `pytest -m slow`       -> runs ONLY the slow gates.

A test is "slow" by EITHER of two independent routes, and it matters which — they are very
differently sized (measured 2026-07-31, 1002 tests, 224 slow):

  (1) An EXPLICIT `@pytest.mark.slow` in the test file. 122 decorator sites across 18 files,
      expanding to ~194 nodeids — 87 % of the partition. A rung author marking their own
      FINDING sweep is the normal case, and it is a DECLARATION of intent that no timing can
      overturn: a hand-marked gate stays slow however fast the box or the interpreter gets.
  (2) This module, from LEARNED DURATION: >= SLOW_SECONDS in the last recorded run. 30 nodeids,
      13 % of the partition. Every run records per-test durations into pytest's own cache
      (`.pytest_cache`) and the next collection marks them; the set is seeded below from a full
      baseline run so the very first `pytest` in a fresh clone already skips the right tests
      without needing a warm cache.

Route (2) is therefore a BACKSTOP for expensive gates nobody hand-marked — not the policy. Do
not read a change to SLOW_SECONDS as re-cutting the suite; it can only move that 13 %. What the
threshold does keep is the property that the policy lives in ONE place and never edits a test
file to DESELECT one, so the rung gates stay pristine (the derive/reduce spine is untouched).

ONE override (`_is_spine`): the bit-for-bit REDUCE gates (`test_reduce_*`, `test_cycle_untouched_*`,
`*_bit_for_bit`) are NEVER slow-tagged regardless of cost, so bare `pytest` always guards the
"each rung reduces to its predecessor, exactly and by test" invariant — the project's spine
(user's explicit choice, 2026-07-21). Only the expensive FINDING / robustness sweeps are deferred
to `--runslow`. This is what keeps a fast routine run from silently dropping the reduce check.

Parallelism (`-n auto --dist load --maxschedchunk=1`, set in pytest.ini) is orthogonal: it
speeds BOTH the fast and the full run. It is already at its floor — this box has 8 PHYSICAL
cores behind 16 logical, and for these CPU-bound float loops an LPT pack of the measured
durations onto 8 workers is 155.5 s against an OBSERVED full run of 167.8 s (1002 tests, PyPy,
2026-07-31, idle box). The pack models 93 % of the observed wall clock, so there is no
scheduling slack left to reclaim: the ONLY lever on the full gate's cost is running fewer
tests. That is what `--affected` (below) is for. The hard per-test floor is rung 24's
`test_ei_stays_monotone` (~46 s), so no selection takes the full gate below ~1 min.

THE THIRD MODE — `--affected` (see § affected-set selection below):
  * `pytest --affected`    -> every fast test, PLUS the slow gates of the modules the working
                              diff can actually reach. A strict superset of `pytest` and a
                              strict subset of `pytest --runslow`.
This is the per-rung SHIP gate; `--runslow` becomes a periodic (every 3rd rung) full gate.

INTERPRETER NOTE (2026-07-31, the PyPy switch — docs/plans/todo-pypy-switch.md slice 4). Every
number above was re-measured; the CPython figures it replaced were 1581 s modelled / 1331-1347 s
observed / a 518 s floor. The whole suite got 6.58x faster in total call-time (per-test p10
3.38x, median 5.96x, p90 8.30x — NON-uniform, because the JIT amortises over long tests and
barely helps short ones). Two consequences that outlive the numbers:

  (a) SLOW_SECONDS was NOT rescaled, deliberately — see the note on the constant below.
  (b) The learned durations are now SCHEDULE-DEPENDENT in a way they were not under CPython.
      PyPy attributes JIT warm-up to whichever test first touches a code path on a worker, so a
      trivial test can record seconds it does not intrinsically cost (measured: rung 17's
      `test_identity_is_witnessed_not_a_test`, 0.00 s on CPython -> 2.67 s here; rung 23's
      `test_correlation_concentrated_under_penetration`, 0.35 s -> 4.70 s). Nothing sits near
      8.0 s, so this does not bite today — but both the fast subset and `--affected` read this
      cache, so a test near ANY threshold can now flip side between runs. That is a property of
      the interpreter, not a bug to fix, and it is the reason a PyPy-scale threshold was refused.
"""
import ast
import os
import re
import subprocess
import sys
import time

import pytest

SLOW_SECONDS = 8.0          # a test at/above this (call phase) is tagged `slow`
# SURVIVED THE PYPY SWITCH UNCHANGED (2026-07-31), and the reason INVERTS the original one.
# Under CPython 8.0 was a COST CUT: route (2) tagged 159 tests, on top of the ~194 hand-marked
# ones, to keep bare `pytest` near 5 min. Under PyPy the WHOLE gate is 167.8 s, so no threshold
# is over budget and the cost argument has no discriminating power left. Two things replace it:
#   * A NOISE FLOOR. Rescaling by the measured 6.58x would put the threshold at ~1.2 s —
#     precisely where the JIT-warm-up attribution described in the module docstring lives
#     (2.7-4.7 s recorded on tests that intrinsically cost ~0). Tags would flip run-to-run: the
#     same nondeterminism `_is_spine`'s golden-fingerprint override exists to prevent. And it
#     would not even cut cost: at 1.2 s route (2) tags 167, MORE than CPython's 159. At 0.5 s,
#     249. The rescale is strictly backwards; 8.0 s is ~2.5x above the warm-up band.
#   * ASYMMETRIC RISK. At 8.0 route (2) tags 30. Against the CPython partition that is 0 tests
#     newly SLOW and 107 newly FAST — every disagreement in the SAFE direction. 107 gates that
#     `--runslow` used to own now run on EVERY invocation, for free. That is the switch's real
#     risk reduction, and it needed no constant touched.
#   * Raising it buys nothing: by 12 s route (2) is already down to the 30 the SEED pins, so the
#     threshold has stopped selecting anything and only the seed is left. Lowering it is worse.
# The honest summary: under CPython this number bought TIME; under PyPy it buys DETERMINISM.
_CACHE_KEY = "durations/call"

# Seed set: (module basename, function name) pairs observed >= SLOW_SECONDS in the baseline
# `-n auto --runslow` full run (PyPy, 2026-07-31, 1002 tests; regenerated from the CPython set
# of 61 pairs, which over-tagged 42 — the seed is an OR with the learned cache, so a stale entry
# can only ever ADD a slow tag, never remove one). Function names match ALL parametrizations
# (the "[param]" suffix is stripped before comparison). The learned cache extends this over
# time; this seed only has to be right enough that the first cold `pytest` is already fast.
#
# The regeneration applied a ONE-TIME filter: an entry whose CPython duration was < 1 s was
# excluded, so a PyPy JIT-warm-up artefact could not be frozen into the cold-cache path. None
# qualified. That filter is NOT an invariant and cannot be re-applied — a test added after the
# switch has no CPython duration to check. For new tests the live mechanism is the learned
# cache, which is what it has always been; the seed is only a cold-start hint.
#
# NOTE on the cold-cache check: because this set was regenerated as exactly `pypy >= SLOW_SECONDS`,
# a cold run (seed only) and a warm one (seed OR learned) necessarily agree — 224 deselected
# either way, measured. That agreement is BY CONSTRUCTION. It confirms the regeneration was
# applied correctly; it is NOT independent evidence that 8.0 is the right threshold. The
# arguments for 8.0 are the ones on the constant above, and they stand on their own.
_SEED_SLOW = {
    "test_rung16": {"test_clamp_dormant_over_pockets",
                    "test_far_flank_erosion_vs_rung15",
                    "test_zoned_nox_matches_ei16_helper"},
    "test_rung22": {"test_derived_floor_sits_below_the_hump_peak"},
    "test_rung23": {"test_clamp_dormant_at_station4",
                    "test_correlation_adds_no_at_design_point",
                    "test_g_below_two_stream_ceiling",
                    "test_helper_matches_production",
                    "test_production_g_matches_spatialpdf"},
    "test_rung24": {"test_does_not_claim_the_emissions_global_min_location",
                    "test_ei_stays_monotone_the_emissions_optimum_is_not_recovered",
                    "test_g_below_two_stream_ceiling",
                    "test_local_rate_moves_ei_only_modestly_vs_rung23",
                    "test_production_width_matches_spatial_pdf"},
    "test_rung31": {"test_running_line_and_direction"},
    "test_rung37": {"test_heat_soak_cold_below_hot_below_adiabatic"},
    "test_rung40": {"test_finding_stability_is_rho_free"},
    "test_rung46": {"test_governor_holds_and_the_surge_relief_split",
                    "test_the_lever_fast_ramp_switches_on_lp_relief"},
    "test_rung59": {"test_p1_lp_stator_matching_is_a_no_op",
                    "test_the_clamp_blocker_stays_clear"},
    "test_rung60": {"test_p1_a_floor_pins_its_own_coordinate_so_the_composite_is_a_tautology"},
    "test_rung63": {"test_the_floor_is_disarmed_inside_the_band_and_tautological_above_it"},
    "test_rung65": {"test_the_degeneracy_is_CONSERVED_a_marginal_mode_with_an_edge"},
    # The golden fingerprint's three HEAVY arms (the rungs 16/23/24 mixing closures added by
    # slice 2 of docs/plans/todo-pypy-switch.md). Measured idle on PyPy: r24 11.9 s, r23 11.6 s,
    # r16 9.3 s (CPython: 90.9 / 91.7 / 70.5 s). Seeded because a COLD cache has no learned
    # duration, so without this the very first `pytest` in a fresh clone runs them at full cost —
    # the exact thing the seed set exists to prevent. Rung 17's arm was seeded under CPython
    # (25.0 s) and is NOT any more: PyPy runs it in 3.5 s, so it joins the fast subset. The
    # module's CHEAP arms are deliberately absent: they carry the `test_golden_fingerprint_`
    # prefix and are spine-overridden anyway.
    "test_numeric_fingerprint": {"test_golden_kernel_r16_pocket_quench_pdf",
                                 "test_golden_kernel_r23_spatial_dwell_pdf",
                                 "test_golden_kernel_r24_spatial_local_pdf"},
}


def _module_of(nodeid: str) -> str:
    # "tests/test_rung23.py::test_x[4]" -> "test_rung23"
    path = nodeid.split("::", 1)[0]
    return path.replace("\\", "/").rsplit("/", 1)[-1][:-3] if path.endswith(".py") else path


def _func_of(nodeid: str) -> str:
    # strip the "::" chain to the function, then drop any "[param]" suffix
    tail = nodeid.split("::")[-1]
    return tail.split("[", 1)[0]


def _seed_says_slow(nodeid: str) -> bool:
    return _func_of(nodeid) in _SEED_SLOW.get(_module_of(nodeid), ())


def _is_spine(nodeid: str) -> bool:
    """The reduce SPINE — the bit-for-bit 'each rung reduces to its predecessor' gates and the
    cycle-untouched gates — is NEVER slow-tagged, so bare `pytest` guards that invariant on EVERY
    run (the user's explicit choice, 2026-07-21), even for the expensive rungs. This OVERRIDES
    both the seed set and the learned cache. All three name forms are used across the suite
    (`test_reduce_*`, `test_cycle_untouched_*`, and `..._bit_for_bit` / `..._bitforbit`); every
    match is a genuine reduce/cycle gate (checked — no false positives).

    FOURTH FORM, added 2026-07-31: `test_golden_fingerprint_*`. The reduce spine compares two
    quantities computed in the SAME run, so it cannot see a change that moves both sides
    together — an interpreter swap, a library update, the rung-30 closed-form fix. The golden
    fingerprint (`tests/test_numeric_fingerprint.py`) is the absolute-value counterpart, and it
    is only worth having if it runs as often as the spine does. THAT is the whole argument, and
    it is the only one that survives; it does not depend on any timing.

    The MEASURED justification this docstring originally carried has since inverted TWICE, which
    is why it is now recorded as history rather than as the reason. It read: kernel E (the
    off-design matcher on the equilibrium gas) learned 7.71 s against SLOW_SECONDS = 8.0, a
    3.6 % margin — close enough that the tag would flip with whatever else the box was doing,
    making the gate drop in and out of bare `pytest` NONDETERMINISTICALLY. Then slice 2 grew the
    fingerprint and kernel E went to 11.51 s on CPython — CLEARLY over the line, so the override
    became load-bearing outright rather than a hedge against jitter. Then the PyPy switch took it
    to 2.86 s, a 2.8x margin BELOW the threshold, so today the override changes nothing for any
    arm it matches. Keep it anyway: the margin has moved by a factor of 4 in both directions
    inside two weeks, and the reason to run an absolute-value gate every time was never its cost.

    The pattern is deliberately NARROW — `test_golden_fingerprint`, not `test_golden` — so that
    extending the fingerprint to the expensive rungs 7-24 kernels cannot drag a multi-minute gate
    into the fast subset by accident. That narrowness is doing real work: the `test_golden_kernel_*
    r16/r23/r24` arms are 9-12 s even on PyPy and are seed-tagged slow. Anything slower belongs
    behind `--runslow` like every other FINDING sweep."""
    f = _func_of(nodeid)
    return (f.startswith("test_reduce") or f.startswith("test_cycle_untouched")
            or f.startswith("test_golden_fingerprint")
            or "bit_for_bit" in f or "bitforbit" in f)


# ------------------------------------------------------------- § affected-set selection
# The full gate is 167.8 s and cannot be scheduled below ~155 s (see the module docstring), so the
# only lever is to run fewer tests. `--affected` re-enables the SLOW gates for just the modules
# the working diff can reach, and leaves every fast test in place. Two properties make the
# risk acceptable: the reduce SPINE is never slow-tagged so it runs on EVERY invocation, and the
# selector ESCALATES TO THE FULL GATE whenever it cannot reason about a change (below).
#
# HOW A CHANGE IS MAPPED (why this is not a coverage map): rung commits are ~99 % additive to
# `engine.py` (+404/-2 for rung 57), so `git diff` answers "what existing code moved?" directly.
# We diff the AST — top-level class/def SOURCE TEXT, old vs new — rather than line ranges,
# because a line-range map orphans the banner comments between two newly-added classes and
# escalates the whole module. Rules:
#   * a top-level symbol in BOTH revisions whose source differs -> CHANGED (seeds the closure)
#   * a top-level symbol only in the NEW revision               -> NEW; seeds NOTHING (no existing
#       code can depend on it, and its own test file is picked up as a changed test file)
#   * a top-level symbol only in the OLD revision (deleted)     -> CHANGED
#   * any change to module-level STATEMENTS (imports, constants) -> escalate to the FULL gate
# The seed is then closed in the CALLER direction to a fixpoint (a package symbol whose body
# mentions a seeded name joins), and a test module is affected if it mentions any name in the
# closed set. The closure is what makes `ComponentMap` correctly fan out to all of rungs 31-54
# while rung 55's purely-additive `StageStack` reaches only `test_rung55`.
#
# BASELINE: the working tree is compared against the sha of the last PASSING full gate (recorded
# in the pytest cache), not against HEAD. That is deliberate — it makes `--affected` CUMULATIVE
# across the rungs that were affected-gated since the last full run, so a clean tree right after
# a rung commit still selects that rung's changes instead of selecting nothing.
_ROOT = os.path.dirname(os.path.abspath(__file__))
_FULL_GATE_KEY = "affected/last-full-gate"
_FULL_GATE_EVERY = 3                       # rung commits between full gates (user's cadence)

_SYMBOL_MAPPED = ("turbojet/engine.py",)   # files we can map change -> symbol
# Touching any of these means we cannot reason narrowly: gas.py/components.py are the CORE that
# every rung reads (and have not been touched since rung 31), and the two config files define the
# selection policy itself. Anything else under turbojet/ escalates too, via _affected_modules.
_ESCALATE_FILES = ("turbojet/gas.py", "turbojet/components.py", "turbojet/__init__.py",
                   "conftest.py", "pytest.ini")


def _git(*args):
    """Run git in the repo root. Returns stdout, or None on any failure (which escalates)."""
    try:
        r = subprocess.run(["git", "-C", _ROOT, *args], capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=60)
    except (OSError, subprocess.SubprocessError):
        return None
    return r.stdout if r.returncode == 0 else None


def _symbol_sources(src):
    """({top-level name: source text}, [module-level statement dumps]), or (None, None)."""
    try:
        tree = ast.parse(src)
    except (SyntaxError, ValueError):
        return None, None
    lines = src.splitlines()
    syms, module_stmts = {}, []
    for n in tree.body:
        if isinstance(n, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
            syms[n.name] = "\n".join(lines[n.lineno - 1:n.end_lineno])
        else:
            module_stmts.append(ast.dump(n))     # comments / blank lines are invisible to ast
    return syms, module_stmts


def _seed_symbols(old_src, new_src):
    """Changed top-level symbol names, or None to signal 'escalate to the full gate'."""
    old, old_mod = _symbol_sources(old_src)
    new, new_mod = _symbol_sources(new_src)
    if old is None or new is None or old_mod != new_mod:
        return None                              # unparseable, or module-level statements moved
    seed = {n for n, body in new.items() if n in old and old[n] != body}
    return seed | (set(old) - set(new))          # deletions count as changes


def _closure(seed):
    """Fixpoint in the CALLER direction over the package's top-level symbols."""
    bodies = {}
    for rel in _SYMBOL_MAPPED + _ESCALATE_FILES:
        if not rel.startswith("turbojet/"):
            continue
        try:
            with open(os.path.join(_ROOT, rel), encoding="utf-8") as fh:
                syms, _ = _symbol_sources(fh.read())
        except OSError:
            continue
        bodies.update(syms or {})
    cur = set(seed)
    while True:
        grown = set(cur)
        for name, body in bodies.items():
            if name not in grown and any(re.search(rf"\b{re.escape(s)}\b", body) for s in cur):
                grown.add(name)
        if grown == cur:
            return cur
        cur = grown


def _baseline(config):
    """The sha to diff against: the last passing full gate if it is still an ancestor of HEAD."""
    since = config.getoption("--affected-since")
    if since:
        return since
    cache = getattr(config, "cache", None)
    rec = cache.get(_FULL_GATE_KEY, None) if cache is not None else None
    sha = (rec or {}).get("sha")
    if sha and _git("merge-base", "--is-ancestor", sha, "HEAD") is not None:
        return sha
    return "HEAD"


def _affected_modules(config):
    """Test-module basenames whose SLOW gates must run. None => escalate to the full gate.

    Returns (modules, why) — `why` is a short human-readable line for the report header."""
    base = _baseline(config)
    diff = _git("diff", "--name-only", base)
    if diff is None:
        return None, f"cannot diff against {base!r} (not a git repo?)"
    untracked = _git("ls-files", "--others", "--exclude-standard") or ""
    changed = {p.strip().replace("\\", "/") for p in (diff + "\n" + untracked).splitlines()
               if p.strip()}

    seed, mods = set(), set()
    for path in sorted(changed):
        if path.startswith("tests/") and path.endswith(".py"):
            mods.add(os.path.basename(path)[:-3])          # a changed test file is affected
        elif path in _ESCALATE_FILES:
            return None, f"{path} changed — the core / the policy itself"
        elif path in _SYMBOL_MAPPED:
            try:
                with open(os.path.join(_ROOT, path), encoding="utf-8") as fh:
                    new_src = fh.read()
            except OSError:
                return None, f"cannot read {path}"
            old_src = _git("show", f"{base}:{path}")
            s = _seed_symbols(old_src if old_src is not None else "", new_src)
            if s is None:
                return None, f"module-level statements changed in {path}"
            seed |= s
        elif path.startswith("turbojet/"):
            return None, f"{path} changed — no symbol map for it"
        # Everything else (docs/, main.py, memory/, CLAUDE.md) reaches no slow gate. That is the
        # ACCEPTED RISK named in CLAUDE.md § Commands — main.py in particular is covered by no
        # test at all, a pre-existing hole this selector neither creates nor closes.

    closed = _closure(seed) if seed else set()
    if closed:
        for name in sorted(os.listdir(os.path.join(_ROOT, "tests"))):
            if not (name.startswith("test_") and name.endswith(".py")):
                continue
            try:
                with open(os.path.join(_ROOT, "tests", name), encoding="utf-8") as fh:
                    src = fh.read()
            except OSError:
                continue
            if any(re.search(rf"\b{re.escape(s)}\b", src) for s in closed):
                mods.add(name[:-3])
    why = (f"baseline {base[:12]} · {len(seed)} changed symbol(s) -> {len(closed)} in closure "
           f"-> {len(mods)} module(s)")
    return mods, why


def _affected_for(config):
    """Memoised (modules, why) for this session. Lazy because pytest_report_header runs BEFORE
    pytest_collection_modifyitems, and both need the answer."""
    info = getattr(config, "_affected_info", None)
    if info is None:
        info = _affected_modules(config)
        config._affected_info = info
    return info


def _rungs_since_full_gate(config):
    """(n rung commits, sha) since the last passing full gate — drives the cadence banner."""
    cache = getattr(config, "cache", None)
    rec = cache.get(_FULL_GATE_KEY, None) if cache is not None else None
    sha = (rec or {}).get("sha")
    if not sha or _git("merge-base", "--is-ancestor", sha, "HEAD") is None:
        return None, sha
    out = _git("log", "--oneline", "--extended-regexp", "--grep=^feat\\(rung", f"{sha}..HEAD")
    return (len([l for l in out.splitlines() if l.strip()]) if out is not None else None), sha


def pytest_addoption(parser):
    parser.addoption("--runslow", action="store_true", default=False,
                     help="run the slow gates too (default: the fast subset only)")
    parser.addoption("--affected", action="store_true", default=False,
                     help="every fast test PLUS the slow gates of the modules the working diff "
                          "can reach (the per-rung ship gate). Escalates to the full gate if the "
                          "diff touches the core or module-level statements.")
    parser.addoption("--affected-since", action="store", default=None, metavar="REV",
                     help="diff against REV instead of the last passing full gate")


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
    config.addinivalue_line("markers", "slow: an inherently-expensive gate (deselected unless --runslow)")
    config._jet_niced = _run_at_below_normal_priority()


def pytest_collection_modifyitems(config, items):
    """Tag slow items (from the learned cache OR the seed set), then deselect them unless
    --runslow / -m slow was asked for. Runs on the controller and every xdist worker; the
    cache read is identical everywhere, so the collected set stays consistent across workers."""
    cache = getattr(config, "cache", None)
    durations = cache.get(_CACHE_KEY, {}) if cache is not None else {}
    for item in items:
        recorded = durations.get(item.nodeid)
        is_slow = (not _is_spine(item.nodeid)
                   and (_seed_says_slow(item.nodeid)
                        or (recorded is not None and recorded >= SLOW_SECONDS)))
        if is_slow:
            item.add_marker(pytest.mark.slow)

    # --affected: keep every fast test, and keep a slow gate only if its module is reachable
    # from the working diff. `affected is None` means the selector escalated -> behave as
    # --runslow. Computed once and stashed so the report header can explain the selection.
    affected = None
    if config.getoption("--affected") and not config.option.markexpr:
        affected, _why = _affected_for(config)

    full_gate = config.getoption("--runslow") and not config.getoption("--affected")
    if not full_gate and not config.option.markexpr:
        # respect an explicit `-m` expression (e.g. `-m slow`); otherwise drop the slow gates —
        # all of them for a bare `pytest`, the unreachable ones for `--affected`.
        keep_slow = (lambda it: False) if not config.getoption("--affected") else (
            (lambda it: True) if affected is None
            else (lambda it: _module_of(it.nodeid) in affected))
        selected, deselected = [], []
        for item in items:
            drop = item.get_closest_marker("slow") and not keep_slow(item)
            (deselected if drop else selected).append(item)
        if deselected:
            config.hook.pytest_deselected(items=deselected)
            items[:] = selected

    # NB: "was this a full gate?" is decided in pytest_sessionfinish, NOT here. Under xdist the
    # WORKERS collect and this hook never runs on the controller — but the controller is the only
    # process that may write the cache. Anything stashed on config here is therefore invisible to
    # the recorder. (This bit us: the first green --runslow recorded nothing.)

    # LPT scheduling: get every multi-minute pole started at t=0 so the makespan approaches
    # the single longest test rather than a stacked tail. xdist hands items to workers in
    # collection order AND seeds TWO tests per worker up front — so a naive longest-first order
    # would pair the two longest poles on one worker (rung-24's 365 s scan + the 273 s one =
    # 640 s on a single core). Instead interleave the cost-sorted list from both ends
    # ([longest, shortest, 2nd-longest, 2nd-shortest, ...]): every worker's initial pair is one
    # long pole + one sub-second filler, and the long poles fan out across the workers. A pure
    # scheduling hint — the tests are independent (the parallel baseline proved order-
    # independence), so this changes wall-clock only, never results.
    def _cost(item):
        rec = durations.get(item.nodeid)
        if rec is not None:
            return rec
        return 1e6 if item.get_closest_marker("slow") else 0.0   # cold cache: seed-slow go early
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
# cache-clobber race that per-worker writes would cause.
def pytest_runtest_logreport(report):
    if report.when == "call":
        _RECORDED[report.nodeid] = report.duration
    _SEEN.add(report.nodeid)          # any phase — a skipped test never reaches `call`


_RECORDED: dict = {}
_SEEN: set = set()


def pytest_report_header(config):
    """Say out loud which gate is running, what it selected, and when the full one is due."""
    lines = []
    # Report the priority drop rather than assume it: the Win32 call fails silently (see
    # `_run_at_below_normal_priority`), and a courtesy that quietly no-ops is worse than none.
    if not getattr(config, "_jet_niced", False) and os.environ.get("JET_TEST_NICE", "1") != "0":
        lines.append("gate: WARNING — could not drop to below-normal priority; the run will "
                     "compete with your foreground apps")
    if config.getoption("--affected") and not config.option.markexpr:
        affected, why = _affected_for(config)
        if affected is None:
            lines.append(f"gate: --affected ESCALATED TO FULL — {why}")
        else:
            shown = ", ".join(sorted(affected)) if affected else "(none)"
            lines.append(f"gate: --affected — {why}")
            lines.append(f"      slow gates re-enabled for: {shown}")
    n, sha = _rungs_since_full_gate(config)
    if sha is None:
        lines.append("gate: no full `pytest --runslow` recorded yet — run one to start the clock")
    elif n is not None and n >= _FULL_GATE_EVERY:
        lines.append(f"gate: *** {n} rung commits since the last full gate ({sha[:12]}) — "
                     f"cadence is every {_FULL_GATE_EVERY}; run `pytest --runslow` ***")
    elif n is not None:
        lines.append(f"gate: {n}/{_FULL_GATE_EVERY} rung commits since the full gate at {sha[:12]}")
    return lines


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
    # A clean, unnarrowed `--runslow` pass is what resets the cadence clock AND becomes the
    # baseline every later `--affected` run diffs against. Decided here, from the config alone,
    # because this is the only hook that runs on the xdist CONTROLLER (see the note in
    # pytest_collection_modifyitems). Every narrowing flag we know of disqualifies the run, and
    # `_SEEN` must be non-empty so a `--collect-only` pass can never reset the clock. Unknown
    # narrowing errs toward NOT recording — i.e. toward running the full gate sooner.
    narrowed = (config.option.markexpr or config.option.keyword
                or getattr(config.option, "lf", False)
                or getattr(config.option, "failedfirst", False)
                or getattr(config.option, "stepwise", False)
                or config.getoption("--collect-only")
                or list(config.args) != list(config.getini("testpaths")))
    was_full = (config.getoption("--runslow") and not config.getoption("--affected")
                and not narrowed)
    if was_full and exitstatus == 0 and _SEEN:
        head = (_git("rev-parse", "HEAD") or "").strip()
        if head:
            cache.set(_FULL_GATE_KEY, {"sha": head, "when": time.strftime("%Y-%m-%d %H:%M:%S")})
