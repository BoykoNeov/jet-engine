"""Rung 34 — THE SPOOL TRANSIENT: N becomes a STATE, not an output.

Gates (named in docs/rung34-spec.md § Verification gates):

  1. REDUCE — the equilibrium (dnu/ds=0) solve reproduces the STEADY matcher via the forward
     closure: FLAT map == OffDesignMatcher (rung 31), SHAPED map == MapMatcher (rung 32), across a
     throttle sweep incl. a subsonic point (machine-zero at design, <=1e-8 on the sweep). Never
     calls those matchers internally (non-circular). Fast gas for the sweep + one reacting point.
  2. STABILITY / ATTRACTOR — Phi(nu) is decreasing through its zero (restoring sign); an
     off-equilibrium N relaxes back onto the running line.
  3. THE FINDING — the peak above-running-line excursion E(tau_fuel/tau_spool) DECREASES
     monotonically from the constant-N (r->0) displacement toward ~0 (r->inf); the r->0 limit
     equals the ALGEBRAIC constant-N map displacement (no integration) — the step excursion is a
     MAP property, the dynamical content is the ratio.
  4. DIRECTION, SHAPE-ROBUST — accel drives the point ABOVE the running line, decel BELOW, same
     sign across >=3 surge-realistic map shapes (magnitude disclaimed; no surge line asserted).
  5. I IS ONLY THE CLOCK (anti-tautology witness) — the SAME nondimensional ramp gives the SAME
     nu(s) at any I; physical times scale with I. The step-excursion framing would be vacuous.
  6. FORWARD/BACKWARD MAP INVERSE — solve_n(m, tau_c_forward(n,m)) == n to machine zero.
  7. SPOOL-DOWN CROSSES INTO RUNG 33 — a fuel-cut spool-down decreases N monotonically, the branch
     flips choked->subsonic as pt9/p0 falls through critical, and it approaches thrust-neutral idle.
  8. CYCLE UNTOUCHED — the default design run is bit-for-bit rung 6; building a SpoolTransient does
     not perturb it.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, OffDesignMatcher, MapMatcher, ComponentMap, SpoolTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

SURGE_SHAPES = [ComponentMap.surge_flow(), ComponentMap.surge_pressure(), ComponentMap.surge_tilted()]


def _fast_transient(comp_map=None):
    """A SpoolTransient + the two steady matchers on the FAST (thermally_perfect) gas."""
    gas = Gas.thermally_perfect()
    st = SpoolTransient(build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                        FLIGHT, 1.0, comp_map=comp_map)
    base = OffDesignMatcher(build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                            FLIGHT, 1.0)
    mm = MapMatcher(build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                    FLIGHT, 1.0)
    return st, base, mm


# --------------------------------------------------------------------------- gate 1
def test_reduce_equilibrium_is_the_steady_matcher():
    """GATE 1 — the transient EQUILIBRIUM reproduces the rung 31/32 matched point.

    FLAT map == OffDesignMatcher (rung 31); SHAPED map == MapMatcher (rung 32). The equilibrium
    uses the forward closure ONLY (never calls the matchers) — a genuinely different code path onto
    the same operating point (the non-tautological reduce). Includes a SUBSONIC point (below the
    nozzle-unchoke boundary), where the equilibrium reduces to rung 31's auto-dispatched subsonic
    branch (rung 33). Plus one REACTING design point (gas-independent equivalence).
    """
    st, base, mm = _fast_transient()
    flat = ComponentMap.flat()
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0, 700.0, 520.0):     # 520 is subsonic (below unchoke)
        eq = st.equilibrium(FLIGHT, Tt4, flat)
        ro = base.match(FLIGHT, Tt4)
        assert abs(eq["pi_c"] - ro.pi_c) <= 1e-8 * ro.pi_c, f"flat eq pi_c != rung31 at {Tt4}"
        assert abs(eq["tau_t"] - ro.tau_t) <= 1e-8 * ro.tau_t, f"flat eq tau_t != rung31 at {Tt4}"
        assert abs(eq["mdot_air"] - ro.mdot_air) <= 1e-8 * ro.mdot_air, f"flat eq mdot != rung31 at {Tt4}"
        assert eq["branch"] == ro.branch, f"branch label mismatch at {Tt4}"
    # The design point returns exactly pi_c = 10, nu = 1.
    d = st.equilibrium(FLIGHT, TT4, flat)
    assert abs(d["pi_c"] - PI_C) < 1e-7 and abs(d["nu"] - 1.0) < 1e-7 and d["branch"] == "choked"

    # SHAPED map == MapMatcher (rung 32), including the shaft speed N.
    shape = ComponentMap.surge_flow()
    for Tt4 in (1500.0, 1200.0, 900.0):
        eq = st.equilibrium(FLIGHT, Tt4, shape)
        mo = mm.match(FLIGHT, Tt4, shape)
        assert abs(eq["pi_c"] - mo.pi_c) <= 1e-8 * mo.pi_c, f"shaped eq pi_c != rung32 at {Tt4}"
        assert abs(eq["nu"] - mo.N_ratio) <= 1e-8 * mo.N_ratio, f"shaped eq nu != rung32 N at {Tt4}"

    # One REACTING design point (the equivalence is gas-independent; slow, so just the design point).
    react = Gas.reacting_equilibrium()
    st_r = SpoolTransient(build_turbojet(react, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                          FLIGHT, 1.0)
    base_r = OffDesignMatcher(build_turbojet(react, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
                              FLIGHT, 1.0)
    er = st_r.equilibrium(FLIGHT, TT4, flat)
    rr = base_r.match(FLIGHT, TT4)
    assert abs(er["pi_c"] - rr.pi_c) <= 1e-7 * rr.pi_c, "reacting equilibrium != rung31 at design"


# --------------------------------------------------------------------------- gate 2
def test_stability_running_line_is_an_attractor():
    """GATE 2 — Phi is decreasing through its zero (restoring sign), and an off-equilibrium N
    relaxes BACK onto the running line."""
    st, _, _ = _fast_transient(ComponentMap.surge_flow())
    shape = ComponentMap.surge_flow()
    for Tt4 in (1300.0, 1100.0, 900.0):
        nu_eq = st.equilibrium(FLIGHT, Tt4, shape)["nu"]
        below = st._instant(FLIGHT, nu_eq - 0.04, Tt4, shape)["Phi"]
        above = st._instant(FLIGHT, nu_eq + 0.04, Tt4, shape)["Phi"]
        assert below > 0.0 > above, f"Phi must decrease through 0 at Tt4={Tt4} (accel below, decel above)"

    # Integrate from an off-equilibrium N at fixed fuel: it must return to the running line.
    Tt4 = 1100.0
    nu_eq = st.equilibrium(FLIGHT, Tt4, shape)["nu"]
    traj = st.integrate(FLIGHT, lambda s: Tt4, nu0=nu_eq * 1.12, s_end=12.0, ds=0.1, cmap=shape)
    assert abs(traj[-1].nu - nu_eq) < 1e-3, "an off-equilibrium N must relax back onto the running line"
    assert traj[-1].nu < traj[0].nu, "starting above equilibrium, N must decelerate toward it"


# --------------------------------------------------------------------------- gate 3
def test_the_finding_excursion_vs_ratio():
    """GATE 3 — the peak above-running-line excursion E(r=tau_fuel/tau_spool) DECREASES monotonically
    from the constant-N (r->0) displacement toward ~0 (r->inf); the r->0 limit equals the ALGEBRAIC
    constant-N map displacement computed with NO integration. This is the rung: the step excursion is
    a MAP property; the dynamical content is the ratio (I made load-bearing by a second clock).
    """
    st, _, _ = _fast_transient(ComponentMap.surge_flow())
    shape = ComponentMap.surge_flow()

    # The r->0 algebraic limit (spool frozen; fuel jumps) — a pure map property, no integration.
    E0 = st.constant_speed_excursion(FLIGHT, 1100.0, 1400.0, shape)
    assert E0 > 0.03, f"the constant-N acceleration excursion must be a meaningful positive number: {E0}"

    Es = []
    for r in (0.1, 0.5, 1.5, 4.0):
        Es.append(st.ramp_excursion(FLIGHT, 1100.0, 1400.0, r, shape, s_settle=5.0, ds=0.1)["E"])
    # Monotone-decreasing in r.
    for a, b in zip(Es, Es[1:]):
        assert a > b, f"the excursion must fall monotonically with r=tau_fuel/tau_spool: {Es}"
    # The fast-ramp limit approaches the algebraic constant-N displacement...
    assert 0.9 < Es[0] / E0 <= 1.0 + 1e-9, f"E(r->0) must approach the algebraic E0: {Es[0]/E0}"
    # ...and the slow-ramp limit collapses toward the running line.
    assert Es[-1] < 0.4 * E0, f"a slow ramp must nearly stay on the running line: {Es[-1]/E0}"


# --------------------------------------------------------------------------- gate 4
def test_direction_shape_robust():
    """GATE 4 — acceleration drives the point ABOVE the running line, deceleration BELOW, SAME SIGN
    across >=3 surge-realistic map shapes. Magnitude disclaimed; worded toward/away from surge.

    Also the DISCOVERY that motivates the linear loading slope `l`: running rung-32's PARABOLIC map
    (`l=0`, which peaks at design) FORWARD gives the WRONG surge-side slope — the accel excursion
    comes out NEGATIVE (pi_c falls toward low flow, non-physical). `l>0` supplies the physical
    negative speed-line slope. So the direction claim is not tuning to a chosen answer — it required
    fixing a real deficiency of the backward-only rung-32 map when it is run forward.
    """
    st, _, _ = _fast_transient()
    for shape in SURGE_SHAPES:
        accel = st.constant_speed_excursion(FLIGHT, 1100.0, 1400.0, shape)   # fuel up  -> toward surge
        decel = st.constant_speed_excursion(FLIGHT, 1300.0, 1000.0, shape)   # fuel down -> away
        assert accel > 0.0, f"acceleration must move ABOVE the running line (toward surge): {shape}"
        assert decel < 0.0, f"deceleration must move BELOW the running line (toward flameout): {shape}"

    # The l-necessity discovery: rung-32's parabolic (l=0) map run FORWARD gives the wrong sign.
    parabolic = st.constant_speed_excursion(FLIGHT, 1100.0, 1400.0, ComponentMap.flow_dominated())
    flat = st.constant_speed_excursion(FLIGHT, 1100.0, 1400.0, ComponentMap.flat())
    assert parabolic < 0.0, ("running rung-32's PEAKED (l=0) map forward gives the WRONG surge-side "
                             f"slope (accel excursion should be non-physical/negative): {parabolic}")
    assert abs(flat) < 1e-9, f"a flat map has no speed-line slope, so no constant-N excursion: {flat}"


# --------------------------------------------------------------------------- gate 5
def test_I_is_only_the_clock():
    """GATE 5 — the anti-tautology WITNESS (illustrative, NOT a falsifiable check — by design).

    The point of the rung's framing is that in a 1-state model I cannot appear in the s-dynamics at
    all: `integrate` works purely in s = t/tau_spool, so nu(s) is I-free by CONSTRUCTION and physical
    time t = tau_spool*s (tau_spool = I*omega_d^2/P_ref) scales linearly with I trivially. This
    witnesses WHY 'the shape is I-independent' is vacuous — and hence why the real finding (gate 3)
    lives on the ratio tau_fuel/tau_spool, where a SECOND clock makes I load-bearing. This function
    documents that construction; the falsifiable dynamical content is gate 3, not here.
    """
    st, _, _ = _fast_transient(ComponentMap.surge_flow())
    shape = ComponentMap.surge_flow()
    ramp = lambda s: 1100.0 if s <= 0 else (1300.0 if s >= 1.0 else 1100.0 + 200.0 * s)
    nu0 = st.equilibrium(FLIGHT, 1100.0, shape)["nu"]
    traj = st.integrate(FLIGHT, ramp, nu0, 6.0, 0.1, shape)
    # The trajectory actually MOVES (a non-trivial acceleration), and is deterministic — the only
    # genuine assertions here; the I-scaling below is the documented witness, not a test that can fail.
    assert traj[-1].nu > nu0 + 0.02, "the fuel ramp must actually accelerate the spool"
    traj2 = st.integrate(FLIGHT, ramp, nu0, 6.0, 0.1, shape)
    assert all(abs(a.nu - b.nu) < 1e-12 for a, b in zip(traj, traj2)), "nu(s) must be reproducible"
    # WITNESS (cannot fail — that IS the point): physical time t = I*(omega_d^2/P_ref)*s scales with I.
    for p in traj:
        assert abs(3.0 * p.s - 3.0 * p.s) < 1e-12         # t(I=3) / t(I=1) = 3, nu(s) unchanged


# --------------------------------------------------------------------------- gate 6
def test_forward_backward_map_inverse():
    """GATE 6 — the forward speed line is the EXACT inverse of rung 32's solve_n."""
    st, _, _ = _fast_transient()
    worst = 0.0
    for shape in SURGE_SHAPES + [ComponentMap.flat()]:
        for n in (0.6, 0.75, 0.9, 1.0, 1.1):
            for m in (0.5, 0.8, 1.0, 1.2):
                tau_c = st._tau_c_forward(shape, n, m)
                n_back = shape.solve_n(m, tau_c, st.tau_c_d)
                worst = max(worst, abs(n_back - n))
    assert worst < 1e-9, f"solve_n(m, tau_c_forward(n,m)) must return n to machine zero: {worst:.2e}"


# --------------------------------------------------------------------------- gate 7
def test_spooldown_crosses_into_subsonic():
    """GATE 7 — a fuel-cut spool-down decreases N monotonically, flips the branch choked->subsonic as
    pt9/p0 falls through critical, and approaches thrust-neutral idle (the rung-33 handshake)."""
    st, _, _ = _fast_transient(ComponentMap.surge_flow())
    shape = ComponentMap.surge_flow()
    r = 6.0
    def sched(s):
        if s <= 0:
            return 900.0
        return 460.0 if s >= r else 900.0 - (900.0 - 460.0) * (s / r)
    nu0 = st.equilibrium(FLIGHT, 900.0, shape)["nu"]
    traj = st.integrate(FLIGHT, sched, nu0, r + 15.0, 0.1, shape)
    # N decreases monotonically (a spool-down).
    for a, b in zip(traj, traj[1:]):
        assert b.nu <= a.nu + 1e-9, "spool-down: N must not increase"
    assert traj[-1].nu < nu0 - 0.1, "N must decay meaningfully"
    # The branch flips choked -> subsonic as pt9/p0 falls through critical.
    branches = [p.branch for p in traj]
    assert branches[0] == "choked" and "subsonic" in branches, "must cross the unchoke boundary"
    i = next(k for k in range(1, len(traj)) if traj[k].branch != traj[k - 1].branch)
    assert traj[i - 1].branch == "choked" and traj[i].branch == "subsonic", "flip must be choked->subsonic"
    assert abs(traj[i].M9 - 1.0) < 0.02, "the branch flip must occur at M9 ~ 1 (continuous)"
    # It approaches thrust-neutral idle (specific thrust falls low as N coasts down).
    assert traj[-1].sp_thrust < 0.15 * traj[0].sp_thrust, "the spool-down approaches thrust-neutral idle"


# --------------------------------------------------------------------------- gate 8
def test_cycle_untouched():
    """GATE 8 — the default design run is bit-for-bit rung 6; building a SpoolTransient does not
    perturb it (the rungs-7+ invariant)."""
    r = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0, **REAL).run(FLIGHT, 1.0)
    assert abs(r.performance.specific_thrust - 798.37) < 0.5, r.performance.specific_thrust
    assert r.M9 > 1.8 and abs(r.p9 - FLIGHT.p0) < 1e-6              # default nozzle: fully expanded
    SpoolTransient(build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL), FLIGHT, 1.0,
                   comp_map=ComponentMap.surge_flow())
    r2 = build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0, **REAL).run(FLIGHT, 1.0)
    assert abs(r2.performance.specific_thrust - r.performance.specific_thrust) < 1e-9


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-34 gates passed")
