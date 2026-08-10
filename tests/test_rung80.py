"""Rung 80 — THE SPLIT WALL: `docs/rung74-arrest-interval.md` § 8's seam.

Every floor since rung 49 came from ONE margin. This rung gives the AIRFLOW legs their own:

    phi_lim = (1 + sm    ) * phi_surge     the fuel leg
    phi_air = (1 + sm_air) * phi_surge     the valve and the stator

HEADLINE: **a LEVEL split separates loops on the CONSTRAINT; it cannot separate the two that
share the ACTUATOR.** The split OPENS the four-loop cell in `demand` — the seam's own object,
empty at every shared wall — but `min` still masks one fuel-side leg with an EXACTLY zero
column, so `n_live` (loops holding AUTHORITY) is STILL <= 3, a SIXTH time.

AND IT CORRECTS RUNG 74: the arrest belongs to neither floor, but to their COINCIDENCE. With the
walls split, `phi(0)` is lifted onto 0.78 / 0.80 and the plant marches anyway.

All three pre-registered predictions were REFUTED — see `docs/plans/rung80-anchor-split-wall.md`
§ 5a. Scoring + tables: `docs/rung80-spec.md`.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    SplitWallTransient, StateCoordinateTransient, BleedLimiter, StatorLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX, TAU, TAU_S = 0.10, 0.20, 0.05, 0.05

# THE WINDOW, both edges READ OFF `docs/rung74-arrest-interval.md` and not chosen here:
#   free droop (demand)   0.7464354455  -- below it the fuel leg is DORMANT
#   free operating point  0.7731162133  -- at/above it a SHARED wall ARRESTS
PHI_FUEL = 0.75          # strictly inside the window
FREE_PHI0 = 0.7731162133

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


def _rig(design, cls=SplitWallTransient):
    sm = 0.80 / FLOOR - 1.0
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


@pytest.fixture(scope="module")
def liveness(design):
    return _rig(design).split_liveness(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL)


@pytest.fixture(scope="module")
def arrest(design):
    return _rig(design).split_arrest(FLIGHT, LO, HI, TT4_MAX, phi_lim_lo=PHI_FUEL)


@pytest.fixture(scope="module")
def gains(design):
    return _rig(design).split_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                    phi_airs=(None, 0.77, 0.80), coord="clip")


@pytest.fixture(scope="module")
def gains_demand(design):
    return _rig(design).split_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                    phi_airs=(None, 0.77, 0.80), coord="demand")


# --- § 9.1: THE REDUCE, BOTH ARMS AND BOTH EXACT ---------------------------------------------

def test_reduce_none_path_is_rung79_identical(design):
    """`_sm_air = None` must dispatch to rung 79 with nothing recomputed. Compared through a
    shipped rung-79 reader, not through this rung's own."""
    a = _rig(design, StateCoordinateTransient).coord_scan(FLIGHT, LO, HI, TT4_MAX)
    b = _rig(design).coord_scan(FLIGHT, LO, HI, TT4_MAX)
    assert repr(a) == repr(b), "rung 80 moved a rung-79 float on the unarmed path"


def test_reduce_sm_air_equal_sm_is_bit_for_bit(design):
    """`sm_air == sm` rebuilds the SAME floors from the SAME factory, so it must equal the
    `None` path to the last bit. Any difference is a bug in the rebuild, never a finding."""
    m = _rig(design)
    sm = PHI_FUEL / FLOOR - 1.0
    args = (FLIGHT, LO, HI, TT4_MAX, sm, (0.05,) * 4, 0.5, 1.2, 0.005, V_MAX, False,
            "demand", "sched")
    r0 = m._coord_march(*args)[3]
    r1 = m._with_air(sm, m._coord_march, *args)[3]
    assert len(r0) == len(r1) == 341
    for p, q in zip(r0, r1):
        for k in ("phi_lp", "Tt4", "b", "v", "mf"):
            assert p[k] == q[k], f"{k} moved at s = {p['s']}"


def test_the_knob_is_loud(design):
    """A split that fails to reach the plant would make every reader here report *the levers
    did nothing* — this rung's own anchor P1 — so the walls are read BACK off the limiters the
    rig will march with, and a lower airflow wall dies by name."""
    m = _rig(design)
    sm = PHI_FUEL / FLOOR - 1.0
    rig, surge, _ = m._with_air(0.80 / FLOOR - 1.0, m._shared_rig, sm, TAU, TAU_S, V_MAX,
                                TT4_MAX)
    w = rig._walls_of(rig, surge)
    assert w["phi_lim"] == pytest.approx(PHI_FUEL, rel=1e-12)
    assert w["phi_air"] == pytest.approx(0.80, rel=1e-12)
    assert w["phi_valve"] == pytest.approx(w["phi_stator"], rel=1e-12), (
        "both airflow legs must sit on ONE airflow wall")
    with pytest.raises(AssertionError, match="AIRFLOW wall sits AT or ABOVE"):
        m._with_air(0.70 / FLOOR - 1.0, m._shared_rig, sm, TAU, TAU_S, V_MAX, TT4_MAX)


# --- § 9.2: THE CONTROL — RUNG 74's BRACKET, REPRODUCED --------------------------------------

def test_shared_wall_control_reproduces_rung74_bracket(arrest):
    """THE CONTROL ARM FIXES THE RIG. If rung 74's own bracket does not reappear here, nothing
    in the two split arms is interpretable. Both edges are DERIVED (the lower one is the free
    operating point, read independently), so this cannot be satisfied by tuning."""
    sh = arrest["arms"]["shared"]
    assert arrest["control_bracket"] == (0.7731, 0.7732)
    assert sh["monotone"], f"marched {sh['marched']} interleaves arrested {sh['arrested']}"
    assert sh["last_march"] < FREE_PHI0 < sh["first_arrest"], (
        "the free operating point must lie INSIDE the bracket — that is what makes the edge "
        "derived rather than fitted")


# --- § 9.3: THE ARREST BELONGS TO THE COINCIDENCE, NOT TO A FLOOR ----------------------------

def test_neither_split_arm_arrests(arrest):
    """CORRECTS rung 74 § 2.2 / the arrest doc § 4: the arrest is not owned by the highest
    floor, nor by the fuel leg's. Split the walls in EITHER direction and it is gone."""
    assert arrest["owner"] == [], f"a split arm arrested: {arrest['owner']}"
    for arm in ("air", "fuel"):
        d = arrest["arms"][arm]
        assert not d["arrested"], f"{arm} arrested at {d['arrested']}"
        assert all(x["max_Tt4"] > LO * 1.15 for x in d["rows"]), (
            f"{arm} did not accelerate on some wall")


def test_the_lift_is_present_so_the_null_is_not_a_dormant_knob(arrest):
    """The arrest doc's MECHANISM is a floor lifting `phi(0)` onto itself. That lift must be
    demonstrably HAPPENING in the split arms, or "no arrest" would just mean "no floor acted"
    and this rung would have measured nothing."""
    for arm, key in (("air", "phi_air"), ("fuel", "phi_air")):
        lifted = [x for x in arrest["arms"][arm]["rows"] if x[key] > FREE_PHI0]
        assert lifted, f"{arm} never put a floor above the free operating point"
        for x in lifted:
            assert x["phi0"] == pytest.approx(x[key], rel=1e-9), (
                f"{arm}: phi(0) was not lifted ONTO the airflow wall at {x['wall']}")
            assert x["b0_frac"] > 0.0, f"{arm}: the valve never opened at {x['wall']}"


# --- § 9.4: THE FOUR-LOOP CELL OPENS ---------------------------------------------------------

def test_clip_positive_control_fires(liveness):
    """`clip` HAS four-loop cells at the shared wall (arrest doc § 5). Zero motion there means
    the READER is broken, and every `demand` zero in the same table is uninterpretable."""
    assert liveness["control_ok"]
    clip = [x for x in liveness["rows"] if x["coord"] == "clip"]
    assert clip and all(x["valve_moved"] > 0 and x["stator_moved"] > 0 for x in clip)


def test_split_opens_the_demand_four_loop_cell(liveness):
    """THE SEAM'S OWN OBJECT, at last non-empty — and the shared-wall row is the baseline that
    proves the SPLIT is what created it, at identical settings."""
    dem = [x for x in liveness["rows"] if x["coord"] == "demand"]
    shared = [x for x in dem if x["phi_air"] is None]
    split = [x for x in dem if x["phi_air"] is not None]
    assert len(shared) == 1 and split
    assert shared[0]["n_riding4"] == 0 and shared[0]["valve_moved"] == 0, (
        "rung 74's result must reproduce: a shared wall leaves the levers inert")
    for x in split:
        assert x["riding4_valid"], "n_riding4 is meaningless on an arrested plant (§ 8)"
        assert x["n_riding4"] > 0, f"no four-loop point at phi_air = {x['phi_air']}"
        assert x["valve_moved"] > 0 and x["stator_moved"] > 0


def test_fuel_leg_stays_live_and_erodes_monotonically(liveness):
    """§ 1.1's DERIVATION, scored on the counterfactual noun. The fuel leg's cut is evaluated
    at the SCHEDULED fuel, so a lever that raises the ACHIEVED phi erodes it without
    extinguishing it. Anchor P1 predicted extinction; this is the refutation."""
    dem = sorted([x for x in liveness["rows"] if x["coord"] == "demand"],
                 key=lambda x: x["phi_air"] or 0.0)
    cuts = [x["n_cut_fuel"] for x in dem]
    assert all(c > 0 for c in cuts), f"the fuel leg went dormant: {cuts}"
    assert cuts == sorted(cuts, reverse=True) and cuts[0] > cuts[-1], (
        f"the erosion is not monotone in phi_air: {cuts}")


# --- § 9.5: THE MASK SURVIVES THE SPLIT — AND THE ZERO IS GUARDED ----------------------------

def test_one_authority_per_point_and_the_mask_is_exactly_zero(gains, gains_demand):
    """RUNG 72, UNMOVED BY A LEVEL SPLIT. Exactly one fuel-side leg holds the actuator at every
    interior point, and the masked one's column is EXACTLY zero however far apart the walls."""
    for out in (gains, gains_demand):
        assert not out["ever_two_authorities"]
        assert out["max_mask_leak"] == 0.0, (
            f"the masked leg leaked {out['max_mask_leak']} into the plant")
        for a in out["arms"]:
            for c in a["cells"]:
                assert c["masked"] != c["authority"]


def test_the_zero_has_a_positive_control_and_an_empty_skip_count(gains, gains_demand):
    """AN EXACT ZERO CAN MEAN "NOTHING WAS COMPUTED" — rung 78 § 5.1's logged trap, and in
    `demand` every interior cell masks the SAME leg, so `mask_leak` and the cyclic product's
    `V_f` factor are ONE zero seen twice. Two discriminators:

      * the `clip` cell where the GOVERNOR is masked returns |cyclic| ~ 1 on the same code path;
      * every split arm differenced all of its points (`skipped == 0/0`).
    """
    assert gains["control_nonzero"] is not None
    assert gains["control_nonzero"] == pytest.approx(1.0, rel=1e-6), (
        "the instrument must produce a NON-zero on the same path, or the zeros below are "
        "unfalsifiable")
    for out in (gains, gains_demand):
        for a in out["arms"]:
            if a["phi_air"] is None:
                continue                      # the baseline is the control, not the subject
            assert a["n_interior"] > 0
            assert a["skipped"] == {"switch": 0, "regime": 0}, (
                f"points were dropped at phi_air = {a['phi_air']}: {a['skipped']}")
            assert a["max_cyc"] == 0.0


def test_demand_shared_wall_is_vacuous_and_says_so(gains_demand):
    """The `demand` baseline HAS no interior point — that is rung 74's finding, and the reader
    must REPORT it rather than average it away."""
    base = next(a for a in gains_demand["arms"] if a["phi_air"] is None)
    assert base["n_riding" ] == 0 and base["n_interior"] == 0
    assert gains_demand["vacuous"], "a zero-point arm must set the vacuity flag"


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-q"]))
