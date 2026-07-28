"""Rung 59 — THE MATCHED SCHEDULE: the ORDINATE cannot see the stator.

Rung 58 refused the matched-schedule variant — the fuel leg re-derived on the machine it
actually runs on, which is what a FADEC burns in — as a confounded experiment, on the stated
premise that "a stator-armed machine derives a DIFFERENT kappa_ss table". That premise is
FALSE in the ordinate and TRUE only in the abscissa.

THE HEADLINE: a derived schedule's ORDINATE cannot see a stator; only its INDEX can. kappa_ss
= pi_b*f*MFP_A4/[(1+f)*sqrt(Tt4)] is a function of Tt4 ALONE — A4 is choked so the corrected
group is hardware, and Tt3 is pinned by the map-free shaft balances (rung 31's (*)). So
matching is PURE RE-INDEXING: a no-op exactly when the lever leaves the schedule's own
abscissa alone (an LP stator cannot move n_H(Tt4) — rung 39's ONE ARROW), and worth 100 % of
the effect when it does not (an HP stator moves it 3.3–6.7 %).

MEASURED: the abscissa carries 100.00 % of delta_match and the ordinate 0.00 %, proven by
splicing the two tables. And an UNMATCHED schedule MANUFACTURES an interaction — 48–96x too
large on the LP spool, and of the WRONG SIGN on the spool carrying the stator.

CROSS-RUNG: rung 58's concession is discharged as VACUOUS (it ran an LP stator, so its leg was
already the matched leg and its four-cell numbers were never confounded) and its
stator-invariance BOUNDED to one spool.

Reduces: v = 0 gives tuple-level table identity and delta_match EXACTLY 0.0; the same leg
object reproduces rung 58's composite cell bit-for-bit; the design run is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    AccelSchedule, ScheduledStatorTransient, StatorSchedule,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR, V = 0.55, 0.20
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.01, 1.2
N_LO = 0.7557
MARGIN = 0.25
V_HP = 0.10                     # the HP branch; authority saturates at ~0.15 (see the spec)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _st(design=None, **kw):
    return ScheduledStatorTransient(design if design is not None else _design(), FLIGHT, 1.0,
                                    map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _sched(v_max=V, n_lo=N_LO):
    return StatorSchedule(v_max=v_max, n_lo=n_lo)


def _matched(r=0.5, ds=DS, spool="lp", **kw):
    return _st(**kw).matched_credit(FLIGHT, LO, HI, MARGIN, r=r, s_settle=SETTLE, ds=ds,
                                    spool=spool)


# =============================================================================
# THE REDUCE
# =============================================================================

def test_reduce_v_zero_gives_tuple_identity_and_exactly_zero_delta_match():
    """THE STRONG IDENTITY REDUCE. At v = 0 the armed machine IS the bare machine, so it
    derives the SAME equilibria and hence a table equal by PYTHON TUPLE EQUALITY — not to a
    tolerance — and delta_match is exactly 0.0.

    Both arming routes, because they are different code paths: a zero-v_max SCHEDULE (`_arm`
    hands back the same map object) and a zero CONSTANT setting (the constructor never swaps).
    """
    bare = _st()
    L_bare = bare.accel_schedule(FLIGHT, LO, HI, MARGIN)
    for tag, kw in (("sched v_max=0", dict(vsv_sched_lp=_sched(v_max=0.0))),
                    ("const v=0", dict(vsv_lp=0.0))):
        L = _st(**kw).accel_schedule(FLIGHT, LO, HI, MARGIN)
        assert L.kappa == L_bare.kappa, tag
        assert L.n_H == L_bare.n_H, tag


def test_reduce_matched_cell_is_bit_for_bit_rung58_composite():
    """DISPATCH REDUCE. `matched_credit`'s `both_bare_leg` cell runs the SAME machine with the
    SAME bare-derived leg as rung 58's `composite_credit`, so it must reproduce rung 58's
    `both` cell bit-for-bit — and its `neither`/`stator`/`fuel` cells likewise. Rung 59 adds
    cells beside rung 58's; it does not perturb them."""
    m = _st(vsv_sched_lp=_sched())
    L_bare = m.at_stator().accel_schedule(FLIGHT, LO, HI, MARGIN)
    r58 = m.composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS, spool="lp",
                             accel=L_bare)
    r59 = _matched(vsv_sched_lp=_sched())
    for a, b in (("neither", "neither"), ("stator", "stator"), ("fuel", "fuel"),
                 ("both", "both_bare_leg")):
        for key in ("m_i", "m_phi", "s", "v", "min_phi", "fuel_removed", "npts"):
            assert r58["cells"][a][key] == r59["cells"][b][key], (a, b, key)
    assert r58["credit_bare"] == r59["credit_bare"]
    assert r58["interaction"] == r59["interaction_bare_leg"]


def test_reduce_rung58_readers_untouched():
    """Rung 58's own entry points still run and still return its published objects — rung 59
    only ADDS methods to the class."""
    m = _st(vsv_sched_lp=_sched())
    L = m.at_stator().accel_schedule(FLIGHT, LO, HI, MARGIN)
    d = m.engagement_shift(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS, accel=L)
    assert d["leg"] == "accel"
    assert abs(d["rel_limited"]) < 0.01          # rung 58's 0.16 %, at this coarser grid


def test_cycle_untouched_by_rung59_bit_for_bit_rung6():
    """The design run never sees any of this — the whole rung is a separate entry point."""
    gas = Gas.reacting_equilibrium()

    def design():
        return build_turbojet(gas, PI_LPC * PI_HPC, TT4, FLIGHT.p0, **{
            k: v for k, v in REAL.items()
            if k not in ("eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt")
        }, eta_c=0.90, eta_t=0.92).run(FLIGHT, 50.0)

    a = design()
    _st(vsv_hp=V_HP).matched_credit(FLIGHT, LO, HI, MARGIN, r=0.5, ds=0.02)
    b = design()
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.performance.tsfc == b.performance.tsfc


# =============================================================================
# THE GUARDS
# =============================================================================

def test_synthetic_leg_refuses_a_margin_mismatch():
    """The splice exists to isolate the abscissa from the ordinate. Splicing two tables of
    DIFFERENT schedule margins would reintroduce the very leg-change it excludes."""
    bare = _st()
    a = bare.accel_schedule(FLIGHT, LO, HI, 0.25)
    b = bare.accel_schedule(FLIGHT, LO, HI, 0.40)
    ScheduledStatorTransient._synthetic_leg(a, a)          # same margin: fine
    with pytest.raises(AssertionError, match="ONE schedule margin"):
        ScheduledStatorTransient._synthetic_leg(a, b)


def test_matched_credit_needs_an_armed_stator():
    """It differences an ARMED stator against its own bare sibling."""
    with pytest.raises(AssertionError, match="ARMED stator"):
        _st().matched_credit(FLIGHT, LO, HI, MARGIN, ds=DS)


# =============================================================================
# THE FINDINGS
# =============================================================================

@pytest.mark.slow
def test_p1_the_ordinate_is_a_function_of_Tt4_alone():
    """THE ORDINATE CLAIM, checked on the three factors it is built from rather than asserted.

        kappa_ss = pi_b * f(Tt3,Tt4) * MFP_A4 / [(1+f)*sqrt(Tt4)]

    A4 is choked so MFP is hardware; Tt3 is pinned by the map-free shaft balances. So Tt25,
    Tt3, f, MFP and mdot/pt3 — and hence kappa itself — are stator-INVARIANT at fixed Tt4, on
    EITHER spool, to the equilibrium solver's own noise floor.

    NOT to the last bit: `equilibrium`'s Newton converges to a tolerance, so a nonzero setting
    lands ~1e-13 away. Tuple equality is claimed only at v = 0 (the reduce above), and
    asserting it here would claim more than the solver can deliver."""
    for tag, kw in (("LP const", dict(vsv_lp=V)),
                    ("LP sched", dict(vsv_sched_lp=_sched())),
                    ("HP const", dict(vsv_hp=V_HP))):
        d = _st(**kw).schedule_invariance(FLIGHT, LO, HI, MARGIN)
        assert d["d_ordinate"] < 1e-12, (tag, d["d_ordinate"])
        for row in d["chain"]:
            for key in ("d_Tt25", "d_Tt3", "d_f", "d_mfp", "d_ratio", "d_kappa"):
                assert abs(row[key]) < 1e-12, (tag, row["Tt4"], key, row[key])


@pytest.mark.slow
def test_p1_the_abscissa_is_what_splits_the_two_spools():
    """THE SPLIT. An LP stator cannot move n_H(Tt4) — rung 39's ONE ARROW, pi_LPC cancels out
    of the HP-face corrected flow — so its whole table is invariant. An HP stator moves the
    face itself, so the SAME CURVE comes back RE-INDEXED."""
    for tag, kw in (("LP const", dict(vsv_lp=V)), ("LP sched", dict(vsv_sched_lp=_sched()))):
        d = _st(**kw).schedule_invariance(FLIGHT, LO, HI, MARGIN)
        assert d["d_abscissa"] < 1e-12, (tag, d["d_abscissa"])
    d = _st(vsv_hp=V_HP).schedule_invariance(FLIGHT, LO, HI, MARGIN)
    assert d["d_abscissa"] > 0.03, d["d_abscissa"]          # measured 6.69 %
    assert d["d_ordinate"] < 1e-12, d["d_ordinate"]         # ... with the ordinate STILL flat


@pytest.mark.slow
def test_p1_lp_stator_matching_is_a_no_op():
    """RUNG 58's CONCESSION, DISCHARGED AS VACUOUS. It ran an LP stator, so the leg it derived
    once on the bare machine already WAS the matched leg: not merely the margin but the
    engagement time, the fuel removed and the minimum's location are identical."""
    for kw in (dict(vsv_sched_lp=_sched()), dict(vsv_lp=V)):
        d = _matched(**kw)
        assert abs(d["delta_match"]) < 1e-12, (kw, d["delta_match"])
        assert d["interaction_matched"] == pytest.approx(d["interaction_bare_leg"], abs=1e-12)
        assert d["s_eng_matched"] == pytest.approx(d["s_eng_bare_leg"], abs=1e-12)
        assert d["removed_matched"] == pytest.approx(d["removed_bare_leg"], abs=1e-12)


@pytest.mark.slow
def test_p2_the_abscissa_carries_all_of_it():
    """THE ISOLATION, and the answer to "you just swapped in a tighter schedule". Splice the
    two tables: the ARMED index with the BARE values reproduces the matched leg, and the BARE
    index with the ARMED values reproduces the bare one. Measured 100.00 % / 0.00 %."""
    d = _matched(vsv_hp=V_HP)
    assert d["delta_match"] > 1e-3, d["delta_match"]        # a real effect to decompose
    assert d["abscissa_share"] == pytest.approx(1.0, abs=1e-6), d["abscissa_share"]
    assert abs(d["ordinate_share"]) < 1e-6, d["ordinate_share"]


@pytest.mark.slow
def test_p3_an_unmatched_schedule_manufactures_an_interaction():
    """THE PRACTICAL RESULT. On an HP-statored machine, rung 58's bare-machine leg reports an
    interaction far larger than the leg the machine would actually be given — and on the spool
    CARRYING the stator, of the wrong SIGN. Matched, the pair very nearly superposes."""
    lp = _matched(vsv_hp=V_HP, spool="lp")
    assert abs(lp["interaction_bare_leg"]) > 10.0 * abs(lp["interaction_matched"])
    hp = _matched(vsv_hp=V_HP, spool="hp")
    assert hp["interaction_bare_leg"] < 0.0 < hp["interaction_matched"], (
        hp["interaction_bare_leg"], hp["interaction_matched"])


@pytest.mark.slow
def test_the_clamp_blocker_stays_clear():
    """THE STANDING BLOCKER. `AccelSchedule.cap` CLAMPS outside its abscissa bracket, and this
    rung RE-INDEXES that very abscissa — so a leg consulted outside its bracket would be
    running on kappa[0], the envelope edge (rung 48's m -> 0, rung 58's r = 2.0 dormancy), and
    would counterfeit the finding. `matched_credit` asserts it internally; this pins the
    audit itself so a moved band or margin cannot silently void the rung."""
    for kw in (dict(vsv_sched_lp=_sched()), dict(vsv_hp=V_HP)):
        d = _matched(**kw)
        for tag, a in d["audits"].items():
            assert a["clamped"] == 0, (kw, tag, a)
            assert a["n_cuts"] > 0, (kw, tag, "the leg never binds — nothing was audited")
            assert a["lo"] < a["cut_lo"] and a["cut_hi"] < a["hi"], (kw, tag, a)
