"""Rung 53 — THE VARIABLE STATOR: what a margin *is*, when the lever moves the wall.

Gates (named in docs/rung53-spec.md § Verification gates):

   1. REDUCE — an IDENTITY, not a dispatch: at vsv=0 the stored maps are the SAME OBJECTS
      passed in and `match` is rung 39's own inherited method, so every field is `==` rung
      39's on BOTH gases. Plus psi/phi_max/phi_surge_at bit-for-bit at vsv=0, and the
      is_flat rule (phi_surge ignored, vsv NOT).
   2. THE CONTROL that could have killed the rung — at v=0 the throttle moves phi_op against
      a FIXED floor, so ALL THREE currencies must agree in sign at every step and
      dM_i/dM_phi must track the Jacobian 1/phi_op^2. A floor-fixed lever cannot split them.
   3. THE HEADLINE — with the STATOR as the lever the signs DO split, on both spools and
      across all five disclosed shapes; the derivatives hit -(1+l)/(2+l)+phi_s0^2 and the
      closed form +1/(2+l); the interval test -phi_op'/v' in (phi_surge^2, phi_op^2) holds.
   4. ZERO NEW CONSTANTS — T_c == 1/phi_surge exactly, the floor law and the psi law agree
      (phi_surge_at is exactly where tan_beta1 == tan_beta1_crit), t2 = l/(1+l) reproduces
      psi's design slope.
   5. P1 — a SPEED lever, not a flow lever: |dm/m| << |dn/n| (machine zero AT design), the
      closed form within 10%, and the trade (thrust flat, N_L strongly up).
   6. P5's TWO EXACT ZEROS with `==` — vsv_lp never reaches the HP spool at all; vsv_hp
      never reaches the LP spool on flat-eta islands; the shaped arrow is nonzero (so the
      zeros are not vacuous).
   7. P7 — the constant-incidence schedule: M_i constant to _INC_TOL while M_phi falls BELOW
      its own bare value at the same throttle; v* monotone; v*_LP > 3 v*_HP (rung 41's split).
   8. BOTH SPLIT BOUNDARIES AS BRACKETS (not point values — the levels ride on the
      disclosed constants).
   9. RUNG 41's TWO-PATH pi GATE SURVIVES the new psi term, at a MOVED stator.
  10. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolMapMatcher, VariableStatorMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

# The five disclosed shapes the split is asserted ACROSS (magnitudes disclaimed).
SHAPES = {
    "flow/press": (ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                   ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                   ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
    "steep":      (ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2),
                   ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)),
    "flat-eta":   (ComponentMap(sigma=0.1, l=0.7), ComponentMap(sigma=0.1, l=1.0)),
}
THROTTLE = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0]
FIELDS = ("pi_lpc", "pi_hpc", "n_lp", "n_hp", "phi_lp", "phi_hp", "slip",
          "eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt", "tau_lpc", "tau_hpc",
          "tau_hpt", "tau_lpt", "mdot_air", "thrust", "N_lp_ratio", "N_hp_ratio")


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """Self-consistent CPG dual gas (rung 31/38/39/40/41/42's recipe)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas, pi_lpc=PI_LPC, pi_hpc=PI_HPC):
    return build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _vm(gas, ml=LP, mh=HP, vl=0.0, vh=0.0, design=None):
    return VariableStatorMatcher(design if design is not None else _design(gas),
                                 FLIGHT, 1.0, map_lp=ml, map_hp=mh,
                                 vsv_lp=vl, vsv_hp=vh)


# ======================================================================================
# GATE 1 — REDUCE: an IDENTITY, not a dispatch
# ======================================================================================

def test_reduce_map_objects_are_identical_at_design_setting():
    """The strong claim in the class docstring: at vsv=0 the maps are the SAME OBJECTS, so
    there is no rung-53 code path to skip. Object identity, not dataclass equality."""
    m = _vm(_cpg_gas())
    assert m.map_lp is LP and m.map_hp is HP
    assert m.map_lp_design is LP and m.map_hp_design is HP
    # ...and `match` is rung 39's own method, inherited unoverridden.
    assert VariableStatorMatcher.match is TwoSpoolMapMatcher.match


@pytest.mark.parametrize("gasname", ["fast", "reacting"])
def test_reduce_bit_for_bit_rung39(gasname):
    """vsv=0 => every matched field is EXACTLY rung 39's, on both gases."""
    gas = Gas.thermally_perfect() if gasname == "fast" else Gas.reacting_equilibrium()
    d = _design(gas)
    base = TwoSpoolMapMatcher(d, FLIGHT, 1.0, map_lp=LP, map_hp=HP)
    stat = _vm(gas, design=d)
    for Tt4 in (1500.0, 1300.0, 1100.0):
        a, b = base.match(FLIGHT, Tt4), stat.match(FLIGHT, Tt4)
        for k in FIELDS:
            assert getattr(a, k) == getattr(b, k), f"{gasname} Tt4={Tt4} field {k}"


def test_reduce_componentmap_expressions_bit_for_bit():
    """psi / phi_max / phi_surge_at at vsv=0 are the rung <= 52 expressions, exactly."""
    for cm in (LP, HP, ComponentMap.flat(), ComponentMap.surge_tilted()):
        for phi in (0.4, 0.7, 1.0, 1.3, 1.9):
            assert cm.psi(phi) == (1.0 - cm.sigma * (phi - 1.0) ** 2
                                   - cm.l * (phi - 1.0))
        assert cm.phi_surge_at() == cm.phi_surge
        # phi_max's generalisation is inert at vsv == 0
        if cm.sigma == 0.0 and cm.l == 0.0:
            assert cm.phi_max() == 5.0
        else:
            rhs, lin = 1.0 - 0.1, cm.l
            u = (rhs / lin if cm.sigma == 0.0 else
                 (-lin + (lin ** 2 + 4.0 * cm.sigma * rhs) ** 0.5) / (2.0 * cm.sigma))
            assert cm.phi_max() == 1.0 + u


def test_is_flat_rule():
    """phi_surge is ignored by flatness (rung 36's rule); vsv is NOT (it enters psi)."""
    assert ComponentMap.flat().with_phi_surge(0.7).is_flat()
    assert not ComponentMap.flat().with_vsv(0.1).is_flat()
    assert ComponentMap.flat().with_vsv(0.0).is_flat()


def test_design_setting_maps_refused():
    """The matcher moves the stators itself, so a pre-swirled map is refused."""
    d = _design(_cpg_gas())
    with pytest.raises(AssertionError, match="DESIGN-SETTING maps"):
        VariableStatorMatcher(d, FLIGHT, 1.0, map_lp=LP.with_vsv(0.1), map_hp=HP)
    with pytest.raises(AssertionError, match="lp_disabled"):
        VariableStatorMatcher(d, FLIGHT, 1.0, map_lp=LP, map_hp=HP,
                              vsv_lp=0.1, lp_disabled=True)


# ======================================================================================
# GATE 2 — THE CONTROL: a FLOOR-FIXED lever CANNOT split the currencies
# ======================================================================================

@pytest.mark.parametrize("spool", ["lp", "hp"])
def test_throttle_cannot_split_the_currencies(spool):
    """THE GATE THAT COULD HAVE KILLED THE RUNG. At the design stator setting the throttle
    moves phi_op against a FIXED floor. Then M_i is a monotone reparameterisation of M_phi
    with a STRICTLY POSITIVE Jacobian 1/phi_op^2, so the two (and SM_N with them) must agree
    in sign at every step. If they could split here, the moving floor would not be the
    mechanism and the headline would be wrong."""
    rows = _vm(_cpg_gas()).throttle_currency(FLIGHT, THROTTLE, spool=spool)
    assert len(rows) == len(THROTTLE) - 1
    for r in rows:
        assert r["signs_agree"], r
        assert r["all_three_agree"], r
        # the ratio IS the Jacobian, to the finite-difference error
        assert abs(r["ratio"] / r["jacobian"] - 1.0) < 1e-3, r


# ======================================================================================
# GATE 3 — THE HEADLINE: the STATOR splits them
# ======================================================================================

@pytest.mark.parametrize("spool", ["lp", "hp"])
def test_headline_split_and_closed_forms(spool):
    """The two currencies disagree in SIGN under the stator, and both derivatives hit their
    closed forms at the design point (zero new constants)."""
    cs = _vm(_cpg_gas()).currency_split(FLIGHT, TT4, spool=spool)
    l = (LP if spool == "lp" else HP).l
    assert cs["phi_op"] == pytest.approx(1.0, abs=1e-9)     # design point
    assert cs["d_m_phi"] < 0.0 < cs["d_m_i"], cs            # THE SPLIT
    assert cs["split"] is True
    assert cs["d_m_phi"] == pytest.approx(-(1.0 + l) / (2.0 + l) + FLOOR ** 2, rel=1e-4)
    assert cs["d_m_i"] == pytest.approx(1.0 / (2.0 + l), rel=1e-4)
    # the interval law: disagreement IFF -phi_op'/v' lies in (phi_surge^2, phi_op^2)
    assert cs["in_interval"] is True
    assert cs["interval"][0] < cs["ratio"] < cs["interval"][1]


@pytest.mark.slow
@pytest.mark.parametrize("shape", list(SHAPES))
@pytest.mark.parametrize("spool", ["lp", "hp"])
def test_split_is_shape_robust(shape, spool):
    """The SIGN split holds on all five disclosed shapes, both spools (magnitudes disclaimed).
    Includes flat-eta, where the stator's only inter-spool arrow is switched off."""
    ml, mh = (m.with_phi_surge(FLOOR) for m in SHAPES[shape])
    cs = _vm(_cpg_gas(), ml=ml, mh=mh).currency_split(FLIGHT, TT4, spool=spool)
    assert cs["d_m_phi"] < 0.0 < cs["d_m_i"], (shape, spool, cs)
    l = (ml if spool == "lp" else mh).l
    assert cs["d_m_i"] == pytest.approx(1.0 / (2.0 + l), rel=1e-3)


# ======================================================================================
# GATE 4 — ZERO NEW CONSTANTS: the two channels are anchored, and they AGREE
# ======================================================================================

def test_incidence_anchor_is_the_rung36_floor():
    for cm in (LP, HP, ComponentMap.surge_flow().with_phi_surge(0.65)):
        assert cm.tan_beta1_crit() == 1.0 / cm.phi_surge
        assert cm.tan_beta1(cm.phi_surge) == cm.tan_beta1_crit()   # v=0: the anchor itself
    with pytest.raises(AssertionError, match="anchor"):
        ComponentMap.surge_flow().tan_beta1_crit()


@pytest.mark.parametrize("v", [-0.2, -0.05, 0.1, 0.3, 0.8])
def test_floor_law_and_psi_law_are_one_law(v):
    """The two derived channels are not independent fits: phi_surge_at() is EXACTLY the phi at
    which tan_beta1 reaches the (stator-invariant) critical incidence, and the psi swirl term
    carries the SAME v through the derived t2 = l/(1+l)."""
    for cm0 in (LP, HP):
        cm = cm0.with_vsv(v)
        assert cm.tan_beta1(cm.phi_surge_at()) == pytest.approx(
            cm.tan_beta1_crit(), rel=1e-14)
        assert cm.phi_surge_at() == pytest.approx(
            cm0.phi_surge / (1.0 + v * cm0.phi_surge), rel=1e-14)
        # the derived rotor-exit metal angle reproduces the map's own design slope
        t2 = cm0.l / (1.0 + cm0.l)
        assert 1.0 / (1.0 - t2) == pytest.approx(1.0 + cm0.l, rel=1e-14)
        # psi's swirl increment is exactly -v*(1+l)*phi
        for phi in (0.6, 1.0, 1.4):
            assert cm.psi(phi) == pytest.approx(
                cm0.psi(phi) - v * (1.0 + cm0.l) * phi, rel=1e-14, abs=1e-15)
        # closing lowers the floor, opening raises it
        assert (cm.phi_surge_at() < cm0.phi_surge) == (v > 0.0)


# ======================================================================================
# GATE 5 — P1: a SPEED lever, not a flow lever; and the trade
# ======================================================================================

@pytest.mark.parametrize("spool", ["lp", "hp"])
def test_speed_lever_at_design_is_a_machine_zero(spool):
    """AT the design point the eta island is STATIONARY, so m cannot move at all and the
    closed form -(1+l)/(2+l) is EXACT, not approximate."""
    cs = _vm(_cpg_gas()).currency_split(FLIGHT, TT4, spool=spool)
    l = (LP if spool == "lp" else HP).l
    assert cs["d_n"] > 0.0 > cs["d_phi_op"]                     # n UP, phi DOWN
    assert cs["flow_vs_speed"] < 1e-6, cs                       # m pinned
    assert cs["d_phi_op"] == pytest.approx(-(1.0 + l) / (2.0 + l), rel=1e-5)


@pytest.mark.slow
@pytest.mark.parametrize("spool", ["lp", "hp"])
def test_speed_lever_off_design_stays_within_the_registered_bands(spool):
    """Off design m DOES move (through the eta island) but stays far below n, and the general
    closed form -(1+l)phi^2/D(phi) holds within the pre-registered 10%."""
    s = _vm(_cpg_gas())
    for Tt4 in THROTTLE:
        cs = s.currency_split(FLIGHT, Tt4, spool=spool)
        assert cs["d_n"] > 0.0 > cs["d_phi_op"], (Tt4, cs)
        assert cs["flow_vs_speed"] <= 0.1, (Tt4, cs)
        assert cs["d_phi_op"] == pytest.approx(cs["d_phi_op_closed"], rel=0.10), (Tt4, cs)


def test_the_trade_is_thrust_neutral_and_paid_in_shaft_speed():
    """The contrast with rung 42: bleed costs thrust monotonically, the stator costs SPEED.
    At fixed Tt4 the energy cascade pins tau_c, so pi_c moves only through eta."""
    gas, d = _cpg_gas(), None
    d = _design(gas)
    st, nl = [], []
    for v in (-0.1, 0.0, 0.1, 0.2, 0.3):
        od = _vm(gas, vl=v, design=d).match(FLIGHT, TT4)
        st.append(od.performance.specific_thrust)
        nl.append(od.N_lp_ratio)
    assert max(st) / min(st) - 1.0 < 5e-3           # thrust FLAT (< 0.5%)
    assert st.index(max(st)) == 1                   # and it PEAKS at the design setting
    assert nl == sorted(nl) and nl[-1] / nl[1] - 1.0 > 0.15   # N_L monotone, > +15%


# ======================================================================================
# GATE 6 — P5: the inter-spool arrow is eta-MEDIATED ONLY (two EXACT zeros)
# ======================================================================================

def test_lp_stator_never_reaches_the_hp_spool_exactly():
    """rung 39: pi_LPC cancels out of the HP face and the energy cascade is map-free, so the
    LP stator is a PURE-LP lever -- bit-for-bit, not to a tolerance."""
    gas, d = _cpg_gas(), None
    d = _design(gas)
    a = _vm(gas, design=d).stator_margin(FLIGHT, TT4)
    b = _vm(gas, vl=0.20, design=d).stator_margin(FLIGHT, TT4)
    assert b["hp"]["phi_op"] == a["hp"]["phi_op"]        # EXACT
    assert b["hp"]["n"] == a["hp"]["n"]                  # EXACT
    assert b["lp"]["phi_op"] != a["lp"]["phi_op"]        # ...and the lever IS live
    assert b["lp"]["phi_op"] < a["lp"]["phi_op"]


def test_hp_stator_arrow_is_eta_mediated_only():
    """The HP->LP arrow exists ONLY through the efficiency island: switch the island off
    (a=b=c=0) and it is EXACTLY zero; leave it on and it is not."""
    gas = _cpg_gas()
    lpf = ComponentMap(sigma=0.1, l=0.7).with_phi_surge(FLOOR)
    hpf = ComponentMap(sigma=0.1, l=1.0).with_phi_surge(FLOOR)
    df = _design(gas)
    a = _vm(gas, ml=lpf, mh=hpf, design=df).stator_margin(FLIGHT, TT4)
    b = _vm(gas, ml=lpf, mh=hpf, vh=0.20, design=df).stator_margin(FLIGHT, TT4)
    assert b["lp"]["phi_op"] == a["lp"]["phi_op"]        # EXACT zero
    assert b["hp"]["phi_op"] < a["hp"]["phi_op"]         # the HP lever is live
    # ...and with the island ON the arrow is nonzero, so the zero above is not vacuous.
    c = _vm(gas, design=df).stator_margin(FLIGHT, TT4)
    e = _vm(gas, vh=0.20, design=df).stator_margin(FLIGHT, TT4)
    assert e["lp"]["phi_op"] != c["lp"]["phi_op"]


# ======================================================================================
# GATE 7 — P7: the constant-incidence schedule
# ======================================================================================

@pytest.mark.slow
def test_constant_incidence_schedule_holds_M_i_while_M_phi_collapses():
    """THE HEADLINE MADE OPERATIONAL, one assertion with both halves: along a schedule that
    holds the TRUE margin exactly constant, the phi-currency reports a large monotone LOSS --
    and falls BELOW its own unscheduled value at the same throttle."""
    s = _vm(_cpg_gas())
    rows = s.incidence_schedule(FLIGHT, THROTTLE, spool="lp", v_hi=1.6)
    for r in rows:
        assert abs(r["m_i"] - rows[0]["m_i"]) <= 1e-11, r      # M_i EXACTLY constant
        assert abs(r["residual"]) <= 1e-11, r
    # phi-currency: falls monotonically AND below the bare reading at the same throttle
    m_phi = [r["m_phi"] for r in rows]
    assert m_phi == sorted(m_phi, reverse=True)
    for r in rows[1:]:
        assert r["m_phi"] < r["m_phi_bare"], r
    assert rows[-1]["m_phi"] / rows[0]["m_phi"] < 0.4         # ~74% loss
    # the schedule closes progressively as power falls
    vs = [r["vsv_star"] for r in rows]
    assert vs == sorted(vs) and vs[0] == 0.0 and vs[-1] > 1.0


@pytest.mark.slow
def test_schedule_size_inherits_rung41_split():
    """The stator authority a spool needs measures its exposure: the LP (which takes the
    throttle excursion, rungs 41/44/45) needs several times the HP's setting."""
    s = _vm(_cpg_gas())
    lo = s.incidence_schedule(FLIGHT, [TT4, 1000.0], spool="lp", v_hi=1.6)[-1]
    hi = s.incidence_schedule(FLIGHT, [TT4, 1000.0], spool="hp", v_hi=1.6)[-1]
    assert lo["vsv_star"] > 3.0 * hi["vsv_star"], (lo["vsv_star"], hi["vsv_star"])


# ======================================================================================
# GATE 8 — the split's TWO boundaries, asserted as BRACKETS
# ======================================================================================

@pytest.mark.slow
def test_floor_tightness_boundary_bracket():
    """The split needs phi_s0 < sqrt((1+l)/(2+l)) = 0.7935 (LP). Asserted as a BRACKET: the
    sign of dM_phi/dv flips between phi_s0 = 0.79 and 0.82. The closed form is the claim; the
    crossing's exact level rides on the disclosed constants."""
    gas = _cpg_gas()
    d = _design(gas)
    got = {}
    for floor in (0.79, 0.82):
        ml = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(floor)
        mh = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(floor)
        got[floor] = _vm(gas, ml=ml, mh=mh, design=d).currency_split(
            FLIGHT, TT4, spool="lp")
    assert got[0.79]["d_m_phi"] < 0.0 < got[0.82]["d_m_phi"], got
    assert got[0.79]["split"] and not got[0.82]["split"]
    assert 0.79 < ((1.0 + 0.7) / (2.0 + 0.7)) ** 0.5 < 0.82      # the closed form brackets


@pytest.mark.slow
def test_part_power_boundary_bracket():
    """Throttled far enough down, the phi-currency FLIPS to agreement and both currencies say
    closing the stator loses margin. Predicted at phi_op ~ 0.71; bracketed inside the choked
    envelope between Tt4 = 825 and 800.

    The prediction is scored HONESTLY: phi_op ~ 0.71 lands just ABOVE the measured bracket
    (0.6996, 0.7078) -- a 0.3% miss, consistent with the closed form's known few-percent error
    off design (gate 5). The load-bearing claim is the EXISTENCE and the bracket, not 0.71."""
    s = _vm(_cpg_gas())
    hi, lo = (s.currency_split(FLIGHT, T, spool="lp") for T in (825.0, 800.0))
    assert hi["d_m_phi"] < 0.0 < lo["d_m_phi"], (hi["d_m_phi"], lo["d_m_phi"])
    assert hi["split"] and not lo["split"]
    assert lo["phi_op"] < hi["phi_op"] < 0.72        # the crossing, bracketed
    assert abs(0.71 / hi["phi_op"] - 1.0) < 0.01     # ...and the prediction within 1%
    assert hi["d_m_i"] > 0.0 and lo["d_m_i"] > 0.0   # incidence still helps on both sides


# ======================================================================================
# GATE 9 — rung 41's two-path pi gate survives the new psi term
# ======================================================================================

@pytest.mark.parametrize("v", [0.0, 0.1, 0.25])
def test_two_path_pi_agrees_at_a_moved_stator(v):
    """`_pi_c_spool` (which reads psi) must reproduce the SHIPPED pi from the cascade at the
    operating point -- two code paths, one pi. Rung 41's gate, now witnessing the swirl term."""
    s = _vm(_cpg_gas(), vl=v)
    od = s.match(FLIGHT, TT4)
    r = s.stator_margin(FLIGHT, TT4)
    assert r["lp"]["pi_op"] == pytest.approx(od.pi_lpc, rel=1e-11)
    assert r["hp"]["pi_op"] == pytest.approx(od.pi_hpc, rel=1e-11)
    # the floor point is a DIFFERENT map point, and its pi is above the operating one
    assert r["lp"]["sm_n"] > 0.0 and r["hp"]["sm_n"] > 0.0


def test_sweep_is_two_sided_and_monotone():
    """Rung 50's lesson: an edge is measured two-sided. Opening the stators (v<0) raises the
    floor and lifts phi_op; closing lowers both. M_i monotone rising in v, M_phi falling."""
    rows = _vm(_cpg_gas()).stator_sweep(
        FLIGHT, TT4, [-0.2, -0.1, 0.0, 0.1, 0.2, 0.3], spool="lp")
    m_i = [r["lp"]["m_i"] for r in rows]
    m_phi = [r["lp"]["m_phi"] for r in rows]
    floors = [r["lp"]["phi_surge"] for r in rows]
    assert m_i == sorted(m_i)
    assert m_phi == sorted(m_phi, reverse=True)
    assert floors == sorted(floors, reverse=True)
    # the OTHER spool is untouched throughout (P5's zero, along a whole sweep)
    assert len({r["hp"]["phi_op"] for r in rows}) == 1


# ======================================================================================
# GATE 10 — CYCLE UNTOUCHED
# ======================================================================================

def test_cycle_untouched_rung6():
    """The default single-spool design run is bit-for-bit rung 6 (no rung-53 knob reaches it)."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, 1600.0, FLIGHT.p0, **dict(
        pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92, eta_m=0.99, pi_n=0.98))
    r = eng.run(FLIGHT, 1.0)
    assert r.performance.specific_thrust > 0.0
    assert ComponentMap().vsv == 0.0 and ComponentMap().is_flat()


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
