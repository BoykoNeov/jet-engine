"""Rung 81 — THE AUTHORITY CLOCK: `docs/rung80-spec.md` § 10's first seam.

Rungs 72–80 read `n_live <= 3` six times off ONE side of a switch: every cell measured had the
`Tt4` governor holding the actuator and the φ fuel leg masked. Rung 80 booked the other side as
untested. This rung throws the switch — and the knob that throws it is one the plant has carried
since rung 47.

HEADLINE (§ 3): **a leg that never holds the actuator has no clock.** In `clip` at the split wall
a 10× sweep of the fuel leg's own time constant moves **not one bit** of the trajectory — 0 of
1 364 floats, at three governor clocks — while the identical sweep at the SHARED wall, where that
leg does take the actuator, is live. Rung 72's *"`min` is flat in the masked leg"* promoted from a
Jacobian zero to an exact invariance of the plant.

AND THE MECHANISM (§ 1): **authority is decided by the LAG, not by the SET POINT.** The governor's
limit is the more severe one in every fuel-authority cell here and it loses the actuator anyway.

P1's tie-locality clause and P5 were REFUTED, and neither is gated — see
`docs/plans/rung81-anchor-authority-clock.md` § 6a. Scoring + tables: `docs/rung81-spec.md`.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    AuthorityClockTransient, SplitWallTransient, BleedLimiter, StatorLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX, TAU, TAU_S = 0.10, 0.20, 0.05, 0.05

# RUNG 80's OWN CELL, unchanged — a wall sweep is refused at this rung (anchor § 4.4): it would
# confound the clock with the set-point gap, which is what the § 1 criterion is a statement about.
PHI_FUEL, PHI_AIR = 0.75, 0.77
MATCHED = (0.05, 0.05, 0.05, 0.05)      # rung 80's clocks: the control
SLOW_FUEL = (0.20, 0.05, 0.05, 0.05)    # the mirror cell, with `tau_q` PINNED at rung 80's value

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg():
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=g, cp_t=cp, R_t=(g - 1.0) / g * cp, hPR=42.8e6)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _rig(design, cls=AuthorityClockTransient):
    sm = 0.80 / FLOOR - 1.0
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


@pytest.fixture(scope="module")
def clock(design):
    return _rig(design).authority_clock(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                        phi_air=PHI_AIR)


@pytest.fixture(scope="module")
def mask(design):
    return _rig(design).authority_mask(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                       phi_air=PHI_AIR)


def _march(m, taus, coord="demand", phi_air=PHI_AIR):
    return m._split_march(FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, phi_air, coord, taus,
                          0.5, 1.2, 0.005, V_MAX, False)


# --- § 7.1: THE REDUCE — AN IDENTITY, BECAUSE THE RUNG ADDS NOTHING --------------------------

def test_reduce_is_bit_for_bit_rung80(design):
    """A reader-only rung's whole contract. `AuthorityClockTransient` adds no state, no knob and
    no constant, so at rung 80's clocks and walls its march must be `SplitWallTransient`'s TO THE
    LAST BIT — including both legs' states and both demands, which are what this rung reads.

    A difference here would not be a finding; it would be a rung-80 regression wearing a new
    class name."""
    a = _march(_rig(design), MATCHED)[3]
    b = _march(_rig(design, SplitWallTransient), MATCHED)[3]
    assert len(a) == len(b) == 341
    keys = ("phi_lp", "Tt4", "b", "v", "w_fuel", "w_gov", "required_fuel", "required_gov")
    for p, q in zip(a, b):
        for k in keys:
            assert p[k] == q[k], f"{k} moved at s = {p['s']}"


# --- § 7.2: THE CONTROL — RUNG 80's OWN CELL, REPRODUCED -------------------------------------

@pytest.mark.slow
def test_matched_clock_control_reproduces_rung80(clock):
    """THE CONTROL FIXES THE RIG. `(0.05, 0.05)` at rung 80's walls is rung 80's own cell and
    must return its own number — 33 four-loop points, every one held by the GOVERNOR. If it does
    not, this grid is not the shipped plant and no row in it is interpretable."""
    assert clock["control_all_gov"], "rung 80's cell no longer reports an all-governor census"
    assert clock["control_n_riding4"] == [33], (
        f"rung 80 § 2 measured 33 four-loop points here; got {clock['control_n_riding4']}")
    assert not clock["all_fuel"], (
        "fuel authority at EVERY cell would mean the reader is not reading the clock at all "
        "(anchor V4) — and it would contradict a measured pre-check fact")
    assert clock["n_invalid"] == 0, "a row arrested; its counts are void (rung 80 § 8)"


# --- § 7.3: THE SEAM'S CELL — THE FUEL LEG HOLDS, WITH `tau_q` PINNED ------------------------

@pytest.mark.slow
def test_the_fuel_leg_takes_the_actuator(clock):
    """THE SEAM'S OWN OBJECT: a `demand` four-loop cell with the φ fuel leg AUTHORITATIVE — the
    first in this family, and the cell rung 80 § 5's table could not reach.

    Scored on a VALID row, because `n_riding4` is meaningless on an arrested plant (rung 80
    § 8's 320-point frozen march)."""
    hot = [x for x in clock["rows"] if x["coord"] == "demand" and x["tau_f"] == 0.20]
    assert hot, "the grid no longer carries the slow-fuel row"
    for x in hot:
        assert x["riding4_valid"], "the plant never left Tt4_lo — the count is void"
        assert x["n_fuel"] > 0 and x["n_gov"] == 0, (
            f"tau_f = 0.20, tau_gov = {x['tau_gov']}: expected an all-fuel census, "
            f"got {x['census']}")


@pytest.mark.slow
def test_the_valve_clock_is_not_the_cause(clock):
    """ANCHOR P2. The § 0 pre-check that found the mirror cell moved the fuel clock, the governor
    clock AND the valve clock in one step, so it could not say which one opened it. This grid
    pins `tau_q` at rung 80's 0.05 throughout and the cell opens anyway."""
    assert clock["tau_q"] == 0.05 and clock["tau_s"] == 0.05
    assert clock["fuel_cells"]["demand"], (
        "with the valve clock pinned there is no fuel-authority cell at all — the pre-check's "
        "result would then have been the VALVE's, not the fuel leg's")
    assert min(tf for tf, _, _ in clock["fuel_cells"]["demand"]) >= 0.08, (
        "the threshold moved below the measured 0.08 — the spec's § 1 number is stale")


@pytest.mark.slow
def test_the_losing_leg_is_the_more_severe_one(clock):
    """THE HEADLINE SENTENCE, GATED. *Authority is decided by the lag, not by the set point* is
    only worth saying if the two nouns actually DISAGREE — if the fuel leg took the actuator by
    also demanding the deeper cut, this rung would be reporting arithmetic.

    So: at every point where the fuel leg holds, the GOVERNOR's own demand must still be the
    larger one. Measured over the whole fuel-authority region (not just the § 0 pre-check's
    cell), and a single non-positive gap here would retire § 5's claim."""
    fuel = [c for x in clock["rows"] if x["riding4_valid"]
            for c in x["cells"] if c["measured"] == "fuel"]
    assert len(fuel) > 200, f"only {len(fuel)} fuel-held points scored — too few to claim this"
    worst = min(c["setpoint_gap"] for c in fuel)
    assert worst > 0.0, (
        f"the fuel leg held the actuator while ALSO demanding the deeper cut (set-point gap "
        f"{worst:.4e}) — the two nouns agree there, and the headline does not hold")


@pytest.mark.slow
def test_tau_gov_still_modulates_the_window(clock):
    """THE ANTI-OVER-CLAIM GATE. § 1 says the threshold in `tau_f` is crossed at EVERY `tau_gov`
    — it does NOT say the region is independent of `tau_gov`, and this repo's own recorded
    failure is over-claiming a consequence (rung 63). At fixed `tau_f` the governor's clock moves
    the fuel-held count by ~2×, so a spec sentence claiming independence dies here."""
    row = {x["tau_gov"]: x["n_fuel"] for x in clock["rows"]
           if x["coord"] == "demand" and x["tau_f"] == 0.12}
    assert len(set(row.values())) > 1, (
        f"tau_gov no longer moves the fuel-held count at tau_f = 0.12: {row}")


# --- § 7.4: THE HEADLINE — A MASKED LEG'S CLOCK IS AN EXACT NULL KNOB ------------------------

@pytest.mark.slow
def test_the_masked_legs_clock_moves_not_one_bit(clock):
    """§ 3, THE HEADLINE, and the NEGATIVE half. In `clip` at the split wall the fuel leg is
    masked for the whole ramp, and a 10× sweep of its own time constant is compared BIT-FOR-BIT
    over 341 points × `phi_lp`/`Tt4`/`b`/`v`.

    Compared exactly and NOT through a scalar: an inert knob changes NOTHING, and a reduced
    number would let a compensating pair read as inertness (rung 77's closure returned a perfect
    1.000e+00 having outlived its state block)."""
    cols = [(k, v) for k, v in clock["tau_f_inert"].items() if k.startswith("clip@")]
    assert len(cols) == 3, f"expected three clip columns, got {[k for k, _ in cols]}"
    for k, v in cols:
        assert v is not None and v["all_valid"] and v["n_tau_f"] == 6
        assert v["n_floats"] == 1364, f"{k}: the comparison lost its resolution"
        assert v["n_differing"] == 0 and v["march_identical"], (
            f"{k}: a masked leg's clock moved {v['n_differing']} of {v['n_floats']} floats")
        assert v["riding4_identical"], f"{k}: the four-loop count moved: {v['n_riding4']}"


@pytest.mark.slow
def test_the_same_clock_is_live_where_the_leg_holds(clock):
    """§ 3, THE POSITIVE HALF, and without it the test above is a statement about `clip` rather
    than about MASKING. At the SHARED wall the fuel leg does take the actuator (rung 80 § 5), and
    there the identical 10× sweep MOVES the march — and moves it monotonically in the direction
    § 1's derivation requires: in `clip` the state lags `required` from BELOW, so a SLOWER leg
    cuts LESS and holds the actuator on FEWER points."""
    assert clock["control_clip_fuel"] > 0, (
        "the reader cannot say `fuel` in `clip` at all — the null above is then a broken "
        "reader, not a masked leg (anchor V3)")
    assert clock["control_clip_tau_f_live"], (
        "the same 10× sweep is inert at the SHARED wall too — the null is about the coordinate, "
        "not about masking, and § 3's headline does not hold")
    n = [x["n_fuel"] for x in sorted(clock["control_clip_rows"], key=lambda x: x["tau_f"])]
    assert n[0] > n[-1], f"a slower fuel leg did not lose authority in clip: {n}"
    assert all(a >= b for a, b in zip(n, n[1:])), f"the fall is not monotone: {n}"


@pytest.mark.slow
def test_demand_columns_are_not_inert(clock):
    """The contrast that makes § 3 a SPLIT and not a null. The same knob, over the same range, on
    the same rig: inert in one coordinate, decisive in the other."""
    for k, v in clock["tau_f_inert"].items():
        if not k.startswith("demand@"):
            continue
        assert v is not None and not v["march_identical"], f"{k}: demand went inert"
        assert v["n_differing"] > 1000, f"{k}: only {v['n_differing']} floats moved"
        assert len(v["n_riding4"]) > 1, f"{k}: the four-loop count never moved"


# --- § 7.5: THE MIRROR MASK — RUNG 72's BLOCK, OTHER SIDE OF THE SWITCH ----------------------

def test_the_mask_survives_the_switch(mask):
    """ANCHOR P4. Every mask measurement rungs 72–80 made had the GOVERNOR holding. This is the
    mirror: `min` masks the governor instead, and rung 72's block must be indifferent to which
    leg that is — one authority per interior point, `mask_leak` EXACTLY zero, `n_live <= 3` a
    seventh time."""
    assert not mask["vacuous"], (
        f"only one authority regime is present (fuel {mask['n_fuel_interior']}, "
        f"gov {mask['n_gov_interior']}) — nothing about the mask is scored (anchor V1)")
    assert mask["n_fuel_interior"] > 0 and mask["n_gov_interior"] > 0
    assert mask["max_mask_leak"] == 0.0, (
        f"the masked leg reached the plant: {mask['max_mask_leak']}")
    assert not mask["ever_two_authorities"], "two legs held the actuator at one point"
    assert mask["all_differenced"], (
        "some four-loop point was skipped, so the zeros above are computed over a subset "
        "(rung 78 § 5.1's trap)")


def test_the_zero_is_falsifiable_on_one_code_path(mask):
    """THE DISCRIMINATOR, and it is better than rung 80's because both branches sit in ONE table
    rather than being imported from another coordinate's arm.

    The three-φ-loop cycle runs THROUGH the fuel leg, so it is exactly 0 where that leg is masked
    and non-zero where the GOVERNOR is. An instrument returning only zeros would be
    indistinguishable from one that measured nothing."""
    assert mask["cyc_gov_auth"] == 0.0, (
        f"`min` is not flat in the masked fuel leg: {mask['cyc_gov_auth']}")
    assert mask["cyc_fuel_auth"] is not None and mask["cyc_fuel_auth"] > 0.5, (
        f"the SAME reader returns {mask['cyc_fuel_auth']} where the governor is masked — if "
        "that is zero too, the zero above is unfalsifiable and scores nothing")


# --- § 7.6: THE CRITERION, AT ITS REGISTERED BAR ---------------------------------------------

@pytest.mark.slow
def test_the_criterion_predicts_the_authority_label(clock):
    """ANCHOR P1, at the bar that was REGISTERED (>= 95 % worst cell) and not at the value that
    was measured. The criterion is rung 74's own lag law read as an inequality, with no constant
    and no knob:

        fuel holds iff  required_gov - required_fuel  <  tau_f * dc_f/ds - tau_gov * dc_r/ds

    P1's tie-locality clause is NOT gated: it was measured REFUTED (3 of 9 misses above its 10 %
    line, worst 11.77 %), and gating a refuted clause at its measured value is fitting the test
    to the result."""
    assert clock["n_scored"] > 900, f"only {clock['n_scored']} points scored"
    assert clock["agreement"] >= 0.95, (
        f"worst cell agrees at {clock['agreement']:.4f}, below the registered 0.95")
    for x in clock["rows"]:
        assert x["n_edge"] == 0, (
            f"{x['coord']} ({x['tau_f']}, {x['tau_gov']}): {x['n_edge']} four-loop points had "
            "no central difference and were dropped — the spec's count is stale")


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
