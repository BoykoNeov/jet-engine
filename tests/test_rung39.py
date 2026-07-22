"""Rung 39 — TWO-SPOOL + COMPONENT MAPS: the cascade acquires a DIRECTION.

Gates (named in docs/rung39-spec.md § Verification gates):

   1. REDUCE — FLAT maps on both spools reproduce rung 38's TwoSpoolMatcher (bit-for-bit).
   2. REDUCE (the ladder) — lp_disabled dispatch: flat -> rung 31 OffDesignMatcher,
      shaped -> rung 32 MapMatcher, both bit-for-bit BY CONSTRUCTION (no LP hardware built).
   3. NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-spool MAP cascade (no Gas/Component/
      ComponentMap/TwoSpoolMapMatcher calls; its own bisections, speed-line inversions and
      efficiency fixed points by damped substitution rather than the shipped secant)
      reproduces (pi_LPC, pi_HPC, eta_LPC, eta_HPC, n_L, n_H) across a throttle sweep.
   4. FINDING A — THE ASYMMETRY. Compressor maps only: eta_LPC leaves pi_HPC BIT-FOR-BIT
      unchanged (the (dagger) cancellation) while eta_HPC MOVES pi_LPC (negative sign).
      Contrast: eta_HPT/eta_LPT move BOTH -- so this is not "the spools don't talk".
   5. FINDING A — the weak back-arrow: a turbine map DOES open the closed leaf, but >=50x
      weaker than the HP->LP arrow (the RATIO is disclaimed; measured 119x-548x).
   6. FINDING B1 — slip == 1 on CPG + flat maps is a STRUCTURAL identity: exact at every
      throttle AND under a deliberately perturbed f ((1+f) cancels in N_L/N_H).
   7. FINDING B2 — the rung-31-gate-5 MIRROR: the same flat maps break the identity on the
      variable-cp gases, and on the SAME CPG gas the MAP channel is the larger of the two.
   8. FINDING B3 — direction: N_L/N_H falls monotonically with throttle, >=3 shape pairs.
      Magnitude DISCLAIMED -- only the sign is gated.
   9. SCOPE GUARD — nozzle unchoke raises the documented rung-38 scope error.
  10. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    OffDesignMatcher, MapMatcher, TwoSpoolMatcher, TwoSpoolMapMatcher, ram_recovery,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC = 3.0, 6.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

# Compressor-island-only shape pairs (a_t = 0): the CLEAN structural test for finding A.
SHAPES_C = {
    "flow_dom":  (ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                  ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "press_dom": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                  ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0)),
    "tilted":    (ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                  ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
    "mixed":     (ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                  ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)),
}


def _cpg_gas():
    """Self-consistent CPG dual gas (rung 31/38's recipe): R_t = (g-1)/g*cp_t exactly."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def _two_spool_design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _single_design():
    """A plain single-spool design (its compressor plays the HPC role) — the reduce ladder."""
    return build_turbojet(Gas.reacting_equilibrium(), PI_HPC, TT4, FLIGHT.p0,
                          pi_d=REAL["pi_d"], eta_c=REAL["eta_hpc"], eta_b=REAL["eta_b"],
                          pi_b=REAL["pi_b"], eta_t=REAL["eta_hpt"], eta_m=REAL["eta_m"],
                          pi_n=REAL["pi_n"], nozzle_convergent=True)


def _mm(gas, map_lp=None, map_hp=None):
    return TwoSpoolMapMatcher(_two_spool_design(gas), FLIGHT, 1.0,
                              map_lp=map_lp, map_hp=map_hp)


def _fixed_inputs(mm, Tt4):
    """The converged (Tt2, pt2, f, pt4) of a matched point — rung 38 gate-3's isolation
    protocol, so the outer f loop's own (separately disclosed) cross-talk cannot confound
    a perturbation reading."""
    od = mm.match(FLIGHT, Tt4)
    state0, _ = mm._fs_engine.freestream(FLIGHT, mm.mdot_air_design)
    Tt2 = state0.Tt
    pt2 = mm.pi_d_max * ram_recovery(FLIGHT.M0) * state0.pt
    f = od.stations["4"].far
    pt4 = mm.pi_b * od.pi_hpc * od.pi_lpc * pt2
    return Tt2, pt2, f, pt4


def _perturbed(mm, wgas, Tt2, pt2, Tt4, f, attr, delta):
    old = getattr(mm, attr)
    setattr(mm, attr, old - delta)
    try:
        return mm._cascade_map(wgas, Tt2, pt2, Tt4, f)
    finally:
        setattr(mm, attr, old)


# --------------------------------------------------------------------------- gate 1
def test_reduce_flat_maps_is_rung38():
    """GATE 1 — FLAT maps on both spools reproduce rung 38's TwoSpoolMatcher."""
    gas = Gas.reacting_equilibrium()
    design = _two_spool_design(gas)
    r38 = TwoSpoolMatcher(design, FLIGHT, 1.0)
    r39 = TwoSpoolMapMatcher(design, FLIGHT, 1.0)   # flat by default

    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0):
        a, b = r38.match(FLIGHT, Tt4), r39.match(FLIGHT, Tt4)
        # The flat map holds every eta at design and the remaining arithmetic is rung 38's
        # with two independent sub-expressions reordered -- which lands bit-for-bit.
        assert a.pi_lpc == b.pi_lpc, (Tt4, a.pi_lpc, b.pi_lpc)
        assert a.pi_hpc == b.pi_hpc, (Tt4, a.pi_hpc, b.pi_hpc)
        assert a.tau_hpt == b.tau_hpt and a.tau_lpt == b.tau_lpt
        assert a.mdot_air == b.mdot_air
        assert a.thrust == b.thrust
        # ...and the efficiencies really are the design ones (the map is inert).
        assert b.eta_lpc == REAL["eta_lpc"] and b.eta_hpc == REAL["eta_hpc"]
        assert b.eta_hpt == REAL["eta_hpt"] and b.eta_lpt == REAL["eta_lpt"]


# --------------------------------------------------------------------------- gate 2
def test_reduce_lp_disabled_ladder():
    """GATE 2 — lp_disabled dispatch completes the ladder: flat -> rung 31, shaped -> rung 32."""
    flat_deg = TwoSpoolMapMatcher(_single_design(), FLIGHT, 1.0, lp_disabled=True)
    r31 = OffDesignMatcher(_single_design(), FLIGHT, 1.0)

    shaped = ComponentMap.surge_pressure()
    shp_deg = TwoSpoolMapMatcher(_single_design(), FLIGHT, 1.0,
                                 map_hp=shaped, lp_disabled=True)
    r32 = MapMatcher(_single_design(), FLIGHT, 1.0, comp_map=shaped)

    for Tt4 in (1500.0, 1300.0, 1000.0):
        a, b = r31.match(FLIGHT, Tt4), flat_deg.match(FLIGHT, Tt4)
        assert a.pi_c == b.pi_c and a.mdot_air == b.mdot_air and a.thrust == b.thrust

        p, q = r32.match(FLIGHT, Tt4), shp_deg.match(FLIGHT, Tt4)
        assert p.pi_c == q.pi_c and p.eta_c == q.eta_c and p.N_ratio == q.N_ratio
        assert p.thrust == q.thrust


# --------------------------------------------------------------------------- gate 3
def test_independent_cpg_map_cascade():
    """GATE 3 — an INDEPENDENT bare-math CPG two-spool MAP cascade reproduces the solver.

    No Gas / Component / ComponentMap / TwoSpoolMapMatcher calls inside the reference: closed-form
    CPG thermodynamics, its own turbine bisection, its own speed-line bisection, and efficiency
    fixed points by DAMPED SUBSTITUTION (a genuinely different iteration from the shipped secant).
    Two code paths, one operating point.
    """
    gas = _cpg_gas()
    map_lp, map_hp = SHAPES_C["mixed"]
    mm = _mm(gas, map_lp, map_hp)

    gc = (gas.gamma_c - 1.0) / gas.gamma_c
    gt = (gas.gamma_t - 1.0) / gas.gamma_t
    cp_c, cp_t, hPR = gas.cp_c, gas.cp_t, gas.hPR
    e_lpc0, e_hpc0 = REAL["eta_lpc"], REAL["eta_hpc"]
    eta_hpt, eta_lpt = REAL["eta_hpt"], REAL["eta_lpt"]
    eta_m, eta_b, pi_n = REAL["eta_m"], REAL["eta_b"], REAL["pi_n"]

    # Freestream + the design point, in closed form.
    stag = 1.0 + 0.5 * (gas.gamma_c - 1.0) * FLIGHT.M0 ** 2
    Tt2 = FLIGHT.T0 * stag
    Tt25_d = Tt2 * (1.0 + (PI_LPC ** gc - 1.0) / e_lpc0)
    Tt3_d = Tt25_d * (1.0 + (PI_HPC ** gc - 1.0) / e_hpc0)
    f_d = (cp_t * TT4 - cp_c * Tt3_d) / (eta_b * hPR - cp_t * TT4)
    tau_lpc_d, tau_hpc_d = Tt25_d / Tt2, Tt3_d / Tt25_d

    def bisect(fn, lo, hi, tol=1e-14):
        flo = fn(lo)
        assert flo * fn(hi) < 0.0, "bare bracket fails"
        for _ in range(300):
            mid = 0.5 * (lo + hi)
            fm = fn(mid)
            if flo * fm <= 0.0:
                hi = mid
            else:
                lo, flo = mid, fm
            if hi - lo < tol:
                break
        return 0.5 * (lo + hi)

    def turbine(area_ratio, eta_t):
        """pi_t/sqrt(tau_t) = area_ratio; tau_t = 1 - eta_t*(1-pi_t**gt). MFP* is Tt-independent
        on CPG, so the area ratio alone is the target (rung 38 gate 2's relation)."""
        tau = lambda p: 1.0 - eta_t * (1.0 - p ** gt)                      # noqa: E731
        pi_t = bisect(lambda p: p / tau(p) ** 0.5 - area_ratio, 0.02, 0.999)
        return pi_t, tau(pi_t)

    def psi(cm, phi):
        return 1.0 - cm.sigma * (phi - 1.0) ** 2 - cm.l * (phi - 1.0)

    def solve_n(cm, mflow, tau_c, tau_c_d):
        target = (tau_c - 1.0) / (tau_c_d - 1.0)
        return bisect(lambda n: psi(cm, mflow / n) * n * n - target, 0.1, 2.0)

    def eta_at(cm, base, phi, n):
        return (base - cm.a * (phi - 1.0) ** 2 - cm.b * (n - 1.0) ** 2
                - cm.c * (phi - 1.0) * (n - 1.0))

    area_hp, area_lp = mm.A4 / mm.A45, mm.A45 / (mm.A8 * pi_n)

    def bare(Tt4):
        pi_hpt, tau_hpt = turbine(area_hp, eta_hpt)
        Tt45 = Tt4 * tau_hpt
        pi_lpt, tau_lpt = turbine(area_lp, eta_lpt)
        Tt5 = Tt45 * tau_lpt

        f = f_d
        for _ in range(300):
            Tt25 = Tt2 + eta_m * (1.0 + f) * cp_t * (Tt45 - Tt5) / cp_c
            Tt3 = Tt25 + eta_m * (1.0 + f) * cp_t * (Tt4 - Tt45) / cp_c

            # HP efficiency fixed point — CLOSED (no LP quantity anywhere), by (dagger):
            #   m_H = (pi_HPC/pi_HPC_d)*sqrt(Tt25/Tt25_d)*sqrt(Tt4_d/Tt4)*(1+f_d)/(1+f)
            # (MFP* and A4*pi_b cancel against the design normalization on a CPG gas.)
            e_h = e_hpc0
            for _ in range(600):
                pi_hpc = (1.0 + e_h * (Tt3 / Tt25 - 1.0)) ** (1.0 / gc)
                m_h = ((pi_hpc / PI_HPC) * (Tt25 / Tt25_d) ** 0.5
                       * (TT4 / Tt4) ** 0.5 * (1.0 + f_d) / (1.0 + f))
                n_h = solve_n(map_hp, m_h, Tt3 / Tt25, tau_hpc_d)
                tgt = eta_at(map_hp, e_hpc0, m_h / n_h, n_h)
                if abs(tgt - e_h) < 1e-15:
                    break
                e_h += 0.5 * (tgt - e_h)

            # LP efficiency fixed point — carries pi_HPC, by (ddagger).
            e_l = e_lpc0
            for _ in range(600):
                pi_lpc = (1.0 + e_l * (Tt25 / Tt2 - 1.0)) ** (1.0 / gc)
                m_l = ((pi_hpc * pi_lpc / (PI_HPC * PI_LPC)) * (Tt2 / Tt2) ** 0.5
                       * (TT4 / Tt4) ** 0.5 * (1.0 + f_d) / (1.0 + f))
                n_l = solve_n(map_lp, m_l, Tt25 / Tt2, tau_lpc_d)
                tgt = eta_at(map_lp, e_lpc0, m_l / n_l, n_l)
                if abs(tgt - e_l) < 1e-15:
                    break
                e_l += 0.5 * (tgt - e_l)

            f_new = (cp_t * Tt4 - cp_c * Tt3) / (eta_b * hPR - cp_t * Tt4)
            if abs(f_new - f) < 1e-14:
                f = f_new
                break
            f = f_new
        return pi_lpc, pi_hpc, e_l, e_h, n_l, n_h

    for Tt4 in (1500.0, 1300.0, 1100.0, 1000.0):
        pl, ph, el, eh, nl, nh = bare(Tt4)
        od = mm.match(FLIGHT, Tt4)
        assert abs(od.pi_lpc - pl) < 1e-8 * pl, (Tt4, od.pi_lpc, pl)
        assert abs(od.pi_hpc - ph) < 1e-8 * ph, (Tt4, od.pi_hpc, ph)
        assert abs(od.eta_lpc - el) < 1e-9, (Tt4, od.eta_lpc, el)
        assert abs(od.eta_hpc - eh) < 1e-9, (Tt4, od.eta_hpc, eh)
        assert abs(od.n_lp - nl) < 1e-8, (Tt4, od.n_lp, nl)
        assert abs(od.n_hp - nh) < 1e-8, (Tt4, od.n_hp, nh)


# --------------------------------------------------------------------------- gate 4
def test_finding_a_the_asymmetry():
    """GATE 4 — THE FINDING: the map opens EXACTLY ONE arrow (HP -> LP).

    eta_LPC leaves pi_HPC BIT-FOR-BIT unchanged (the (dagger) cancellation is a code-level
    guarantee -- the HP eta loop reads no LP quantity), while eta_HPC MOVES pi_LPC. Asserted
    across shape pairs x throttles, on CPG AND the reacting gas. The CONTRAST (turbine
    parameters move BOTH) is asserted in the same test, so this cannot be misread as
    "the two spools don't talk".
    """
    d = 0.01
    for gas_name, gas_fn in (("cpg", _cpg_gas), ("reacting", Gas.reacting_equilibrium)):
        shapes = SHAPES_C if gas_name == "cpg" else {"mixed": SHAPES_C["mixed"]}
        for name, (map_lp, map_hp) in shapes.items():
            assert map_lp.a_t == 0.0 and map_hp.a_t == 0.0, "gate 4 is the a_t=0 structural test"
            for Tt4 in (1400.0, 1200.0, 1000.0):
                mm = _mm(gas_fn(), map_lp, map_hp)
                Tt2, pt2, f, pt4 = _fixed_inputs(mm, Tt4)
                wgas = mm._working_gas(f, Tt4, pt4)
                base = mm._cascade_map(wgas, Tt2, pt2, Tt4, f)

                # THE LEAF THAT SURVIVES: eta_LPC cannot reach pi_HPC. Bit-for-bit.
                qL = _perturbed(mm, wgas, Tt2, pt2, Tt4, f, "eta_lpc", d)
                assert qL["pi_hpc"] == base["pi_hpc"], (
                    gas_name, name, Tt4, qL["pi_hpc"], base["pi_hpc"])
                assert qL["Tt3"] == base["Tt3"] and qL["Tt25"] == base["Tt25"]
                assert qL["pi_lpc"] != base["pi_lpc"], "eta_LPC must move its OWN ratio"

                # THE ARROW THE MAP OPENS: eta_HPC DOES reach pi_LPC, negative.
                qH = _perturbed(mm, wgas, Tt2, pt2, Tt4, f, "eta_hpc", d)
                arrow = qH["pi_lpc"] / base["pi_lpc"] - 1.0
                assert arrow < 0.0, (gas_name, name, Tt4, arrow)
                assert abs(arrow) > 1e-5, (gas_name, name, Tt4, arrow)

                # CONTRAST: the turbine/energy-path parameters move BOTH ratios.
                for attr in ("eta_hpt", "eta_lpt"):
                    q = _perturbed(mm, wgas, Tt2, pt2, Tt4, f, attr, d)
                    assert q["pi_lpc"] != base["pi_lpc"] and q["pi_hpc"] != base["pi_hpc"], (
                        gas_name, name, Tt4, attr)


# --------------------------------------------------------------------------- gate 5
def test_finding_a_weak_back_arrow():
    """GATE 5 — a TURBINE map DOES open the closed leaf, but it is orders weaker.

    This is rung 32's "the turbine is pinned in corrected speed" transplanted onto the LP
    spool. Only the SIGN (it opens) and the order-of-magnitude weakness are gated; the exact
    ratio is DISCLAIMED. The asserted bound is a loose 50x against a measured 119x-548x --
    loose BECAUSE the measured ratio rides on the fixed representative a_t=0.02 (a stiffer
    turbine map would shrink it), NOT because 50x is a physically meaningful threshold.
    """
    d = 0.01
    for name, (mL, mH) in SHAPES_C.items():
        map_lp = ComponentMap(a=mL.a, b=mL.b, c=mL.c, sigma=mL.sigma, l=mL.l, a_t=0.02)
        map_hp = ComponentMap(a=mH.a, b=mH.b, c=mH.c, sigma=mH.sigma, l=mH.l, a_t=0.02)
        for Tt4 in (1400.0, 1200.0, 1000.0):
            mm = _mm(_cpg_gas(), map_lp, map_hp)
            Tt2, pt2, f, pt4 = _fixed_inputs(mm, Tt4)
            wgas = mm._working_gas(f, Tt4, pt4)
            base = mm._cascade_map(wgas, Tt2, pt2, Tt4, f)

            qL = _perturbed(mm, wgas, Tt2, pt2, Tt4, f, "eta_lpc", d)
            qH = _perturbed(mm, wgas, Tt2, pt2, Tt4, f, "eta_hpc", d)
            back = abs(qL["pi_hpc"] / base["pi_hpc"] - 1.0)
            arrow = abs(qH["pi_lpc"] / base["pi_lpc"] - 1.0)
            assert back > 0.0, (name, Tt4, "turbine map must OPEN the leaf")
            assert arrow > 50.0 * back, (name, Tt4, arrow, back, arrow / back)


# --------------------------------------------------------------------------- gate 6
def test_finding_b1_slip_identity_is_structural():
    """GATE 6 — slip == 1 on CPG + flat maps, at every throttle AND under a perturbed f.

    Both shaft works are eta_m*(1+f)*cp_t*Tt4*[pure geometry], so (1+f) AND Tt4 cancel in
    N_L/N_H: the identity is f- and Tt4-independent, not a design-point coincidence.
    """
    mm = _mm(_cpg_gas())                                  # flat maps
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0):
        od = mm.match(FLIGHT, Tt4)
        assert abs(od.slip - 1.0) < 1e-9, (Tt4, od.slip)
        assert abs(od.N_lp_ratio - od.N_hp_ratio) < 1e-9 * od.N_hp_ratio

    # The (1+f) cancellation, exercised directly: force f far off its solved value.
    Tt2, pt2, f, pt4 = _fixed_inputs(mm, 1200.0)
    for f_forced in (0.5 * f, f, 2.0 * f, 4.0 * f):
        wgas = mm._working_gas(f_forced, 1200.0, pt4)
        c = mm._cascade_map(wgas, Tt2, pt2, 1200.0, f_forced)
        assert abs(c["slip"] - 1.0) < 1e-9, (f_forced, c["slip"])


# --------------------------------------------------------------------------- gate 7
def test_finding_b2_mirror_and_map_dominance():
    """GATE 7 — the rung-31-gate-5 MIRROR, plus which channel dominates.

    The SAME flat maps that give the exact CPG identity leave a real drift on the
    variable-cp gases (the gas-model channel); on the SAME CPG gas a shaped map breaks it
    harder (the map channel). Both asserted side by side, so the identity above is isolated
    as a CPG statement and the dominance is not a cross-gas comparison.
    """
    COLD = 900.0
    cpg_flat = _mm(_cpg_gas()).match(FLIGHT, COLD).slip
    assert abs(cpg_flat - 1.0) < 1e-9

    # MIRROR: variable-cp gases drift on the SAME flat maps.
    gas_channel = None
    for gas_fn in (Gas.thermally_perfect, Gas.reacting_equilibrium):
        slip = _mm(gas_fn()).match(FLIGHT, COLD).slip
        drift = abs(1.0 - slip)
        assert drift > 5e-3, (gas_fn.__name__, slip)
        gas_channel = max(gas_channel or 0.0, drift)

    # DOMINANCE: on the SAME CPG gas, the map channel is the larger of the two.
    map_lp, map_hp = SHAPES_C["mixed"]
    map_channel = abs(1.0 - _mm(_cpg_gas(), map_lp, map_hp).match(FLIGHT, COLD).slip)
    assert map_channel > gas_channel, (map_channel, gas_channel)


# --------------------------------------------------------------------------- gate 8
def test_finding_b3_slip_direction_shape_robust():
    """GATE 8 — N_L/N_H falls MONOTONICALLY with throttle: the LP spool falls away from the
    HP spool. Sign gated across >=3 disclosed shape pairs; MAGNITUDE DISCLAIMED."""
    grid = (1500.0, 1300.0, 1100.0, 900.0)
    for name, (mL, mH) in SHAPES_C.items():
        map_lp = ComponentMap(a=mL.a, b=mL.b, c=mL.c, sigma=mL.sigma, l=mL.l, a_t=0.02)
        map_hp = ComponentMap(a=mH.a, b=mH.b, c=mH.c, sigma=mH.sigma, l=mH.l, a_t=0.02)
        mm = _mm(_cpg_gas(), map_lp, map_hp)
        slips = [mm.match(FLIGHT, T).slip for T in grid]
        assert abs(slips[0] - 1.0) < 1e-9, (name, slips[0])       # design is the datum
        for i in range(len(slips) - 1):
            assert slips[i + 1] < slips[i], (name, slips)


# --------------------------------------------------------------------------- gate 9
def test_scope_guard_unchoke_raises():
    """GATE 9 — throttling into nozzle-unchoke raises the documented scope error."""
    mm = _mm(_cpg_gas(), *SHAPES_C["mixed"])
    raised = False
    for Tt4 in (700.0, 650.0, 600.0, 550.0, 500.0, 450.0):
        try:
            mm.match(FLIGHT, Tt4)
        except AssertionError as e:
            if "OUT OF SCOPE" in str(e):
                raised = True
                break
        except Exception:
            break
    assert raised, "deep throttle must raise the rung-38 'OUT OF SCOPE' unchoke error"


# --------------------------------------------------------------------------- gate 10
def test_cycle_untouched_rung6():
    """GATE 10 — building/running the rung-39 matcher does not perturb the default cycle."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, pi_d=REAL["pi_d"], eta_c=0.88,
                         eta_b=REAL["eta_b"], pi_b=REAL["pi_b"], eta_t=REAL["eta_hpt"],
                         eta_m=REAL["eta_m"], pi_n=REAL["pi_n"])
    before = eng.run(FLIGHT, 1.0)

    mm = _mm(_cpg_gas(), *SHAPES_C["mixed"])
    mm.match(FLIGHT, 1200.0)

    after = eng.run(FLIGHT, 1.0)
    assert before.performance.specific_thrust == after.performance.specific_thrust
    assert before.performance.tsfc == after.performance.tsfc
    for k in before.stations:
        assert before.stations[k].Tt == after.stations[k].Tt
        assert before.stations[k].pt == after.stations[k].pt


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"  ok  {name}")
    print("rung 39: all gates pass")
