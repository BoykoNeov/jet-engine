"""Rung 76 — THE FUEL-DEPENDENT CAP: rung 73 § 11's second seam, deferred by rungs 73, 74
AND 75.

Every cap in this family is a SET-POINT SOLVE, so it is a function of the STATE alone and
`d(cap)/d(mf) = 0` — which is what collapses rung 73's applied reference from a continuum to
three readings. Rung 48's `Wf/pt3` leg is the ONE whose law is not a solve: its own docstring
states it as an inequality ON THE FUEL, and a real limiter EVALUATES that from the delivery
pressure it senses.

    solve    cap = w*  with  w* = (1+margin)*kappa(n_H(w*))*pt3(w*)   -- rung 48, shipped
    sensed   cap(w) = (1+margin)*kappa(n_H(w))*pt3(w)  at  w = mf_app -- AS WRITTEN

THE HEADLINE: **a device in a leg's LAW reaches only the MASKED leg; a device in the PLANT the
legs READ reaches only the AUTHORITATIVE one.** Min-select masks a law; it cannot mask a plant,
because the plant is shared. So this writes `c/tau_f` on the authoritative fuel diagonal — the
one entry rungs 73, 74 and 75 each measure as *moved 0.0 relative* — and leaves the masked one
alone. And `n_live` is STILL <= 3, a FIFTH time: the obstruction is `min`'s flatness in the
masked state, which is neither a law nor a plant.

AND THE SET-POINT SOLVE WAS A GAIN, NOT A RELOCATION: `d(cap_solve)/dq = (d(cap_sensed)/dq) /
(1 - c)`, so writing a limiter as a solve makes it a STIFFER limiter than the schedule it
implements.

Anchor + scoring: `docs/plans/rung76-anchor-sensed-cap.md`, `docs/rung76-spec.md` § 7.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    SensedCapTransient, AntiWindupTransient,
    BleedLimiter, StatorIncidenceLimiter, StatorLimiter, SurgeLimiter, AsymmetricLag,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, V_MAX, TT4_MAX = 0.10, 0.20, 1200.0
TAUS = (0.05, 0.05, 0.05, 0.05)
TAU, TAU_S, TAU_GOV = 0.05, 0.05, 0.05
TAU_ATT, TAU_REL = 0.05, 0.15
TAU_T = 0.05

# The two floors, inherited from rungs 74/75 rather than chosen: `0.80` is where all four legs
# ride and every JACOBIAN is read; `0.76` is the both-legs-ride arm where every TRAJECTORY is
# marched.
PHI_JAC, PHI_BOTH = 0.80, 0.76

# RUNG 48's OWN already-imposed scalar, and the ONE imposition this rung carries. At 0.10 the
# accel leg is the binding cap on this trajectory; above ~0.20 the phi leg takes over and the
# knob is INERT by construction (spec § 1.3).
MARGIN = 0.10

# The DIFFERENCING FLOOR, and it is arithmetic rather than taste: `_rhs_gains_at` central-
# differences at `dg = 1e-7`, so roundoff alone is `eps/dg ~ 2.2e-9`. The anchor asked for
# `< 1e-9` and § 7 scores that tolerance as optimistic by exactly this much.
JAC_FLOOR = 3e-9

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _sm(phi_lim):
    return phi_lim / FLOOR - 1.0


def _rig(design, cls, sm, inc=False, coord="demand", ref="sched", law="none", tau_t=None,
         cap_law="solve"):
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                        else None),
            stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law = coord, ref
    m._windup_law, m._tau_t = law, tau_t
    if cls is SensedCapTransient:
        m._cap_law = cap_law
    return m


def _accel(design, sm, inc=False, margin=MARGIN):
    """THE SCHEDULE IS BUILT ON THE RIG THAT WILL MARCH IT. `kappa_ss` is read off the plant's
    OWN equilibria, so a schedule built on a bare machine and marched on `_shared_rig`'s would
    be a schedule for a DIFFERENT ENGINE — the trap rungs 61–75 hit on knobs, wearing its other
    face (spec § 7)."""
    return _rig(design, SensedCapTransient, sm, inc=inc).accel_for(
        FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, margin)


def _march(m, sm, acc):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TT4_MAX, tau_gov=TAU_GOV,
                           accel=acc, surge=SurgeLimiter.from_margin(LP, "lp", sm),
                           lag=AsymmetricLag(tau_att=TAU_ATT, tau_rel=TAU_REL))[0]


def _keys(traj, ks=("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "v",
                    "w_fuel", "w_gov")):
    return [tuple(p[k] for k in ks if k in p) for p in traj]


# ======================================================================================
# THE REDUCE SPINE — ONE arm, by DISPATCH, on FIVE cells. `_cap_law = 'solve'` is not a
# limit of anything: the hook's branch is simply not taken and the floats are rung 75's.
#
# NOT MARKED `slow`, on rungs 72/73/74/75's reasoning: the reduce spine is the project's
# spine and `conftest.py` is explicit that `-m "not slow"` has no backstop.
# ======================================================================================

def test_reduces_to_rung75_bit_for_bit(design):
    """The accel-armed plant is one this ladder has ALWAYS supported and never marched, so
    the reduce runs on it and not on rungs 72–75's phi-only rig (spec § 0.2)."""
    sm = _sm(PHI_BOTH)
    acc = _accel(design, sm)
    for coord, ref, law, tt in (("clip", "applied", "none", None),
                                ("demand", "sched", "none", None),
                                ("demand", "sched", "track", TAU_T),
                                ("demand", "applied", "track", TAU_T),
                                ("demand-latched", "applied", "none", None)):
        a = _keys(_march(_rig(design, SensedCapTransient, sm, coord=coord, ref=ref, law=law,
                              tau_t=tt), sm, acc))
        b = _keys(_march(_rig(design, AntiWindupTransient, sm, coord=coord, ref=ref, law=law,
                              tau_t=tt), sm, acc))
        assert a == b, f"{coord}|{ref}|{law}"


def test_the_reduce_is_not_vacuous(design):
    """ARM 1 MUST BE A TEST, NOT A TAUTOLOGY (rung 73's `charpoly_selftest` discipline, rungs
    74/75's own): if `_cap_law` were ignored, the reduce above would compare rung 75 with rung
    75 and pass. The SAME machine under `sensed` must DIFFER."""
    sm = _sm(PHI_BOTH)
    acc = _accel(design, sm)
    a = _keys(_march(_rig(design, SensedCapTransient, sm), sm, acc))
    b = _keys(_march(_rig(design, SensedCapTransient, sm, cap_law="sensed"), sm, acc))
    assert a != b


def test_the_refusals_are_refusals(design):
    """`clip x sensed` is refused because `clip` DISPATCHES OUT of this ladder before
    `_cap_fuel` is ever called — the march would silently be rung 73 and be reported as this
    rung. And `sensed` without a schedule has nothing to re-read."""
    sm = _sm(PHI_BOTH)
    acc = _accel(design, sm)
    with pytest.raises(AssertionError, match="REFUSED in the CLIP coordinate"):
        _march(_rig(design, SensedCapTransient, sm, coord="clip", ref="applied",
                    cap_law="sensed"), sm, acc)
    with pytest.raises(AssertionError, match="there must BE one"):
        m = _rig(design, SensedCapTransient, sm, cap_law="sensed")
        m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TT4_MAX, tau_gov=TAU_GOV,
                        surge=SurgeLimiter.from_margin(LP, "lp", sm),
                        lag=AsymmetricLag(tau_att=TAU_ATT, tau_rel=TAU_REL))
    with pytest.raises(AssertionError, match="CAP LAW is this rung's subject"):
        _march(_rig(design, SensedCapTransient, sm, cap_law="fitted"), sm, acc)


def test_the_knob_is_carried_by_at_lever(design):
    """THE FOURTEENTH INSTANCE of the trap rungs 61–75 each hit — and the first rung that
    wrote the carry BEFORE the first reader ran rather than after a reader lied."""
    m = _rig(design, SensedCapTransient, _sm(PHI_BOTH), cap_law="sensed")
    m._lag_coord = "demand"
    assert m.at_lever(bleed_lim=m.bleed_lim, stator_lim=m.stator_lim)._cap_law == "sensed"
    assert m._shared_rig(_sm(PHI_BOTH), TAU, TAU_S, V_MAX, TT4_MAX)[0]._cap_law == "sensed"


# ======================================================================================
# § 0.3 — `c` IS MEASURED, and `c < 1` is NOT implied by the shipped solver working
# ======================================================================================

@pytest.mark.slow
def test_c_is_strictly_inside_the_unit_interval(design):
    """A bracketing root-finder converges on a SIGN CHANGE whether or not `G = w - cap(w)` is
    monotone, so `_sched_fuel` bracketing buys *a root exists*, never `G' > 0`. `c` is
    therefore measured, on both stator arms and across margins."""
    for inc in (False, True):
        for margin in (0.05, MARGIN, 0.40):
            sm = _sm(PHI_JAC)
            acc = _accel(design, sm, inc=inc, margin=margin)
            m = _rig(design, SensedCapTransient, sm, inc=inc)
            sg = m.solve_gain(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=margin,
                              taus=TAUS, inc=inc, r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX)
            assert sg["n"] > 0, (inc, margin)
            cs = [x["c"] for x in sg["rows"]]
            assert 0.0 < min(cs) and max(cs) < 1.0, (inc, margin, min(cs), max(cs))
            del acc


# ======================================================================================
# § 1 — THE AUTHORITATIVE DIAGONAL, which nothing in this family has ever moved
# ======================================================================================

@pytest.fixture(scope="module")
def gains(design):
    m = _rig(design, SensedCapTransient, _sm(PHI_JAC))
    return m.cap_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN, taus=TAUS,
                       tau_t=TAU_T, r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX, every=8)


def _live(g):
    return {k: c for k, c in g["cells"].items() if c.get("n")}


@pytest.mark.slow
def test_the_authoritative_diagonal_moves_and_the_law_is_c_minus_one(gains):
    """THE RUNG. `d(mf_app)/dw_auth = 1` where the leg holds, so `d(cap)/dw_auth = c` and the
    diagonal is `(c-1)/tau_f` — against `-1/tau_f`, which rungs 73, 74 AND 75 each report as
    *moved 0.0 relative*. Scored PER POINT against THAT point's own `c`, never pooled: `c`
    varies 5% along the trajectory and a pooled residual would smear it (rung 73 § 4)."""
    live = _live(gains)
    assert live, "no interior riding cell"
    for k, c in live.items():
        assert c["auth_err"] < JAC_FLOOR, (k, c["auth_err"])
        assert c["auth_moved"] > 0.15, (k, c["auth_moved"])
        # AND WHAT IT MOVED FROM IS `-1/tau_f`, TO THE SAME DIFFERENCING FLOOR AND NOT BETTER
        # — because rung 73 WEAKENED `_jac4` to *measure* this diagonal rather than construct
        # it, and § 1.3 of that rung priced the weakening at five orders of magnitude. A gate
        # here at `1e-12` would be asserting against the construction rung 73 removed.
        assert abs(c["auth_diag0"][0] + 1.0 / TAUS[0]) < JAC_FLOOR, (k, c["auth_diag0"])


@pytest.mark.slow
def test_the_move_is_identical_in_both_references(gains):
    """`_demand_reference` returns `cap` ITSELF when `mf_app == w_own`, i.e. the applied
    reference is the IDENTITY on the leg that holds — so it cannot change what a plant-side
    gain does there. A sharp asymmetry against rung 75, whose masked diagonal is
    reference-DEPENDENT (`-1/tau_t` against `-(1/tau + 1/tau_t)`)."""
    live = _live(gains)
    by_ref = {}
    for k, c in live.items():
        by_ref.setdefault(c["ref"], []).append(c["auth_err"])
    assert set(by_ref) == {"sched", "applied"}
    for ref, errs in by_ref.items():
        assert max(errs) < JAC_FLOOR, (ref, errs)


@pytest.mark.slow
def test_the_masked_leg_is_untouched_and_the_rank_does_not_move(gains):
    """`min()` is flat in what the masked leg holds, so `d(mf_app)/dw_masked = 0` and the
    masked diagonal cannot move — and the masked COLUMN stays zero, which is `n_live <= 3` a
    FIFTH running. The same flatness that gives rungs 72–76 their triangularity is what
    confines this rung's device to the authoritative leg."""
    live = _live(gains)
    for k, c in live.items():
        assert c["masked_moved"] == 0.0, (k, c["masked_moved"])
        assert c["mask_leak"] == 0.0 and c["mask_leak0"] == 0.0, (k, c["mask_leak"])
        assert c["zeros"] == c["zeros0"], (k, c["zeros"], c["zeros0"])


@pytest.mark.slow
def test_the_governors_row_is_bit_identical(gains):
    """`_cap_gov` has NO sensed branch — a floor on a STATE is not a formula for a FUEL — so
    nothing in that row can move in any cell. The knob's DOMAIN, measured."""
    for k, c in _live(gains).items():
        assert c["gov_row"] == 0.0, (k, c["gov_row"])


@pytest.mark.slow
def test_det_J_scales_by_roughly_one_minus_c_and_the_residual_is_not_noise(gains):
    """The masked column is zero, so `det J` = masked diagonal x det(live 3x3) and only the
    authoritative fuel row moves. Anchor P7 asked for `1-c` EXACTLY and § 7 scores it REFUTED:
    the whole row scales by `1-c` only when both laws are read at the SAME `w`, and they are
    not (`mf_app` against `cap_solve`). The residual is bounded and § 3 names it.

    `applied x solve` is excluded because `det J == 0` there (rung 73's dead determinant) and
    the ratio is 0/0 — which is exactly what the anchor excluded in advance."""
    checked = 0
    for k, c in _live(gains).items():
        if c["ref"] == "applied" and c["law"] == "none":
            assert abs(c["det0"][0]) < 1e-6, (k, c["det0"])   # still dead
            continue
        assert c["det_err"] is not None and c["det_err"] < 0.06, (k, c["det_err"])
        assert 0.75 < c["det_ratio"][0] and c["det_ratio"][1] < 0.90, (k, c["det_ratio"])
        checked += 1
    assert checked >= 2


@pytest.mark.slow
def test_the_masked_cell_is_structurally_unreachable_with_the_accel_leg_binding(gains):
    """§ 1.4, and it is a CONSEQUENCE rather than a gap. Rung 48's `Wf/pt3` leg is FEEDFORWARD
    ON THE CAUSE and fires early; the topping governor is FEEDBACK ON A CONSEQUENCE and fires
    late. So where the accel leg BINDS the cap it binds early, and the leg that binds the cap
    is then also the leg that HOLDS the actuator. Anchor P6 is scored UNREACHED on this."""
    for k, c in gains["cells"].items():
        if c["auth"] == "gov":
            assert c["n"] == 0, (k, c["n"])


# ======================================================================================
# § 3 — WHAT THE SOLVE WAS BUYING: a GAIN, not a relocation
# ======================================================================================

@pytest.mark.slow
def test_the_two_laws_agree_at_the_solves_own_answer(design):
    """D2, where it actually lives. `cap_solve` is BY CONSTRUCTION the fixed point of
    `cap_sensed`, so the two laws agree there EXACTLY — and that is why the equilibrium of a
    leg that holds does not move. The march's TAIL is not that equilibrium (the spools are
    still spinning up), which is why anchor P10's trajectory form is scored REFUTED."""
    for inc in (False, True):
        m = _rig(design, SensedCapTransient, _sm(PHI_JAC), inc=inc)
        sg = m.solve_gain(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN, taus=TAUS,
                          inc=inc, r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX)
        assert sg["n"] > 0 and sg["fixed_point"] < 1e-15, (inc, sg["fixed_point"])


@pytest.mark.slow
def test_the_solve_amplifies_the_cap_by_one_over_one_minus_c(design):
    """THE FINDING THAT WAS NOT PREDICTED AT ALL, and the correction P7's refutation bought.
    Differentiating the fixed point `cap = cap_sensed(cap, q)` gives
    `d(cap_solve)/dq = (d(cap_sensed)/dq)/(1-c)` in one line — so writing a limiter as a SOLVE
    multiplies its sensitivity to every other state by `1/(1-c)`. A limiter written as a solve
    is a STIFFER limiter than the schedule it claims to implement."""
    for inc in (False, True):
        m = _rig(design, SensedCapTransient, _sm(PHI_JAC), inc=inc)
        sg = m.solve_gain(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN, taus=TAUS,
                          inc=inc, r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX)
        assert sg["gain_err"] < 1e-7, (inc, sg["gain_err"])
        assert sg["gain"][0] > 1.0, (inc, sg["gain"])


# ======================================================================================
# § 2 — THE BILL: the path moves, the destination does not
# ======================================================================================

@pytest.mark.slow
def test_the_sensed_leg_cuts_harder_over_the_whole_ramp(design):
    """During the ramp `mf_app < cap_solve`, so the droop identity gives
    `cap_sensed = cap_solve + c*(mf_app - cap_solve) < cap_solve`. **Rung 48's set-point solve
    has been granting the engine the fuel it would be self-consistent WITH, which is more fuel
    than the schedule it implements allows.** The SIGN is the claim; the magnitudes are
    reported in § 2 with their grid band."""
    m = _rig(design, SensedCapTransient, _sm(PHI_BOTH))
    b = m.cap_bill(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, margin=MARGIN, taus=TAUS,
                   ref="sched", law="none", r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX)
    assert b["cuts_harder"]
    assert b["max_Tt4"][1] < b["max_Tt4"][0]          # peak TIT falls
    assert b["min_phi"][1] > b["min_phi"][0]          # surge margin rises
    assert b["fuel_int"][1] < b["fuel_int"][0]        # and it burns less


@pytest.mark.slow
def test_the_trajectories_do_not_converge_at_the_tail(design):
    """ANCHOR P10, SCORED REFUTED AND GATED AS SUCH. The prediction was the right claim in the
    wrong coordinate: D2 is about an EQUILIBRIUM and this march never reaches one — the
    schedule stops at `s = r` but the spools are still spinning up, so the cap keeps moving
    and both legs keep chasing it. Gating the refutation is what stops a later rung from
    quietly re-deriving the claim that failed."""
    m = _rig(design, SensedCapTransient, _sm(PHI_BOTH))
    b = m.cap_bill(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, margin=MARGIN, taus=TAUS,
                   ref="sched", law="none", r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX)
    assert b["wf_tail"] > 1e-3
    assert b["wf_tail"] > b["wf_ramp"]


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
