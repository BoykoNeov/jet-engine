"""Rung 40 — THE TWO-SHAFT TRANSIENT: the LP map opens a COMPLEX mode.

Gates (named in docs/rung40-spec.md § Verification gates):

   1. REDUCE — the 2-D equilibrium (Phi_L = Phi_H = 0) reproduces rung 39's
      TwoSpoolMapMatcher.match, via the FORWARD closure only (never calling the matcher,
      so the reduce is non-circular). CPG and reacting.
   2. REDUCE — lp_disabled EXACT DISPATCH to rung 34's SpoolTransient, bit-for-bit (==).
   3. NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-shaft closure (no Gas/Component/
      ComponentMap/TwoSpoolTransient calls; own CPG thermodynamics, own bisections, own
      forward speed lines, own 2-D equilibrium) reproduces (nu_L, nu_H, pi_lpc, pi_hpc) AND
      sigma_crit ON SHAPED MAPS. The shaped value is what ties the object down — reproducing
      the ==1 identity alone would only re-check the reduce (mirrors rung 39 gate 3).
   4. sigma_crit — the INHERITED identity (== 1 on flat+CPG, from rung 39 B1) + its two
      breaking channels (cp(T) curve, the map; map larger), + the REFUTATION that the map's
      shift direction is shape-dependent.
   5. FINDING (i) — STABILITY: a<0, d<0, a*d>b*c at every sampled point (MEASURED), hence
      both eigenvalues negative at every rho in [0.05, 100] (DERIVED from those signs).
   6. FINDING (ii) — THE COMPLEX MODE: b*c<0 for every SHAPED-LP pair, b*c>=0 for every
      FLAT-LP pair (hp-only is the discriminator: HP shaped, LP flat -> no band). Existence
      + sign + mechanism only; the band LOCATION and |Im/Re| are deliberately NOT gated.
   7. SCOPE — sigma_crit is FIRST-INSTANT only: the marched threshold does NOT converge to
      it (asserted as a deliberate NON-convergence so the withdrawn claim cannot creep back).
   8. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    SpoolTransient, TwoSpoolMapMatcher, TwoSpoolTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC = 3.0, 6.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

FLAT = ComponentMap.flat()
LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

# Disclosed shape pairs (a_t = 0 throughout — compressor islands only).
SHAPES = {
    "flat":       (FLAT, FLAT),
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                   ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
    "steep":      (ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2),
                   ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)),
    "lp-only":    (LP_SHAPED, FLAT),      # LP shaped, HP flat
    "hp-only":    (FLAT, HP_SHAPED),      # HP shaped, LP FLAT — the discriminator
}
LP_IS_FLAT = {"flat", "hp-only"}


def _cpg_gas():
    """Self-consistent CPG dual gas (rung 31/38/39's recipe): R_t = (g-1)/g*cp_t exactly."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def _two_spool_design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _tt(gas, map_lp=None, map_hp=None, rho=1.0):
    return TwoSpoolTransient(_two_spool_design(gas), FLIGHT, 1.0,
                             map_lp=map_lp, map_hp=map_hp, rho=rho)


def _single_design():
    """A plain single-spool design (its compressor plays the HPC role) — the reduce ladder."""
    return build_turbojet(Gas.reacting_equilibrium(), PI_HPC, TT4, FLIGHT.p0,
                          pi_d=REAL["pi_d"], eta_c=REAL["eta_hpc"], eta_b=REAL["eta_b"],
                          pi_b=REAL["pi_b"], eta_t=REAL["eta_hpt"], eta_m=REAL["eta_m"],
                          pi_n=REAL["pi_n"], nozzle_convergent=True)


# --------------------------------------------------------------------------- gate 1
def test_reduce_2d_equilibrium_is_rung39():
    """GATE 1 — the 2-D root (Phi_L = Phi_H = 0) lands on rung 39's matched point.

    NON-CIRCULAR: `equilibrium` uses the FORWARD closure only and never calls
    TwoSpoolMapMatcher.match. Rung 34's reduce was a 1-D bracket; this is a genuine 2-D
    Newton from the design start, so it also witnesses that the design point is reachable.
    """
    for gas, sweep in ((_cpg_gas(), (1500.0, 1300.0, 1200.0)),
                       (Gas.reacting_equilibrium(), (1500.0, 1200.0))):
        t = _tt(gas, LP_SHAPED, HP_SHAPED)
        for Tt4 in sweep:
            od = t.match(FLIGHT, Tt4)
            eq = t.equilibrium(FLIGHT, Tt4)
            assert abs(eq["nu_lp"] / od.N_lp_ratio - 1.0) < 1e-10, (Tt4, eq["nu_lp"])
            assert abs(eq["nu_hp"] / od.N_hp_ratio - 1.0) < 1e-10, (Tt4, eq["nu_hp"])
            assert abs(eq["pi_lpc"] / od.pi_lpc - 1.0) < 1e-9, (Tt4, eq["pi_lpc"])
            assert abs(eq["pi_hpc"] / od.pi_hpc - 1.0) < 1e-9, (Tt4, eq["pi_hpc"])
            assert abs(eq["mdot_air"] / od.mdot_air - 1.0) < 1e-9, (Tt4, eq["mdot_air"])
            # And the residuals really are zero (not just the speeds agreeing).
            assert abs(eq["Phi_lp"]) < 1e-9 and abs(eq["Phi_hp"]) < 1e-9


# --------------------------------------------------------------------------- gate 2
def test_reduce_lp_disabled_is_rung34_bit_for_bit():
    """GATE 2 — EXACT DISPATCH: lp_disabled builds NO two-shaft state at all.

    `__init__` constructs and holds a plain rung-34 SpoolTransient and forwards to it, so
    the fields compare == (not a converged limit) — the rung 38/39 contract, one rung on.
    """
    design = _single_design()
    deg = TwoSpoolTransient(design, FLIGHT, 1.0, map_hp=HP_SHAPED, lp_disabled=True)
    ref = SpoolTransient(design, FLIGHT, 1.0, comp_map=HP_SHAPED)
    for Tt4 in (1500.0, 1200.0):
        a = deg._degenerate.equilibrium(FLIGHT, Tt4)
        b = ref.equilibrium(FLIGHT, Tt4)
        for k in ("nu", "pi_c", "tau_c", "tau_t", "mdot_air", "f", "Phi", "sp_thrust"):
            assert a[k] == b[k], (Tt4, k, a[k], b[k])


# --------------------------------------------------------------------------- gate 3
def test_independent_cpg_two_shaft_closure():
    """GATE 3 — an INDEPENDENT bare-math CPG two-shaft closure reproduces the solver.

    No Gas / Component / ComponentMap / TwoSpoolTransient calls inside the reference: closed
    form CPG thermodynamics, its own choke bisection, its own FORWARD speed lines, its own
    2-D equilibrium by damped Newton. Reproduces (nu_L, nu_H, pi_lpc, pi_hpc) AND — the
    load-bearing part — sigma_crit ON SHAPED MAPS (~1.2), which the ==1 identity could not
    anchor. Two code paths, one operating point (the rung-31/33/38/39 gate pattern).
    """
    gas = _cpg_gas()
    map_lp, map_hp = LP_SHAPED, HP_SHAPED
    t = _tt(gas, map_lp, map_hp)

    gc = (gas.gamma_c - 1.0) / gas.gamma_c
    gt = (gas.gamma_t - 1.0) / gas.gamma_t
    cp_c, cp_t, hPR = gas.cp_c, gas.cp_t, gas.hPR
    e_lpc0, e_hpc0 = REAL["eta_lpc"], REAL["eta_hpc"]
    eta_hpt, eta_lpt = REAL["eta_hpt"], REAL["eta_lpt"]
    eta_m, eta_b, pi_n = REAL["eta_m"], REAL["eta_b"], REAL["pi_n"]

    # Design point, closed form (same freestream as the solver: Tt2 == Tt2_d here).
    stag = 1.0 + 0.5 * (gas.gamma_c - 1.0) * FLIGHT.M0 ** 2
    Tt2 = FLIGHT.T0 * stag
    Tt25_d = Tt2 * (1.0 + (PI_LPC ** gc - 1.0) / e_lpc0)
    Tt3_d = Tt25_d * (1.0 + (PI_HPC ** gc - 1.0) / e_hpc0)
    f_d = (cp_t * TT4 - cp_c * Tt3_d) / (eta_b * hPR - cp_t * TT4)
    tau_lpc_d, tau_hpc_d = Tt25_d / Tt2, Tt3_d / Tt25_d
    Pref_lp, Pref_hp = cp_c * (Tt25_d - Tt2), cp_c * (Tt3_d - Tt25_d)

    def bisect(fn, lo, hi, tol=1e-15):
        flo = fn(lo)
        assert flo * fn(hi) < 0.0, "bare bracket fails"
        for _ in range(400):
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
        """pi_t/sqrt(tau_t) = area_ratio (MFP* is Tt-independent on CPG — rung 38 gate 2)."""
        def tau(p):
            return 1.0 - eta_t * (1.0 - p ** gt)
        pi_t = bisect(lambda p: p / tau(p) ** 0.5 - area_ratio, 0.02, 0.999)
        return pi_t, tau(pi_t)

    def psi(cm, phi):
        return 1.0 - cm.sigma * (phi - 1.0) ** 2 - cm.l * (phi - 1.0)

    def eta_at(cm, base, phi, n):
        return (base - cm.a * (phi - 1.0) ** 2 - cm.b * (n - 1.0) ** 2
                - cm.c * (phi - 1.0) * (n - 1.0))

    area_hp, area_lp = t.A4 / t.A45, t.A45 / (t.A8 * pi_n)
    _, tau_hpt = turbine(area_hp, eta_hpt)
    _, tau_lpt = turbine(area_lp, eta_lpt)

    def phis(nu_l, nu_h, Tt4):
        """The bare forward closure: one bisection in m_L, then both power residuals."""
        def ev(m_l):
            phi_l = m_l / nu_l                       # Tt2 == Tt2_d  =>  n_L == nu_L
            tau_lpc = 1.0 + (tau_lpc_d - 1.0) * psi(map_lp, phi_l) * nu_l * nu_l
            Tt25 = Tt2 * tau_lpc
            e_l = eta_at(map_lp, e_lpc0, phi_l, nu_l)
            pi_lpc = (1.0 + e_l * (tau_lpc - 1.0)) ** (1.0 / gc)
            # Corrected-flow transfer to the HP face.
            m_h = m_l * (PI_LPC / pi_lpc) * (Tt25 / Tt25_d) ** 0.5
            n_h = nu_h * (Tt25_d / Tt25) ** 0.5
            phi_h = m_h / n_h
            tau_hpc = 1.0 + (tau_hpc_d - 1.0) * psi(map_hp, phi_h) * n_h * n_h
            Tt3 = Tt25 * tau_hpc
            e_h = eta_at(map_hp, e_hpc0, phi_h, n_h)
            pi_hpc = (1.0 + e_h * (tau_hpc - 1.0)) ** (1.0 / gc)
            f = (cp_t * Tt4 - cp_c * Tt3) / (eta_b * hPR - cp_t * Tt4)
            # NGV choke, referred to design (MFP* cancels on CPG).
            m_imp = ((pi_lpc * pi_hpc / (PI_LPC * PI_HPC)) * (TT4 / Tt4) ** 0.5
                     * (1.0 + f_d) / (1.0 + f))
            return dict(m_imp=m_imp, Tt25=Tt25, Tt3=Tt3, pi_lpc=pi_lpc, pi_hpc=pi_hpc, f=f)

        m_l = bisect(lambda m: m - ev(m)["m_imp"], 0.05, 1.6)
        s = ev(m_l)
        Tt45, Tt5 = Tt4 * tau_hpt, Tt4 * tau_hpt * tau_lpt
        f = s["f"]
        Pt_hp = eta_m * (1.0 + f) * cp_t * (Tt4 - Tt45)
        Pt_lp = eta_m * (1.0 + f) * cp_t * (Tt45 - Tt5)
        Pc_hp = cp_c * (s["Tt3"] - s["Tt25"])
        Pc_lp = cp_c * (s["Tt25"] - Tt2)
        return (m_l * (Pt_lp - Pc_lp) / (Pref_lp * nu_l),
                m_l * (Pt_hp - Pc_hp) / (Pref_hp * nu_h), s)

    def equilibrium(Tt4):
        nl, nh = 1.0, 1.0
        for _ in range(80):
            fl, fh, s = phis(nl, nh, Tt4)
            if max(abs(fl), abs(fh)) < 1e-13:
                return nl, nh, s
            h = 1e-6
            al, ah, _ = phis(nl + h, nh, Tt4)
            bl, bh, _ = phis(nl, nh + h, Tt4)
            j11, j12, j21, j22 = (al-fl)/h, (bl-fl)/h, (ah-fh)/h, (bh-fh)/h
            det = j11 * j22 - j12 * j21
            dl, dh = (-fl*j22 + fh*j12)/det, (-j11*fh + j21*fl)/det
            damp = min(1.0, 0.25 / max(abs(dl), abs(dh), 1e-30))
            nl, nh = nl + damp * dl, nh + damp * dh
        raise AssertionError("bare 2-D equilibrium did not converge")

    for Tt4 in (1500.0, 1300.0, 1100.0):
        nl, nh, s = equilibrium(Tt4)
        od = t.match(FLIGHT, Tt4)
        assert abs(od.N_lp_ratio - nl) < 1e-8, (Tt4, od.N_lp_ratio, nl)
        assert abs(od.N_hp_ratio - nh) < 1e-8, (Tt4, od.N_hp_ratio, nh)
        assert abs(od.pi_lpc - s["pi_lpc"]) < 1e-8 * s["pi_lpc"], (Tt4, od.pi_lpc)
        assert abs(od.pi_hpc - s["pi_hpc"]) < 1e-8 * s["pi_hpc"], (Tt4, od.pi_hpc)

        # THE LOAD-BEARING PART: sigma_crit on SHAPED maps, from the bare closure.
        d = 5.0
        pl_p, ph_p, _ = phis(nl, nh, Tt4 + d)
        pl_m, ph_m, _ = phis(nl, nh, Tt4 - d)
        bare_sigma = ((pl_p - pl_m) / nl) / ((ph_p - ph_m) / nh)
        assert bare_sigma > 1.1, ("shaped sigma_crit must be materially off 1", bare_sigma)
        ship_sigma = t.lead_threshold(FLIGHT, Tt4, d=d)
        assert abs(ship_sigma - bare_sigma) < 1e-6 * bare_sigma, (
            Tt4, ship_sigma, bare_sigma)


# --------------------------------------------------------------------------- gate 4
def test_sigma_crit_identity_channels_and_direction():
    """GATE 4 — sigma_crit: the INHERITED identity, its two channels, and the REFUTATION.

    The == 1 identity is rung 39 B1 restated for the transient (on the running line
    sigma_crit reduces to the steady slip, which B1 pins at 1) — this rung's reduce SPINE,
    labelled inherited, not billed as discovery.
    """
    # (a) the identity — flat maps + CPG, every throttle.
    t = _tt(_cpg_gas(), FLAT, FLAT)
    for Tt4 in (900.0, 1100.0, 1300.0, 1500.0):
        assert abs(t.lead_threshold(FLIGHT, Tt4, d=25.0) - 1.0) < 1e-11, Tt4

    # (b) the two channels, measured identically — the rung-31-gate-5 mirror + rung 39 B2.
    flat_cpg = abs(t.lead_threshold(FLIGHT, 1100.0, d=25.0) - 1.0)
    gas_ch = abs(_tt(Gas.thermally_perfect(), FLAT, FLAT)
                 .lead_threshold(FLIGHT, 1100.0, d=25.0) - 1.0)
    map_ch = abs(_tt(_cpg_gas(), LP_SHAPED, HP_SHAPED)
                 .lead_threshold(FLIGHT, 1100.0, d=25.0) - 1.0)
    assert flat_cpg < 1e-11 < gas_ch, (flat_cpg, gas_ch)     # the mirror
    assert map_ch > gas_ch, ("the map channel is the larger", map_ch, gas_ch)

    # (c) THE REFUTATION (kept visible): the direction of the map's shift is SHAPE-dependent.
    lp_only = _tt(_cpg_gas(), LP_SHAPED, FLAT).lead_threshold(FLIGHT, 1100.0)
    hp_only = _tt(_cpg_gas(), FLAT, HP_SHAPED).lead_threshold(FLIGHT, 1100.0)
    assert lp_only < 1.0 < hp_only, (
        "'the map favours the LP spool' is FALSE — both signs are reachable",
        lp_only, hp_only)


# --------------------------------------------------------------------------- gate 5
@pytest.mark.parametrize("gas_name", ["cpg", "reacting"])
def test_finding_stability_is_rho_free(gas_name):
    """GATE 5 — FINDING (i): the clock ratio cannot destabilize the two-shaft pair.

    The MEASURED part is the sign structure a<0, d<0, a*d>b*c (no rho in it). The DERIVED
    part — asserted on top — is that those signs give tr<0 and det>0, hence both eigenvalues
    negative, at EVERY rho>0. Spot-checked over rho in [0.05, 100] (a 2000x range).
    """
    def gas():
        return _cpg_gas() if gas_name == "cpg" else Gas.reacting_equilibrium()

    for name, (lp, hp) in SHAPES.items():
        t = _tt(gas(), lp, hp)
        for Tt4 in (1500.0, 1200.0, 950.0):
            od = t.match(FLIGHT, Tt4)
            nu = (od.N_lp_ratio, od.N_hp_ratio)
            t.rho = 1.0
            J = t.jacobian(FLIGHT, Tt4, nu=nu)
            a, b, c, d = J[0][0], J[0][1], J[1][0], J[1][1]
            assert a < 0.0 and d < 0.0, (name, Tt4, a, d)
            assert a * d > b * c, (name, Tt4, a * d, b * c)
            for rho in (0.05, 0.2, 1.0, 5.0, 20.0, 100.0):
                Jr = [[a / rho, b / rho], [c, d]]
                assert max(t.eigenvalues(Jr)) < 0.0, (name, Tt4, rho)


# --------------------------------------------------------------------------- gate 6
def test_finding_complex_mode_is_created_by_the_lp_map():
    """GATE 6 — FINDING (ii): a COMPLEX inter-spool mode exists iff the LP map is SHAPED.

    `hp-only` is the DISCRIMINATOR — the HP map is shaped there and NO band appears, so the
    mechanism is the LP map specifically, not shaping in general.

    Gated: existence + the sign of b*c + the mechanism. DELIBERATELY NOT gated: the band's
    LOCATION and |Im/Re| (<=0.25 in these maps) — both ride on the representative shapes and
    are disclaimed, exactly as rung 39 disclaims its slip depth.
    """
    for name, (lp, hp) in SHAPES.items():
        t = _tt(_cpg_gas(), lp, hp)
        for Tt4 in (1500.0, 1200.0):
            od = t.match(FLIGHT, Tt4)
            nu = (od.N_lp_ratio, od.N_hp_ratio)
            t.rho = 1.0
            J = t.jacobian(FLIGHT, Tt4, nu=nu)
            bc = J[0][1] * J[1][0]
            band = t.oscillatory_band(FLIGHT, Tt4, nu=nu)
            if name in LP_IS_FLAT:
                assert bc >= 0.0, ("flat LP map must keep b*c >= 0", name, Tt4, bc)
                assert band is None, ("no complex band with a flat LP map", name, Tt4, band)
                assert t.damping_ratio_max(FLIGHT, Tt4, nu=nu) == 0.0
            else:
                assert bc < 0.0, ("a shaped LP map must flip b*c negative", name, Tt4, bc)
                assert band is not None, (name, Tt4)
                lo, hi = band
                assert 0.0 < lo < hi, (name, Tt4, band)
                # The returned band really brackets the discriminant's sign change.
                a, b, c, d = J[0][0], J[0][1], J[1][0], J[1][1]

                def disc(rho):
                    return (a / rho - d) ** 2 + 4.0 * b * c / rho

                mid = (lo * hi) ** 0.5
                assert disc(mid) < 0.0, ("complex inside the band", name, Tt4, disc(mid))
                assert disc(0.5 * lo) > 0.0 and disc(2.0 * hi) > 0.0, (name, Tt4)
                assert t.damping_ratio_max(FLIGHT, Tt4, nu=nu) > 0.0


# --------------------------------------------------------------------------- gate 7
@pytest.mark.slow
def test_scope_sigma_crit_is_first_instant_only():
    """GATE 7 — SCOPE: sigma_crit does NOT govern the finite-amplitude ramp.

    Asserted as a DELIBERATE NON-convergence so the withdrawn claim cannot silently creep
    back into the rung. The marched threshold rho* (bisected on the sign of the running-line-
    referenced slip excursion) sits far from sigma_crit, because that excursion is dominated
    by the steady slip SCHEDULE moving with Tt4 while the speeds lag — schedule-slaved, not
    lead-governed.
    """
    t = _tt(_cpg_gas(), LP_SHAPED, HP_SHAPED)
    Tt4_lo, dT = 1100.0, 50.0
    sc = t.lead_threshold(FLIGHT, Tt4_lo)

    def exc(rho):
        t.rho = rho
        return t.slip_excursion(FLIGHT, Tt4_lo, dT, s_end=1.2, ds=0.05)

    lo, hi = 0.6 * sc, 1.6 * sc
    elo, ehi = exc(lo), exc(hi)
    assert elo * ehi < 0.0, ("a threshold exists in the bracket", elo, ehi)
    for _ in range(18):
        mid = 0.5 * (lo + hi)
        if exc(mid) * elo > 0.0:
            lo = mid
        else:
            hi = mid
    rho_star = 0.5 * (lo + hi)
    assert abs(rho_star / sc - 1.0) > 0.2, (
        "sigma_crit must NOT be billed as the marched threshold — this gate exists to keep "
        "the withdrawn claim withdrawn", rho_star, sc)


# --------------------------------------------------------------------------- gate 8
def test_cycle_untouched_rung6():
    """GATE 8 — the default single-spool design path is untouched by rung 40."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, pi_d=REAL["pi_d"], eta_c=0.88,
                         eta_b=REAL["eta_b"], pi_b=REAL["pi_b"], eta_t=REAL["eta_hpt"],
                         eta_m=REAL["eta_m"], pi_n=REAL["pi_n"])
    a = eng.run(FLIGHT, 1.0)
    # Building a rung-40 object must not perturb the design cycle in any way.
    _tt(_cpg_gas(), LP_SHAPED, HP_SHAPED).match(FLIGHT, 1200.0)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].Tt == b.stations["4"].Tt
    assert a.stations["9"].pt == b.stations["9"].pt


if __name__ == "__main__":
    for fn in (test_reduce_2d_equilibrium_is_rung39,
               test_reduce_lp_disabled_is_rung34_bit_for_bit,
               test_independent_cpg_two_shaft_closure,
               test_sigma_crit_identity_channels_and_direction,
               test_finding_complex_mode_is_created_by_the_lp_map,
               test_scope_sigma_crit_is_first_instant_only,
               test_cycle_untouched_rung6):
        fn()
        print(f"OK  {fn.__name__}")
    for g in ("cpg", "reacting"):
        test_finding_stability_is_rho_free(g)
        print(f"OK  test_finding_stability_is_rho_free[{g}]")
