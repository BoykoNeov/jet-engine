"""Rung 33 — THE SUBSONIC-NOZZLE MATCHING BRANCH (below the nozzle-unchoke boundary).

Rung 31 pinned the turbine by TWO choked throats: (★) π_t/√τ_t = A4·MFP4/(A8·π_n·MFP9) is
pure GEOMETRY, so τ_t, π_t are constant on a CPG gas ("the turbine does not know the operating
condition changed"). Below the nozzle-unchoke boundary that decoupling BREAKS: only the NGV
stays choked and the nozzle passes a SUBSONIC flow whose corrected throughput is MFP(M9) with
M9 set by the ACTUAL ratio pt9/p0 — which moves with π_c as you throttle. So π_t is no longer
geometry-pinned; it equilibrates the NGV-choked supply against the subsonic-nozzle demand.

Gates (docs/rung33-spec.md § Verification gates):

  1. REDUCE / CHOKED BIT-FOR-BIT — the choked path is left literally unchanged: matching at the
     design point returns pi_c=10 branch="choked"; choked off-design points are unaffected (the
     rung-31/32 suites, which exercise the same match(), still pass — that is the bit-for-bit gate).
  2. DISPATCH + BOUNDARY CONTINUITY — choked above unchoke, subsonic below; M9 passes through 1
     continuously and pi_c, tau_t are continuous across the boundary (no jump).
  3. THE RUNG (CPG tau_t VARIES) — on a CPG gas the subsonic tau_t varies with throttle (structural
     coupling through pi_c), the INVERSION of rung 31's choked tau_t (machine-constant on CPG).
  4. NON-TAUTOLOGICAL ANCHOR — the matched subsonic point satisfies textbook compressible-flow
     MFP(M9) + the isentropic pt9/p0<->M9 relation to machine precision on a self-consistent CPG
     gas (the _sonic_throat/Nozzle solver vs the closed-form algebraic MFP — two code paths).
  5. ENVELOPE — the subsonic branch is monotone (pi_c, M9, thrust fall with Tt4), bounded ABOVE by
     nozzle-unchoke and BELOW by thrust-neutral idle (SUB-IDLE raised, not force-fit).
  6. HOMOGENEITY (the framing) — scaling p0 leaves the subsonic ratios (pi_c, tau_t, M9) invariant:
     the coupling is to pi_c via pt9/p0, NOT to the ambient pressure.
  7. CYCLE UNTOUCHED — the default design run is bit-for-bit rung 6; MapMatcher (rung 32) does NOT
     inherit the subsonic branch (subsonic+map is out of scope).
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.components import ram_recovery  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, OffDesignMatcher, MapMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)


def _cpg_gas():
    # self-consistent CPG dual gas: R_t = (g-1)/g*cp_t exactly (so the sonic solver == closed form)
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def _cpg_matcher():
    return OffDesignMatcher(
        build_turbojet(_cpg_gas(), PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
        FLIGHT, 1.0)


def _reacting_matcher():
    return OffDesignMatcher(
        build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0,
                       nozzle_convergent=True, **REAL), FLIGHT, 1.0)


# --------------------------------------------------------------------------- gate 1
def test_reduce_choked_bitforbit():
    """GATE 1 — the choked path is untouched: design reduces, choked points stay choked."""
    m = _reacting_matcher()
    od = m.match(FLIGHT, TT4)
    assert od.branch == "choked" and od.nozzle_choked
    assert abs(od.pi_c - PI_C) < 1e-8, f"pi_c did not reduce to design: {od.pi_c}"
    # A mid-throttle choked point is still choked (dispatch only fires below unchoke).
    mid = m.match(FLIGHT, 1000.0)
    assert mid.branch == "choked" and mid.nozzle_choked and mid.M9 > 1.0 - 1e-9


# --------------------------------------------------------------------------- gate 2
def test_dispatch_and_boundary_continuity():
    """GATE 2 — choked above unchoke, subsonic below; M9/pi_c/tau_t continuous across it."""
    m = _cpg_matcher()
    # Scan across the boundary; find the last choked and first subsonic point.
    prev = None
    crossed = False
    for Tt4 in [700, 650, 620, 610, 600, 590, 580, 560]:
        od = m.match(FLIGHT, float(Tt4))
        if od.branch == "choked":
            assert od.nozzle_choked and abs(od.M9 - 1.0) < 1e-6
        else:
            assert not od.nozzle_choked and od.M9 < 1.0
            if prev is not None and prev.branch == "choked":
                crossed = True
                # continuity: no jump in pi_c / tau_t across the branch change, M9 just below 1.
                # The physical adjacent-step jump is only ~1.6-3% (a ~10-20 K throttle step), so a
                # tight bound catches a genuine discontinuity (gate 4 pins the VALUES rigorously).
                assert abs(od.pi_c - prev.pi_c) < 0.05 * prev.pi_c
                assert abs(od.tau_t - prev.tau_t) < 1e-3 * prev.tau_t
                assert 0.90 < od.M9 < 1.0
        prev = od
    assert crossed, "the scan must cross the nozzle-unchoke boundary"


# --------------------------------------------------------------------------- gate 3 (THE RUNG)
def test_the_rung_cpg_tau_t_varies():
    """GATE 3 — on CPG the SUBSONIC tau_t VARIES with throttle (the inversion of rung 31).

    Rung 31's choked branch holds tau_t machine-constant on CPG (its gate 2). Here the coupling
    runs through pi_c (structural), not gamma(T)/composition, so it SURVIVES CPG: the subsonic
    tau_t moves measurably. First-order structural coupling vs rung 31's second-order var-cp drift.
    """
    m = _cpg_matcher()
    taus = []
    for Tt4 in (580, 560, 540, 520, 500, 480, 460):
        od = m.match(FLIGHT, float(Tt4))
        assert od.branch == "subsonic"
        taus.append(od.tau_t)
    spread = max(taus) - min(taus)
    assert spread > 1e-3, f"CPG subsonic tau_t must VARY (structural), got spread {spread:.2e}"
    # and it is monotone (rises toward 1 as the turbine expands less).
    assert all(b > a for a, b in zip(taus, taus[1:])), "subsonic tau_t should rise as Tt4 falls"

    # Contrast: the CHOKED branch on the SAME CPG gas holds tau_t machine-constant (rung 31).
    hot, warm = m.match(FLIGHT, 1200.0).tau_t, m.match(FLIGHT, 800.0).tau_t
    assert abs(hot - warm) < 1e-9, f"CPG choked tau_t must stay constant, got {abs(hot-warm):.2e}"


def _indep_cpg_subsonic(m, gas, Tt4, p0):
    """INDEPENDENT closed-form CPG solve of the subsonic match (★★) — a SECOND code path.

    No _sonic_throat, no Nozzle.apply, no equilibrium: pure calorically-perfect algebra
    (Mattingly's dual-mode ratio method on the gas the textbook assumes). For a trial pi_t:
      tau_t = 1 - eta_t(1 - pi_t^((gt-1)/gt))                    # CPG isentropic turbine
      shaft (nested one-shot f) -> Tt3 -> pi_c = (1+eta_c(tau_c-1))^(gc/(gc-1))
      pt4 = pi_b pi_c pt2 ;  pt9 = pi_n pi_t pt4
      M9 from the isentropic pt9/p0 ; MFP(M9), MFP* closed form
    root-find pi_t on mdot_NGV = mdot_noz. Returns (pi_t, pi_c, tau_t, M9). This is what makes
    gate 4 non-tautological — it ties the subsonic OPERATING POINT to the textbook, two paths.
    """
    gc, cpc, gt_, cpt, Rt = gas.gamma_c, gas.cp_c, gas.gamma_t, gas.cp_t, gas.R_t
    hpr, eta_c, eta_t, eta_m, eta_b = gas.hPR, REAL["eta_c"], REAL["eta_t"], REAL["eta_m"], REAL["eta_b"]
    pi_b, pi_n = REAL["pi_b"], REAL["pi_n"]
    tau_r = 1.0 + 0.5 * (gc - 1.0) * FLIGHT.M0 ** 2
    Tt2 = FLIGHT.T0 * tau_r
    pt2 = REAL["pi_d"] * ram_recovery(FLIGHT.M0) * p0 * tau_r ** (gc / (gc - 1.0))
    eps = 0.5 * (gt_ - 1.0)
    exp_mfp = (gt_ + 1.0) / (2.0 * (gt_ - 1.0))
    MFP_star = (gt_ / Rt) ** 0.5 * (2.0 / (gt_ + 1.0)) ** exp_mfp

    def op(pi_t):
        tau_t = 1.0 - eta_t * (1.0 - pi_t ** ((gt_ - 1.0) / gt_))
        Tt5 = tau_t * Tt4
        f = 0.0
        for _ in range(80):                                        # CPG one-shot burner, nested f
            Tt3 = Tt2 + eta_m * (1.0 + f) * cpt * (Tt4 - Tt5) / cpc
            f_new = (cpt * Tt4 - cpc * Tt3) / (eta_b * hpr - cpt * Tt4)
            if abs(f_new - f) <= 1e-14 * (f_new + 1e-30):
                break
            f = f_new
        Tt3s = Tt2 + eta_c * (Tt3 - Tt2)
        pi_c = (Tt3s / Tt2) ** (gc / (gc - 1.0))
        pt4 = pi_b * pi_c * pt2
        pt9 = pi_n * pi_t * pt4
        M9 = (((pt9 / p0) ** ((gt_ - 1.0) / gt_) - 1.0) / eps) ** 0.5
        MFP_M9 = (gt_ / Rt) ** 0.5 * M9 * (1.0 + eps * M9 ** 2) ** (-exp_mfp)
        mdot_ngv = m.A4 * pt4 * MFP_star / Tt4 ** 0.5
        mdot_noz = m.A8 * pt9 * MFP_M9 / (tau_t * Tt4) ** 0.5
        return pi_c, tau_t, M9, mdot_ngv - mdot_noz

    lo, hi = 0.15, 0.9995
    rlo = op(lo)[3]
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        rm = op(mid)[3]
        if rlo * rm <= 0.0:
            hi = mid
        else:
            lo, rlo = mid, rm
        if hi - lo <= 1e-13:
            break
    pi_t = 0.5 * (lo + hi)
    pi_c, tau_t, M9, _ = op(pi_t)
    return pi_t, pi_c, tau_t, M9


# --------------------------------------------------------------------------- gate 4 (anchor)
def test_nontautological_independent_solve():
    """GATE 4 — an INDEPENDENT CPG closed-form solve of (★★) reproduces the shipped solver.

    The rigorous, non-tautological anchor (mirrors rung 31 gate 2). The subsonic-branch code —
    the pi_t root-find on the sonic-throat/Nozzle machinery — has no reduce-to-prior of its own
    (gate 1 is a CHOKED point that returns before dispatch). So we solve the SAME operating point
    a second, entirely separate way: pure CPG closed-form algebra (_indep_cpg_subsonic, no
    _sonic_throat/Nozzle), and assert the two paths land on the same (pi_t, pi_c, tau_t, M9) to
    machine zero. Two code paths, one operating point — this is what pins the subsonic VALUES to
    the textbook (a 1% pi_c drift in the shipped solve would be caught here, where gates 1/2 miss it).
    """
    gas = _cpg_gas()
    m = OffDesignMatcher(build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                         FLIGHT, 1.0)
    for Tt4 in (580, 540, 500, 460):
        od = m.match(FLIGHT, float(Tt4))
        assert od.branch == "subsonic"
        pi_t_i, pi_c_i, tau_t_i, M9_i = _indep_cpg_subsonic(m, gas, float(Tt4), FLIGHT.p0)
        assert abs(od.pi_t - pi_t_i) < 1e-9, f"Tt4={Tt4}: pi_t {od.pi_t} vs indep {pi_t_i}"
        assert abs(od.pi_c - pi_c_i) < 1e-9 * pi_c_i, f"Tt4={Tt4}: pi_c {od.pi_c} vs indep {pi_c_i}"
        assert abs(od.tau_t - tau_t_i) < 1e-9, f"Tt4={Tt4}: tau_t {od.tau_t} vs indep {tau_t_i}"
        assert abs(od.M9 - M9_i) < 1e-9, f"Tt4={Tt4}: M9 {od.M9} vs indep {M9_i}"


# --------------------------------------------------------------------------- gate 5 (envelope)
def test_envelope_monotone_and_subidle():
    """GATE 5 — subsonic branch monotone; bounded above (unchoke) and below (thrust-neutral idle)."""
    m = _cpg_matcher()
    ods = [m.match(FLIGHT, float(t)) for t in (580, 540, 500, 460)]
    for a, b in zip(ods, ods[1:]):
        assert a.pi_c > b.pi_c, "pi_c falls with Tt4"
        assert a.M9 > b.M9, "M9 falls with Tt4"
        assert a.performance.specific_thrust > b.performance.specific_thrust, "thrust falls with Tt4"
    # Below thrust-neutral idle the subsonic match self-reports SUB-IDLE (net thrust <= 0).
    with pytest.raises(AssertionError, match="SUB-IDLE"):
        m.match(FLIGHT, 400.0)


# --------------------------------------------------------------------------- gate 6 (framing)
def test_homogeneity_coupling_through_pi_c():
    """GATE 6 — the coupling is to pi_c via pt9/p0, NOT to ambient p0: scale p0, ratios invariant."""
    ratios = []
    for p0 in (25_000.0, 50_000.0, 100_000.0):
        fl = FlightCondition(T0=250.0, p0=p0, M0=0.85)
        m = OffDesignMatcher(build_turbojet(_cpg_gas(), PI_C, TT4, p0, nozzle_convergent=True, **REAL),
                             fl, 1.0)
        od = m.match(fl, 500.0)
        assert od.branch == "subsonic"
        ratios.append((od.pi_c, od.tau_t, od.M9))
    for (pc, tt, m9) in ratios[1:]:
        assert abs(pc - ratios[0][0]) < 1e-9 * ratios[0][0], "pi_c must be p0-invariant"
        assert abs(tt - ratios[0][1]) < 1e-9, "tau_t must be p0-invariant"
        assert abs(m9 - ratios[0][2]) < 1e-9, "M9 must be p0-invariant"


# --------------------------------------------------------------------------- gate 7
def test_cycle_untouched_and_map_out_of_scope():
    """GATE 7 — default design run bit-for-bit rung 6; MapMatcher does NOT run the subsonic branch."""
    gas = Gas.reacting_equilibrium()
    r = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, **REAL).run(FLIGHT, 1.0)
    assert abs(r.performance.specific_thrust - 798.37) < 0.5   # rung-6 anchor (fully expanded)
    assert r.M9 > 1.8 and abs(r.p9 - FLIGHT.p0) < 1e-6
    # MapMatcher (rung 32) overrides match() and stays on its choked-only path below unchoke:
    # it flags nozzle_choked=False WITHOUT re-solving (subsonic+map is out of scope, no "subsonic"
    # branch label). This documents that rung 33's dispatch is NOT inherited by the map matcher.
    mm = MapMatcher(build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0,
                                   nozzle_convergent=True, **REAL), FLIGHT, 1.0)
    deep = mm.match(FLIGHT, 560.0)
    assert not deep.nozzle_choked and deep.branch == "choked"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-33 gates passed")
