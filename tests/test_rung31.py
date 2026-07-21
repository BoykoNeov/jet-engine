"""Rung 31 — OFF-DESIGN MATCHING: the operating point becomes an OUTPUT.

Gates (named in docs/rung31-spec.md § Verification gates):

  1. REDUCE TO DESIGN (the spine) — the matching solver at the design flight + Tt4
     reproduces the design-run pi_c, stations, mdot, thrust (to solver tolerance). This is
     reduce-BY-CONSTRUCTION: A4/A8 captured from the design run, the compressor inverse is
     the exact inverse of Compressor.apply, the f fixed point starts converged.
  2. THE SOLVER IS RIGHT (non-tautological) — on a CPG gas the TPG matching solve reproduces
     Mattingly's closed-form referencing: tau_t, pi_t EXACTLY constant across the throttle
     sweep; pi_c = [1+eta_c(tau_c-1)]^(gc/(gc-1)); the Tt4/(tau_r T0) slaving factor constant.
     Without this, gate 1 only exercises the design point itself.
  3. CYCLE UNTOUCHED — the default (specified-pi_c) design path is bit-for-bit rung 6; the
     matcher is a separate entry point.
  4. THE VERDICT / RUNNING LINE — throttle down and pi_c, mdot, thrust fall together
     (compressor slaved); the nozzle-unchoke Tt4 bounds the branch and is flagged, not lied
     about.
  5. THE DRIFT (the finding) — on the reacting gas tau_t is NOT constant along the throttle
     sweep, while on the CPG gas it is constant to machine precision; the drift is the
     variable-cp physics.
  6. DIRECTION — hotter Tt4 => higher pi_c, higher mdot, higher thrust.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, OffDesignMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)


def _reacting_matcher():
    gas = Gas.reacting_equilibrium()
    design = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    return OffDesignMatcher(design, FLIGHT, 1.0), design.run(FLIGHT, 1.0)


def _cpg_gas():
    # self-consistent CPG dual gas: R_t = (g-1)/g*cp_t exactly (so the sonic solver == closed form)
    g, cp = 1.3, 1239.0
    Rt = (g - 1.0) / g * cp
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp, R_t=Rt, hPR=42.8e6)


# --------------------------------------------------------------------------- gate 1
def test_reduce_to_design():
    """GATE 1 — matching AT the design condition reproduces the design run."""
    m, ref = _reacting_matcher()
    od = m.match(FLIGHT, TT4)
    assert abs(od.pi_c - PI_C) < 1e-8, f"pi_c did not reduce to design: {od.pi_c}"
    assert abs(od.mdot_ratio - 1.0) < 1e-8, f"mdot did not reduce: {od.mdot_ratio}"
    assert abs(od.performance.specific_thrust - ref.performance.specific_thrust) < 1e-6
    for k in ("2", "3", "4", "5"):
        a, b = od.stations[k], ref.stations[k]
        assert abs(a.Tt - b.Tt) < 1e-6 * b.Tt, f"station {k} Tt drifted"
        assert abs(a.pt - b.pt) < 1e-6 * b.pt, f"station {k} pt drifted"
    assert abs(od.stations["4"].far - ref.stations["4"].far) < 1e-9
    # The design reference itself is the choked-convergent (rung-30) point.
    assert od.nozzle_choked and abs(od.M9 - 1.0) < 1e-6


# --------------------------------------------------------------------------- gate 2
def test_cpg_closed_form_referencing():
    """GATE 2 — on a CPG gas the matching solve == Mattingly's closed-form referencing."""
    gas = _cpg_gas()
    design = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    m = OffDesignMatcher(design, FLIGHT, 1.0)

    gc = (1.4 - 1.0) / 1.4
    tau_r = 1.0 + 0.2 * FLIGHT.M0 ** 2
    rows = [m.match(FLIGHT, float(Tt4)) for Tt4 in (1500, 1300, 1100, 1000)]

    # (a) tau_t, pi_t EXACTLY constant (the choked-turbine + choked-nozzle pin, CPG).
    tau_t0, pi_t0 = rows[0].tau_t, rows[0].pi_t
    for od in rows:
        assert abs(od.tau_t - tau_t0) < 1e-9, f"CPG tau_t not constant: {od.tau_t} vs {tau_t0}"
        assert abs(od.pi_t - pi_t0) < 1e-9, f"CPG pi_t not constant: {od.pi_t} vs {pi_t0}"

    # (b) the Tt4/(tau_r T0) slaving factor is constant.
    def slave(od):
        f = od.stations["4"].far
        return (od.tau_c - 1.0) / ((1.0 + f) * od.Tt4 / (tau_r * FLIGHT.T0))
    s0 = slave(rows[0])
    for od in rows:
        assert abs(slave(od) - s0) < 1e-9 * s0, "CPG slaving factor not constant"

    # (c) pi_c == the closed-form compressor map.
    for od in rows:
        closed = (1.0 + 0.88 * (od.tau_c - 1.0)) ** (1.0 / gc)
        assert abs(od.pi_c - closed) < 1e-9 * closed, "pi_c != closed-form compressor map"


# --------------------------------------------------------------------------- gate 3
def test_cycle_untouched():
    """GATE 3 — the default design path is unchanged by rung 31 (bit-for-bit rung 6)."""
    gas = Gas.reacting_equilibrium()
    r = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, **REAL).run(FLIGHT, 1.0)
    # Rung-6 anchors: the ideal (fully expanded) design specific thrust ~798, M9 supersonic.
    assert abs(r.performance.specific_thrust - 798.37) < 0.5, r.performance.specific_thrust
    assert r.M9 > 1.8 and abs(r.p9 - FLIGHT.p0) < 1e-6   # default nozzle: fully expanded
    # Building a matcher (which runs a convergent design) must not perturb this default run.
    gas2 = Gas.reacting_equilibrium()
    design2 = build_turbojet(gas2, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    OffDesignMatcher(design2, FLIGHT, 1.0)
    r2 = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0, **REAL).run(FLIGHT, 1.0)
    assert abs(r2.performance.specific_thrust - r.performance.specific_thrust) < 1e-9


# --------------------------------------------------------------------------- gate 4/6
def test_running_line_and_direction():
    """GATE 4/6 — the running line is monotone; hotter pumps harder; unchoke is flagged."""
    m, _ = _reacting_matcher()
    Tt4s = [1500, 1300, 1100, 900]
    ods = [m.match(FLIGHT, float(t)) for t in Tt4s]
    # Monotone: throttle up => higher pi_c, mdot, thrust (compressor slaved to the line).
    for a, b in zip(ods, ods[1:]):
        assert a.pi_c > b.pi_c, "pi_c must fall as Tt4 falls"
        assert a.mdot_ratio > b.mdot_ratio, "mdot must fall as Tt4 falls"
        assert a.thrust > b.thrust, "thrust must fall as Tt4 falls"
    # All these are on the choked branch.
    assert all(od.nozzle_choked for od in ods)
    # The nozzle unchokes when throttled far enough — flagged, not lied about.
    deep = m.match(FLIGHT, 550.0)
    assert not deep.nozzle_choked, "deep throttle should unchoke the nozzle (branch boundary)"


# --------------------------------------------------------------------------- gate 5
def test_tau_t_drift_is_the_finding():
    """GATE 5 — reacting tau_t DRIFTS along the throttle sweep; CPG holds it constant."""
    # Reacting gas: measurable drift.
    m, _ = _reacting_matcher()
    hot = m.match(FLIGHT, 1500.0).tau_t
    cold = m.match(FLIGHT, 800.0).tau_t
    drift = abs(hot - cold) / hot
    assert drift > 0.02, f"reacting tau_t should drift >2% over 2:1 throttle, got {drift:.4f}"

    # CPG gas: constant to machine precision (isolates the drift as variable-cp physics).
    gas = _cpg_gas()
    mc = OffDesignMatcher(build_turbojet(gas, PI_C, TT4, FLIGHT.p0,
                                         nozzle_convergent=True, **REAL), FLIGHT, 1.0)
    hot_c = mc.match(FLIGHT, 1500.0).tau_t
    cold_c = mc.match(FLIGHT, 800.0).tau_t
    assert abs(hot_c - cold_c) < 1e-9, f"CPG tau_t must be constant, drift {abs(hot_c-cold_c):.2e}"


def test_tau_t_drift_killtest_gamma_curve():
    """GATE 5 (kill-test) — the drift's DRIVER is the gamma_t(T) curve, not composition.

    Three-gas ladder, drift measured over the CHOKED branch (1500 vs 800, both choked):
      CPG (fixed gamma)                    -> 0 (machine)
      thermally_perfect (var cp(T), FROZEN comp) -> most of the drift    [the gamma(T) curve]
      reacting_equilibrium (var cp + comp) -> full drift
    Within one operating point both throats carry the SAME frozen composition, so R cancels in
    MFP4/MFP9 and the residual IS a gamma_t(T)-curve effect. The frozen-composition gas must
    reproduce the MAJORITY of the reacting drift (composition is the minority contributor).
    """
    def drift(gas):
        m = OffDesignMatcher(build_turbojet(gas, PI_C, TT4, FLIGHT.p0,
                                            nozzle_convergent=True, **REAL), FLIGHT, 1.0)
        h, c = m.match(FLIGHT, 1500.0).tau_t, m.match(FLIGHT, 800.0).tau_t   # both choked
        return abs(h - c) / h

    d_cpg = drift(_cpg_gas())
    d_tpg = drift(Gas.thermally_perfect())          # variable cp(T), frozen composition
    d_react = drift(Gas.reacting_equilibrium())
    assert d_cpg < 1e-9, "CPG must not drift"
    assert d_react > 0.02, "reacting drift should exceed 2%"
    # the gamma(T) curve alone carries the MAJORITY of the drift (measured ~81%)
    assert d_tpg > 0.6 * d_react, f"gamma(T) curve should drive most of the drift: {d_tpg/d_react:.2f}"
    assert d_tpg < d_react, "composition adds a (minority) further contribution"


# --------------------------------------------------------------------------- M0 axis
def test_m0_ram_lapse():
    """The flight-Mach trends: pi_c falls, mdot rises with M0 (ram) at fixed Tt4."""
    m, _ = _reacting_matcher()
    lo = m.match(FlightCondition(T0=250.0, p0=50_000.0, M0=0.5), TT4)
    hi = m.match(FlightCondition(T0=250.0, p0=50_000.0, M0=2.0), TT4)
    assert hi.pi_c < lo.pi_c, "pi_c must fall as flight Mach rises (ram raises Tt2)"
    assert hi.mdot_ratio > lo.mdot_ratio, "mdot must rise as flight Mach rises (higher pt4)"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-31 gates passed")
