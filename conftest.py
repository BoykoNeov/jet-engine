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

Parallelism (`-n auto --dist worksteal`, set in pytest.ini) is orthogonal: it speeds BOTH
the fast and the full run. worksteal (not the default `load`) is chosen because the suite's
durations are very uneven — a few multi-minute items among many sub-second ones — and
work-stealing packs the long poles far better than static batching.
"""
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


def pytest_addoption(parser):
    parser.addoption("--runslow", action="store_true", default=False,
                     help="run the slow gates too (default: the fast subset only)")


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

    if not config.getoption("--runslow") and not config.option.markexpr:
        # respect an explicit `-m` expression (e.g. `-m slow`); otherwise drop the slow gates.
        selected, deselected = [], []
        for item in items:
            (deselected if item.get_closest_marker("slow") else selected).append(item)
        if deselected:
            config.hook.pytest_deselected(items=deselected)
            items[:] = selected

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


_RECORDED: dict = {}


def pytest_sessionfinish(session):
    config = session.config
    cache = getattr(config, "cache", None)
    if cache is None or not _RECORDED:
        return
    if hasattr(config, "workerinput"):     # an xdist worker — the controller does the write
        return
    stored = cache.get(_CACHE_KEY, {})
    stored.update(_RECORDED)               # last-seen wins; keeps durations for tests not run this time
    cache.set(_CACHE_KEY, stored)
