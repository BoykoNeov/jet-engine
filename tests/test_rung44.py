"""Rung 44 — THE TRANSIENT TWO-SPOOL SURGE LINE: the excursion is SCHEDULE-slaved, LP eats it.

Rungs 40 and 41 both closed by naming the SAME open seam: the transient surge line, measuring
rung 40's marched trajectory against rung 41's imposed stability boundary (rung 41 is steady-only,
so no surge-SURVIVAL claim yet). Rung 44 marches the running point against the surge line.

THE FINDING (a clean split, sign-space only — phi_surge stays imposed, no survival claim):
  (1) the two-shaft acceleration drives BOTH spools TOWARD surge, and the LP eats ~1.6-2.2x the
      HP's excursion — rung 41's steady exposure split SURVIVES dynamically;
  (2) the excursion is SCHEDULE-slaved: rho-INVARIANT (<2% over 25x rho), RAMP-RATE-driven (~5x),
      and MODE-INDEPENDENT (the mode-free hp-only pair has the LARGEST LP/HP ratio; |Im/Re|<=0.164)
      — rung 40's two dynamically interesting objects (rho, the complex mode) are BOTH
      surge-irrelevant; what governs is how fast you slam the throttle;
  (3) the transient CAN cross a line the steady running line clears — report the crossing, gate
      the flip (rung 36 discipline).

Gates (named in docs/rung44-spec.md § Verification gates):
   1. REDUCE — the rung-44 methods are READ-ONLY: arming phi_surge leaves rung 40's
      integrate/equilibrium/jacobian bit-for-bit (==); default design run bit-for-bit rung 6.
   2. NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-shaft accel closure reproduces the
      excursion SIGN, the LP-over-HP ordering, and the schedule-slaving on a shaped map.
   3. THE SPLIT SURVIVES DYNAMICALLY — ext<0 both spools on accel (toward surge), >0 on decel;
      |ext_lp| > 1.4|ext_hp| every shape pair incl. mode-free hp-only.
   4. SCHEDULE-SLAVED — rho-invariant, ramp-rate-monotone, mode-independent (hp-only band None,
      largest ratio, |Im/Re|<0.25).
   5. REPORT THE CROSSING, GATE THE FLIP — transient min LP margin < steady min LP margin on
      accel (> on decel); a floor in the gap => crossed_lp True while steady clears.
   6. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
STEEP = ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)
FLAT = ComponentMap.flat()

SHAPES = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (TILTED, TILTED),
    "steep":      (STEEP, STEEP),
    "hp-only":    (FLAT, HP_SHAPED),   # rung 40's DISCRIMINATOR: LP flat => NO complex mode
}


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _tt(gas, ml=None, mh=None, rho=1.0):
    return TwoSpoolTransient(_design(gas), FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=rho)


def _floor(cm, phi_surge):
    return cm.with_phi_surge(phi_surge)


# ======================================================================================
# GATE 1 — REDUCE: the rung-44 methods are READ-ONLY (bit-for-bit rung 40)
# ======================================================================================

def test_reduce_rung44_methods_are_read_only_bit_for_bit():
    """Arming phi_surge (needed only by transient_surge_margin) must leave rung 40's
    integrate / equilibrium / jacobian bit-for-bit identical — rung 44 adds no state."""
    gas = _cpg_gas()
    d = _design(gas)
    for ml, mh in (SHAPES["flow/press"], SHAPES["tilted"]):
        bare = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.5)
        armed = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=_floor(ml, 0.60),
                                  map_hp=_floor(mh, 0.55), rho=1.5)

        def sched(s):
            return 1200.0 + 150.0 * min(1.0, s / 0.5)

        pa = bare.integrate(FLIGHT, sched, (0.95, 0.97), 2.0, 0.05)
        pb = armed.integrate(FLIGHT, sched, (0.95, 0.97), 2.0, 0.05)
        assert len(pa) == len(pb)
        for a, b in zip(pa, pb):
            assert (a.nu_lp, a.nu_hp, a.phi_lp, a.phi_hp, a.pi_lpc, a.slip) == \
                   (b.nu_lp, b.nu_hp, b.phi_lp, b.phi_hp, b.pi_lpc, b.slip)
        for Tt4 in (1500.0, 1100.0):
            assert bare.equilibrium(FLIGHT, Tt4) == armed.equilibrium(FLIGHT, Tt4)
            assert bare.jacobian(FLIGHT, Tt4) == armed.jacobian(FLIGHT, Tt4)
        # the running-line excursion itself is identical (the trajectory never reads phi_surge)
        assert bare.phi_excursion(FLIGHT, 1000.0, 300.0) == \
               armed.phi_excursion(FLIGHT, 1000.0, 300.0)


def test_default_design_run_bit_for_bit_rung6():
    """GATE 1/6 — the default single-spool design path is untouched (bit-for-bit rung 6)."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    # Constructing and exercising the rung-44 diagnostics must not perturb it.
    tt = _tt(_cpg_gas(), _floor(LP_SHAPED, 0.60), _floor(HP_SHAPED, 0.55))
    tt.phi_excursion(FLIGHT, 1000.0, 300.0)
    tt.transient_surge_margin(FLIGHT, 1000.0, 300.0)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# ======================================================================================
# GATE 3 — THE SPLIT SURVIVES DYNAMICALLY (accel toward surge, LP eats more)
# ======================================================================================

def test_split_survives_dynamically():
    """Both spools swing TOWARD surge on an accel (ext<0), AWAY on a decel (ext>0), and the
    LP eats > 1.4x the HP's excursion at EVERY shape pair — including the mode-free hp-only.
    Sign + ordering only; magnitudes disclaimed."""
    gas = _cpg_gas()
    d = _design(gas)
    for name, (ml, mh) in SHAPES.items():
        tt = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        acc = tt.phi_excursion(FLIGHT, 1000.0, 400.0)
        dec = tt.phi_excursion(FLIGHT, 1400.0, -400.0)
        assert acc["ext_lp"] < 0.0 and acc["ext_hp"] < 0.0, (name, "accel toward surge")
        assert dec["ext_lp"] > 0.0 and dec["ext_hp"] > 0.0, (name, "decel away from surge")
        assert abs(acc["ext_lp"]) > 1.4 * abs(acc["ext_hp"]), (name, "LP eats more", acc)


# ======================================================================================
# GATE 4 — SCHEDULE-SLAVED: not rho, not the complex mode
# ======================================================================================

def test_excursion_is_rho_invariant():
    """The accel excursion moves < 5% over rho in [0.2, 5.0] (a 25x range) — the inter-spool
    clock ratio is POWERLESS over the surge excursion (rung 40's scope-limit, surge axis)."""
    gas = _cpg_gas()
    d = _design(gas)
    vals = []
    for rho in (0.2, 0.5, 1.0, 2.0, 5.0):
        tt = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=rho)
        vals.append(tt.phi_excursion(FLIGHT, 1000.0, 400.0)["ext_lp"])
    spread = (max(vals) - min(vals)) / abs(sum(vals) / len(vals))
    assert spread < 0.05, ("rho-invariant", vals, spread)


def test_excursion_is_ramp_rate_driven():
    """|ext_lp| increases monotonically as the ramp gets FASTER (r_ramp falls) — the schedule
    against the shaft clock is the governing variable, not rho, not the mode."""
    tt = _tt(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0)
    prev = None
    for r in (5.0, 2.0, 1.0, 0.5, 0.3, 0.1):
        e = abs(tt.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=r, s_end=6.0)["ext_lp"])
        if prev is not None:
            assert e > prev, ("faster ramp => deeper excursion", r, e, prev)
        prev = e


def test_complex_mode_is_surge_irrelevant():
    """The AIRTIGHT leg: every |Im/Re|max < 0.25 => the ring e-folds before a quarter cycle
    and cannot cross a line the steady point clears (the mode is irrelevant by its own
    strength). CORROBORATION (not necessary): hp-only (LP flat) has NO complex band yet the
    LARGEST LP/HP ratio, so the asymmetry is present with no mode => the mode is not its cause
    (the 2.23-vs-1.90 value also swaps LP shaped->flat, a partial artifact, named -- not a
    proof; the damping ratio carries the claim)."""
    gas = _cpg_gas()
    d = _design(gas)
    ratios = {}
    for name, (ml, mh) in SHAPES.items():
        tt = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        acc = tt.phi_excursion(FLIGHT, 1000.0, 400.0)
        ratios[name] = abs(acc["ext_lp"]) / abs(acc["ext_hp"])
        assert tt.damping_ratio_max(FLIGHT, 1200.0) < 0.25, (name, "ring e-folds fast")
        band = tt.oscillatory_band(FLIGHT, 1200.0)
        if name == "hp-only":
            assert band is None, "hp-only (LP flat) must carry NO complex mode"
        else:
            assert band is not None, (name, "shaped-LP pair must carry a complex mode")
    # the mode-free discriminator carries the LARGEST ratio => the mode is not the cause
    assert ratios["hp-only"] == max(ratios.values()), ("mode-free pair eats the most", ratios)


# ======================================================================================
# GATE 5 — REPORT THE CROSSING, GATE THE FLIP
# ======================================================================================

def test_report_the_crossing_gate_the_flip():
    """transient_surge_margin ALLOWS phi < phi_surge and records it. On an accel the transient
    min LP margin sits BELOW the steady min LP margin at the same Tt4 (the flip); with a floor
    placed in the gap the LP crosses while the steady point clears. Decel is the mirror.
    The flip's SIGN is gated; the crossing DEPTH is disclaimed."""
    gas = _cpg_gas()
    d = _design(gas)
    ml, mh = _floor(LP_SHAPED, 0.76), _floor(HP_SHAPED, 0.55)
    tt = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)

    # THE FLIP (sign): transient min LP margin strictly below steady min LP margin on accel.
    acc = tt.transient_surge_margin(FLIGHT, 1000.0, 400.0, r_ramp=0.3)
    assert acc["margin_min_lp"] < acc["steady_min_lp"], ("accel flip (LP toward surge)", acc)
    dec = tt.transient_surge_margin(FLIGHT, 1400.0, -400.0, r_ramp=0.3)
    assert dec["margin_min_lp"] > dec["steady_min_lp"], ("decel flip (LP away)", dec)

    # THE CROSSING (reported, not asserted as a magnitude): the floor 0.76 sits in the gap so
    # the LP transient crosses while every steady point clears it, and it lands on the LP spool.
    assert acc["steady_min_lp"] > 0.0, ("steady CLEARS the floor", acc)
    assert acc["crossed_lp"] is True and acc["crossed_hp"] is False, (
        "the transient crossing lands on the LP spool", acc)

    # unarmed maps => the method asserts (the surge line is genuinely off when absent)
    bare = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    with pytest.raises(AssertionError):
        bare.transient_surge_margin(FLIGHT, 1000.0, 400.0)


# ======================================================================================
# GATE 2 — NON-TAUTOLOGICAL: an INDEPENDENT bare-math CPG two-shaft accel closure
# ======================================================================================

def test_bare_math_accel_excursion_sign_ordering_and_slaving():
    """An INDEPENDENT bare-math CPG two-shaft closure (no Gas / Component / ComponentMap /
    TwoSpoolTransient inside the reference — closed-form CPG thermodynamics, own choke
    bisection, own FORWARD speed lines, own 2-D running-line root, own EULER shaft march)
    reproduces the shipped excursion's SIGN (negative, both spools), the LP-over-HP ordering,
    and the schedule-slaving (rho-invariance + ramp-rate monotonicity) on a shaped map.
    Different integrators (RK4 vs Euler), so SIGN + ordering + qualitative slaving are the
    gated ties, not a bit-for-bit value (the rung-40 gate-3 pattern, on a marched object)."""
    gas = _cpg_gas()
    map_lp, map_hp = LP_SHAPED, HP_SHAPED
    t = _tt(gas, map_lp, map_hp)

    gc = (gas.gamma_c - 1.0) / gas.gamma_c
    gt = (gas.gamma_t - 1.0) / gas.gamma_t
    cp_c, cp_t, hPR = gas.cp_c, gas.cp_t, gas.hPR
    e_lpc0, e_hpc0 = REAL["eta_lpc"], REAL["eta_hpc"]
    eta_hpt, eta_lpt = REAL["eta_hpt"], REAL["eta_lpt"]
    eta_m, eta_b, pi_n = REAL["eta_m"], REAL["eta_b"], REAL["pi_n"]

    stag = 1.0 + 0.5 * (gas.gamma_c - 1.0) * FLIGHT.M0 ** 2
    Tt2 = FLIGHT.T0 * stag
    Tt25_d = Tt2 * (1.0 + (PI_LPC ** gc - 1.0) / e_lpc0)
    Tt3_d = Tt25_d * (1.0 + (PI_HPC ** gc - 1.0) / e_hpc0)
    f_d = (cp_t * TT4 - cp_c * Tt3_d) / (eta_b * hPR - cp_t * TT4)
    tau_lpc_d, tau_hpc_d = Tt25_d / Tt2, Tt3_d / Tt25_d
    Pref_lp, Pref_hp = cp_c * (Tt25_d - Tt2), cp_c * (Tt3_d - Tt2 * tau_lpc_d)

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
        def ev(m_l):
            phi_l = m_l / nu_l                        # Tt2 == Tt2_d => n_L == nu_L
            tau_lpc = 1.0 + (tau_lpc_d - 1.0) * psi(map_lp, phi_l) * nu_l * nu_l
            Tt25 = Tt2 * tau_lpc
            e_l = eta_at(map_lp, e_lpc0, phi_l, nu_l)
            pi_lpc = (1.0 + e_l * (tau_lpc - 1.0)) ** (1.0 / gc)
            m_h = m_l * (PI_LPC / pi_lpc) * (Tt25 / Tt25_d) ** 0.5
            n_h = nu_h * (Tt25_d / Tt25) ** 0.5
            phi_h = m_h / n_h
            tau_hpc = 1.0 + (tau_hpc_d - 1.0) * psi(map_hp, phi_h) * n_h * n_h
            Tt3 = Tt25 * tau_hpc
            e_h = eta_at(map_hp, e_hpc0, phi_h, n_h)
            pi_hpc = (1.0 + e_h * (tau_hpc - 1.0)) ** (1.0 / gc)
            f = (cp_t * Tt4 - cp_c * Tt3) / (eta_b * hPR - cp_t * Tt4)
            m_imp = ((pi_lpc * pi_hpc / (PI_LPC * PI_HPC)) * (TT4 / Tt4) ** 0.5
                     * (1.0 + f_d) / (1.0 + f))
            return dict(m_imp=m_imp, Tt25=Tt25, Tt3=Tt3, phi_l=phi_l, phi_h=phi_h, f=f)

        m_l = bisect(lambda m: m - ev(m)["m_imp"], 0.05, 1.6)
        s = ev(m_l)
        Tt45, Tt5 = Tt4 * tau_hpt, Tt4 * tau_hpt * tau_lpt
        f = s["f"]
        Pt_hp = eta_m * (1.0 + f) * cp_t * (Tt4 - Tt45)
        Pt_lp = eta_m * (1.0 + f) * cp_t * (Tt45 - Tt5)
        Pc_hp = cp_c * (s["Tt3"] - s["Tt25"])
        Pc_lp = cp_c * (s["Tt25"] - Tt2)
        Phi_l = m_l * (Pt_lp - Pc_lp) / (Pref_lp * nu_l)
        Phi_h = m_l * (Pt_hp - Pc_hp) / (Pref_hp * nu_h)
        return Phi_l, Phi_h, s

    def equilibrium(Tt4):
        nl, nh = 1.0, 1.0
        for _ in range(120):
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

    def steady_phi(Tt4, _cache={}):
        if Tt4 not in _cache:
            _, _, s = equilibrium(Tt4)
            _cache[Tt4] = (s["phi_l"], s["phi_h"])
        return _cache[Tt4]

    def excursion(Tt4_lo, dTt4, r_ramp, rho, s_end=6.0, ds=0.01):
        """Bare EULER march of (dnu_L/ds, dnu_H/ds) = (Phi_L/rho, Phi_H) on the ramp."""
        nl, nh, _ = equilibrium(Tt4_lo)
        ext_l = ext_h = 0.0
        s = 0.0
        n = int(round(s_end / ds))
        for _ in range(n + 1):
            Tt4 = Tt4_lo + dTt4 * min(1.0, s / r_ramp)
            Phi_l, Phi_h, sdict = phis(nl, nh, Tt4)
            pl, ph = steady_phi(round(Tt4, 3))
            e_l, e_h = sdict["phi_l"] - pl, sdict["phi_h"] - ph
            if abs(e_l) > abs(ext_l):
                ext_l = e_l
            if abs(e_h) > abs(ext_h):
                ext_h = e_h
            nl = max(0.2, nl + ds * Phi_l / rho)
            nh = max(0.2, nh + ds * Phi_h)
            s += ds
        return ext_l, ext_h

    # SIGN + ordering (accel toward surge, LP eats more).
    el, eh = excursion(1000.0, 400.0, 0.5, 1.0)
    assert el < 0.0 and eh < 0.0, ("bare accel toward surge", el, eh)
    assert abs(el) > 1.4 * abs(eh), ("bare LP eats more", el, eh)
    # decel mirror.
    dl, dh = excursion(1400.0, -400.0, 0.5, 1.0)
    assert dl > 0.0 and dh > 0.0, ("bare decel away from surge", dl, dh)

    # SCHEDULE-SLAVING: rho-invariance.
    rho_vals = [excursion(1000.0, 400.0, 0.5, r)[0] for r in (0.2, 1.0, 5.0)]
    spread = (max(rho_vals) - min(rho_vals)) / abs(sum(rho_vals) / len(rho_vals))
    assert spread < 0.05, ("bare rho-invariance", rho_vals, spread)
    # SCHEDULE-SLAVING: ramp-rate monotonicity.
    fast = abs(excursion(1000.0, 400.0, 0.2, 1.0)[0])
    slow = abs(excursion(1000.0, 400.0, 2.0, 1.0)[0])
    assert fast > slow, ("bare faster ramp => deeper", fast, slow)

    # loose tie to the shipped object (different integrators): same sign, within 25%.
    ship = t.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=0.5)["ext_lp"]
    assert abs(el - ship) < 0.25 * abs(ship), ("bare ~ shipped", el, ship)


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
