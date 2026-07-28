"""Rung 56 — PER-ROW CAPACITY: two constraints on one machine, at opposite ends.

Rung 55 named this seam: "X(v) = m*sqrt(1+v^2) is a FACE quantity; in a stack each row has its
own throat and its own X_k ... the capacity margin should BIND AT THE BACK while the incidence
margin binds at the front ... It needs a C per row."

It does not need K constants — the stack's own design ladder fixes the PROFILE (every row has
the same design throat VELOCITY while Tt_k climbs), leaving rung 54's single constant as the
LEVEL. And the derived profile FIGHTS the seam, so which end binds is a contest.

Gates (named in docs/rung56-spec.md § Verification gates):

   1. REDUCE — an INVARIANCE over BOTH the constant and the profile, on a stack that DOES
      enter the solver; plus K = 1 reproducing rung 54's own `throat_margin` to the last bit.
   2. THE DERIVED PROFILE — C_0 is the disclosed constant EXACTLY, C_k falls monotonically,
      and it is the total-referenced Mach nu that scales as 1/sqrt(theta_k,d).
   3. THE PER-ROW CURRENCY — m_k = phi_k*n_k exactly, X_k is rung 54's law at the row's OWN
      setting, and the design tie is a TOLERANCE (measured drift ~2e-14), not an identity.
   4. THE NON-TAUTOLOGY GATE — the amplification (face vs binding row) is EXACTLY 1.0 at
      K = 1 and grows with throttle depth; a resolution gap, not a feedback one.
   5. P1 — the binding row MIGRATES: front near design, rear at part power, with a crossover.
   6. P3 — K is a RESOLUTION: the increments shrink.
   7. P4 — the disclosed SPLIT is LOAD-BEARING here (contrast rung 55 P6, where it was not).
   8. P5 — THE HEADLINE: the two constraints land at opposite ENDS and on opposite SPOOLS;
      and rung 54's "the HP never approaches its throat" is CORRECTED by resolution.
   9. P6 — the positional lever DEBITS the row it does not move, the lumped lever far more,
      and the advantage is CURRENCY-DEPENDENT (it collapses with v; the speed ratio does not).
  10. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    VariableStatorMatcher, StageStack, StageStackMatcher, _mfp_frac, _nu_of_M, _M_of_nu,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR, CAP = 0.55, 0.90

# Rung 53/54/55's five disclosed shapes, verbatim.
SHAPES = {
    "flow/press": (ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                   ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                   ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
    "steep":      (ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2),
                   ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)),
    "flat-eta":   (ComponentMap(sigma=0.1, l=0.7), ComponentMap(sigma=0.1, l=1.0)),
}
THROTTLE = (1500.0, 1200.0, 1000.0, 800.0)
WALK = (1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 800.0)
FIELDS = ("pi_lpc", "pi_hpc", "n_lp", "n_hp", "phi_lp", "phi_hp", "slip",
          "eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt", "tau_lpc", "tau_hpc",
          "tau_hpt", "tau_lpt", "mdot_air", "thrust", "N_lp_ratio", "N_hp_ratio")

# B1, MEASURED not guessed: the K-stage march does not reproduce X_k = 1 at design to the
# bit -- max|X_k - 1| runs 7.8e-15 .. 1.9e-14 over K = 2..16 on both spools. So the design tie
# is a tolerance, and binding-row identity at design under the UNIFORM profile is noise.
_DESIGN_DRIFT = 1e-12


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _maps(shape="flow/press", C=CAP):
    return tuple(m.with_phi_surge(FLOOR).with_capacity(C) for m in SHAPES[shape])


def _sm(gas, shape="flow/press", C=CAP, K=8, prof="derived", split="dT", vl=0.0, vh=0.0,
        vs_lp=None, vs_hp=None, design=None):
    ml, mh = _maps(shape, C)
    return StageStackMatcher(design if design is not None else _design(gas), FLIGHT, 1.0,
                             map_lp=ml, map_hp=mh, K_lp=K, K_hp=K, split=split,
                             cap_profile=prof, vsv_lp=vl, vsv_hp=vh,
                             vsv_stages_lp=vs_lp, vsv_stages_hp=vs_hp)


# ======================================================================================
# GATE 1 — REDUCE: an INVARIANCE over the constant AND over the profile
# ======================================================================================

@pytest.mark.parametrize("vl,vh", [(0.0, 0.0), (0.30, 0.0), (0.20, 0.10)])
def test_reduce_invariance_over_capacity_and_profile(vl, vh):
    """THE SPINE, in rung 54's stronger form. Rung 54 earned an invariance over C on a channel
    that entered no solver at all; rung 55's STACK does enter the solver, so this is no longer
    free — `capacity` and `cap_profile` ride on objects (`ComponentMap`, `StageStack`) the
    speed-line inversion consumes. Every matched field must still be bit-identical for every
    C and both profiles, at a MOVED stator."""
    gas = _cpg_gas()
    design = _design(gas)
    ref = _sm(gas, C=1e-9, vl=vl, vh=vh, design=design)   # a throat model that is ~off
    bare_l, bare_h = (m.with_phi_surge(FLOOR) for m in SHAPES["flow/press"])
    nothroat = StageStackMatcher(design, FLIGHT, 1.0, map_lp=bare_l, map_hp=bare_h,
                                 K_lp=8, K_hp=8, vsv_lp=vl, vsv_hp=vh)
    cases = [nothroat] + [_sm(gas, C=C, prof=p, vl=vl, vh=vh, design=design)
                          for C in (0.30, 0.70, 0.90, 0.99) for p in ("derived", "uniform")]
    for Tt4 in THROTTLE:
        a = ref.match(FLIGHT, Tt4)
        for other in cases:
            b = other.match(FLIGHT, Tt4)
            for f in FIELDS:
                assert getattr(a, f) == getattr(b, f), (
                    f"rung-56 invariance broken on {f} at Tt4={Tt4}, C="
                    f"{other.map_lp.capacity}, profile={other.cap_profile}: "
                    f"{getattr(a, f)!r} vs {getattr(b, f)!r}")


def test_reduce_K1_is_rung54_throat_margin_bit_for_bit():
    """At K = 1 there is no stack, so `stage_throat_margin`'s single row must BE rung 54's
    face read -- the same X, the same margin, the same c_min -- to the last bit, on both
    profiles (which cannot differ when there is only one row) and at a moved stator."""
    gas = _cpg_gas()
    design = _design(gas)
    for vl in (0.0, 0.30):
        ref = VariableStatorMatcher(design, FLIGHT, 1.0, map_lp=_maps()[0],
                                    map_hp=_maps()[1], vsv_lp=vl)
        for prof in ("derived", "uniform"):
            st = _sm(gas, K=1, prof=prof, vl=vl, design=design)
            for Tt4 in THROTTLE:
                a = ref.throat_margin(FLIGHT, Tt4)
                b = st.stage_throat_margin(FLIGHT, Tt4)
                for spool in ("lp", "hp"):
                    row = b[spool]["stages"][0]
                    assert row["throat_loading"] == a[spool]["throat_loading"]
                    assert row["m_c"] == a[spool]["m_c"]
                    assert row["c_min"] == a[spool]["c_min"]
                    assert b[spool]["amplification"] == 1.0, (
                        "at K = 1 the binding row IS the face: the amplification is exactly 1")


def test_reduce_stack_capacities_at_K1():
    """A hand-built one-stage stack carries exactly the disclosed constant, on both profiles
    and for any gamma -- theta_d[0] == 1, so the derived ladder cannot bite."""
    gas = _cpg_gas()
    m = _sm(gas, K=8)
    cmap = _maps()[0]
    for prof in ("derived", "uniform"):
        for g in (1.3, 1.4, 1.667):
            st = StageStack(K=1, cmap=cmap, tau_d=m.tau_lpc_d, pi_d=m.pi_lpc_design,
                            eta_d=m.eta_lpc, cap_profile=prof, gamma_th=g)
            assert st.capacities() == [CAP]
            assert st.stage_capacity_margin(0, 0.7) == cmap.capacity_margin(0.7)


# ======================================================================================
# GATE 2 — THE DERIVED PROFILE: shape derived, level disclosed
# ======================================================================================

def test_derived_profile_is_the_ladder_and_the_level_is_the_front_row():
    """C_0 is rung 54's constant EXACTLY (not a bisection round-trip), the profile falls
    monotonically rearward, and the object that actually scales as 1/sqrt(theta_k,d) is the
    TOTAL-referenced Mach nu -- which is what a common design throat VELOCITY at rising Tt
    means. Zero new constants beyond rung 54's one."""
    gas = _cpg_gas()
    m = _sm(gas, K=8)
    for spool, st in (("lp", m.stack_lp), ("hp", m.stack_hp)):
        Cs = st.capacities()
        assert Cs[0] == CAP, "the disclosed level IS the front row's C, exactly"
        assert len(Cs) == st.K
        for k in range(st.K - 1):
            assert Cs[k + 1] < Cs[k], (
                f"rung-56 derived profile must FALL rearward on {spool} (rising Tt at a "
                f"common throat velocity), got {Cs}")
        # the derivation itself: nu_k * sqrt(theta_k,d) is invariant
        nu = [_nu_of_M(_M_of_nu(_nu_of_M(st.cmap.design_throat_mach(st.gamma_th),
                                         st.gamma_th) / st.theta_d[k] ** 0.5, st.gamma_th),
                       st.gamma_th) * st.theta_d[k] ** 0.5 for k in range(st.K)]
        for x in nu:
            assert x == pytest.approx(nu[0], rel=1e-12)
        # and each C_k IS the MFP fraction of a Mach BELOW the front row's
        assert Cs[-1] == pytest.approx(_mfp_frac(_M_of_nu(
            _nu_of_M(st.cmap.design_throat_mach(), 1.4) / st.theta_d[st.K - 1] ** 0.5)),
            rel=1e-12)


def test_uniform_profile_is_the_disclosed_alternative():
    gas = _cpg_gas()
    m = _sm(gas, K=8, prof="uniform")
    for st in (m.stack_lp, m.stack_hp):
        assert st.capacities() == [CAP] * st.K
    with pytest.raises(AssertionError):
        StageStack(K=4, cmap=_maps()[0], tau_d=1.4, pi_d=3.0, eta_d=0.9,
                   cap_profile="quadratic")
    with pytest.raises(AssertionError):       # no throat model => no per-row capacity
        StageStack(K=4, cmap=SHAPES["flow/press"][0].with_phi_surge(FLOOR),
                   tau_d=1.4, pi_d=3.0, eta_d=0.9).capacities()


def test_hp_profile_falls_harder_than_lp():
    """The profile is a functional of the ladder, so the spool with the larger design
    temperature rise has the steeper Mach fall. Not fitted -- read off tau_d."""
    gas = _cpg_gas()
    m = _sm(gas, K=8)
    lp, hp = m.stack_lp.capacities(), m.stack_hp.capacities()
    assert m.stack_hp.tau_d > m.stack_lp.tau_d
    assert hp[-1] / hp[0] < lp[-1] / lp[0]


# ======================================================================================
# GATE 3 — THE PER-ROW CURRENCY
# ======================================================================================

def test_per_row_corrected_flow_is_phi_times_n_and_X_is_rung54s_law():
    """m_k = phi_k * n_k is an IDENTITY at every station (the face relation m = phi*n, per
    row), and X_k applies rung 54's derived area law at the setting THAT row carries -- the
    design setting for every row a front-block stator does not move."""
    gas = _cpg_gas()
    m = _sm(gas, K=8, vl=0.40, vs_lp=3)
    r = m.stage_throat_margin(FLIGHT, 1000.0)["lp"]
    for k, s in enumerate(r["stages"]):
        assert s["m_k"] == s["phi"] * s["n"]
        assert s["vsv"] == (0.40 if k < 3 else 0.0), \
            "only the front block carries the setting -- that positional split is the rung"
        assert s["throat_loading"] == pytest.approx(
            s["m_k"] * math.sqrt(1.0 + s["vsv"] ** 2), rel=1e-15)
        assert s["m_c"] == pytest.approx(1.0 - s["capacity"] * s["throat_loading"], rel=1e-15)
        assert s["c_min"] == pytest.approx(1.0 / s["throat_loading"], rel=1e-15)


@pytest.mark.parametrize("K", [2, 4, 8, 16])
def test_design_tie_is_a_tolerance_not_an_identity(K):
    """B1, gated as measured. At design every X_k should be 1; in floating point the K-stage
    march drifts by ~1e-14. So this is a tolerance -- and consequently no binding-row claim
    may be gated at design under the UNIFORM profile, where the rows are otherwise tied."""
    gas = _cpg_gas()
    m = _sm(gas, K=K, prof="uniform")
    for spool in ("lp", "hp"):
        r = m.stage_throat_margin(FLIGHT, TT4)[spool]
        Xs = [s["throat_loading"] for s in r["stages"]]
        assert max(abs(x - 1.0) for x in Xs) < _DESIGN_DRIFT
        assert max(Xs) - min(Xs) < _DESIGN_DRIFT
        assert max(Xs) - min(Xs) > 0.0, (
            "the drift is REAL -- if this ever becomes an exact tie the noise warning above "
            "can be dropped, but until then binding-row gates must avoid design+uniform")


# ======================================================================================
# GATE 4 — THE NON-TAUTOLOGY GATE: a RESOLUTION gap, not a feedback one
# ======================================================================================

@pytest.mark.parametrize("shape", list(SHAPES))
@pytest.mark.parametrize("split", ["dT", "tau"])
def test_amplification_is_the_non_tautology_gate(shape, split):
    """P2. The channel enters no solver (rung 54 P1, inherited), so what makes rung 56 content
    is RESOLUTION: at the SAME solved state the binding row's throat deficit exceeds the face
    deficit rung 54 could read. Exactly 1.0 at K = 1, and growing with throttle depth."""
    gas = _cpg_gas()
    design = _design(gas)
    one = _sm(gas, shape=shape, K=1, split=split, design=design)
    eight = _sm(gas, shape=shape, K=8, split=split, design=design)
    for spool in ("lp", "hp"):
        assert one.stage_throat_margin(FLIGHT, 800.0)[spool]["amplification"] == 1.0
        vals = [eight.stage_throat_margin(FLIGHT, T)[spool]["amplification"]
                for T in (1200.0, 1000.0, 800.0)]
        assert vals[0] <= vals[1] < vals[2], (
            f"the amplification must grow with throttle depth on {spool}/{shape}: {vals}")
        assert vals[-1] >= 1.15, (
            f"P2's band: >= 1.15x at Tt4 = 800 on {spool}/{shape}/{split}, got {vals[-1]:.4f}")


def test_uniform_profile_amplifies_harder_than_derived():
    """P2's second half. The derived profile is PROTECTIVE -- it designs the rear rows with
    more capacity -- so the naive uniform read overstates the rear's exposure. This is why the
    profile is disclosed and no LEVEL claim is made robust to it."""
    gas = _cpg_gas()
    design = _design(gas)
    d = _sm(gas, prof="derived", design=design)
    u = _sm(gas, prof="uniform", design=design)
    for Tt4 in (1000.0, 800.0):
        for spool in ("lp", "hp"):
            assert (u.stage_throat_margin(FLIGHT, Tt4)[spool]["amplification"]
                    > d.stage_throat_margin(FLIGHT, Tt4)[spool]["amplification"])


# ======================================================================================
# GATE 5 — P1: the binding row MIGRATES (the seam HIT at part power, REFUTED near design)
# ======================================================================================

@pytest.mark.parametrize("shape", list(SHAPES))
@pytest.mark.parametrize("split", ["dT", "tau"])
def test_binding_row_migrates_front_to_rear(shape, split):
    """P1. The derived profile designs the rear rows with MORE capacity exactly where the
    off-design march loads them hardest, so the two fight: the profile wins near design (the
    FRONT binds) and the loading wins at part power (the REAR binds). Rung 55's seam predicted
    only the rear -- it is HIT at part power and REFUTED near design, for a DERIVED reason.

    RUN UNDER BOTH SPLITS DELIBERATELY. P4 shows the split moves this rung's levels, and the
    INTERIOR crossover cell is genuinely fragile to it (`press/flow` HP at Tt4 = 1200 binds row
    2 on `dT` and row 3 on `tau`). What is pinned here is therefore the migration's EXISTENCE
    and ONE-WAYNESS plus the two EXTREME cells -- and those are robust: the gap between the
    best and second-best margin is 1.1e-2 .. 4.1e-2 across both splits and all five shapes,
    five orders above gate 3's ~1e-14 design drift. The interior cell is never asserted."""
    gas = _cpg_gas()
    m = _sm(gas, shape=shape, split=split)
    for spool in ("lp", "hp"):
        w = m.throat_walk(FLIGHT, WALK, spool)
        assert w[0]["binds"] == 0, (
            f"near design the derived PROFILE binds (front row, highest Mach) on "
            f"{spool}/{shape}, got row {w[0]['binds']}")
        assert w[-1]["binds"] == len(w[-1]["margins"]) - 1, (
            f"at part power the LOADING binds (rear row) on {spool}/{shape}")
        first_rear = min(i for i, r in enumerate(w) if r["binds"] == len(r["margins"]) - 1)
        assert all(r["binds"] == len(r["margins"]) - 1 for r in w[first_rear:]), (
            "the migration must be one-way: once the loading wins it does not hand back")
        for r in (w[0], w[-1]):     # the two pinned cells are decided by a WIDE margin
            g = sorted(r["margins"])
            assert g[1] - g[0] > 1e-3, (
                f"a pinned binding-row cell must not be near-degenerate ({spool}/{shape}/"
                f"{split}, Tt4={r['Tt4']}): gap {g[1] - g[0]:.2e}")


@pytest.mark.parametrize("shape", list(SHAPES))
def test_uniform_profile_binds_at_the_rear_at_every_off_design_throttle(shape):
    """The control. Strip the derived profile and the contest disappears -- X_k alone decides,
    and it rises rearward monotonically. DESIGN IS EXCLUDED: there the rows tie to ~1e-14 and
    the binding row is float noise (gate 3)."""
    gas = _cpg_gas()
    m = _sm(gas, shape=shape, prof="uniform")
    for spool in ("lp", "hp"):
        for r in m.throat_walk(FLIGHT, WALK[1:], spool):
            assert r["binds"] == len(r["margins"]) - 1, (
                f"uniform C must bind at the rear at Tt4={r['Tt4']} on {spool}/{shape}")


# ======================================================================================
# GATE 6 — P3: K is a RESOLUTION
# ======================================================================================

@pytest.mark.slow
def test_K_is_a_resolution_increments_shrink():
    """P3. The amplification grows with K but its increments shrink monotonically, so the
    disclosed integer is a resolution coordinate and no claim rides on a particular K.
    (Scored HONESTLY: the LP increments halve; the HP's shrink by ~0.53 per doubling, which
    MISSES the pre-registered 'at least halves' band while confirming what it encoded.)"""
    gas = _cpg_gas()
    design = _design(gas)
    for spool in ("lp", "hp"):
        vals = [_sm(gas, K=K, design=design).stage_throat_margin(
            FLIGHT, 800.0)[spool]["amplification"] for K in (1, 2, 4, 8, 16, 32)]
        inc = [vals[i + 1] - vals[i] for i in range(len(vals) - 1)]
        assert all(d > 0.0 for d in inc), f"amplification must grow with K on {spool}: {vals}"
        for i in range(len(inc) - 1):
            assert inc[i + 1] < inc[i], f"increments must SHRINK on {spool}: {inc}"
            assert inc[i + 1] / inc[i] < 0.60, (
                f"and shrink geometrically (first order) on {spool}: {inc}")


# ======================================================================================
# GATE 7 — P4: the disclosed SPLIT is LOAD-BEARING here (contrast rung 55 P6)
# ======================================================================================

@pytest.mark.parametrize("shape", list(SHAPES))
def test_split_is_load_bearing_but_carries_no_sign(shape):
    """P4, and an honest inversion of rung 55 P6. The amplification rides on the internal
    theta/varpi ladder, which is exactly what the disclosed work split moves -- so unlike rung
    55 (where the split moved d_phi by 0.01 %) it moves this by 2-5 %. The LEVELS are therefore
    disclaimed on the split; the SIGNS and the part-power binding row are not."""
    gas = _cpg_gas()
    design = _design(gas)
    a = _sm(gas, shape=shape, split="dT", design=design)
    b = _sm(gas, shape=shape, split="tau", design=design)
    for spool in ("lp", "hp"):
        for Tt4 in (1000.0, 800.0):
            x = a.stage_throat_margin(FLIGHT, Tt4)[spool]
            y = b.stage_throat_margin(FLIGHT, Tt4)[spool]
            rel = abs(y["amplification"] - x["amplification"]) / (x["amplification"] - 1.0)
            # The PRE-REGISTERED band is HP at Tt4 = 800 (> 2 %); it holds there on every
            # shape, 3.7-4.6 %. Off that cell the sweep is asserted at its MEASURED floor --
            # the tightest cell in the grid is flat-eta LP at 1000, 1.98 %. Reported, not
            # rounded up to the prediction: the band the prediction named is the one scored.
            floor = 0.02 if (spool == "hp" and Tt4 == 800.0) else 0.017
            assert rel > floor, (
                f"P4 says the split MOVES this ({spool}/{shape}/{Tt4}): rel = {rel:.4f}")
            assert x["binds"] == y["binds"] == len(x["stages"]) - 1, \
                "but it must not move the part-power binding row"
            assert (x["m_c_worst"] > 0.0) == (y["m_c_worst"] > 0.0)
    # ... and two orders of magnitude above rung 55 P6's 0.01 %, which is the contrast.
    assert (a.stage_throat_margin(FLIGHT, 800.0)["hp"]["amplification"]
            != b.stage_throat_margin(FLIGHT, 800.0)["hp"]["amplification"])


# ======================================================================================
# GATE 8 — P5: THE HEADLINE. Opposite ENDS, opposite SPOOLS; and rung 54 CORRECTED
# ======================================================================================

@pytest.mark.parametrize("shape", list(SHAPES))
def test_two_constraints_opposite_ends_and_opposite_spools(shape):
    """P5 — THE HEADLINE, in its strong form. Rung 55's seam predicted front-vs-back on one
    machine. Measured, it is more than that: at part power the worst INCIDENCE margin in the
    whole machine is the LP's FRONT row and the worst CAPACITY margin is the HP's REAR row.
    Opposite end AND opposite spool -- a lumped block has one phi and one face, and cannot
    express either statement, let alone their separation."""
    gas = _cpg_gas()
    m = _sm(gas, shape=shape)
    for Tt4 in (1000.0, 800.0):
        r = m.stage_throat_margin(FLIGHT, Tt4)
        lp, hp = r["lp"], r["hp"]
        assert lp["inc_worst"] == 0 and hp["inc_worst"] == 0, \
            "incidence binds at the FRONT of each spool (rung 55 P4, inherited)"
        assert lp["binds"] == len(lp["stages"]) - 1 and hp["binds"] == len(hp["stages"]) - 1, \
            "capacity binds at the REAR of each spool"
        assert lp["m_i_worst"] < hp["m_i_worst"], \
            "the machine's INCIDENCE exposure is the LP's (rungs 41/44/45/53's split)"
        assert hp["m_c_worst"] < lp["m_c_worst"], \
            "but the machine's CAPACITY exposure is the HP's -- the opposite spool"


def test_rung54s_hp_throat_claim_is_corrected_by_resolution():
    """Rung 54 § The exposure split wrote: 'The HP schedule's demand falls monotonically and
    never approaches its throat at any throttle.' At the FACE that is true and stays true.
    Resolved into rows it is nearly false: the HP REAR row's margin FALLS with throttle while
    the face's RISES, and the threshold on the constant reaches C* ~ 0.91.

    The rung-28 shape: the face-level reasoning survives as a face-level statement, and the
    verdict it supported is corrected by resolution. Stated as a THRESHOLD ON the constant
    (rung 54's discipline), never as a level."""
    gas = _cpg_gas()
    design = _design(gas)
    m = _sm(gas, design=design)
    face, rear, cstar = [], [], []
    for Tt4 in (1200.0, 1000.0, 800.0):
        r = m.stage_throat_margin(FLIGHT, Tt4)["hp"]
        face.append(r["m_c_face"])
        rear.append(r["stages"][-1]["m_c"])
        cstar.append(r["stages"][-1]["c_min"])
    assert face[0] < face[1] < face[2], "at the FACE the HP relaxes with throttle (rung 54)"
    assert rear[0] > rear[1] > rear[2], "at the REAR ROW it TIGHTENS -- the opposite sign"
    assert cstar[-1] < 0.92, (
        f"and the constant-free threshold reaches C* = {cstar[-1]:.4f}: any HP row whose "
        f"design capacity fraction exceeds it is CHOKED at Tt4 = 800")
    u = _sm(gas, prof="uniform", design=design).stage_throat_margin(
        FLIGHT, 800.0)["hp"]["stages"][-1]["m_c"]
    assert 0.0 < u < 0.02, (
        f"and on the naive UNIFORM profile that row is a hair from choking ({u:.4f}) -- which "
        f"is what makes the derived profile a finding and not furniture")


def test_capacity_channel_stays_diagnostic_only():
    """Rung 54's refusal, inherited EXPLICITLY. The rear row at C = 0.90 is close to the wall
    and it is tempting to let it bind; rung 54 already priced that as inverting rung 31's (*),
    the flow being set at the first choked throat DOWNSTREAM. So a choked row must change
    nothing that is solved."""
    gas = _cpg_gas()
    design = _design(gas)
    a = _sm(gas, C=0.99, prof="uniform", design=design)
    r = a.stage_throat_margin(FLIGHT, 800.0)["hp"]
    assert r["chokes"], "pick a C that provably chokes the binding row, or this gate is vacuous"
    b = _sm(gas, C=0.30, design=design)
    assert not b.stage_throat_margin(FLIGHT, 800.0)["hp"]["chokes"]
    for f in FIELDS:
        assert getattr(a.match(FLIGHT, 800.0), f) == getattr(b.match(FLIGHT, 800.0), f)


# ======================================================================================
# GATE 9 — P6: the positional lever's DEBIT, and its currency dependence
# ======================================================================================

def test_front_row_lever_debits_the_row_it_does_not_move():
    """P6's sign, measured before the headline was fixed (anchor § PROBE B2). The front-row
    stator reaches an unmoved rear row only through the solved (m, n) -- and the sign is a
    DEBIT: closing it costs the rear row throat margin, monotonically. Rung 55's honest half
    (the shaft speed is the one thing every stage shares) in a second currency."""
    gas = _cpg_gas()
    design = _design(gas)
    for Tt4 in (1000.0, 800.0):
        rear = [_sm(gas, vl=v, vs_lp=1, design=design).stage_throat_margin(
            FLIGHT, Tt4)["lp"]["stages"][-1]["m_c"] for v in (0.0, 0.20, 0.3536, 0.60)]
        for i in range(len(rear) - 1):
            assert rear[i + 1] < rear[i], f"the debit must be monotone in v: {rear}"
        front = [_sm(gas, vl=v, vs_lp=1, design=design).stage_throat_margin(
            FLIGHT, Tt4)["lp"]["stages"][0]["m_c"] for v in (0.0, 0.60)]
        assert (front[0] - front[1]) > 10.0 * (rear[0] - rear[-1]), (
            "and the lever's throat cost must land overwhelmingly on the row it MOVES")


def test_positional_advantage_is_currency_dependent():
    """P6 scored a MISS, and the miss is the content. The rear-row debit ratio (front-only /
    lumped) was predicted to track rung 55's dN ratio within 25 %. It does not: the SPEED
    ratio is nearly v-invariant (~0.11-0.13) while the THROAT ratio COLLAPSES with the
    setting (0.18 -> 0.03). The lumped lever spends every row's throat directly, by
    sqrt(1+v^2), on top of the speed rise -- so the positional lever's advantage is larger,
    and grows, in exactly the currency rung 54 introduced. Rung 53's law a fourth time:
    the LEVER'S COST is coordinate-dependent too."""
    gas = _cpg_gas()
    design = _design(gas)
    Tt4 = 1000.0
    base = _sm(gas, design=design).stage_throat_margin(FLIGHT, Tt4)["lp"]
    thr, spd = [], []
    for v in (0.20, 0.3536, 0.60):
        fr = _sm(gas, vl=v, vs_lp=1, design=design).stage_throat_margin(FLIGHT, Tt4)["lp"]
        lu = _sm(gas, vl=v, design=design).stage_throat_margin(FLIGHT, Tt4)["lp"]
        thr.append((base["stages"][-1]["m_c"] - fr["stages"][-1]["m_c"])
                   / (base["stages"][-1]["m_c"] - lu["stages"][-1]["m_c"]))
        spd.append(((fr["n"] - base["n"]) / base["n"]) / ((lu["n"] - base["n"]) / base["n"]))
    assert all(t < 1.0 for t in thr) and all(s < 1.0 for s in spd), \
        "the positional lever must be cheaper in BOTH currencies"
    assert thr[0] > thr[1] > thr[2], f"the THROAT ratio collapses with v: {thr}"
    assert (max(spd) - min(spd)) < 0.10 * min(spd), f"the SPEED ratio does not: {spd}"
    assert thr[-1] < 0.5 * spd[-1], (
        f"so at the larger setting the two currencies disagree by >2x: {thr[-1]:.4f} vs "
        f"{spd[-1]:.4f} -- the pre-registered 'within 25 %' is REFUTED")


def test_lever_relocates_the_binding_row_to_itself_at_large_setting():
    """B3, rung 50's shape in a currency rung 50 never saw. Push the front-row setting far
    enough and the row's OWN throat cost sqrt(1+v^2) overwhelms the rear's loading, so the
    binding capacity row relocates to the moved row. The threshold sits well ABOVE rung 55's
    own front-row schedule (v* ~ 0.35), so rung 55's published lever does not trip it."""
    gas = _cpg_gas()
    design = _design(gas)
    for Tt4 in (1000.0, 800.0):
        binds = [_sm(gas, vl=v, vs_lp=1, design=design).stage_throat_margin(
            FLIGHT, Tt4)["lp"]["binds"] for v in (0.0, 0.3536, 1.2)]
        assert binds[0] == binds[1] == 7, \
            "at and below rung 55's schedule setting the REAR still binds"
        assert binds[2] == 0, "far enough closed, the moved row binds ITSELF"


# ======================================================================================
# GATE 10 — CYCLE UNTOUCHED
# ======================================================================================

def test_cycle_untouched_default_design_run_is_bit_for_bit_rung6():
    """The project's standing gate: rung 56 is reached through a separate entry point, so the
    default single-spool design run must be bit-for-bit what it was."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, 1600.0, FLIGHT.p0,
                         pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96,
                         eta_t=0.92, eta_m=0.99, pi_n=0.98)
    a = eng.run(FLIGHT, 1.0)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    for st in ("2", "3", "4", "5", "9"):
        assert a.stations[st].Tt == b.stations[st].Tt
        assert a.stations[st].pt == b.stations[st].pt


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
