"""Rung 28 — the rung-26-coupled NO march (docs/rung28-spec.md).

Gates, in the priority order of the spec:

  1. RUNGS 26/27 UNTOUCHED — the trajectory recorder is a pure observer; a rung-28 call leaves
     `freeze_out_nozzle` and `no_freeze_out_nozzle` bit-for-bit.
  2. REDUCE (LOAD-BEARING) — `couple=False` ⇒ rung 27's march BIT-FOR-BIT (structural: the same
     expression sequence, via `_frozen_no_trajectory`), and `rate_scale→0` ⇒ the rung-14/17 clamp.
  3. THE HEADLINE SURVIVES — entry `Da_NO` is path-INDEPENDENT ⇒ bit-for-bit rung 27's at every Tt4;
     `frozen_from_entry` everywhere.
  4. THE MECHANISTIC CORRECTION — rung 27's "can ONLY slow NO further" is one-sided: BOTH channels are
     real, and the opposing one grows monotonically with Tt4.
  5. THE CONCLUSION SURVIVES — `net_factor < 1` at every in-band Tt4 (deeper into frozen).
  6. THE STRUCTURAL WIN — depletion is UNBOUNDED, heat release SATURATES (the pool-rate limit gate).
  7. THE β REPAIR — rung 27's a≫1 surrogate is a genuine rate bound in BOTH regimes (β<1), even though
     its stated premise (NO arrives super-equilibrium) is FALSE at the entry.
  8. NO MOTION — `relaxed_fraction ≈ 0`; the Da ratios are the clock's depth, not NO's motion.
  9. CYCLE UNTOUCHED + GUARDS.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import (
    Gas, FreezeOut, NOFreezeOut, CoupledNOFreezeOut,
    _equilibrium_composition, _freeze_out_expand, _tau_chem_recomb,
    _tau_no_destroy, _tau_no_exact, _frozen_no_trajectory,
)
from turbojet.engine import FlightCondition
from main import build_turbojet, PI_C, REAL_LOSSES

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
BAND = (1500.0, 1650.0, 1800.0, 2000.0, 2200.0, 2400.0)


def _run(Tt4):
    eq = Gas.reacting_equilibrium()
    r = build_turbojet(eq, PI_C, Tt4, FLIGHT.p0, **REAL_LOSSES).run(FLIGHT, 1.0)
    s3, s4, s9 = r.stations["3"], r.stations["4"], r.stations["9"]
    return eq, r, (s4.far, s3.Tt, s4.Tt, s4.pt, s9.Tt, s9.pt, r.p9, 1.0)


# ---------------------------------------------------------------- gate 1: neighbours untouched ---- #

def test_rung26_27_untouched_across_a_rung28_call():
    """A rung-28 call must not perturb rungs 26/27 — the recorder only reads."""
    eq, r, a = _run(2200.0)
    far, Tt3, Tt4, pt4, Tt9, pt9, p9, phi = a
    fz_before = eq.freeze_out_nozzle(far, Tt4, pt4, Tt9, pt9, p9, FreezeOut())
    n27_before = eq.no_freeze_out_nozzle(*a, NOFreezeOut())

    eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut())

    fz_after = eq.freeze_out_nozzle(far, Tt4, pt4, Tt9, pt9, p9, FreezeOut())
    n27_after = eq.no_freeze_out_nozzle(*a, NOFreezeOut())
    assert fz_after.V9_freeze == fz_before.V9_freeze
    assert fz_after.s_freeze == fz_before.s_freeze
    assert fz_after.Da_entry == fz_before.Da_entry
    assert n27_after.max_a == n27_before.max_a
    assert n27_after.Da_entry == n27_before.Da_entry


def test_trajectory_recorder_is_a_pure_observer():
    """`record=` must not change a single returned float of rung 26's marcher (the gate-1 mechanism)."""
    eq, r, a = _run(2200.0)
    far, _Tt3, Tt4, pt4, Tt9, pt9, p9, _phi = a
    comp = _equilibrium_composition(far, Tt4, pt4)
    nf = eq.nozzle_flow(far, Tt4, pt4, Tt9, pt9, p9, x_no_frozen=1e-4)
    tau_res = 0.5 / (0.6 * nf.V9_frozen)

    def da(c, T, p):
        return tau_res / _tau_chem_recomb(c, T, p)

    plain = _freeze_out_expand(comp, far, Tt9, pt9, p9, da, 400)
    rec = []
    watched = _freeze_out_expand(comp, far, Tt9, pt9, p9, da, 400, record=rec)
    T_p, V_p, c_p, dS_p, s_p, de_p, dx_p = plain
    T_w, V_w, c_w, dS_w, s_w, de_w, dx_w = watched
    assert (T_w, V_w, dS_w, s_w, de_w, dx_w) == (T_p, V_p, dS_p, s_p, de_p, dx_p)
    assert all(c_w[k] == c_p[k] for k in c_p)
    assert len(rec) == 401                      # nstep step-starts + the exit


# ------------------------------------------------------- gate 2: the reduce (LOAD-BEARING) -------- #

def test_uncoupled_is_rung27_bit_for_bit():
    """`couple=False` feeds the FROZEN trajectory ⇒ the identical expression sequence as rung 27."""
    for Tt4 in BAND:
        eq, r, a = _run(Tt4)
        n27 = eq.no_freeze_out_nozzle(*a, NOFreezeOut(nstep=400))
        un = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut(nstep=400), couple=False)
        assert un.max_a == n27.max_a, f"max_a drift at Tt4={Tt4}"
        assert un.x_no_relaxed == n27.x_no_relaxed, f"x_no drift at Tt4={Tt4}"
        assert un.Da_entry == n27.Da_entry
        assert un.x_no_e_exit == n27.x_no_e_exit
        assert un.net_factor == 1.0             # uncoupled ⇒ the clock IS the rung-27 clock


def test_frozen_trajectory_matches_the_rung27_path():
    """`_frozen_no_trajectory` must reproduce rung-27's own frozen path (k=0 is Tt9 exactly)."""
    eq, r, a = _run(2200.0)
    far, _Tt3, Tt4, pt4, Tt9, pt9, p9, _phi = a
    comp = _equilibrium_composition(far, Tt4, pt4)
    traj = _frozen_no_trajectory(comp, Tt9, pt9, p9, 400)
    assert len(traj) == 401
    assert traj[0][1] == pt9 and traj[0][2] == Tt9      # entry: no bisection, Tt9 exactly
    assert traj[-1][1] == p9
    assert all(t[3] is comp for t in traj)              # frozen: the SAME composition throughout
    Ts = [t[2] for t in traj]
    assert all(Ts[i] > Ts[i + 1] for i in range(len(Ts) - 1)), "frozen path must cool monotonically"


def test_no_rate_off_recovers_the_clamp():
    """`rate_scale→0` ⇒ NO never moves ⇒ the rung-14/17 clamp number, as rung 27."""
    for Tt4 in (1500.0, 2200.0):
        eq, r, a = _run(Tt4)
        off = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut(rate_scale=1e-30))
        assert off.x_no_relaxed == off.x_no_frozen
        assert off.max_a == off.max_a_frozen
        assert off.relaxed_fraction == 0.0


# ------------------------------------------------- gate 3: the headline is structurally safe ------ #

def test_entry_da_is_path_independent_and_frozen_everywhere():
    """The nozzle-ENTRY state is an input, not a marched quantity ⇒ the coupling cannot reach it.

    THE MARGIN THINS AT THE HOT EDGE, and this gate records it rather than hiding it. Rung 27 quotes a
    "3–9 order" entry margin, measured on ITS band (topping out at 2200 K). Extending to 2400 K the
    entry Da_NO climbs to 2.1e-2 — still frozen, but only ~1.7 orders clear, not 3. The verdict is
    unchanged on the runnable band; the COMFORT of it is not uniform, and rung 27's headline number
    should be read as band-specific."""
    das = []
    for Tt4 in BAND:
        eq, r, a = _run(Tt4)
        n27 = eq.no_freeze_out_nozzle(*a, NOFreezeOut(nstep=400))
        cp = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut(nstep=400))
        assert cp.Da_entry == n27.Da_entry, f"entry Da moved at Tt4={Tt4} — impossible if path-indep"
        assert cp.frozen_from_entry, f"NO not frozen from entry at Tt4={Tt4}"
        das.append(cp.Da_entry)
    assert das[0] < 1e-7, f"lean margin must be enormous, got {das[0]}"
    assert das[-1] < 0.1, f"hot margin must still be clearly frozen, got {das[-1]}"
    assert all(das[i] < das[i + 1] for i in range(len(das) - 1)), \
        f"the entry margin must thin monotonically with Tt4 (the honest trend): {das}"


# ------------------------------- gate 4: the mechanistic correction (BOTH channels are real) ------ #

def test_both_channels_are_real_and_oppose():
    """Rung 27's "can ONLY slow NO further" is one-sided: depletion slows it, heat release SPEEDS it."""
    for Tt4 in (1800.0, 2200.0, 2400.0):
        eq, r, a = _run(Tt4)
        cp = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut())
        assert cp.depletion_factor < 1.0, f"channel 1 must deepen at Tt4={Tt4}"
        assert cp.heat_release_factor > 1.0, f"channel 2 must OPPOSE at Tt4={Tt4}"
        assert cp.T9_pool > cp.T9_frozen, "the relaxing pool must run WARMER (exothermic recombination)"
        assert cp.x_radical_exit_pool < cp.x_radical_entry, "radicals must deplete on the relaxing path"


def test_opposing_channel_grows_monotonically_with_tt4():
    """The certified trend: |ln ch2 / ln ch1| rises across the band, to ~half at the hot edge.

    (The NET's turnaround is deliberately NOT asserted — it rides on how far the pool relaxes, i.e. on
    L/τ_res, exactly as rungs 26/27 disclaim `s_freeze`.)"""
    ratios = []
    for Tt4 in BAND:
        eq, r, a = _run(Tt4)
        ratios.append(eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut()).channel_ratio)
    assert all(ratios[i] < ratios[i + 1] for i in range(len(ratios) - 1)), \
        f"channel_ratio must rise monotonically with Tt4: {ratios}"
    assert ratios[0] < 0.01, f"negligible lean, got {ratios[0]}"
    assert ratios[-1] > 0.3, f"substantial at the hot edge, got {ratios[-1]}"


# --------------------------------------------- gate 5: rung 27's CONCLUSION survives -------------- #

def test_net_is_deeper_frozen_across_the_band():
    """Rung 27's conclusion (the coupling pushes deeper into frozen) holds at every in-band Tt4."""
    for Tt4 in BAND:
        eq, r, a = _run(Tt4)
        cp = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut())
        assert cp.net_factor < 1.0, f"net must deepen at Tt4={Tt4}, got {cp.net_factor}"
        assert cp.deeper_frozen


# ------------------------------ gate 6: the STRUCTURAL win (unbounded vs saturating) -------------- #

def test_depletion_unbounded_heat_release_saturates():
    """Drive the pool rate up: depletion runs away to 0, heat release hits a ceiling and stops.

    This is what makes "deeper frozen" structural rather than incidental — at ANY chemistry faster
    than anchored, depletion wins decisively."""
    eq, r, a = _run(2200.0)
    scales = (1.0, 1e1, 1e2, 1e3, 1e4, 1e6)
    dep, heat, net = [], [], []
    for rs in scales:
        cp = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut(pool_rate_scale=rs))
        dep.append(cp.depletion_factor)
        heat.append(cp.heat_release_factor)
        net.append(cp.net_factor)
    # channel 1 collapses monotonically toward zero — UNBOUNDED
    assert all(dep[i] > dep[i + 1] for i in range(len(dep) - 1)), f"depletion not monotone: {dep}"
    assert dep[-1] < 1e-3, f"depletion should run away, got {dep[-1]}"
    # channel 2 rises but SATURATES — the last decades barely move it
    assert all(heat[i] < heat[i + 1] for i in range(len(heat) - 1)), f"heat not monotone: {heat}"
    assert heat[-1] < 1.5, f"heat release must stay bounded, got {heat[-1]}"
    assert abs(heat[-1] - heat[-2]) < 1e-3, f"heat release must SATURATE, got {heat[-2]}→{heat[-1]}"
    # ⇒ depletion wins by orders at every rate
    assert all(n < 1.0 for n in net)
    assert net[-1] < 1e-3


def test_depletion_wins_at_every_tt4_in_the_limit():
    """No in-band Tt4 lets the (saturating) heat release overturn the (unbounded) depletion."""
    for Tt4 in (1800.0, 2200.0, 2400.0):
        eq, r, a = _run(Tt4)
        cp = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut(pool_rate_scale=1e6))
        assert cp.net_factor < 1e-2, f"depletion must dominate at Tt4={Tt4}, got {cp.net_factor}"


# ------------------------------------------- gate 7: the β repair ------------------------------- #

def test_no_arrives_sub_equilibrium_yet_the_surrogate_still_bounds():
    """Rung 27's premise is FALSE at the entry — and its bound survives anyway, via β<1.

    Rung 27 justified its a≫1 clock with "exhaust NO arrives SUPER-equilibrium". At the ENTRY it does
    NOT for Tt4 ≥ 1800 K (a = 0.31–0.61: NO is BELOW the local ceiling and initially tries to FORM).
    Since freeze-from-entry is decided exactly there, the premise fails where it is needed. What
    actually holds is β<1, which makes τ_surrogate a uniform LOWER bound on τ in BOTH regimes."""
    for Tt4 in BAND:
        eq, r, a = _run(Tt4)
        cp = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut())
        assert cp.a_exit > 1.0, f"NO must be super-eq at the COLD exit (the clamp) at Tt4={Tt4}"
        assert cp.beta_max < 1.0, f"β must stay < 1 at Tt4={Tt4}, got {cp.beta_max}"
        assert cp.tau_ratio_min >= 1.0, f"surrogate must not exceed the exact rate at Tt4={Tt4}"
        assert cp.surrogate_bounds_rate
        if Tt4 >= 1800.0:
            assert cp.sub_equilibrium_entry, f"expected SUB-eq entry at Tt4={Tt4}, a={cp.a_entry}"


def test_exact_tau_ratio_matches_the_closed_form():
    """τ_exact/τ_surr must equal (1+u)²/[(1+u)²−(1−β²)] — the algebra the bound rests on."""
    eq, r, a = _run(2200.0)
    far, _Tt3, Tt4, pt4, Tt9, pt9, p9, _phi = a
    comp = _equilibrium_composition(far, Tt4, pt4)
    zn = eq.zoned_nox(far, r.stations["3"].Tt, Tt4, pt4, 1.0)
    for traj_pt in _frozen_no_trajectory(comp, Tt9, pt9, p9, 400)[::40]:
        _s, p, T, c = traj_pt
        tau_e, beta, a_loc = _tau_no_exact(c, T, p, zn.x_no_mix)
        tau_s = _tau_no_destroy(c, T, p)
        u = beta * a_loc
        expect = (1.0 + u) ** 2 / ((1.0 + u) ** 2 - (1.0 - beta * beta))
        assert math.isclose(tau_e / tau_s, expect, rel_tol=1e-9), \
            f"closed form mismatch at T={T:.0f}: {tau_e/tau_s} vs {expect}"
        assert tau_e >= tau_s                    # the bound, pointwise


def test_beta_margin_is_disclosed_not_comfortable():
    """β RISES with Tt4 and reaches ~half the β=1 threshold — the honest weak point, asserted so it
    cannot silently drift into a violation."""
    betas = []
    for Tt4 in BAND:
        eq, r, a = _run(Tt4)
        betas.append(eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut()).beta_max)
    assert betas[0] < 0.15, f"β small lean, got {betas[0]}"
    assert betas[-1] > 0.3, f"β must be materially larger hot (the disclosed margin), got {betas[-1]}"
    assert max(betas) < 1.0, f"β must never reach 1 on the runnable band, got {max(betas)}"


# ------------------------------------------- gate 8: NO does not actually move -------------------- #

def test_no_barely_moves_despite_the_da_ratios():
    """The Da ratios are the CLOCK's depth, not NO's motion — `relaxed_fraction` stays ≈ 0.

    Slightly NEGATIVE hot (sub-equilibrium entry ⇒ a tiny FORMATION drift). Both are ≈0."""
    for Tt4 in BAND:
        eq, r, a = _run(Tt4)
        cp = eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut())
        assert abs(cp.relaxed_fraction) < 1e-2, \
            f"NO must stay frozen at Tt4={Tt4}, got {cp.relaxed_fraction}"
        assert math.isclose(cp.max_a, cp.max_a_frozen, rel_tol=1e-2), \
            f"the clamp must be unmoved at Tt4={Tt4}"
        assert cp.clamp_fires, f"the clamp must still fire at Tt4={Tt4}"


# ------------------------------------------- gate 9: cycle untouched + guards -------------------- #

def test_cycle_untouched():
    eq, r, a = _run(2200.0)
    far_before, V9_before = r.stations["4"].far, r.V9
    eq.coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut())
    assert r.stations["4"].far == far_before
    assert r.V9 == V9_before


def test_guards():
    eq, r, a = _run(1800.0)
    for kw in (dict(L=0.0), dict(nstep=10), dict(rate_scale=0.0), dict(pool_rate_scale=0.0)):
        try:
            CoupledNOFreezeOut(**kw)
        except AssertionError:
            pass
        else:
            raise AssertionError(f"CoupledNOFreezeOut({kw}) should have been rejected")

    # needs the equilibrium gas
    try:
        Gas.reacting().coupled_no_freeze_out_nozzle(*a, CoupledNOFreezeOut())
    except AssertionError:
        pass
    else:
        raise AssertionError("non-equilibrium gas should have been rejected")

    # rejects an impossible back-pressure
    far, Tt3, Tt4, pt4, Tt9, pt9, p9, phi = a
    try:
        eq.coupled_no_freeze_out_nozzle(far, Tt3, Tt4, pt4, Tt9, pt9, pt9 * 2.0, phi,
                                        CoupledNOFreezeOut())
    except AssertionError:
        pass
    else:
        raise AssertionError("p9 > pt9 should have been rejected")


if __name__ == "__main__":
    for name, fn in sorted(list(globals().items())):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"  ok  {name}")
    print("rung 28: all gates green")
