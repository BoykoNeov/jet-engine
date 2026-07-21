"""Rung 32 — COMPONENT-MAP MATCHING: the map re-labels the choke-pinned work.

Gates (named in docs/rung32-spec.md § Verification gates):

  1. REDUCE TO RUNG 31 (the spine) — the FLAT map makes MapMatcher.match reproduce
     OffDesignMatcher.match (pi_c, mdot, tau_t, stations, thrust) across a throttle sweep
     (machine-zero at design; <=1e-9 on the sweep). N present but inert. On the REACTING gas.
  2. CYCLE UNTOUCHED — the default design run is bit-for-bit rung 6; building a MapMatcher does
     not perturb it (the rungs-7+ invariant).
  3. THE FINDING (shape-robust) — for >=3 map shapes, off-design pi_c AND mdot fall BELOW the
     flat-map (rung-31) values, same sign, gap growing with throttle. rung 31's "without a map"
     over-claimed: the map bites pi_c/mdot first-order.
  4. WORK IS MAP-FREE — tau_c from the map matcher equals rung 31's to ~1e-4 across the sweep; the
     choke-pinned compressor WORK is unmoved by the map (isolates WHAT the map moves).
  5. TURBINE PINNED IN CORRECTED SPEED (sub-finding) — nu_t stays within ~1% of design, so
     |d eta_t| << |d eta_c| EVEN for a steep turbine map (structural, single-spool N/sqrt(Tt4)).
  6. N ATTACHES + MONOTONE — N/N_d = 1 at design, falls monotonically with Tt4; leading schedule
     robust across the speed-line sigma (bounded spread), magnitude disclaimed.
  7. DIRECTION / CONVERGENCE — hotter Tt4 => higher pi_c, mdot, N; the outer secant converges.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, OffDesignMatcher, MapMatcher, ComponentMap,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

SHAPES = [ComponentMap.flow_dominated(), ComponentMap.pressure_dominated(), ComponentMap.tilted()]


def _fast_matchers():
    """A MapMatcher + a rung-31 OffDesignMatcher on the FAST (thermally_perfect) gas.

    The finding gates are gas-independent physics; the fast gas keeps them cheap (the reacting gas
    re-freezes equilibrium per inner iteration, and the outer secant multiplies that).
    """
    mm = MapMatcher(build_turbojet(Gas.thermally_perfect(), PI_C, TT4, FLIGHT.p0,
                                   nozzle_convergent=True, **REAL), FLIGHT, 1.0)
    base = OffDesignMatcher(build_turbojet(Gas.thermally_perfect(), PI_C, TT4, FLIGHT.p0,
                                           nozzle_convergent=True, **REAL), FLIGHT, 1.0)
    return mm, base


# --------------------------------------------------------------------------- gate 1
def test_reduce_to_rung31():
    """GATE 1 — the FLAT map reproduces rung 31 bit-for-bit (on the REACTING gas)."""
    mm = MapMatcher(build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0,
                                   nozzle_convergent=True, **REAL), FLIGHT, 1.0)
    base = OffDesignMatcher(build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0,
                                           nozzle_convergent=True, **REAL), FLIGHT, 1.0)
    flat = ComponentMap.flat()
    for Tt4 in (1500.0, 1200.0, 900.0):
        mo = mm.match(FLIGHT, Tt4, flat)
        ro = base.match(FLIGHT, Tt4)
        assert abs(mo.pi_c - ro.pi_c) <= 1e-9 * ro.pi_c, f"flat pi_c != rung31 at {Tt4}"
        assert abs(mo.mdot_air - ro.mdot_air) <= 1e-9 * ro.mdot_air, f"flat mdot != rung31 at {Tt4}"
        assert abs(mo.tau_t - ro.tau_t) <= 1e-9 * ro.tau_t, f"flat tau_t != rung31 at {Tt4}"
        assert abs(mo.performance.specific_thrust - ro.performance.specific_thrust) <= 1e-6
        for k in ("2", "3", "4", "5"):
            assert abs(mo.stations[k].Tt - ro.stations[k].Tt) <= 1e-8 * ro.stations[k].Tt
            assert abs(mo.stations[k].pt - ro.stations[k].pt) <= 1e-8 * ro.stations[k].pt
    # At design the flat map returns pi_c = 10 and N = design.
    od = mm.match(FLIGHT, TT4, flat)
    assert abs(od.pi_c - PI_C) < 1e-8 and abs(od.N_ratio - 1.0) < 1e-8 and abs(od.n_corr - 1.0) < 1e-8


# --------------------------------------------------------------------------- gate 2
def test_cycle_untouched():
    """GATE 2 — the default design path is unchanged by rung 32 (bit-for-bit rung 6)."""
    r = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0, **REAL).run(FLIGHT, 1.0)
    assert abs(r.performance.specific_thrust - 798.37) < 0.5, r.performance.specific_thrust
    assert r.M9 > 1.8 and abs(r.p9 - FLIGHT.p0) < 1e-6      # default nozzle: fully expanded
    # Building a MapMatcher (runs a convergent design) must not perturb the default run.
    MapMatcher(build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0,
                              nozzle_convergent=True, **REAL), FLIGHT, 1.0,
               comp_map=ComponentMap.flow_dominated())
    r2 = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0, **REAL).run(FLIGHT, 1.0)
    assert abs(r2.performance.specific_thrust - r.performance.specific_thrust) < 1e-9


# --------------------------------------------------------------------------- gate 3
def test_finding_pi_c_mdot_droop_shape_robust():
    """GATE 3 — a peaked map droops pi_c AND mdot off-design, SAME SIGN across >=3 shapes."""
    mm, base = _fast_matchers()
    for cmap in SHAPES:
        dpc_prev = 0.0
        for Tt4 in (1300.0, 1100.0, 900.0):
            mo = mm.match(FLIGHT, Tt4, cmap)
            ro = base.match(FLIGHT, Tt4)
            dpc = (mo.pi_c - ro.pi_c) / ro.pi_c
            dmd = (mo.mdot_air - ro.mdot_air) / ro.mdot_air
            assert dpc < 0.0, f"pi_c must droop below rung31 off-design (shape {cmap}): {dpc}"
            assert dmd < 0.0, f"mdot must droop below rung31 off-design: {dmd}"
            assert dpc < dpc_prev, "the droop must grow with throttle (deeper Tt4 => larger gap)"
            dpc_prev = dpc
        # At the design point the peaked map still sits at the peak => no droop.
        od = mm.match(FLIGHT, TT4, cmap)
        assert abs(od.eta_c - REAL["eta_c"]) < 1e-6, "eta_c must equal design at the design point"


# --------------------------------------------------------------------------- gate 4
def test_work_tau_c_is_map_free():
    """GATE 4 — tau_c (the compressor WORK) is choke-pinned: map matches rung 31 to ~1e-4."""
    mm, base = _fast_matchers()
    for cmap in SHAPES:
        for Tt4 in (1300.0, 1100.0, 900.0):
            mo = mm.match(FLIGHT, Tt4, cmap)
            ro = base.match(FLIGHT, Tt4)
            rel = abs(mo.tau_c - ro.tau_c) / ro.tau_c
            assert rel < 1e-4, f"tau_c should be map-free (shape {cmap}, Tt4 {Tt4}): rel {rel:.2e}"
            # And it moves pi_c by FAR more than tau_c (the map bites pi_c, not the work).
            dpc = abs(mo.pi_c - ro.pi_c) / ro.pi_c
            if Tt4 <= 1100.0:
                assert dpc > 30.0 * rel, "the map must move pi_c far more than tau_c"


# --------------------------------------------------------------------------- gate 5
def test_turbine_pinned_in_corrected_speed():
    """GATE 5 — nu_t barely moves (single spool), so |d eta_t| << |d eta_c| even for a STEEP map."""
    mm, _ = _fast_matchers()
    steep = ComponentMap(a=0.25, b=0.05, sigma=0.3, a_t=0.5)   # 25x the representative turbine curvature
    for Tt4 in (1300.0, 1100.0, 900.0, 700.0):
        mo = mm.match(FLIGHT, Tt4, steep)
        assert abs(mo.nu_t - 1.0) < 0.01, f"turbine corrected speed should stay within 1%: {mo.nu_t}"
        d_eta_t = abs(mo.eta_t - REAL["eta_t"])
        d_eta_c = abs(mo.eta_c - REAL["eta_c"])
        assert d_eta_t < 1e-3, f"turbine eta must barely move even for a steep map: {d_eta_t:.2e}"
        assert d_eta_t < 0.02 * d_eta_c, "turbine droop must be orders below the compressor droop"


# --------------------------------------------------------------------------- gate 6
def test_N_attaches_monotone_and_schedule_robust():
    """GATE 6 — N/N_d = 1 at design, falls monotonically; leading schedule robust across sigma."""
    mm, _ = _fast_matchers()
    flat = ComponentMap.flat()
    od = mm.match(FLIGHT, TT4, flat)
    assert abs(od.N_ratio - 1.0) < 1e-8, "N/N_d must equal 1 at the design point"
    Ns = [mm.match(FLIGHT, float(t), flat).N_ratio for t in (1500, 1300, 1100, 900, 700)]
    for a, b in zip(Ns, Ns[1:]):
        assert a > b, "N/N_d must fall monotonically as Tt4 falls"
    # Leading schedule robust across speed-line sigma: bounded spread at a fixed Tt4.
    variants = [ComponentMap(sigma=s) for s in (0.0, 0.3, 0.6, 1.0)]
    N_sig = [mm.match(FLIGHT, 900.0, v).N_ratio for v in variants]
    spread = (max(N_sig) - min(N_sig)) / N_sig[0]
    assert spread < 0.05, f"N schedule should be robust across sigma (spread {spread:.3f})"
    assert spread > 1e-4, "but N IS genuinely sigma-dependent (not a tautology)"


# --------------------------------------------------------------------------- gate 7
def test_direction_and_convergence():
    """GATE 7 — hotter Tt4 => higher pi_c, mdot, N; the outer secant converges across the sweep."""
    mm, _ = _fast_matchers()
    cmap = ComponentMap.flow_dominated()
    ods = [mm.match(FLIGHT, float(t), cmap) for t in (1500, 1300, 1100, 900)]
    for a, b in zip(ods, ods[1:]):
        assert a.pi_c > b.pi_c, "pi_c must fall as Tt4 falls"
        assert a.mdot_air > b.mdot_air, "mdot must fall as Tt4 falls"
        assert a.N_ratio > b.N_ratio, "N must fall as Tt4 falls"
        assert a.nozzle_choked, "these points are on the choked branch"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-32 gates passed")
