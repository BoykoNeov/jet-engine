"""Test-suite speed policy — fast by default, full on demand.

The full suite is dominated by a handful of inherently-expensive tests (the mixing-PDF
per-pocket-quench sweeps of rungs 16/20-24, the transient marches of rungs 29/31/34/37).
Those are the load-bearing FINDING and robustness gates; they are correct to be thorough,
but they cost minutes and you do not want them on every quick iteration.

Policy (see the Commands section of CLAUDE.md):
  * `pytest`               -> runs the FAST subset only (slow gates deselected). Routine.
  * `pytest --runslow`     -> runs EVERYTHING (every gate). Use at commit / session-end / CI.
  * `pytest -m slow`       -> runs ONLY the slow gates.

A test is "slow" if its last recorded call-duration was >= SLOW_SECONDS. That threshold is
learned automatically: every run records per-test durations into pytest's own cache
(`.pytest_cache`), and the next collection marks the slow ones. The set is seeded below
from a full baseline run so the very first `pytest` already skips the right tests without
needing a warm cache. This keeps the policy in ONE place and never edits a test file, so
the rung gates stay pristine (the project's derive/reduce spine is untouched).

ONE override (`_is_spine`): the bit-for-bit REDUCE gates (`test_reduce_*`, `test_cycle_untouched_*`,
`*_bit_for_bit`) are NEVER slow-tagged regardless of cost, so bare `pytest` always guards the
"each rung reduces to its predecessor, exactly and by test" invariant — the project's spine
(user's explicit choice, 2026-07-21). Only the expensive FINDING / robustness sweeps are deferred
to `--runslow`. This is what keeps a fast routine run from silently dropping the reduce check.

Parallelism (`-n auto --dist load --maxschedchunk=1`, set in pytest.ini) is orthogonal: it
speeds BOTH the fast and the full run. It is already at its floor — this box has 8 PHYSICAL
cores behind 16 logical, and for these CPU-bound float loops an LPT pack of the measured
durations onto 8 workers is 1581 s against an OBSERVED full run of 1331-1347 s (2026-07-28,
two runs). The pack is already tighter than the 8-core model, so there is no scheduling slack
left to reclaim: the ONLY lever on the full gate's cost is running fewer tests. That is what
`--affected` (below) is for. The hard per-test floor is rung 24's `test_ei_stays_monotone`
(~518 s), so no amount of selection takes the full gate below ~9 min.

THE THIRD MODE — `--affected` (see § affected-set selection below):
  * `pytest --affected`    -> every fast test, PLUS the slow gates of the modules the working
                              diff can actually reach. A strict superset of `pytest` and a
                              strict subset of `pytest --runslow`. ~330-940 s vs ~1340 s.
This is the per-rung SHIP gate; `--runslow` becomes a periodic (every 3rd rung) full gate.
"""
import ast
import os
import re
import subprocess
import time

import pytest

SLOW_SECONDS = 8.0          # a test at/above this (call phase) is tagged `slow`
_CACHE_KEY = "durations/call"

# Seed set: (module basename, function name) pairs observed >= SLOW_SECONDS in the baseline
# `-n auto` full run (2026-07, 371 tests). Function names match ALL parametrizations (the
# "[param]" suffix is stripped before comparison). The learned cache extends this over time;
# this seed only has to be right enough that the first cold `pytest` is already fast.
_SEED_SLOW = {
    "test_rung13": {"test_cycle_untouched_by_pdf_call",
                    "test_reduce_g_to_zero_is_well_mixed_point_value"},
    "test_rung15": {"test_cycle_untouched_by_pdf_quench_call",
                    "test_reduce_pdf_quench_none_is_rung13_path",
                    "test_zoned_nox_matches_ei15_helper"},
    "test_rung16": {"test_clamp_dormant_over_pockets",
                    "test_cycle_untouched_by_pocket_quench_call",
                    "test_excess_vanishes_at_c_opt_flanks_up",
                    "test_far_flank_erosion_vs_rung15",
                    "test_reduce_at_c_opt_is_finite_bulk_quench_no",
                    "test_reduce_pocket_quench_none_is_rung15_path",
                    "test_zoned_nox_matches_ei16_helper"},
    "test_rung17": {"test_back_pressure_guard_inherited",
                    "test_cycle_untouched_by_clamp_call",
                    "test_ladder_direction_the_load_bearing_gate",
                    "test_reduce_to_components_exact",
                    "test_scale_sensitivity_ordering_robust_magnitude_not"},
    "test_rung18": {"test_emissions_basin_rounds_the_notch",
                    "test_residual_floor_elevates_the_optimum"},
    "test_rung20": {"test_clamp_stays_dormant_at_station4_with_the_lift",
                    "test_reduce_zoned_and_clamp_flag_off_is_identical",
                    "test_super_eq_o_now_combines_with_ideal_bell_closures"},
    "test_rung21": {"test_hybrid_resolved_and_combines",
                    "test_reduce_super_eq_o_false_is_bit_for_bit"},
    "test_rung22": {"test_derived_floor_sits_below_the_hump_peak",
                    "test_emissions_global_min_at_max_segregation",
                    "test_emissions_local_min_at_C_opt",
                    "test_resolved_width_below_two_stream_ceiling",
                    "test_super_eq_o_lifts_through_the_shared_bell"},
    "test_rung23": {"test_clamp_dormant_at_station4",
                    "test_correlation_adds_no_at_design_point",
                    "test_correlation_sign_one_signed_across_tau_mix",
                    "test_correlation_concentrated_under_penetration",
                    "test_cycle_untouched",
                    "test_g_below_two_stream_ceiling",
                    "test_helper_matches_production",
                    "test_production_g_matches_spatialpdf",
                    "test_reduce_spatial_dwell_none_is_prior_path",
                    "test_terminal_field_reproduces_rung22"},
    "test_rung24": {"test_ei_stays_monotone_the_emissions_optimum_is_not_recovered",
                    "test_does_not_claim_the_emissions_global_min_location",
                    "test_g_below_two_stream_ceiling",
                    "test_g_identical_to_rung22_by_construction",
                    "test_local_rate_moves_ei_only_modestly_vs_rung23",
                    "test_production_width_matches_spatial_pdf"},
    "test_rung28": {"test_uncoupled_is_rung27_bit_for_bit"},
    "test_rung29": {"test_earned_at_design_is_M0_robust",
                    "test_earned_at_design_is_pi_c_robust"},
    "test_rung31": {"test_running_line_and_direction"},
    "test_rung32": {"test_reduce_to_rung31"},
    "test_rung34": {"test_reduce_equilibrium_is_the_steady_matcher"},
    "test_rung37": {"test_heat_soak_accel_time_lag",
                    "test_heat_soak_cold_below_hot_below_adiabatic",
                    "test_plenum_equilibrium_is_rung35"},
    "test_rung47": {"test_lagged_governor_overshoots_erodes_hp_and_misses_lp",
                    "test_overshoot_grows_and_hp_erodes_monotone_in_tau",
                    "test_fast_ramp_lp_relief_eroded_by_lag_never_enhanced"},
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
    match is a genuine reduce/cycle gate (checked — no false positives)."""
    f = _func_of(nodeid)
    return (f.startswith("test_reduce") or f.startswith("test_cycle_untouched")
            or "bit_for_bit" in f or "bitforbit" in f)


# ------------------------------------------------------------- § affected-set selection
# The full gate is 1581 s and cannot be scheduled below that (see the module docstring), so the
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


def pytest_configure(config):
    config.addinivalue_line("markers", "slow: an inherently-expensive gate (deselected unless --runslow)")


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
