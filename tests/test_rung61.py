"""Rung 61 — STATOR + BLEED: a compensating lever buys back the COORDINATE, not the BILL.

Gates (named in docs/rung61-spec.md § Verification gates):

   1. REDUCE — TWO-AXIS. (v=0,b=0) => rung 39, (v!=0,b=0) => rung 53, (v=0,b!=0) => rung 42,
      all bit-for-bit (==), on the fast gas AND the reacting gas.
   2. THE TWO TRAPS — a moved-stator combined matcher really has map_lp.vsv != 0 under the
      MRO, and at_setting returns a StatorBleedMatcher CARRYING the bleed. Both failures
      would have been plausible numbers with no exception.
   3. THE HEADLINE — at b* the phi-debit is fully bought back while >=70% of the stator's
      overspeed SURVIVES; and at v=0.30 the compensated point OVERSPEEDS the bare stator
      (the crossover: undoing the lever is strictly worse than leaving it alone).
   4. THE MECHANISM — the compensated point is MORE unloaded than the stator-only point
      (the rebate forfeited), base(phi) rises as phi falls, and psi_comp matches the closed
      form (gated as a PLUMBING check — it is psi at a known argument, an identity).
   5. RUNG 60's TAUTOLOGY, third route — dM_i == v and dM_phi == v*phi_s0^2/(1+v*phi_s0)
      exactly, throttle-invariant. Gated as an identity, explicitly NOT as a finding.
   6. THE SEAM AS POSED, REFUTED — v_edge and the M_i span fall monotonically in b (the
      valve SHRINKS the stator's authority), M_i(0) rises with b; and artifact-free, the
      four-cell credit interaction is <3% of the credit sum on all five shapes.
   7. SPOOL-DEPENDENCE — b*_LP exists at every throttle, b*_HP at NONE of them, and the HP
      shortfall is throttle-invariant. The two levers do not span the same space.
   8. THE PRICE COLLAPSE on (1+l) — <5% spread across five shapes — PLUS the negative
      control: the ceiling is CAP-dependent and is therefore not claimable.
   9. RUNG 53 CORRECTED — its exact per-spool zero reproduced (so the correction is not
      vacuous), broken by the pair, and STILL broken on the flat-eta island. Plus the cost
      machine-zero: i_F == 0.0 exactly on flat-eta while i_n != 0.
  10. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolMapMatcher, TwoSpoolBleedMatcher, VariableStatorMatcher, StatorBleedMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

# The five disclosed shapes (rungs 53/54/55's set), LP map first.
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

FIELDS = ("pi_lpc", "pi_hpc", "n_lp", "n_hp", "phi_lp", "phi_hp", "slip",
          "eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt", "tau_lpc", "tau_hpc",
          "tau_hpt", "tau_lpt", "mdot_air", "thrust")


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """Self-consistent CPG dual gas (rungs 31/38-42/53's recipe)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _sb(gas=None, ml=LP, mh=HP, vl=0.0, vh=0.0, b=0.0, design=None):
    gas = gas if gas is not None else _cpg_gas()
    return StatorBleedMatcher(design if design is not None else _design(gas), FLIGHT, 1.0,
                              map_lp=ml, map_hp=mh, vsv_lp=vl, vsv_hp=vh, bleed=b)


def _shaped(name, gas=None, vl=0.0, b=0.0):
    ml, mh = SHAPES[name]
    return _sb(gas, ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR), vl=vl, b=b)


# ======================================================================================
# GATE 1 — REDUCE: TWO-AXIS, and stronger than either parent's alone
# ======================================================================================

@pytest.mark.parametrize("gasname", ["fast", "reacting"])
def test_reduce_two_axis_bit_for_bit(gasname):
    """Three corners of the (v, b) plane, each == its own parent rung, field by field."""
    gas = Gas.thermally_perfect() if gasname == "fast" else Gas.reacting_equilibrium()
    for Tt4 in (1500.0, 1200.0):
        d = _design(gas)
        # (0,0) => rung 39
        a = TwoSpoolMapMatcher(d, FLIGHT, 1.0, map_lp=LP, map_hp=HP).match(FLIGHT, Tt4)
        b = _sb(gas, design=d).match(FLIGHT, Tt4)
        for k in FIELDS:
            assert getattr(a, k) == getattr(b, k), f"(0,0) {k} at Tt4={Tt4}"
        # (v,0) => rung 53
        d = _design(gas)
        a = VariableStatorMatcher(d, FLIGHT, 1.0, map_lp=LP, map_hp=HP,
                                  vsv_lp=0.15).match(FLIGHT, Tt4)
        b = _sb(gas, vl=0.15, design=d).match(FLIGHT, Tt4)
        for k in FIELDS:
            assert getattr(a, k) == getattr(b, k), f"(v,0) {k} at Tt4={Tt4}"
        # (0,b) => rung 42
        d = _design(gas)
        a = TwoSpoolBleedMatcher(d, FLIGHT, 1.0, map_lp=LP, map_hp=HP,
                                 bleed=0.08).match(FLIGHT, Tt4)
        b = _sb(gas, b=0.08, design=d).match(FLIGHT, Tt4)
        for k in FIELDS:
            assert getattr(a, k) == getattr(b, k), f"(0,b) {k} at Tt4={Tt4}"


def test_reduce_map_objects_still_identical_at_design_setting():
    """Rung 53's IDENTITY reduce survives the new class: at v=0 the maps are the SAME
    OBJECTS passed in, so there is still no rung-53 code path to skip."""
    m = _sb(b=0.10)
    assert m.map_lp is LP and m.map_hp is HP
    assert m.map_lp_design is LP and m.map_hp_design is HP
    # ...and `match` resolves to rung 42's, which at b=0 forwards to rung 39's.
    assert StatorBleedMatcher.match is TwoSpoolBleedMatcher.match


# ======================================================================================
# GATE 2 — THE TWO SILENT-FAILURE TRAPS
# ======================================================================================

def test_trap_stators_actually_move_under_the_mro():
    """TRAP: rung 42's __init__ forwards no vsv, so a co-operative super() chain would
    leave the stators at the design setting and report plausible WRONG numbers."""
    m = _sb(vl=0.20, vh=0.05, b=0.10)
    assert m.map_lp.vsv == 0.20 and m.map_hp.vsv == 0.05
    assert m.bleed == 0.10
    # the design references are still captured at v=0 (rung 53's construction discipline)
    assert m.map_lp_design.vsv == 0.0 and m.map_hp_design.vsv == 0.0
    # and the moved stator is actually LIVE in the solve
    bare = _sb().match(FLIGHT, TT4)
    assert _sb(vl=0.20).match(FLIGHT, TT4).phi_lp != bare.phi_lp


def test_trap_at_setting_carries_the_bleed():
    """TRAP: every rung 53/54 instrument routes through at_setting. Rung 53's version
    hard-constructs a VariableStatorMatcher, silently dropping the valve."""
    m = _sb(b=0.12)
    sib = m.at_setting(0.05, 0.0)
    assert isinstance(sib, StatorBleedMatcher)
    assert sib.bleed == 0.12 and sib.vsv_lp == 0.05
    # the instruments that route through it therefore see the bled machine
    rows = m.stator_sweep(FLIGHT, TT4, (0.0, 0.05), "lp")
    unbled = _sb().stator_sweep(FLIGHT, TT4, (0.0, 0.05), "lp")
    assert rows[0]["lp"]["phi_op"] != unbled[0]["lp"]["phi_op"]


# ======================================================================================
# GATE 3 — THE HEADLINE: the debit goes, the bill stays
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("Tt4,v", [(1500.0, 0.10), (1500.0, 0.20), (1300.0, 0.20),
                                   (1100.0, 0.20), (1100.0, 0.30)])
def test_headline_overspeed_survives_compensation(Tt4, v):
    """b* removes the WHOLE phi-debit, and >=70% of the stator's overspeed survives it."""
    c = _sb().compensated_point(FLIGHT, Tt4, v, "lp")
    assert c["b_star"] is not None, c["reason"]
    # the coordinate IS bought back
    assert abs(c["phi_comp"] - c["phi_bare"]) <= 1e-10
    assert c["phi_stator"] < c["phi_bare"] - 1e-3        # the stator really spent something
    # the BILL is not
    retained = c["dn_comp"] / c["dn_stator"]
    assert c["dn_stator"] > 0.0 and c["dn_comp"] > 0.0
    assert retained >= 0.70, f"retention {retained:.3f} at Tt4={Tt4}, v={v}"
    # ...and rung 42's own thrust bill is now on top of it
    assert c["dF_comp"] < c["dF_stator"] - 0.02


@pytest.mark.slow
def test_headline_crossover_compensation_is_strictly_worse():
    """THE strongest single number: at v=0.30 the COMPENSATED machine overspeeds the
    UNcompensated one, so undoing the lever is strictly worse than leaving it alone.
    Asserted as a strict inequality — the SIGN is the claim, not the level."""
    c = _sb().compensated_point(FLIGHT, 1500.0, 0.30, "lp")
    assert c["b_star"] is not None
    assert c["dn_comp"] > c["dn_stator"], (
        f"crossover absent: comp {c['dn_comp']:+.5f} vs stator {c['dn_stator']:+.5f}")


# ======================================================================================
# GATE 4 — THE MECHANISM: the phi-debit was carrying a rebate
# ======================================================================================

@pytest.mark.slow
def test_mechanism_compensation_forfeits_the_loading_rebate():
    """The compensated point is MORE unloaded than the stator-only point, because the
    stator's phi-drop raised base(phi) and restoring phi gives that rebate back."""
    m = _sb()
    Tt4, v = 1500.0, 0.20
    c = m.compensating_bleed(FLIGHT, Tt4, v, "lp", "phi")
    assert c["b_star"] is not None
    cells = {}
    for name, vv, bb in (("bare", 0.0, 0.0), ("stator", v, 0.0), ("comp", v, c["b_star"])):
        sib = m.at_point(vv, 0.0, bb)
        od = sib.match(FLIGHT, Tt4)
        cells[name] = (od.phi_lp, sib.map_lp.psi(od.phi_lp))
    # the REBATE: base(phi) rises as the stator drops phi (this is the map's own loading law)
    base = lambda p: 1.0 - LP.sigma * (p - 1.0) ** 2 - LP.l * (p - 1.0)
    assert base(cells["stator"][0]) > base(cells["bare"][0]) + 1e-3
    # ...and forfeiting it leaves the compensated point MORE unloaded than the stator alone
    assert cells["comp"][1] < cells["stator"][1] < cells["bare"][1]


@pytest.mark.slow
@pytest.mark.parametrize("name", list(SHAPES))
def test_psi_closed_form_is_a_plumbing_check(name):
    """psi_comp == base(phi_bare) - v(1+l)phi_bare. This is `psi` evaluated at a KNOWN
    argument — an IDENTITY, gated only to prove that at_point composes the two levers onto
    one map/cascade correctly. It is deliberately NOT presented as a finding."""
    ml = SHAPES[name][0]
    m = _shaped(name)
    Tt4, v = 1500.0, 0.20
    phi_b = m.at_point(0.0, 0.0, 0.0).match(FLIGHT, Tt4).phi_lp
    c = m.compensating_bleed(FLIGHT, Tt4, v, "lp", "phi")
    assert c["b_star"] is not None
    sib = m.at_point(v, 0.0, c["b_star"])
    psi_meas = sib.map_lp.psi(sib.match(FLIGHT, Tt4).phi_lp)
    base = 1.0 - ml.sigma * (phi_b - 1.0) ** 2 - ml.l * (phi_b - 1.0)
    assert abs(psi_meas - (base - v * (1.0 + ml.l) * phi_b)) <= 1e-10


# ======================================================================================
# GATE 5 — RUNG 60's TAUTOLOGY, reached by a THIRD route (an identity, NOT a finding)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("Tt4", [1500.0, 1300.0, 1100.0])
@pytest.mark.parametrize("v", [0.10, 0.20, 0.30])
def test_rung60_tautology_third_route(Tt4, v):
    """Restoring phi (rather than PINNING it, as rung 60's floor does) hands back the SAME
    published value: dM_i = v exactly. So rung 60's tautology needs no floor at all — only
    restoration. Gated for exactness; demoted to a lemma in the spec."""
    c = _sb().compensated_point(FLIGHT, Tt4, v, "lp")
    assert c["b_star"] is not None
    assert abs(c["d_m_i"] - v) <= 1e-10
    assert abs(c["d_m_phi"] - v * FLOOR ** 2 / (1.0 + v * FLOOR)) <= 1e-10


@pytest.mark.slow
def test_tautology_is_throttle_invariant_and_survives_flat_eta():
    """dM_phi takes the SAME value at every throttle (it is pure geometry), and both
    identities hold on the flat-eta island where the maps carry no shaping at all."""
    vals = [_sb().compensated_point(FLIGHT, T, 0.20, "lp")["d_m_phi"]
            for T in (1500.0, 1300.0, 1100.0)]
    assert max(vals) - min(vals) <= 1e-10
    c = _shaped("flat-eta").compensated_point(FLIGHT, 1500.0, 0.20, "lp")
    assert abs(c["d_m_i"] - 0.20) <= 1e-10 and abs(c["d_m_phi"] - vals[0]) <= 1e-10


# ======================================================================================
# GATE 6 — THE SEAM AS POSED, REFUTED (rung 40's convention: assert it, don't drop it)
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("Tt4", [1500.0, 1000.0])
def test_seam_as_posed_valve_shrinks_the_stators_authority(Tt4):
    """'The bleed takes over where the stator's authority ends' predicts the ceiling is
    INDIFFERENT to the valve. It is not: the valve PRE-SPENDS the incidence budget, so the
    stator's remaining authority SHRINKS. (This rung's own prediction had the sign the
    other way and is scored a miss in the anchor.)"""
    rows = _sb().authority_with_bleed(FLIGHT, Tt4, (0.0, 0.05, 0.10, 0.15), "lp")
    edges = [r["v_edge"] for r in rows]
    spans = [r["span"] for r in rows]
    zeros = [r["m_i_0"] for r in rows]
    assert all(a >= b for a, b in zip(edges, edges[1:])) and edges[0] > edges[-1]
    assert all(a > b for a, b in zip(spans, spans[1:])), spans
    assert all(a < b for a, b in zip(zeros, zeros[1:])), zeros    # the valve pre-spends


@pytest.mark.slow
@pytest.mark.parametrize("name", list(SHAPES))
def test_credits_superpose_artifact_free(name):
    """The load-bearing version of the seam's refutation — no reliance on rung 53's
    solve_n artifact edge. The two levers are SUBSTITUTES on one incidence budget: the
    four-cell interaction is under 3% of the credit sum on every shape."""
    m = _shaped(name)
    for Tt4 in (1500.0, 1200.0):
        for v, b in ((0.10, 0.05), (0.20, 0.10)):
            def m_i(vv, bb):
                return m.at_point(vv, 0.0, bb).stator_margin(FLIGHT, Tt4)["lp"]["m_i"]
            base = m_i(0.0, 0.0)
            cs, cb = m_i(v, 0.0) - base, m_i(0.0, b) - base
            inter = (m_i(v, b) - base) - cs - cb
            assert cs > 0.0 and cb > 0.0
            assert abs(inter) / (cs + cb) < 0.03, f"{name} {Tt4} {v},{b}: {inter:+.5f}"


# ======================================================================================
# GATE 7 — SPOOL-DEPENDENCE: the two levers do not span the same space
# ======================================================================================

@pytest.mark.slow
def test_compensability_is_spool_dependent():
    """b*_LP exists at every throttle; b*_HP at NONE of them. Rung 53's stator acts on
    either spool, rung 42's valve on one, so a stator debit is compensable only where the
    two overlap."""
    rows = _sb().compensability(FLIGHT, [1500.0, 1300.0, 1100.0, 900.0], 0.20)
    assert len(rows) >= 4
    assert all(r["b_lp"] is not None and 0.0 < r["b_lp"] < 0.45 for r in rows)
    assert all(r["b_hp"] is None for r in rows)
    assert all(r["why_hp"] == "valve authority exhausted (b >= cap)" for r in rows)
    # b*_LP falls monotonically as power falls (rung 42's near-constant dphi_L into a
    # smaller debit) — the LP branch is well-behaved exactly where the HP one does not exist
    bs = [r["b_lp"] for r in rows]
    assert all(a > b for a, b in zip(bs, bs[1:])), bs


@pytest.mark.slow
def test_hp_shortfall_is_throttle_invariant_not_a_pi_star_divergence():
    """The anchor predicted a DIVERGENCE toward pi*. Measured: uniformly unavailable, by a
    throttle-invariant shortfall. The mechanism (bleed has no HP authority) was right, the
    shape was wrong — asserted so the corrected statement is the gated one."""
    m = _sb()
    short = []
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0):
        c = m.compensating_bleed(FLIGHT, Tt4, 0.20, "hp", "phi")
        assert c["b_star"] is None
        spent = m.at_point(0.0, 0.20, 0.0).stator_margin(FLIGHT, Tt4)["hp"]["phi_op"] \
            - c["goal"]
        returned = c["resid_last"] - spent
        assert spent < 0.0 and returned > 0.0
        assert abs(spent) > 3.0 * returned          # short by >3x at every throttle
        short.append(abs(c["resid_last"]))
    assert (max(short) - min(short)) / (sum(short) / len(short)) < 0.10


# ======================================================================================
# GATE 8 — THE PRICE COLLAPSE, and the control that keeps it honest
# ======================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("Tt4", [1500.0, 1200.0])
def test_price_collapses_on_the_loading_slope(Tt4):
    """b*/[v(1+l)] is the same number across five shapes whose l spans 0.7 -> 1.2, so the
    price's ENTIRE shape-dependence is the map's own loading slope. The coefficient itself
    rides on v and the throttle and is disclaimed."""
    vals = []
    for name in SHAPES:
        l = SHAPES[name][0].l
        c = _shaped(name).compensating_bleed(FLIGHT, Tt4, 0.20, "lp", "phi")
        assert c["b_star"] is not None
        vals.append(c["b_star"] / (0.20 * (1.0 + l)))
    spread = (max(vals) - min(vals)) / (sum(vals) / len(vals))
    assert spread < 0.05, f"{vals} spread {spread:.3%}"


@pytest.mark.slow
def test_the_ceiling_is_cap_dependent_and_therefore_NOT_claimed():
    """THE NEGATIVE CONTROL. The compensable range looked like a derived ceiling scaling as
    1/(1+l). It is not: _B_CAP is this rung's OWN constant, and moving it moves the ceiling.
    Gated so the un-publishable claim cannot creep back in."""
    def last_ok(cap):
        m = _sb()
        m._B_CAP = cap
        ok, v = 0.0, 0.10
        while v < 0.8:
            if m.compensating_bleed(FLIGHT, 1500.0, v, "lp", "phi")["b_star"] is None:
                return ok
            ok, v = v, round(v + 0.05, 4)
        return ok
    lo, hi = last_ok(0.35), last_ok(0.45)
    assert hi > lo, (lo, hi)      # the "ceiling" tracks the cap => not a plant property


@pytest.mark.slow
def test_price_split_two_loci():
    """'Restore the point' and 'restore the reported margin' are different instructions —
    the stator moved the floor between them. The gap grows with v and is throttle-INVARIANT
    while each price separately moves a lot."""
    gaps = {}
    for Tt4 in (1500.0, 1200.0):
        rows = _sb().price_split(FLIGHT, Tt4, (0.10, 0.20, 0.30), "lp")
        for r in rows:
            assert r["b_phi"] is not None and r["b_m_phi"] is not None
            assert r["b_phi"] > r["b_m_phi"] > 0.0
        g = [r["gap"] for r in rows]
        assert all(a < b for a, b in zip(g, g[1:])), g       # grows with v
        gaps[Tt4] = g
        assert rows[1]["b_phi"] > 0.0
    for a, b in zip(gaps[1500.0], gaps[1200.0]):
        assert abs(a - b) / a < 0.02, (a, b)                 # throttle-invariant


# ======================================================================================
# GATE 9 — RUNG 53 CORRECTED, with its own control; and the cost machine-zero
# ======================================================================================

@pytest.mark.slow
def test_rung53_per_spool_cleanliness_lost_under_composition():
    """Rung 53 P5: vsv_lp leaves phi_HP BIT-IDENTICAL, and its inter-spool arrow is
    eta-mediated so a flat-eta island switches it off. Both still hold for the lever ALONE
    (reproduced here, so the correction is not vacuous) — and neither survives the pair,
    because the only lever that buys the LP debit back reaches the HP through the shared
    Tt25 ENERGY channel, which no flat map can switch off."""
    for name in ("flow/press", "flat-eta"):
        m = _shaped(name)
        Tt4, v = 1500.0, 0.20
        r0 = m.at_point(0.0, 0.0, 0.0).stator_margin(FLIGHT, Tt4)["hp"]["phi_op"]
        rv = m.at_point(v, 0.0, 0.0).stator_margin(FLIGHT, Tt4)["hp"]["phi_op"]
        assert rv - r0 == 0.0, f"{name}: rung 53's exact zero is gone"    # the control
        c = m.compensated_point(FLIGHT, Tt4, v, "lp")
        assert c["b_star"] is not None
        assert abs(c["d_phi_other_comp"]) > 1e-3, name    # ...broken by the PAIR, even flat


@pytest.mark.slow
def test_rung53_P1_thrust_neutrality_is_EXACT_on_a_flat_eta_island():
    """RUNG 53's P1 SHARPENED, and the control that stopped this rung overclaiming.

    Rung 53 reported the stator thrust-neutral as a TOLERANCE ("specific thrust flat to
    <0.5%"). With the efficiency island switched off it is a MACHINE ZERO: the stator's own
    thrust effect is exactly 0.0 while n moves >6%. So the whole of the stator's thrust cost
    is the eta island, and it is a PURE speed lever there.

    This is also why the pair's flat-eta THRUST interaction is not a finding of this rung:
    one term of the four-cell difference vanishes identically, so the interaction is zero for
    free. Asserted here in the form that makes the corollary visible."""
    flat = _shaped("flat-eta")
    for Tt4 in (1500.0, 1200.0):
        F00 = flat.at_point(0.0, 0.0, 0.0).match(FLIGHT, Tt4)
        for v in (0.10, 0.20, 0.30):
            od = flat.at_point(v, 0.0, 0.0).match(FLIGHT, Tt4)
            assert od.thrust - F00.thrust == 0.0, (v, od.thrust, F00.thrust)
            # ...while the SPEED bill is large (5.4 % at the mildest point tried, 20 % at
            # the strongest — the contrast with an EXACT zero is the claim, not the level)
            assert od.n_lp / F00.n_lp - 1.0 > 0.04


@pytest.mark.slow
def test_cost_interaction_speed_is_adverse_everywhere():
    """The real cost interaction: the pair always costs MORE shaft speed than the sum of its
    parts — including on the flat-eta island, where the thrust interaction is trivially zero
    but the speed one is not."""
    flat = _shaped("flat-eta")
    for Tt4 in (1500.0, 1200.0):
        for v, b in ((0.10, 0.05), (0.20, 0.10)):
            def cell(vv, bb):
                od = flat.at_point(vv, 0.0, bb).match(FLIGHT, Tt4)
                return od.thrust, od.n_lp
            (F00, n00), (Fv0, nv0) = cell(0.0, 0.0), cell(v, 0.0)
            (F0b, n0b), (Fvb, nvb) = cell(0.0, b), cell(v, b)
            i_F = (Fvb / F00) - (Fv0 / F00) - (F0b / F00) + 1.0
            i_n = (nvb / n00) - (nv0 / n00) - (n0b / n00) + 1.0
            assert i_F == 0.0        # a COROLLARY of the test above, not a claim
            assert i_n > 1e-4, i_n
    shaped = _shaped("flow/press")
    for v, b in ((0.10, 0.05), (0.20, 0.10), (0.30, 0.10)):
        def cell(vv, bb):
            od = shaped.at_point(vv, 0.0, bb).match(FLIGHT, 1500.0)
            return od.thrust, od.n_lp
        (F00, n00), (Fv0, nv0) = cell(0.0, 0.0), cell(v, 0.0)
        (F0b, n0b), (Fvb, nvb) = cell(0.0, b), cell(v, b)
        assert (Fvb / F00) - (Fv0 / F00) - (F0b / F00) + 1.0 > 0.0
        assert (nvb / n00) - (nv0 / n00) - (n0b / n00) + 1.0 > 0.0


# ======================================================================================
# GATE 10 — CYCLE UNTOUCHED
# ======================================================================================

def test_cycle_untouched_bit_for_bit_rung6():
    """The default single-spool design run never sees a stator or a valve."""
    eng = build_turbojet(gas=Gas.reacting_equilibrium(), pi_c=10.0, Tt4=1600.0,
                         p_ambient=FLIGHT.p0, **{k: v for k, v in REAL.items()
                                                 if k in ("pi_d", "eta_b", "pi_b", "eta_m",
                                                          "pi_n")})
    r = eng.run(FLIGHT, 1.0)
    assert r.performance.specific_thrust > 0.0 and r.performance.tsfc > 0.0
    r2 = eng.run(FLIGHT, 1.0)
    assert r2.performance.specific_thrust == r.performance.specific_thrust


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
