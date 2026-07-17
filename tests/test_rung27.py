"""Rung 27 — NO FREEZE-OUT: is the frozen-NO assumption carried since rung 7 actually EARNED?

Every NO number since rung 7 ASSUMES the station-4 exhaust NO freezes through the nozzle; rung 14/17's
dropped-clamp corollary reads `max_a = x_no_frozen/x_no_e(T9) ≫ 1` OFF that assumption. Rung 26 then
showed the MAJOR pool freezes only partway down. Rung 27 applies rung-26's anchored-clock / local-Da
machinery to a `_tau_no_destroy` clock built from rung 7's OWN Zeldovich reverse rates (zero new
constants) and finds the assumption is EARNED: `Da_NO ≪ 1` from entry at EVERY Tt4
(docs/rung27-spec.md, docs/plans/rung27-anchor-no-freeze-out.md).

THE HEADLINE is a DERIVED assumption + an INVERTED kill test, NOT a moving freeze point (rung 26's
headline has no analogue: NO is frozen from entry everywhere). This file certifies the ROBUST
structure — the bit-for-bit reduce to rung 14/17's clamp, the rate_scale limits, the frozen-from-entry
finding at every Tt4, the kill test (the two terms AGREE — both drive — inverting rung 26), the margin
trend (the Da_NO-vs-Da_recomb separation narrows with Tt4, no crossing), the clamp-earned corollary,
robustness to the NO level, cycle-untouched, guards. It DELIBERATELY does NOT assert a freeze LOCATION
or a moving freeze point.
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, NOFreezeOut, FreezeOut,
    _equilibrium_composition, _no_freeze_out_expand, _expand_nozzle,
    _tau_no_destroy, _tau_chem_recomb, _Ru,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)
_PI_C = 10.0

_DP_CACHE: dict = {}


def _dp(Tt4):
    """Cycle state (far, Tt3, Tt4, pt4, Tt9, pt9, p9) at a given combustor temperature, built ONCE."""
    if Tt4 not in _DP_CACHE:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, _PI_C, Tt4, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 1.0)
        st3, st4, st5 = r.stations["3"], r.stations["4"], r.stations["5"]
        _DP_CACHE[Tt4] = dict(g=g, far=st4.far, Tt3=st3.Tt, Tt4=st4.Tt, pt4=st4.pt,
                              Tt9=st5.Tt, pt9=_LOSSES["pi_n"] * st5.pt, p9=r.p9, cycle_V9=r.V9)
    return _DP_CACHE[Tt4]


def _nf(Tt4, phi_p=1.0, no_freeze_out=None):
    d = _dp(Tt4)
    fo = no_freeze_out if no_freeze_out is not None else NOFreezeOut()
    return d["g"].no_freeze_out_nozzle(d["far"], d["Tt3"], d["Tt4"], d["pt4"],
                                       d["Tt9"], d["pt9"], d["p9"], phi_p, fo)


# --- GATE 1: RUNG 14/26 UNTOUCHED — a new method beside nozzle_flow / freeze_out_nozzle ---------- #

def test_neighbors_untouched():
    """no_freeze_out_nozzle is a NEW method beside nozzle_flow (rung 14) and freeze_out_nozzle (rung
    26); calling it does not perturb either, which still return their results on the same inputs."""
    d = _dp(2200.0)
    nf_before = d["g"].nozzle_flow(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                   x_no_frozen=1e-4)
    fz_before = d["g"].freeze_out_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                         FreezeOut())
    _ = _nf(2200.0)
    nf_after = d["g"].nozzle_flow(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                  x_no_frozen=1e-4)
    fz_after = d["g"].freeze_out_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                        FreezeOut())
    assert nf_after.V9_frozen == nf_before.V9_frozen and nf_after.max_a == nf_before.max_a
    assert fz_after.V9_freeze == fz_before.V9_freeze and fz_after.s_freeze == fz_before.s_freeze


# --- GATE 2: REDUCE — Da_NO off ⇒ marched NO == frozen NO == rung 14/17's clamp, BIT-FOR-BIT ----- #

def test_da_off_is_rung14_clamp_bit_for_bit():
    """The load-bearing reduce. With NO relaxation off (Da_NO ≡ 0) the marched NO stays FROZEN at the
    entry value, so the clamp max_a reproduces rung-14/17's fully-frozen number to the ULP. Driven two
    ways: (a) the rate_scale→0 limit through the public method; (b) _no_freeze_out_expand with a literal
    zero da_no_fn (the direct reduce, and the drift tripwire). Also pins the exit T9 == _expand_nozzle's
    (the clamp-denominator hinge)."""
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    nf = d["g"].nozzle_flow(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                            x_no_frozen=None)  # T9/x_no_e only; feed the zoned NO next
    # (a) public method, rate_scale → 0
    s0 = _nf(2200.0, no_freeze_out=NOFreezeOut(rate_scale=1e-12))
    assert s0.x_no_relaxed == s0.x_no_frozen          # NO did not move
    assert s0.max_a == s0.max_a_frozen                # clamp == rung 14/17's, bit-for-bit
    # (b) direct march with a literal-zero da_no_fn — exit == entry, and T9 matches _expand_nozzle
    x_no_in = s0.x_no_frozen
    T9m, x_no_out, _xe, max_a_m, Da_e, Da_x = _no_freeze_out_expand(
        ce, d["far"], d["Tt9"], d["pt9"], d["p9"], x_no_in, (lambda c, T, p: 0.0), 400)
    T9f, _V9f, _ = _expand_nozzle(ce, d["far"], d["Tt9"], d["pt9"], d["p9"], shifting=False)
    assert x_no_out == x_no_in                        # frozen: exit == entry exactly
    assert T9m == T9f                                 # same bisection ⇒ same clamp denominator
    assert Da_e == 0.0 and Da_x == 0.0
    assert max_a_m == s0.max_a_frozen                 # and the clamp lands on rung 14/17's value


# --- GATE 3: LIMIT — rate_scale→∞ ⇒ NO tracks equilibrium ⇒ the clamp goes DORMANT (max_a→1) ----- #

def test_rate_scale_infinity_dormant_clamp():
    """rate_scale scales Da_NO uniformly: →∞ drives the relaxation to completion, so NO tracks
    equilibrium down the nozzle and the dropped clamp goes DORMANT (max_a→1, relaxed_fraction→1). The
    counterpoint to the anchored ≪1 finding — the clamp fires ONLY because the real Da_NO is tiny."""
    fast = _nf(2200.0, no_freeze_out=NOFreezeOut(rate_scale=1e12))
    assert fast.relaxed_fraction > 0.99, f"rate_scale→∞ did not equilibrate: {fast.relaxed_fraction}"
    assert abs(fast.max_a - 1.0) < 0.05, f"clamp not dormant at rate_scale→∞: max_a={fast.max_a}"
    # and it is BELOW the anchored (frozen) clamp — relaxing NO only ever lowers max_a toward 1
    assert fast.max_a < _nf(2200.0).max_a


# --- GATE 4: THE FINDING — FROZEN FROM ENTRY at EVERY Tt4 (unlike the major pool) ---------------- #

def test_frozen_from_entry_at_every_tt4():
    """Da_NO < 1 at the entry for EVERY Tt4 — NO never relaxes, so the frozen-NO assumption is DERIVED.
    This is the clean contrast with rung 26's MAJOR pool, which is frozen-from-entry only LEAN and
    relaxes hot: here the SAME sweep leaves NO frozen throughout. The relaxed fraction is ≈0 (|·|≪1) at
    the anchored rate at every Tt4."""
    Tt4s = (1500.0, 1650.0, 1800.0, 2000.0, 2200.0)
    for Tt4 in Tt4s:
        s = _nf(Tt4)
        assert s.frozen_from_entry and s.Da_entry < 1.0, f"NO not frozen from entry at Tt4={Tt4}"
        assert abs(s.relaxed_fraction) < 1e-2, f"NO relaxed at Tt4={Tt4}: {s.relaxed_fraction}"
    # the contrast: rung 26's major pool RELAXES hot (not frozen from entry) on the same hot point
    d = _dp(2200.0)
    major = d["g"].freeze_out_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                     FreezeOut())
    assert not major.frozen_from_entry               # majors relax hot ...
    assert _nf(2200.0).frozen_from_entry             # ... but NO does not. The rung.


# --- GATE 5: KILL TEST — the two terms AGREE (both drive), INVERTING rung 26 -------------------- #

def test_kill_test_both_terms_drive_inverting_rung26():
    """On the standalone _tau_no_destroy clock (frozen pool pinned): kill-T (k pinned ⇒ density alone)
    GROWS τ_NO (drives freezing); kill-p (c_tot pinned ⇒ T alone) ALSO grows τ_NO (drives freezing).
    Both DRIVE — they AGREE. This INVERTS rung 26, whose two terms OPPOSE (density won DESPITE a k that
    rose on cooling); here the Arrhenius k CRATERS on cooling and joins density. Verified against rung
    26's clock on the same path (kill-p there makes τ SHRINK — opposite sign)."""
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    Tt9, pt9, p9 = d["Tt9"], d["pt9"], d["p9"]
    T9 = _nf(2200.0).T9_frozen                        # the real frozen exit temperature
    c_in = pt9 / (_Ru * Tt9)                          # entry density (SI), for the kill-c pin

    t_in = _tau_no_destroy(ce, Tt9, pt9)
    t_ex = _tau_no_destroy(ce, T9, p9)
    t_killT = _tau_no_destroy(ce, T9, p9, kill_T=Tt9)   # k pinned → density alone
    t_killc = _tau_no_destroy(ce, T9, p9, kill_c=c_in)  # c_tot pinned → T alone
    assert t_ex > t_in                                # net: τ_NO grows ⇒ freezes harder on cooling
    assert t_killT > t_in, "kill-T (density alone) should DRIVE freezing (τ grows)"
    assert t_killc > t_in, "kill-p (T alone) should DRIVE freezing (τ grows) — the INVERSION"
    # rung 26's recombination clock on the SAME path: kill-M (T alone) makes τ SHRINK (opposite sign)
    c_M_in = pt9 / (_Ru * Tt9) / 1.0e6               # entry [M] in CHEMKIN units
    r_in = _tau_chem_recomb(ce, Tt9, pt9)
    r_killM = _tau_chem_recomb(ce, T9, p9, kill_M=c_M_in)
    assert r_killM < r_in, "rung 26's kill-M should make τ SHRINK (Da rises) — the sign rung 27 inverts"


# --- GATE 6: MARGIN TREND — the Da_NO/Da_recomb separation NARROWS with Tt4 (no crossing) -------- #

def test_margin_narrows_with_tt4_no_crossing():
    """Da_NO climbs steeply with Tt4 (steeply Arrhenius) while Da_recomb climbs slowly (Ea=0), so the
    separation τ_NO/τ_recomb at entry NARROWS monotonically with Tt4 — the honest 'motion-like'
    finding. But Da_NO stays ≪ 1 at every Tt4: NO crossing is claimed (extrapolating one would be an
    overclaim)."""
    Tt4s = (1500.0, 1650.0, 1800.0, 2000.0, 2200.0)
    seps, da_no_entries = [], []
    for Tt4 in Tt4s:
        d = _dp(Tt4)
        ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
        sep = _tau_no_destroy(ce, d["Tt9"], d["pt9"]) / _tau_chem_recomb(ce, d["Tt9"], d["pt9"])
        seps.append(sep)
        da_no_entries.append(_nf(Tt4).Da_entry)
    assert all(a > b for a, b in zip(seps, seps[1:])), f"separation not narrowing: {seps}"
    assert seps[0] / seps[-1] > 1e3, f"separation should collapse orders across the band: {seps}"
    assert all(x < 1.0 for x in da_no_entries), f"Da_NO crossed 1 (no crossing claimed): {da_no_entries}"
    assert all(a < b for a, b in zip(da_no_entries, da_no_entries[1:])), "Da_NO entry not monotone"


# --- GATE 7: CLAMP EARNED — the MARCHED NO fires the clamp (max_a > 1), == rung 14/17's number --- #

def test_clamp_fires_on_marched_no():
    """The rung-14/17 clamp firing (max_a ≫ 1) is EARNED, not assumed: the anchored MARCHED NO fires it
    too, and lands on rung 14/17's fully-frozen number to the ≪1 anchored margin. Hot and lean alike."""
    for Tt4 in (1500.0, 2200.0):
        s = _nf(Tt4)
        assert s.clamp_fires and s.max_a > 1.0
        # marched vs fully-frozen: equal to the anchored-margin tolerance (relative)
        assert abs(s.max_a - s.max_a_frozen) / s.max_a_frozen < 1e-2


# --- GATE 8: ROBUST to the NO LEVEL — the clock is [NO]-independent, so max_a ∝ x_no_frozen ------ #

def test_clamp_scales_linearly_with_no_level():
    """The anchored clock `_tau_no_destroy` is [NO]_e- and a-INDEPENDENT (it takes only comp,T,p), so
    the freeze answer does not depend on which frozen NO is fed. Consequence: since NO stays frozen,
    max_a ∝ x_no_frozen — feeding the direct march 2× the NO gives 2× the clamp. (The public method
    fixes the NO at the zoned value, so this drives the march helper directly.)"""
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    args = (ce, d["far"], d["Tt9"], d["pt9"], d["p9"])
    da0 = (lambda c, T, p: 0.0)
    _, _, _, a_1x, _, _ = _no_freeze_out_expand(*args, 1e-4, da0, 400)
    _, _, _, a_2x, _, _ = _no_freeze_out_expand(*args, 2e-4, da0, 400)
    assert abs(a_2x / a_1x - 2.0) < 1e-9              # frozen ⇒ strictly linear in the NO level


# --- GATE 9: CYCLE UNTOUCHED (pure diagnostic) ------------------------------------------------- #

def test_cycle_untouched():
    d = _dp(2200.0)
    st4_far_before, cycle_V9_before = d["far"], d["cycle_V9"]
    _ = _nf(2200.0)
    r = build_turbojet(Gas.reacting_equilibrium(), _PI_C, 2200.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 1.0)
    assert r.stations["4"].far == st4_far_before
    assert r.V9 == cycle_V9_before


# --- GATE 10: GUARDS --------------------------------------------------------------------------- #

def test_guards():
    d = _dp(2200.0)
    with pytest.raises(AssertionError):        # L must be positive
        NOFreezeOut(L=0.0)
    with pytest.raises(AssertionError):        # nstep floor
        NOFreezeOut(nstep=99)
    with pytest.raises(AssertionError):        # rate_scale positive
        NOFreezeOut(rate_scale=0.0)
    with pytest.raises(AssertionError):        # requires the equilibrium gas
        Gas.thermally_perfect().no_freeze_out_nozzle(
            d["far"], d["Tt3"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"], 1.0, NOFreezeOut())
    with pytest.raises(AssertionError):        # rejects a back-pressure above the total pressure
        d["g"].no_freeze_out_nozzle(d["far"], d["Tt3"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"],
                                    d["pt9"] * 1.5, 1.0, NOFreezeOut())


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
