"""Rung 38 — TWO-SPOOL MATCHING: the triangular cascade (no simultaneous solve).

Gates (named in docs/rung38-spec.md § Verification gates):

  1. REDUCE — lp_disabled=True is OffDesignMatcher BY CONSTRUCTION (exact dispatch, not a
     knob-to-zero limit): the two-spool machinery is never entered.
  2. NON-TAUTOLOGICAL — on a self-consistent CPG gas, an INDEPENDENT bare-math cascade
     (no Gas/Component/TwoSpoolMatcher calls) reproduces the shipped solver's
     (pi_lpc, pi_hpc, tau_hpt, tau_lpt) to machine zero, across a throttle sweep; tau_hpt/tau_lpt
     are themselves Tt4-independent on CPG, and (the rung-31 gate-5 mirror) DO drift on the
     reacting gas over the same window -- isolating the CPG constancy as the gas-model effect.
  3. THE FINDING — no 2x2 solve; each compressor's OWN efficiency is a terminal leaf.
     eta_hpc moves pi_hpc but leaves pi_lpc BIT-FOR-BIT unchanged (Step 3 never reads it);
     eta_lpc moves pi_lpc but leaves pi_hpc BIT-FOR-BIT unchanged (Step 4 never reads it).
     Contrast: eta_hpt/eta_lpt (turbine/energy-path parameters) move BOTH ratios -- they
     shape the shared Tt45/Tt5 cascade, so this is NOT a claim that the spools don't talk.
  4. SCOPE GUARD — throttling into nozzle-unchoke raises the documented scope error.
  5. PHYSICALITY / DIRECTION — pi_lpc>1, pi_hpc>1, 0<tau_hpt<1, 0<tau_lpt<1,
     pt4 > pt25 > pt2; hotter Tt4 pumps harder (same rung-31 sanity contract, doubled).
  6. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet,
    OffDesignMatcher, TwoSpoolMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC = 3.0, 6.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)


def _two_spool_design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                     nozzle_convergent=True, **REAL)


def _reacting_matcher():
    gas = Gas.reacting_equilibrium()
    design = _two_spool_design(gas)
    return TwoSpoolMatcher(design, FLIGHT, 1.0), design.run(FLIGHT, 1.0)


def _cpg_gas():
    # self-consistent CPG dual gas (rung-31's own recipe): R_t = (g-1)/g*cp_t exactly.
    g, cp = 1.3, 1239.0
    Rt = (g - 1.0) / g * cp
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp, R_t=Rt, hPR=42.8e6)


# --------------------------------------------------------------------------- gate 1
def test_reduce_lp_disabled_is_offdesign_matcher():
    """GATE 1 — lp_disabled=True IS an OffDesignMatcher by construction, not a limit."""
    gas = Gas.reacting_equilibrium()
    single = build_turbojet(gas, PI_HPC, TT4, FLIGHT.p0, pi_d=REAL["pi_d"],
                             eta_c=REAL["eta_hpc"], eta_b=REAL["eta_b"], pi_b=REAL["pi_b"],
                             eta_t=REAL["eta_hpt"], eta_m=REAL["eta_m"], pi_n=REAL["pi_n"],
                             nozzle_convergent=True)
    plain = OffDesignMatcher(single, FLIGHT, 1.0)
    degenerate = TwoSpoolMatcher(single, FLIGHT, 1.0, lp_disabled=True)

    for Tt4 in (1500.0, 1300.0, 900.0):
        a, b = plain.match(FLIGHT, Tt4), degenerate.match(FLIGHT, Tt4)
        assert a.pi_c == b.pi_c, "lp_disabled must reproduce OffDesignMatcher BIT-FOR-BIT"
        assert a.mdot_air == b.mdot_air
        assert a.thrust == b.thrust
        assert a.tau_t == b.tau_t


# --------------------------------------------------------------------------- gate 2
def test_cpg_independent_cascade():
    """GATE 2 — an independent bare-math cascade reproduces the shipped solver (CPG gas)."""
    gas = _cpg_gas()
    design = _two_spool_design(gas)
    m = TwoSpoolMatcher(design, FLIGHT, 1.0)

    gc = (gas.gamma_c - 1.0) / gas.gamma_c
    gt = (gas.gamma_t - 1.0) / gas.gamma_t
    cp_c, cp_t, hPR = gas.cp_c, gas.cp_t, gas.hPR
    eta_lpc, eta_hpc = REAL["eta_lpc"], REAL["eta_hpc"]
    eta_hpt, eta_lpt = REAL["eta_hpt"], REAL["eta_lpt"]
    eta_m, eta_b, pi_n = REAL["eta_m"], REAL["eta_b"], REAL["pi_n"]

    # Freestream in closed form (rung-1 CPG stagnation relations) -- no Gas/Engine calls.
    stag = 1.0 + 0.5 * (gas.gamma_c - 1.0) * FLIGHT.M0 ** 2
    Tt2 = FLIGHT.T0 * stag
    pt2 = FLIGHT.p0 * stag ** (1.0 / gc) * REAL["pi_d"]

    def bisect_turbine(area_ratio, eta_t):
        """pi_t/sqrt(tau_t) = area_ratio, tau_t = 1 - eta_t*(1 - pi_t**gt) -- CPG closed form
        (MFP* is Tt-INDEPENDENT for a CPG gas, so the area ratio alone is the target)."""
        def tau_of(pi_t):
            return 1.0 - eta_t * (1.0 - pi_t ** gt)

        def resid(pi_t):
            return pi_t / tau_of(pi_t) ** 0.5 - area_ratio

        lo, hi = 0.02, 0.999
        flo, fhi = resid(lo), resid(hi)
        assert flo < 0.0 < fhi
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            fm = resid(mid)
            if flo * fm <= 0.0:
                hi = mid
            else:
                lo, flo = mid, fm
            if hi - lo < 1e-14:
                break
        pi_t = 0.5 * (lo + hi)
        return pi_t, tau_of(pi_t)

    area_hp = m.A4 / m.A45
    area_lp = m.A45 / (m.A8 * pi_n)

    def bare_cascade(Tt4):
        pi_hpt, tau_hpt = bisect_turbine(area_hp, eta_hpt)
        Tt45 = Tt4 * tau_hpt
        pi_lpt, tau_lpt = bisect_turbine(area_lp, eta_lpt)
        Tt5 = Tt45 * tau_lpt

        f = m.f_design
        for _ in range(60):
            dh_lpt = eta_m * (1.0 + f) * cp_t * (Tt45 - Tt5)
            Tt25 = Tt2 + dh_lpt / cp_c
            Tt25s = Tt2 + eta_lpc * (Tt25 - Tt2)
            pi_lpc = (Tt25s / Tt2) ** (1.0 / gc)

            dh_hpt = eta_m * (1.0 + f) * cp_t * (Tt4 - Tt45)
            Tt3 = Tt25 + dh_hpt / cp_c
            Tt3s = Tt25 + eta_hpc * (Tt3 - Tt25)
            pi_hpc = (Tt3s / Tt25) ** (1.0 / gc)

            h4 = cp_t * Tt4
            f_new = (h4 - cp_c * Tt3) / (REAL["eta_b"] * hPR - h4)
            if abs(f_new - f) < 1e-14:
                f = f_new
                break
            f = f_new
        return pi_lpc, pi_hpc, tau_hpt, tau_lpt

    for Tt4 in (1500.0, 1300.0, 1100.0, 1000.0):
        pi_lpc_ref, pi_hpc_ref, tau_hpt_ref, tau_lpt_ref = bare_cascade(Tt4)
        od = m.match(FLIGHT, Tt4)
        assert abs(od.pi_lpc - pi_lpc_ref) < 1e-8 * pi_lpc_ref, (od.pi_lpc, pi_lpc_ref)
        assert abs(od.pi_hpc - pi_hpc_ref) < 1e-8 * pi_hpc_ref, (od.pi_hpc, pi_hpc_ref)
        assert abs(od.tau_hpt - tau_hpt_ref) < 1e-9, (od.tau_hpt, tau_hpt_ref)
        assert abs(od.tau_lpt - tau_lpt_ref) < 1e-9, (od.tau_lpt, tau_lpt_ref)

    # tau_HPT, tau_LPT are Tt4-INDEPENDENT on CPG (the MFP-constant structural fact).
    rows = [m.match(FLIGHT, float(t)) for t in (1500, 1300, 1100, 1000)]
    for od in rows:
        assert abs(od.tau_hpt - rows[0].tau_hpt) < 1e-9
        assert abs(od.tau_lpt - rows[0].tau_lpt) < 1e-9

    # MIRROR (the rung-31 gate-5 parallel, doubled): the reacting gas DOES drift, over the
    # SAME choked throttle window -- isolating the CPG constancy above as the gas-model effect.
    mr, _ = _reacting_matcher()
    hot, cold = mr.match(FLIGHT, 1500.0), mr.match(FLIGHT, 650.0)
    drift_hpt = abs(hot.tau_hpt - cold.tau_hpt) / hot.tau_hpt
    drift_lpt = abs(hot.tau_lpt - cold.tau_lpt) / hot.tau_lpt
    assert drift_hpt > 0.02, f"reacting tau_HPT should drift >2% over the choked window: {drift_hpt:.4f}"
    assert drift_lpt > 0.01, f"reacting tau_LPT should drift >1% over the choked window: {drift_lpt:.4f}"


# --------------------------------------------------------------------------- gate 3
def test_triangularity_is_the_finding():
    """GATE 3 — each compressor's OWN efficiency is a terminal leaf (no 2x2 solve); turbine
    parameters are NOT leaves -- they move both ratios (not a "spools don't talk" claim)."""
    m, _ = _reacting_matcher()
    state0, _ = m._fs_engine.freestream(FLIGHT, m.mdot_air_design)
    Tt2, pt2 = state0.Tt, m.pi_d_max * state0.pt
    f = 0.02
    pt4 = m.pi_b * m.pi_hpc_design * m.pi_lpc_design * pt2
    wgas = m._working_gas(f, TT4, pt4)

    base = m._cascade(wgas, Tt2, TT4, f)

    # eta_hpc: Step 4's OWN pressure-inversion leaf. Must not reach pi_lpc (or Tt25, its input).
    m.eta_hpc = 0.55
    c = m._cascade(wgas, Tt2, TT4, f)
    assert c["pi_lpc"] == base["pi_lpc"], "pi_lpc must be BIT-FOR-BIT unchanged by eta_hpc"
    assert c["Tt25"] == base["Tt25"]
    assert c["pi_hpc"] != base["pi_hpc"], "pi_hpc SHOULD move with its own eta_hpc"
    m.eta_hpc = REAL["eta_hpc"]

    # eta_lpc: Step 3's OWN pressure-inversion leaf. Must not reach pi_hpc (Tt25 unaffected).
    m.eta_lpc = 0.55
    c = m._cascade(wgas, Tt2, TT4, f)
    assert c["pi_lpc"] != base["pi_lpc"], "pi_lpc SHOULD move with its own eta_lpc"
    assert c["pi_hpc"] == base["pi_hpc"], (
        "pi_hpc must be BIT-FOR-BIT unchanged by eta_lpc -- it is a dead end for the HP spool")
    m.eta_lpc = REAL["eta_lpc"]

    # CONTRAST: eta_hpt/eta_lpt are energy-path parameters (shape Tt45/Tt5) -- NOT leaves.
    # They legitimately move BOTH ratios; this is why "no 2x2 solve" != "spools don't talk".
    m.eta_hpt = 0.70
    c = m._cascade(wgas, Tt2, TT4, f)
    assert c["pi_lpc"] != base["pi_lpc"], "eta_hpt (HP turbine) SHOULD move pi_lpc via Tt45"
    assert c["pi_hpc"] != base["pi_hpc"]
    m.eta_hpt = REAL["eta_hpt"]

    m.eta_lpt = 0.70
    c = m._cascade(wgas, Tt2, TT4, f)
    assert c["pi_lpc"] != base["pi_lpc"]
    assert c["pi_hpc"] != base["pi_hpc"], "eta_lpt (LP turbine) SHOULD move pi_hpc via Tt25"
    m.eta_lpt = REAL["eta_lpt"]


# --------------------------------------------------------------------------- gate 4
def test_nozzle_unchoke_is_out_of_scope():
    """GATE 4 — deep throttle (nozzle unchoke) raises the documented scope error."""
    m, _ = _reacting_matcher()
    try:
        m.match(FLIGHT, 600.0)
        raised = False
    except AssertionError as e:
        raised = True
        assert "OUT OF SCOPE" in str(e)
    assert raised, "nozzle unchoke should raise the rung-38 scope guard, not silently mis-solve"


# --------------------------------------------------------------------------- gate 5
def test_physicality_and_direction():
    """GATE 5 — sanity + direction: hotter Tt4 pumps both spools harder."""
    m, ref = _reacting_matcher()
    od = m.match(FLIGHT, TT4)
    assert abs(od.pi_lpc - PI_LPC) < 1e-6, f"pi_lpc did not reduce to design: {od.pi_lpc}"
    assert abs(od.pi_hpc - PI_HPC) < 1e-6, f"pi_hpc did not reduce to design: {od.pi_hpc}"
    assert abs(od.mdot_ratio - 1.0) < 1e-6
    assert abs(od.performance.specific_thrust - ref.performance.specific_thrust) < 1e-4

    hot = m.match(FLIGHT, 1500.0)
    cold = m.match(FLIGHT, 1100.0)
    assert hot.pi_lpc > cold.pi_lpc and hot.pi_hpc > cold.pi_hpc
    assert hot.mdot_ratio > cold.mdot_ratio
    assert hot.thrust > cold.thrust
    for od_ in (hot, cold):
        assert od_.pi_lpc > 1.0 and od_.pi_hpc > 1.0
        assert 0.0 < od_.tau_hpt < 1.0 and 0.0 < od_.tau_lpt < 1.0
        s = od_.stations
        assert s["4"].pt > s["25"].pt > s["2"].pt


# --------------------------------------------------------------------------- gate 6
def test_cycle_untouched():
    """GATE 6 — the default single-spool design path is unchanged (bit-for-bit rung 6)."""
    gas = Gas.reacting_equilibrium()
    r = build_turbojet(gas, 10.0, TT4, FLIGHT.p0,
                        pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
                        eta_t=0.90, eta_m=0.99, pi_n=0.98).run(FLIGHT, 1.0)
    assert abs(r.performance.specific_thrust - 798.37) < 0.5, r.performance.specific_thrust
    assert r.M9 > 1.8 and abs(r.p9 - FLIGHT.p0) < 1e-6

    # Building a two-spool design + matcher must not perturb the single-spool default run.
    gas2 = Gas.reacting_equilibrium()
    _two_spool_design(gas2)
    TwoSpoolMatcher(_two_spool_design(Gas.reacting_equilibrium()), FLIGHT, 1.0)
    r2 = build_turbojet(Gas.reacting_equilibrium(), 10.0, TT4, FLIGHT.p0,
                         pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
                         eta_t=0.90, eta_m=0.99, pi_n=0.98).run(FLIGHT, 1.0)
    assert abs(r2.performance.specific_thrust - r.performance.specific_thrust) < 1e-9


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-38 gates passed")
