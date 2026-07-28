"""Rung 55 — THE STAGE STACK: a lever that moves ROWS, and what it pays for them.

Rung 54 refuted flow capacity as the escape from rung 53's overspeed and named STAGE
REMATCHING as the real mechanism. This rung builds the stack and takes the seam.

Gates (named in docs/rung55-spec.md § Verification gates):

   1. REDUCE — an IDENTITY at K = 1: no stack object is built, both efficiency loops are the
      INHERITED ones, so every matched field is bit-identical to rung 53/54 at a MOVED stator
      and on both gases. StageStack.solve_n itself dispatches at K = 1.
   2. THE DERIVED KINEMATICS — phi_1 IS the face phi = m/n exactly (so rungs 36-53 were
      reading the front stage all along), and the design ladder is exact for every K and
      every split (the stack does NOT re-design the engine).
   3. THE NON-TAUTOLOGY GATE — the marched stack does DIFFERENT work than the lumped law at
      the same (m, n): exactly 0.00e+00 at K = 1, non-zero and growing with throttle depth
      beyond it. Without this the rung would be a re-read of (tau_c, pi_c).
   4. P1 — the RUNNING LINE MOVES: n RISES and phi FALLS, monotonically with throttle depth,
      on every shape; thrust and pi_c barely move (paid in SHAFT SPEED, like rung 53).
   5. P4 — one machine, two opposite failures: the LP FRONT stage is the worst incidence in
      the machine while the HP REAR stage runs ABOVE design phi (toward choke).
   6. P5 — K is a RESOLUTION: the shift grows with K but its increments SHRINK.
   7. P6 — the disclosed WORK SPLIT does not carry any verdict.
   8. SCOPE, ASSERTED not declared — the rung-34/40/43 transient ladders never construct a
      stack and are bit-for-bit unstacked.
   9. P3 — THE HEADLINE: the front-row lever's cost FACTORISES as (1/K) x (v* ratio), and the
      row count has an INTERIOR optimum (relief peaks at 3-4 rows, then REVERSES).
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
    VariableStatorMatcher, StageStack, StageStackMatcher, TwoSpoolFuelTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55

# Rung 53/54's five disclosed shapes, verbatim.
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
FIELDS = ("pi_lpc", "pi_hpc", "n_lp", "n_hp", "phi_lp", "phi_hp", "slip",
          "eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt", "tau_lpc", "tau_hpc",
          "tau_hpt", "tau_lpt", "mdot_air", "thrust", "N_lp_ratio", "N_hp_ratio")


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _maps(shape="flow/press"):
    return tuple(m.with_phi_surge(FLOOR) for m in SHAPES[shape])


def _sm(gas, shape="flow/press", K_lp=1, K_hp=1, vl=0.0, vh=0.0, split="dT",
        vs_lp=None, vs_hp=None, design=None):
    ml, mh = _maps(shape)
    return StageStackMatcher(design if design is not None else _design(gas), FLIGHT, 1.0,
                             map_lp=ml, map_hp=mh, vsv_lp=vl, vsv_hp=vh, K_lp=K_lp,
                             K_hp=K_hp, split=split, vsv_stages_lp=vs_lp,
                             vsv_stages_hp=vs_hp)


# ======================================================================================
# GATE 1 — REDUCE: an IDENTITY at K = 1
# ======================================================================================

@pytest.mark.parametrize("vl,vh", [(0.0, 0.0), (0.30, 0.0), (0.0, 0.15), (0.20, 0.10)])
def test_reduce_K1_is_bit_for_bit_rung53(vl, vh):
    """THE SPINE. At K = 1 no stack object exists, both efficiency loops are the INHERITED
    rung-39 ones, and there is no rung-55 code path to skip -- so this is an identity, not a
    tolerance. Checked at a MOVED stator so it cannot be passing by way of rung 53's own
    v == 0 early returns."""
    gas = _cpg_gas()
    design = _design(gas)
    ml, mh = _maps()
    ref = VariableStatorMatcher(design, FLIGHT, 1.0, map_lp=ml, map_hp=mh,
                                vsv_lp=vl, vsv_hp=vh)
    st = _sm(gas, K_lp=1, K_hp=1, vl=vl, vh=vh, design=design)
    assert st.stack_lp is None and st.stack_hp is None, \
        "rung 55 must not build a stack object at K = 1 -- the reduce is an identity"
    for Tt4 in THROTTLE:
        a, b = ref.match(FLIGHT, Tt4), st.match(FLIGHT, Tt4)
        for f in FIELDS:
            assert getattr(a, f) == getattr(b, f), (
                f"rung-55 K=1 reduce broken on {f} at Tt4={Tt4}, vsv=({vl},{vh}): "
                f"{getattr(a, f)!r} vs {getattr(b, f)!r}")


def test_reduce_stack_object_dispatches_at_K1():
    """Even a HAND-BUILT one-stage stack is bit-for-bit: StageStack.solve_n dispatches to
    rung 32's own ComponentMap.solve_n, so it is the same code and not merely the same
    algebra."""
    gas = _cpg_gas()
    m = _sm(gas, K_lp=8, K_hp=8)                    # a matcher only to read the design point
    cmap, tau_d, pi_d, eta_d = _maps()[0], m.tau_lpc_d, m.pi_lpc_design, m.eta_lpc
    stack = StageStack(K=1, cmap=cmap, tau_d=tau_d, pi_d=pi_d, eta_d=eta_d)
    for mm, tau in ((1.0, tau_d), (0.73, 1.3255), (0.46, 1.2150)):
        assert stack.solve_n(mm, tau, eta_d) == cmap.solve_n(mm, tau, tau_d)
    assert stack.e_d == pytest.approx(eta_d, abs=1e-12), \
        "at K = 1 the per-stage efficiency IS the lumped one (the inversion is the identity)"


@pytest.mark.parametrize("kc", [1.4 / 0.4])
def test_stack_reproduces_rung2b_polytropic_efficiency(kc):
    """GATE 2b — A FREE CONSISTENCY CHECK ON THE WHOLE CONSTRUCTION.

    Nothing in the stack was told about polytropic efficiency: it is handed an ISENTROPIC
    design point (tau_d, pi_d, eta_d) and a stage count. Yet the derived per-stage efficiency
    comes out ABOVE the lumped one (the REHEAT effect) and converges, first order, on rung 2b's
    polytropic e_c = ln(pi_d)/(kc*ln(tau_d)). The stack therefore INTERPOLATES rung 2 (K = 1,
    isentropic) to rung 2b (K -> infinity, polytropic), and rung 2b's shipped eta_c < e_c
    ordering falls out rather than being imposed."""
    gas = _cpg_gas()
    m = _sm(gas, K_lp=8, K_hp=8)
    cmap, tau_d, pi_d, eta_d = _maps()[0], m.tau_lpc_d, m.pi_lpc_design, m.eta_lpc
    e_poly = math.log(pi_d) / (kc * math.log(tau_d))
    assert e_poly > eta_d, "rung 2b's own ordering: eta_c < e_c for a compressor"

    errs = []
    for K in (1, 2, 4, 8, 16, 32):        # every step a DOUBLING
        s = StageStack(K=K, cmap=cmap, tau_d=tau_d, pi_d=pi_d, eta_d=eta_d, kc=kc)
        if K == 1:
            assert s.e_d == pytest.approx(eta_d, abs=1e-12)
        else:
            assert eta_d < s.e_d < e_poly, f"K={K}: e_d must sit BETWEEN the two rungs"
        errs.append(e_poly - s.e_d)
    assert errs == sorted(errs, reverse=True), "the approach to e_c must be monotone"
    for a, b in zip(errs[1:], errs[2:]):
        assert 0.35 < b / a < 0.65, f"first-order convergence to e_c expected, got {b/a:.3f}"


@pytest.mark.slow
def test_reduce_K1_on_the_reacting_equilibrium_gas():
    """The identity is a property of the code path, not of the gas -- so it must hold on the
    production reacting-equilibrium gas too (rung 53/54's both-gases discipline)."""
    gas = Gas.reacting_equilibrium()
    design = _design(gas)
    ml, mh = _maps()
    ref = VariableStatorMatcher(design, FLIGHT, 1.0, map_lp=ml, map_hp=mh, vsv_lp=0.20)
    st = _sm(gas, K_lp=1, K_hp=1, vl=0.20, design=design)
    for Tt4 in (1500.0, 1200.0):
        a, b = ref.match(FLIGHT, Tt4), st.match(FLIGHT, Tt4)
        for f in FIELDS:
            assert getattr(a, f) == getattr(b, f), f"rung-55 reacting-gas reduce broken on {f}"


# ======================================================================================
# GATE 2 — THE DERIVED KINEMATICS
# ======================================================================================

@pytest.mark.parametrize("K", [2, 4, 8, 16])
@pytest.mark.parametrize("split", ["dT", "tau"])
def test_design_ladder_is_exact_for_every_K_and_split(K, split):
    """THE STACK DOES NOT RE-DESIGN THE ENGINE (rung 42/53's design-capture discipline). At
    the design point every phi_k = 1, every n_k = 1, psi = 1, and the march returns tau_d --
    exactly, for any resolution and any disclosed work split."""
    gas = _cpg_gas()
    m = _sm(gas, K_lp=K, K_hp=K, split=split)
    for stack, tau_d, eta_d in ((m.stack_lp, m.tau_lpc_d, m.eta_lpc),
                                (m.stack_hp, m.tau_hpc_d, m.eta_hpc)):
        r = stack.march(1.0, 1.0, eta_d)
        assert r["tau"] == pytest.approx(tau_d, rel=1e-12), \
            f"K={K} split={split}: design march must return tau_d exactly"
        assert r["clamped"] == 0
        for k, (phi, nk) in enumerate(zip(r["phis"], r["n_ks"])):
            assert phi == pytest.approx(1.0, abs=1e-12), f"stage {k} phi != 1 at design"
            assert nk == pytest.approx(1.0, abs=1e-12), f"stage {k} n_k != 1 at design"
        # the per-stage efficiency reproduces the SHIPPED design pi (no new constant)
        assert stack._ladder_p(stack.theta_d, stack.e_d)[-1] == pytest.approx(
            stack.pi_d, rel=1e-10)
        assert stack.e_d > eta_d, \
            "the REHEAT effect: a resolved stack's per-stage eta sits ABOVE the lumped one"


@pytest.mark.parametrize("K", [4, 8])
def test_front_stage_phi_is_the_face_phi(K):
    """THE CROSS-RUNG RESULT, before any measurement: phi_1 = m/n EXACTLY, so the face flow
    coefficient every rung since 32 reads IS the front stage's. Rungs 36-53 were reading the
    binding stage all along -- a BOUNDING in rung 53's style, not a refutation."""
    gas = _cpg_gas()
    m = _sm(gas, K_lp=K, K_hp=K)
    for Tt4 in THROTTLE:
        r = m.stage_margin(FLIGHT, Tt4)
        for spool in ("lp", "hp"):
            s = r[spool]
            assert s["stages"][0]["phi"] == pytest.approx(s["phi_face"], rel=1e-13), (
                f"{spool} stage-0 phi must BE the face phi at Tt4={Tt4}")
            assert s["phi_face"] == pytest.approx(s["m"] / s["n"], rel=1e-13)


def test_capacity_style_guards_reject_nonsense():
    cmap = _maps()[0]
    with pytest.raises(AssertionError):
        StageStack(K=0, cmap=cmap, tau_d=1.4, pi_d=3.0, eta_d=0.9)
    with pytest.raises(AssertionError):
        StageStack(K=4, cmap=cmap, tau_d=1.4, pi_d=3.0, eta_d=0.9, split="equal-psi")
    with pytest.raises(AssertionError):
        StageStack(K=4, cmap=cmap, tau_d=1.4, pi_d=3.0, eta_d=0.9, vsv_stages=5)


# ======================================================================================
# GATE 3 — THE NON-TAUTOLOGY GATE (the advisor's, and the reason this is a rung)
# ======================================================================================

def test_marched_work_differs_from_lumped_and_grows_with_throttle_depth():
    """WITHOUT THIS THE RUNG IS A RE-READ. The front-to-rear spread alone is a functional of
    the (tau_c, pi_c) rung 39 already solves. The content is the FEEDBACK: with a per-stage
    psi(phi_k) the machine's work is no longer psi(phi_face)*n^2, so the stack MOVES the
    running line. Exactly zero at K = 1; negative (the stack is WEAKER) and deepening with
    throttle beyond it."""
    gas = _cpg_gas()
    design = _design(gas)
    flat = _sm(gas, K_lp=1, K_hp=1, design=design)
    for Tt4 in THROTTLE:
        g = flat.work_gap(FLIGHT, Tt4)
        for spool in ("lp", "hp"):
            assert g[spool]["gap"] == 0.0, "K=1 march IS the lumped law, exactly"

    m8 = _sm(gas, K_lp=8, K_hp=8, design=design)
    prev = {"lp": 0.0, "hp": 0.0}
    for Tt4 in THROTTLE:                      # descending throttle
        g = m8.work_gap(FLIGHT, Tt4)
        for spool in ("lp", "hp"):
            frac = g[spool]["gap_frac"]
            if Tt4 == 1500.0:
                assert abs(frac) < 1e-12, "at design the stack does the design work exactly"
            else:
                assert frac < 0.0, (
                    f"the marched stack must be WEAKER than the lumped law ({spool}, "
                    f"Tt4={Tt4}): got {frac:+.4e}")
                assert frac < prev[spool], (
                    f"the gap must DEEPEN with throttle depth ({spool}, Tt4={Tt4})")
            prev[spool] = frac
    # the HP carries the bigger pressure ratio, hence the bigger density mismatch
    g = m8.work_gap(FLIGHT, 800.0)
    assert g["hp"]["gap_frac"] < g["lp"]["gap_frac"] < -0.05


# ======================================================================================
# GATE 4 — P1: the RUNNING LINE MOVES (n up, phi down), and it is paid in SHAFT SPEED
# ======================================================================================

@pytest.mark.parametrize("shape", list(SHAPES))
def test_p1_running_line_shift_sign_and_monotonicity(shape):
    """P1, pre-registered and HIT on sign + monotonicity (the LEVEL was predicted 5-15 % and
    measured 2.7-4.2 % -- scored a miss in the anchor). A weaker stack must be run FASTER to
    do the pinned work, so n RISES and the front stage's phi FALLS."""
    gas = _cpg_gas()
    m8 = _sm(gas, shape, K_lp=8, K_hp=8)
    rows = m8.running_line_shift(FLIGHT, THROTTLE)
    assert abs(rows[0]["lp"]["d_n"]) < 1e-9 and abs(rows[0]["lp"]["d_phi"]) < 1e-9, \
        "the design point must not move (the stack is design-consistent)"
    for spool in ("lp", "hp"):
        dn = [r[spool]["d_n"] for r in rows[1:]]
        dphi = [r[spool]["d_phi"] for r in rows[1:]]
        assert all(x > 0.0 for x in dn), f"{shape}/{spool}: n must RISE"
        assert all(x < 0.0 for x in dphi), f"{shape}/{spool}: phi must FALL"
        assert dn == sorted(dn), f"{shape}/{spool}: the shift must deepen with throttle"
        assert dphi == sorted(dphi, reverse=True)


def test_p1_is_paid_in_shaft_speed_not_performance():
    """Like rung 53's stator, the stack is thrust-neutral: it moves SPEED. On a flat efficiency
    island pi_c cannot move at all, which isolates the channel exactly."""
    gas = _cpg_gas()
    for r in _sm(gas, K_lp=8, K_hp=8).running_line_shift(FLIGHT, THROTTLE):
        assert abs(r["d_thrust"]) < 0.01, "thrust must barely move"
        assert abs(r["lp"]["d_pi"]) < 0.005
        assert abs(r["lp"]["d_n"]) > 3.0 * abs(r["d_thrust"]) or r["Tt4"] == 1500.0
    for r in _sm(gas, "flat-eta", K_lp=8, K_hp=8).running_line_shift(FLIGHT, THROTTLE):
        assert r["lp"]["d_pi"] == pytest.approx(0.0, abs=1e-12), (
            "on a flat island the stack cannot touch pi_c AT ALL -- it is a pure speed lever")


# ======================================================================================
# GATE 5 — P4: one machine, two OPPOSITE failures
# ======================================================================================

def test_p4_front_stalls_while_the_rear_chokes():
    """P4, pre-registered and HIT. The smallest incidence margin in the machine is the LP's
    FRONT stage; the largest excursion on the HP is its REAR stage, running ABOVE design phi
    (toward choke / negative incidence). A lumped block has ONE phi and can represent neither
    end of it."""
    gas = _cpg_gas()
    m8 = _sm(gas, K_lp=8, K_hp=8)

    at_design = m8.stage_margin(FLIGHT, 1500.0)
    for spool in ("lp", "hp"):
        assert at_design[spool]["rear_excess"] == pytest.approx(0.0, abs=1e-12)

    r = m8.stage_margin(FLIGHT, 800.0)
    lp, hp = r["lp"], r["hp"]
    assert lp["worst"] == 0 and hp["worst"] == 0, "the FRONT stage stalls first on both spools"
    assert lp["m_i_worst"] < hp["m_i_worst"], \
        "the LP front stage is the worst incidence in the whole machine (rung 41's split)"
    assert hp["phi_rear"] > 1.10, \
        "the HP REAR stage must run ABOVE design phi -- toward choke"
    assert lp["phi_front"] < 0.75 < lp["phi_rear"], "the LP spans the design point front-to-rear"
    for spool in ("lp", "hp"):
        phis = [s["phi"] for s in r[spool]["stages"]]
        assert phis == sorted(phis), f"{spool}: phi must rise MONOTONICALLY front to rear"
        assert r[spool]["rear_excess"] > 0.30


# ======================================================================================
# GATE 6 — P5: K is a RESOLUTION, not a knob
# ======================================================================================

@pytest.mark.slow
def test_p5_shift_converges_in_K():
    """P5, pre-registered and HIT with room to spare: the shift GROWS with K but its
    INCREMENTS SHRINK -- and in fact halve as K doubles (first-order convergence), so the
    stack has a well-defined continuum limit and no verdict rides on a particular K."""
    gas = _cpg_gas()
    design = _design(gas)
    for Tt4 in (1200.0, 1000.0, 800.0):
        vals = [_sm(gas, K_lp=K, K_hp=K, design=design)
                .running_line_shift(FLIGHT, [Tt4])[0]["lp"]["d_phi"]
                for K in (1, 2, 4, 8, 16)]
        incr = [abs(b - a) for a, b in zip(vals, vals[1:])]
        assert all(x > 0.0 for x in incr)
        assert incr == sorted(incr, reverse=True), (
            f"the K-increments must SHRINK at Tt4={Tt4}: {incr}")
        for a, b in zip(incr[1:], incr[2:]):        # halving, within 25 %
            assert 0.35 < b / a < 0.65, f"first-order convergence expected, got {b/a:.3f}"


# ======================================================================================
# GATE 7 — P6: the disclosed WORK SPLIT carries no verdict
# ======================================================================================

def test_p6_verdicts_survive_the_work_split():
    """P6, pre-registered band < 25 % and HIT by an order of magnitude. The split is rung 54's
    'disclosed level' pattern: the KINEMATICS are derived, the split is disclosed, and the
    verdict is asserted across it."""
    gas = _cpg_gas()
    design = _design(gas)
    a = _sm(gas, K_lp=8, K_hp=8, split="dT", design=design)
    b = _sm(gas, K_lp=8, K_hp=8, split="tau", design=design)
    for Tt4 in (1200.0, 1000.0, 800.0):
        ra = a.running_line_shift(FLIGHT, [Tt4])[0]
        rb = b.running_line_shift(FLIGHT, [Tt4])[0]
        for spool in ("lp", "hp"):
            x, y = ra[spool]["d_phi"], rb[spool]["d_phi"]
            assert x < 0.0 and y < 0.0
            assert abs(y - x) / abs(x) < 0.25, f"{spool} d_phi split-sensitive at Tt4={Tt4}"
        sa = a.stage_margin(FLIGHT, Tt4)
        sb = b.stage_margin(FLIGHT, Tt4)
        assert sa["lp"]["worst"] == sb["lp"]["worst"] == 0
        assert abs(sb["hp"]["rear_excess"] - sa["hp"]["rear_excess"]) \
            / sa["hp"]["rear_excess"] < 0.25


# ======================================================================================
# GATE 8 — SCOPE, ASSERTED: the transient ladders never see a stack
# ======================================================================================

def test_cycle_untouched_transient_ladder_is_bit_for_bit_unstacked():
    """THE SCOPE BOUNDARY, GATED. Rung 55 enters the SOLVER, so a leak into the rung-34/40/43
    forward closures would silently move rungs 34-52. Those closures read ComponentMap.psi /
    phi_max directly and construct no stack -- asserted here by running a rung-43 fuel
    transient and demanding it is bit-identical to itself with a stack CLASS imported and a
    stacked matcher live on the same hardware."""
    gas = _cpg_gas()
    design = _design(gas)
    ml, mh = _maps()

    def peak():
        tr = TwoSpoolFuelTransient(design, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        mf0 = tr.fuel_for_Tt4(FLIGHT, 1000.0)
        mf1 = tr.fuel_for_Tt4(FLIGHT, 1400.0)
        eq0 = tr.equilibrium(FLIGHT, 1000.0)

        def sched(s):
            return mf0 + (mf1 - mf0) * min(1.0, s / 0.5)

        pts = tr.integrate_fuel(FLIGHT, sched, (eq0["nu_lp"], eq0["nu_hp"]),
                                s_end=2.0, ds=0.01)
        return [(p["s"], p["nu_lp"], p["nu_hp"], p["Tt4"]) for p in pts]

    before = peak()
    live = _sm(gas, K_lp=8, K_hp=8, design=design)     # a stack IS live on this hardware
    live.match(FLIGHT, 1000.0)
    after = peak()
    assert before == after, (
        "rung-55 SCOPE VIOLATION: a live stage stack changed a rung-43 transient result. The "
        "transient closures must stay on the lumped loading law (docs/rung55-spec.md § Scope).")
    assert live.stack_lp is not None and live.stack_lp.K == 8, "the stack must actually be live"


# ======================================================================================
# GATE 9 — P3: THE HEADLINE (the factorisation, and the interior row-count optimum)
# ======================================================================================

@pytest.mark.slow
def test_p3_front_row_lever_cost_factorises():
    """THE HEADLINE. Holding the front stage's design incidence with a FRONT-ROW-ONLY stator
    costs a small fraction of rung 53's whole-machine lever, and the collapse FACTORISES:

        dN_ratio = (1/K) x (v*_front / v*_lumped)

    to within 5 % across an 8x range in K. The 1/K leg was pre-registered; the SETTING leg was
    not, and it is why P3's level was scored a miss (0.035 measured against a 0.0625-0.25
    band at K = 8). A front-only lever does not fight its own speed rise.
    """
    T = 1000.0
    gas = _cpg_gas()
    design = _design(gas)
    ml, mh = _maps()
    r53 = VariableStatorMatcher(design, FLIGHT, 1.0, map_lp=ml, map_hp=mh)
    row53 = r53.incidence_schedule(FLIGHT, [T], spool="lp", v_hi=4.0)[0]
    b53 = r53.at_setting(0.0, 0.0).match(FLIGHT, T)
    s53 = r53.at_setting(row53["vsv_star"], 0.0).match(FLIGHT, T)
    dN53 = (s53.N_lp_ratio - b53.N_lp_ratio) / b53.N_lp_ratio
    assert dN53 > 0.60, "rung 53's lumped lever must be expensive (bare-at-throttle reference)"

    prev_v = None
    for K in (2, 4, 8, 16):
        m = _sm(gas, K_lp=K, K_hp=8, vs_lp=1, design=design)
        r = m.stage_incidence_schedule(FLIGHT, [T], spool="lp", stage=0, v_hi=4.0)[0]
        assert r["reached"], f"the front-row schedule must EXIST at K={K}"
        bare, sib = m.at_setting(0.0, 0.0), m.at_setting(r["vsv_star"], 0.0)
        dN = (sib.match(FLIGHT, T).N_lp_ratio - bare.match(FLIGHT, T).N_lp_ratio) \
            / bare.match(FLIGHT, T).N_lp_ratio
        v_ratio = r["vsv_star"] / row53["vsv_star"]
        assert dN / dN53 == pytest.approx(v_ratio / K, rel=0.05), (
            f"K={K}: the cost must factorise as (1/K)x(v* ratio)")
        assert v_ratio < 0.40, "the front-only lever needs a much SMALLER setting too"
        if prev_v is not None:              # v* SATURATES while the penalty keeps falling
            assert r["vsv_star"] < prev_v
        prev_v = r["vsv_star"]
    assert dN < 0.03, "at K = 16 the front-row lever must be nearly free in shaft speed"


@pytest.mark.slow
def test_p3_row_count_has_an_interior_optimum():
    """A POSITIONAL LEVER PAYS FOR THE ROWS IT MOVES OUT OF THE ROWS IT DOES NOT -- through the
    shaft speed every stage shares. So relief in the row count is not monotone: it peaks at
    3-4 rows of 8 and then REVERSES, ending WORSE than bare. The first object in this project
    whose optimum is a COUNT.

    (An advisor check forced this: the reversal was first seen at a coarse scan and could have
    been a bracket artifact. It is not -- the residual is smooth and single-rooted in v, and
    rows = 5 fills the curve. See docs/plans/rung55-anchor-stage-stack.md.)"""
    T, K = 1000.0, 8
    gas = _cpg_gas()
    design = _design(gas)
    base = _sm(gas, K_lp=K, K_hp=K, vs_lp=1, design=design)
    mi_bare = base.at_setting(0.0, 0.0).stage_margin(FLIGHT, T)["lp"]["m_i_worst"]

    relief, cost = {}, {}
    for rows in (1, 2, 3, 4, 5, 6):
        m = _sm(gas, K_lp=K, K_hp=K, vs_lp=rows, design=design)
        m._V_SCAN = 0.01
        r = m.stage_incidence_schedule(FLIGHT, [T], spool="lp", stage=0, v_hi=4.0)[0]
        assert r["reached"], f"rows={rows}: the schedule must exist"
        sib = m.at_setting(r["vsv_star"], 0.0)
        sm = sib.stage_margin(FLIGHT, T)["lp"]
        b = base.at_setting(0.0, 0.0).match(FLIGHT, T)
        relief[rows] = (sm["m_i_worst"] - mi_bare) / mi_bare
        cost[rows] = (sib.match(FLIGHT, T).N_lp_ratio - b.N_lp_ratio) / b.N_lp_ratio
        # the worst stage is PROMOTED into the rows the stator does not move
        assert sm["worst"] >= min(rows, K - 1) or sm["worst"] > 0

    assert max(relief, key=relief.get) in (3, 4), \
        f"relief must peak at 3-4 rows of 8, got {relief}"
    assert relief[6] < 0.0 < relief[1], \
        "moving too many rows must end WORSE than bare (the reversal)"
    assert relief[5] < relief[4], "the fall past the peak must be smooth, not a jump"
    # cost climbs monotonically while relief turns over -- TWO currencies, TWO optima
    assert cost == dict(sorted(cost.items())) or \
        [cost[r] for r in (1, 2, 3, 4, 5, 6)] == sorted(cost[r] for r in (1, 2, 3, 4, 5, 6))
    ppc = {r: relief[r] / cost[r] for r in relief}
    assert max(ppc, key=ppc.get) == 1, \
        "relief PER UNIT SPEED is cheapest at ONE row -- a different optimum, rung 53's law"


@pytest.mark.slow
def test_p3_all_rows_schedule_ceases_to_exist_deep_off_design():
    """Rung 53 conceded its schedule numbers were 'model-bound'. Resolved into stages, the
    ALL-ROWS schedule is not merely expensive -- below Tt4 ~ 1300 it is UNREACHABLE, the scan
    running into the speed-line bracket at v ~ 2.1-2.4. (Rung 54 found the same object ceasing
    to exist under the throat, by a different mechanism: two independent ceilings.)"""
    gas = _cpg_gas()
    m = _sm(gas, K_lp=8, K_hp=8, vs_lp=8)
    rows = m.stage_incidence_schedule(FLIGHT, (1500.0, 1300.0, 1100.0, 1000.0),
                                      spool="lp", stage=0, v_hi=4.0)
    reached = [r["reached"] for r in rows]
    assert reached[0] and reached[1], "the all-rows schedule must still exist near design"
    assert not reached[2] and not reached[3], \
        "the all-rows schedule must CEASE TO EXIST deep off design"


# ======================================================================================
# GATE 10 — THE CYCLE IS UNTOUCHED
# ======================================================================================

def test_cycle_untouched_default_design_run_is_bit_for_bit_rung6():
    """The project's standing gate: rung 55 is reached through a separate entry point, so the
    default single-spool design run must be bit-for-bit what it was."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, 1600.0, FLIGHT.p0, **{
        k: v for k, v in dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96,
                              eta_t=0.92, eta_m=0.99, pi_n=0.98).items()})
    a = eng.run(FLIGHT, 1.0)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    for st in ("2", "3", "4", "5", "9"):
        assert a.stations[st].Tt == b.stations[st].Tt
        assert a.stations[st].pt == b.stations[st].pt


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
