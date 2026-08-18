"""SLICE R step 4 — THE ORACLE for rungs 40 + 44 (`TwoSpoolTransient`), over BOTH suites' grids.

Step 1's smoke touched every path ONCE (1 182 values, nine sections). This dump runs the grids the
two Python suites actually sweep, because step 1 and step 3 both measured that the rung gates
CANNOT see several things the port could get wrong:

  * `best`'s strict `<` versus `<=` inside the Newton — INVISIBLE to all 1 174 smoke values,
    registered at step 1 as *"step 4's larger reacting grid is where it could be"*. It is measured
    here by an `eq_ties` counter on the Rust side and reported either way;
  * the high wall's literal `2.5` arm — visible in the CENSUS ONLY (dropping it moves no value at
    all, only `illinois_evals`), so the census is emitted PER SECTION on the full grid;
  * `min_phi_lp`/`min_phi_hp` and `s_lp`/`s_hp` — emitted by `phi_excursion` and asserted by
    NOTHING in either Python suite, so they are carried on every excursion cell here;
  * the march-in ladder's spelling and the four dead arms — gated against zero at scale.

**THE TWO SUITES RUN TWO DIFFERENT CPG GASES, AND THE DIFFERENCE IS ONE CONSTANT.**
`test_rung40.py` hard-codes `R_c = 286.9`; `test_rung44.py` writes
`R_c = (gamma_c-1)/gamma_c*cp_c = 286.8571428571428`. Both are dumped, under `cpg40` and `cpg44`,
because reading one suite's constant off its neighbour is exactly the defect this step found in
`rust/tests/rung44.rs`. `R_t` IS derived in both and agrees.

THE THREE ARMS:

  main    the CPG grids of both suites — sections A…L. Cheap (seconds on PyPy).
  equil   the REACTING-gas cells — sections P…S. `equilibrium` costs ~10 s per call on PyPy, so
          this arm is separated rather than folded in.
  cpython main + equil in ONE file, for the interpreter arm.

**WHY THE REACTING ARM IS A BIT-EQUALITY ARM AND NOT § 9 DECISION 1's FRAGILE SET.** § 5.15
prediction 1 registered the reacting `equilibrium` keys as the slice's one genuine exposure: probe
4 measured the exit CLASSIFICATION flipping in 5 of 12 cells between CPython and PyPy, because the
`1e-12` bar is ABSOLUTE and sits below the gas sub-solve's own ~1e-10 noise, so a last-bit
difference in `exp`/`log` re-rolls it. Step 4 spiked those exact 12 cells against the shipped Rust
BEFORE writing this file: all 12 agree with PyPy on the exit kind, the pass count and both
converged speeds. So the arm ships at bit-equality and Option B is not invoked. The CPython arm is
where the flips live, and it is read as a DETECTOR with a measured sensitivity, not as coverage.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_two_spool_transient.py main    rust/oracle/two_spool_transient_pypy.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_two_spool_transient.py equil   rust/oracle/two_spool_transient_eq_pypy.tsv
    C:\\Python314\\python.exe  rust/oracle/dump_two_spool_transient.py cpython rust/oracle/two_spool_transient_cpython.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import engine as E                                          # noqa: E402
from turbojet.engine import (FlightCondition, build_turbojet,             # noqa: E402
                             build_two_spool_turbojet, ComponentMap,
                             SpoolTransient, TwoSpoolTransient)
from turbojet.gas import Gas                                              # noqa: E402

ARM = sys.argv[1] if len(sys.argv) > 1 else "main"
OUT = sys.argv[2] if len(sys.argv) > 2 else None
assert ARM in ("main", "equil", "cpython"), ARM

ROWS = []


def put(key, value):
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    n = int(n)
    ROWS.append((key, n, str(n)))


# ================================================================= the instruments
# Identical in intent to `dump_slice_r_smoke.py`'s, and identical in one further respect: every
# NUMBER comes out of the SHIPPED body. The two wrappers that recompute anything (the high wall's
# arm and the eigenvalue arm) recompute a scalar the body also computes, feed no dumped value, and
# say so at their definition.
CENSUS = {}


def reset_census():
    CENSUS.clear()
    CENSUS.update(close_calls=0, close_bracket_fails=0, close_nonreal_propagated=0,
                  powers_calls=0, match_calls=0, instant_calls=0, integrate_calls=0,
                  hi_wall_literal=0, hi_wall_map=0, eig_real=0, eig_complex=0,
                  illinois_calls=0, illinois_evals=0, illinois_exhausted=0)
    MATCH_TT4.clear()
    EQ_POWERS.clear()


MATCH_TT4 = []        # every Tt4 handed to `match`, in call order
LAST_TRAJ = []        # the points of the most recent `integrate` call
EQ_POWERS = []        # `_powers` calls per `equilibrium` call, in call order

reset_census()

_ILL = E._illinois


def _ill(f, a, b, fa, fb, tol=1e-10, maxit=100):
    """The shipped Illinois, counted. `illinois_exhausted` is § 5.15 prediction 7's object: 0 at
    THIS call site against slice Q's 103 of 109 at `_plenum_pt4_at`. The two populations are
    reported side by side in the Rust, each with its grid, and never summed."""
    n = [0]

    def counting(x):
        v = f(x)
        n[0] += 1
        CENSUS["illinois_evals"] += 1
        return v
    CENSUS["illinois_calls"] += 1
    try:
        return _ILL(counting, a, b, fa, fb, tol, maxit)
    finally:
        if n[0] >= maxit:
            CENSUS["illinois_exhausted"] += 1


E._illinois = _ill

_CLOSE = TwoSpoolTransient._close


def _close(self, nu_lp, nu_hp, Tt4, Tt2, pt2):
    """INSTRUMENT NOTE: recomputes `n_lp` — three operations the body also performs — purely to
    classify which arm of `min(2.5, phi_max*n_L)` binds. It feeds NO dumped value. Step 1 measured
    that dropping the literal arm moves NO value anywhere and only `illinois_evals`, so this
    census is the only thing in the project that can see it."""
    CENSUS["close_calls"] += 1
    n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
    if 2.5 <= self.map_lp.phi_max() * n_lp:
        CENSUS["hi_wall_literal"] += 1
    else:
        CENSUS["hi_wall_map"] += 1
    try:
        return _CLOSE(self, nu_lp, nu_hp, Tt4, Tt2, pt2)
    except AssertionError as ex:
        if "does not bracket" in str(ex):
            CENSUS["close_bracket_fails"] += 1
        elif "off-map compressor trial" in str(ex):
            CENSUS["close_nonreal_propagated"] += 1
        raise


TwoSpoolTransient._close = _close

_POWERS = TwoSpoolTransient._powers


def _powers(self, c, flight, nu_lp, nu_hp, Tt4):
    CENSUS["powers_calls"] += 1
    return _POWERS(self, c, flight, nu_lp, nu_hp, Tt4)


TwoSpoolTransient._powers = _powers

_INSTANT = TwoSpoolTransient._instant


def _instant(self, flight, nu_lp, nu_hp, Tt4):
    CENSUS["instant_calls"] += 1
    return _INSTANT(self, flight, nu_lp, nu_hp, Tt4)


TwoSpoolTransient._instant = _instant

_MATCH = TwoSpoolTransient.match


def _match(self, flight, Tt4, *a, **k):
    """Records every `Tt4` the INHERITED rung-39 match is asked for — which is how the `steady`
    memo's MISS sequence is recovered without reaching inside `_ramp_march`'s closure."""
    CENSUS["match_calls"] += 1
    MATCH_TT4.append(float(Tt4))
    return _MATCH(self, flight, Tt4, *a, **k)


TwoSpoolTransient.match = _match

_INTEGRATE = TwoSpoolTransient.integrate


def _integrate(self, flight, schedule, nu0, s_end, ds):
    CENSUS["integrate_calls"] += 1
    out = _INTEGRATE(self, flight, schedule, nu0, s_end, ds)
    LAST_TRAJ[:] = out
    return out


TwoSpoolTransient.integrate = _integrate

_EIG = TwoSpoolTransient.__dict__["eigenvalues"].__func__


def _eigenvalues(J):
    """INSTRUMENT NOTE: recomputes the discriminant (four operations) to classify the arm, then
    returns the SHIPPED body's result. § 5.15 prediction 9 is gated on gate 5's OWN grid inside
    `rung40.rs`; this census belongs to THIS grid and the two are never merged."""
    tr = J[0][0] + J[1][1]
    det = J[0][0] * J[1][1] - J[0][1] * J[1][0]
    if tr * tr - 4.0 * det >= 0.0:
        CENSUS["eig_real"] += 1
    else:
        CENSUS["eig_complex"] += 1
    return _EIG(J)


TwoSpoolTransient.eigenvalues = staticmethod(_eigenvalues)

_EQ = TwoSpoolTransient.equilibrium


def _equilibrium(self, flight, Tt4, start=None):
    """Records `_powers` calls per equilibrium — which makes the EXIT KIND and the PASS COUNT
    recoverable without reaching into the loop. Each Newton pass costs three `_powers` calls
    (residual + two Jacobian columns) and the primary return costs ONE more, so

        primary at pass k  =>  3k + 1 calls        noise (all 80 passes)  =>  240 calls

    and `3k + 1 = 240` has no integer solution, so the classification is unambiguous."""
    n0 = CENSUS["powers_calls"]
    try:
        return _EQ(self, flight, Tt4, start)
    finally:
        EQ_POWERS.append(CENSUS["powers_calls"] - n0)


TwoSpoolTransient.equilibrium = _equilibrium


def eq_kind_and_iters(n_powers):
    """(exit kind, pass count) from the `_powers` call count — see `_equilibrium`."""
    if n_powers == 3 * TwoSpoolTransient._EQ_MAX:
        return 1, TwoSpoolTransient._EQ_MAX          # 1 == the NOISE-FLOOR exit
    assert n_powers % 3 == 1, ("a primary exit costs 3k+1 _powers calls", n_powers)
    return 0, (n_powers - 1) // 3


def emit_census(prefix):
    for k in sorted(CENSUS):
        putd(f"census/{prefix}/{k}", CENSUS[k])
    reset_census()


# ================================================================= the dict emitters
# THE KEY LISTS COME FROM PYTHON'S OWN DICTS, never from the Rust struct: a field forgotten in the
# port must show up as a MISSING comparison, and it only can if the dump enumerates the source.
CLOSE_KEYS = 21
INSTANT_KEYS = 44


def put_close(prefix, c):
    assert len(c) == CLOSE_KEYS, (len(c), sorted(c))
    assert isinstance(c["wgas"], Gas)
    for k in sorted(c):
        if k != "wgas":
            put(f"{prefix}/{k}", c[k])


def put_instant(prefix, i):
    assert len(i) == INSTANT_KEYS, (len(i), sorted(i))
    assert isinstance(i["wgas"], Gas)
    for k in sorted(i):
        if k == "wgas":
            continue
        if k == "branch":
            putd(f"{prefix}/branch_choked", 1 if i[k] == "choked" else 0)
        else:
            put(f"{prefix}/{k}", i[k])


POINT_FIELDS = ("s", "nu_lp", "nu_hp", "Tt4", "slip", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
                "mdot_air", "f", "Phi_lp", "Phi_hp", "sp_thrust")
EXC_FIELDS = ("ext_lp", "ext_hp", "s_lp", "s_hp", "min_phi_lp", "min_phi_hp", "ratio")
SM_FIELDS = ("margin_min_lp", "margin_min_hp", "steady_min_lp", "steady_min_hp",
             "phi_surge_lp", "phi_surge_hp")


def put_exc(prefix, ex):
    """All EIGHT keys, including the four NO Python gate reads (`s_lp`/`s_hp`,
    `min_phi_lp`/`min_phi_hp`) — step 3 measured corrupting them invisible to all 17 rung tests."""
    assert len(ex) == 8, sorted(ex)
    for k in EXC_FIELDS:
        put(f"{prefix}/{k}", ex[k])
    putd(f"{prefix}/npts", ex["npts"])


def put_sm(prefix, sm):
    assert len(sm) == 9, sorted(sm)
    for k in SM_FIELDS:
        put(f"{prefix}/{k}", sm[k])
    putd(f"{prefix}/crossed_lp", 1 if sm["crossed_lp"] else 0)
    putd(f"{prefix}/crossed_hp", 1 if sm["crossed_hp"] else 0)
    putd(f"{prefix}/npts", sm["npts"])


# ======================================================================== the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

FLAT = ComponentMap.flat()
LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
STEEP = ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)
PRESS_LP = ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0)
PRESS_HP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)

# `test_rung40.py`'s seven disclosed pairs, in ITS order. `test_rung44.py`'s five are the SUBSET
# named in RUNG44_SHAPES — the same ComponentMap objects, so one table serves both suites.
SHAPES = (
    ("flat",       FLAT,      FLAT),
    ("flow_press", LP_SHAPED, HP_SHAPED),
    ("press_flow", PRESS_LP,  PRESS_HP),
    ("tilted",     TILTED,    TILTED),
    ("steep",      STEEP,     STEEP),
    ("lp_only",    LP_SHAPED, FLAT),
    ("hp_only",    FLAT,      HP_SHAPED),
)
RUNG44_SHAPES = ("flow_press", "press_flow", "tilted", "steep", "hp_only")


def cpg40():
    """`test_rung40.py`'s CPG dual gas — `R_c` HARD-CODED at 286.9."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def cpg44():
    """`test_rung44.py`'s CPG dual gas — `R_c` DERIVED, so 286.8571428571428, not 286.9."""
    gc, cc, gt, ct = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cc, R_c=(gc - 1.0) / gc * cc,
               gamma_t=gt, cp_t=ct, R_t=(gt - 1.0) / gt * ct, hPR=42.8e6)


def design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def tt(gas, ml, mh, rho=1.0):
    return TwoSpoolTransient(design(gas), FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=rho)


def shape(name):
    for n, ml, mh in SHAPES:
        if n == name:
            return ml, mh
    raise KeyError(name)


RHO_EIG = (0.05, 0.2, 1.0, 5.0, 20.0, 100.0)      # gate 5's spot-check sweep


def main_arm():
    # ------------------------------------------------------- A: the constants, and BOTH gases
    # The one difference between the two suites' cold sections, emitted so a port that reads one
    # off the other fails HERE rather than passing every sign gate in silence.
    put("A/cpg40/R_c", cpg40().R_c)
    put("A/cpg44/R_c", cpg44().R_c)
    put("A/cpg40/R_t", cpg40().R_t)
    put("A/cpg44/R_t", cpg44().R_t)
    t40 = tt(cpg40(), LP_SHAPED, HP_SHAPED)
    t44 = tt(cpg44(), LP_SHAPED, HP_SHAPED)
    for nm, t in (("cpg40", t40), ("cpg44", t44)):
        Tt2, pt2, V0 = t._inlet(FLIGHT)
        put(f"A/{nm}/tt2", Tt2)
        put(f"A/{nm}/pt2", pt2)
        put(f"A/{nm}/v0", V0)
    emit_census("A")

    # ------------------------------------------------------- B: `_close`, driven DIRECTLY
    # Seven shape pairs x two speed pairs x two throttles = 28 closures, 20 float keys each. The
    # HIGH WALL's contested `min` gets its population here: step 1 saw only 2/4 and 6/9.
    Tt2, pt2, _ = t40._inlet(FLIGHT)
    for name, ml, mh in SHAPES:
        t = tt(cpg40(), ml, mh)
        for inu, (a, b) in enumerate(((1.0, 1.0), (0.92, 0.96))):
            for Tt4 in (1500.0, 1200.0):
                put_close(f"B/{name}/{inu}/{Tt4:.0f}", t._close(a, b, Tt4, Tt2, pt2))
    emit_census("B")

    # ------------------------------------------------------- C: `_instant` at the MATCHED point
    # Gate 5 + gate 6's grid: every shape pair at three throttles, at the speeds rung 39's
    # INHERITED `match` returns — so this section also witnesses that edge at scale.
    for name, ml, mh in SHAPES:
        t = tt(cpg40(), ml, mh)
        for Tt4 in (1500.0, 1200.0, 950.0):
            od = t.match(FLIGHT, Tt4)
            put(f"C/{name}/{Tt4:.0f}/nu_lp", od.N_lp_ratio)
            put(f"C/{name}/{Tt4:.0f}/nu_hp", od.N_hp_ratio)
            put(f"C/{name}/{Tt4:.0f}/slip", od.slip)
            put(f"C/{name}/{Tt4:.0f}/phi_lp", od.phi_lp)
            put(f"C/{name}/{Tt4:.0f}/phi_hp", od.phi_hp)
            put_instant(f"C/{name}/{Tt4:.0f}/i", t._instant(FLIGHT, od.N_lp_ratio,
                                                            od.N_hp_ratio, Tt4))
    emit_census("C")

    # ------------------------------------------------------- D: `equilibrium` on CPG
    # Gate 1's CPG sweep, widened to all seven pairs. The exit kind and the pass count are
    # DISCRETE keys: on CPG the primary return always fires (the residual's noise floor is ~1e-14
    # against the ABSOLUTE 1e-12 bar), and that is asserted rather than assumed by dumping it.
    for name, ml, mh in SHAPES:
        t = tt(cpg40(), ml, mh)
        for Tt4 in (1500.0, 1300.0, 1200.0):
            eq = t.equilibrium(FLIGHT, Tt4)
            put_instant(f"D/{name}/{Tt4:.0f}", eq)
            kind, iters = eq_kind_and_iters(EQ_POWERS[-1])
            putd(f"D/{name}/{Tt4:.0f}/exit_noise", kind)
            putd(f"D/{name}/{Tt4:.0f}/passes", iters)
            putd(f"D/{name}/{Tt4:.0f}/powers_calls", EQ_POWERS[-1])
    # The ONE shipped signature branch nothing above takes: an explicit START.
    eq = t40.equilibrium(FLIGHT, 1200.0, start=(0.90, 0.95))
    put_instant("D/start", eq)
    kind, iters = eq_kind_and_iters(EQ_POWERS[-1])
    putd("D/start/exit_noise", kind)
    putd("D/start/passes", iters)
    emit_census("D")

    # ------------------------------------------------------- E: `lead_threshold`
    # Gate 4's whole grid — the flat+CPG identity at four throttles, the two breaking channels
    # (gas curve vs map), and the REFUTATION that the map's shift direction is shape-dependent —
    # plus every pair at three throttles on the DEFAULT `d`, which gate 4 never exercises. The
    # default is `d = 5.0` (`engine.py:3644`) and gate 4 (a)/(b) pass 25.0 explicitly: step 2
    # found that non-uniformity by reading the source, not the call.
    t_flat = tt(cpg40(), FLAT, FLAT)
    for Tt4 in (900.0, 1100.0, 1300.0, 1500.0):
        put(f"E/identity/{Tt4:.0f}", t_flat.lead_threshold(FLIGHT, Tt4, d=25.0))
    put("E/channel/gas", tt(Gas.thermally_perfect(), FLAT, FLAT)
        .lead_threshold(FLIGHT, 1100.0, d=25.0))
    put("E/channel/map", t40.lead_threshold(FLIGHT, 1100.0, d=25.0))
    put("E/refute/lp_only", tt(cpg40(), LP_SHAPED, FLAT).lead_threshold(FLIGHT, 1100.0))
    put("E/refute/hp_only", tt(cpg40(), FLAT, HP_SHAPED).lead_threshold(FLIGHT, 1100.0))
    for name, ml, mh in SHAPES:
        t = tt(cpg40(), ml, mh)
        for Tt4 in (1500.0, 1300.0, 1100.0):
            put(f"E/default_d/{name}/{Tt4:.0f}", t.lead_threshold(FLIGHT, Tt4))
    emit_census("E")

    # ------------------------------------------------------- F: the 2x2 and its two arms
    # Gate 5's grid (7 pairs x 3 throttles) with the eigenvalues at gate 5's own six `rho`, plus
    # gate 6's band, its discriminant probe and its damping. `rho` enters the way the GATE does it
    # — `J` built at `rho = 1` and the LP row divided afterwards — never by mutating the object
    # between calls, which is how the Rust's `jacobian_at_rho` spells the same thing.
    for name, ml, mh in SHAPES:
        t = tt(cpg40(), ml, mh)
        for Tt4 in (1500.0, 1200.0, 950.0):
            od = t.match(FLIGHT, Tt4)
            nu = (od.N_lp_ratio, od.N_hp_ratio)
            t.rho = 1.0
            J = t.jacobian(FLIGHT, Tt4, nu=nu)
            p = f"F/{name}/{Tt4:.0f}"
            for r in range(2):
                for cc in range(2):
                    put(f"{p}/J/{r}{cc}", J[r][cc])
            put(f"{p}/bc", J[0][1] * J[1][0])
            for ir, rho in enumerate(RHO_EIG):
                Jr = [[J[0][0] / rho, J[0][1] / rho], [J[1][0], J[1][1]]]
                lo, hi = t.eigenvalues(Jr)
                put(f"{p}/eig/{ir}/lo", lo)
                put(f"{p}/eig/{ir}/hi", hi)
            band = t.oscillatory_band(FLIGHT, Tt4, nu=nu)
            putd(f"{p}/band_is_none", 1 if band is None else 0)
            if band is not None:
                put(f"{p}/band_lo", band[0])
                put(f"{p}/band_hi", band[1])
                # Gate 6's own discriminant probe, at the three points it evaluates.
                a, b, c, d = J[0][0], J[0][1], J[1][0], J[1][1]
                mid = (band[0] * band[1]) ** 0.5
                for tag, rr in (("mid", mid), ("lo2", 0.5 * band[0]), ("hi2", 2.0 * band[1])):
                    put(f"{p}/disc/{tag}", (a / rr - d) ** 2 + 4.0 * b * c / rr)
            put(f"{p}/damping", t.damping_ratio_max(FLIGHT, Tt4, nu=nu))
    emit_census("F")

    # ------------------------------------------------------- G: the march, EVERY point
    # Three cells, all points, all fields. Two run at `s_end = 1.2, ds = 0.05` — gate 7's pair,
    # and the ONLY one of the four in use where `int(round(s_end/ds))` is not exact
    # (23.99999999999999644729 -> 24; a truncation gives 23, a whole missing step). The third
    # runs rung 44's own default ramp on rung 44's own gas, 151 points, which is what makes the
    # trajectory length a gate at the scale the excursions are actually taken over.
    for tag, gas, sname, Tt4_lo, dT, r_ramp, s_end, ds in (
            ("g7", cpg40(), "flow_press", 1100.0, 50.0, 0.5, 1.2, 0.05),
            ("steep", cpg40(), "steep", 1100.0, 50.0, 0.5, 1.2, 0.05),
            ("r44", cpg44(), "hp_only", 1000.0, 400.0, 0.5, 3.0, 0.02)):
        ml, mh = shape(sname)
        t = tt(gas, ml, mh)
        od_lo = t.match(FLIGHT, Tt4_lo)
        nu0 = (od_lo.N_lp_ratio, od_lo.N_hp_ratio)

        def sched(x, _lo=Tt4_lo, _d=dT, _r=r_ramp):
            return _lo + _d * min(1.0, x / _r)

        pts = t.integrate(FLIGHT, sched, nu0, s_end, ds)
        putd(f"G/{tag}/npts", len(pts))
        for ip, p in enumerate(pts):
            for k in POINT_FIELDS:
                put(f"G/{tag}/{ip}/{k}", getattr(p, k))
        emit_census(f"G/{tag}")

    # ------------------------------------------------------- H: `slip_excursion` + gate 7
    # Gate 7 REPRODUCED, all 18 bisection steps — including `elo * ehi < 0.0`, the bracket check
    # that runs FOUR LINES AHEAD of the headline 0.2 margin and is what a truncated step count
    # actually breaks (step 1 measured that; step 2 preserved the order and said so at the site).
    t = tt(cpg40(), LP_SHAPED, HP_SHAPED)
    Tt4_lo, dT = 1100.0, 50.0
    sc = t.lead_threshold(FLIGHT, Tt4_lo)
    put("H/sigma_crit", sc)

    def exc(rho):
        t.rho = rho
        return t.slip_excursion(FLIGHT, Tt4_lo, dT, s_end=1.2, ds=0.05)

    lo, hi = 0.6 * sc, 1.6 * sc
    elo, ehi = exc(lo), exc(hi)
    put("H/bisect/elo", elo)
    put("H/bisect/ehi", ehi)
    for ib in range(18):
        mid = 0.5 * (lo + hi)
        em = exc(mid)
        put(f"H/bisect/{ib}/mid", mid)
        put(f"H/bisect/{ib}/exc", em)
        if em * elo > 0.0:
            lo = mid
        else:
            hi = mid
    put("H/bisect/rho_star", 0.5 * (lo + hi))
    t.rho = 1.0
    # THE TWO RUNNING-LINE REFERENCES, POINTWISE, on a SATURATING and a NON-saturating ramp. At
    # `r_ramp = 0.5` the extremum lands exactly where the ramp saturates (`u == 1`), so the linear
    # reference IS the endpoint match bit-for-bit and unifying the two moves NOTHING — step 1
    # measured that and added an `r_ramp = 3.0` cell, where the two differ by 2.4 %.
    for rr in (0.5, 3.0):
        put(f"H/ref/{rr}/slip_excursion", t.slip_excursion(FLIGHT, Tt4_lo, dT, r_ramp=rr,
                                                           s_end=1.2, ds=0.05))
        # The trajectory is re-marched EXPLICITLY rather than read out of `LAST_TRAJ`. Reading the
        # side channel would make this side do one march where the Rust does two, and the census —
        # which is what catches the high wall's arm — would disagree for a harness reason.
        od_lo = t.match(FLIGHT, Tt4_lo)
        od_hi = t.match(FLIGHT, Tt4_lo + dT)

        def rsched(x, _lo=Tt4_lo, _d=dT, _r=rr):
            return _lo + _d * min(1.0, x / _r)

        pts = t.integrate(FLIGHT, rsched, (od_lo.N_lp_ratio, od_lo.N_hp_ratio), 1.2, 0.05)
        for ip, p in enumerate(pts):
            u = (p.Tt4 - Tt4_lo) / dT
            linear = od_lo.slip + u * (od_hi.slip - od_lo.slip)
            instant = t.match(FLIGHT, p.Tt4).slip
            put(f"H/ref/{rr}/{ip}/err_linear", p.slip - linear)
            put(f"H/ref/{rr}/{ip}/err_instant", p.slip - instant)
    for rho in (0.5, 2.0):
        t.rho = rho
        put(f"H/rho/{rho}/slip_excursion", t.slip_excursion(FLIGHT, Tt4_lo, dT,
                                                            s_end=1.2, ds=0.05))
    t.rho = 1.0
    emit_census("H")

    # ------------------------------------------------------- I: `phi_excursion`, rung 44's grid
    # Gate 3 (five pairs, accel AND decel), gate 4 (a) (five `rho`), gate 4 (b) (six ramp rates at
    # `s_end = 6.0`). All EIGHT returned keys every time, including the four no Python gate reads.
    for name in RUNG44_SHAPES:
        ml, mh = shape(name)
        t = tt(cpg44(), ml, mh)
        put_exc(f"I/acc/{name}", t.phi_excursion(FLIGHT, 1000.0, 400.0))
        put_exc(f"I/dec/{name}", t.phi_excursion(FLIGHT, 1400.0, -400.0))
        band = t.oscillatory_band(FLIGHT, 1200.0)
        putd(f"I/band_is_none/{name}", 1 if band is None else 0)
        put(f"I/damping/{name}", t.damping_ratio_max(FLIGHT, 1200.0))
    emit_census("I/shapes")

    for rho in (0.2, 0.5, 1.0, 2.0, 5.0):
        t = tt(cpg44(), LP_SHAPED, HP_SHAPED, rho=rho)
        put_exc(f"I/rho/{rho}", t.phi_excursion(FLIGHT, 1000.0, 400.0))
    emit_census("I/rho")

    t = tt(cpg44(), LP_SHAPED, HP_SHAPED)
    for r in (5.0, 2.0, 1.0, 0.5, 0.3, 0.1):
        put_exc(f"I/ramp/{r}", t.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=r, s_end=6.0))
    emit_census("I/ramp")

    # ------------------------------------------------------- J: the memo's KEY SEQUENCE
    # The equivalence relation `round(Tt4, 3)` is the one thing NO value key can see: probe 1
    # measured the single collision that exists (1399.9999999999984 and 1400.0 share the key
    # 1400.0) moving 0 reported values, and confirmed it FIRES inside the measured set. Both
    # keying schemes are built off the SAME trajectory, so this compares the relation and nothing
    # else — and the shipped miss sequence, recovered from the `match` calls, is ASSERTED to be
    # the rounded one, which is what makes the counts a measurement of the memo rather than of a
    # re-implementation of it.
    for tag, r_ramp, s_end in (("collide", 5.0, 6.0), ("default", 0.5, 3.0)):
        ex = t.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=r_ramp, s_end=s_end)
        put_exc(f"J/{tag}", ex)
        misses = MATCH_TT4[1:]          # [0] is `_ramp_march`'s own start-point match
        putd(f"J/{tag}/match_calls", len(MATCH_TT4))
        putd(f"J/{tag}/steady_misses", len(misses))
        putd(f"J/{tag}/steady_calls", 2 * ex["npts"])
        for ik, x in enumerate(misses):
            put(f"J/{tag}/key/{ik}", round(x, 3))
        seen_r, seen_x, keys_r, keys_x = set(), set(), [], []
        for p in LAST_TRAJ:
            kr = round(p.Tt4, 3)
            if kr not in seen_r:
                seen_r.add(kr)
                keys_r.append(kr)
            if p.Tt4 not in seen_x:
                seen_x.add(p.Tt4)
                keys_x.append(p.Tt4)
        assert keys_r == [round(x, 3) for x in misses], "recovered misses are not the rounded keys"
        putd(f"J/{tag}/keys_rounded", len(keys_r))
        putd(f"J/{tag}/keys_exact", len(keys_x))
        putd(f"J/{tag}/collisions", len(keys_x) - len(keys_r))
        emit_census(f"J/{tag}")

    # ------------------------------------------------------- K: `transient_surge_margin`
    # Gate 5's accel/decel at `r_ramp = 0.3` plus the five pairs at the default ramp. The unarmed
    # refusal is a raise and cannot be dumped; it is gated in `rung44.rs` instead.
    for name in RUNG44_SHAPES:
        ml, mh = shape(name)
        t = tt(cpg44(), ml.with_phi_surge(0.86), mh.with_phi_surge(0.90))
        put_sm(f"K/def/{name}", t.transient_surge_margin(FLIGHT, 1000.0, 400.0))
    t = tt(cpg44(), LP_SHAPED.with_phi_surge(0.86), HP_SHAPED.with_phi_surge(0.90))
    put_sm("K/acc", t.transient_surge_margin(FLIGHT, 1000.0, 400.0, r_ramp=0.3))
    put_sm("K/dec", t.transient_surge_margin(FLIGHT, 1400.0, -400.0, r_ramp=0.3))
    emit_census("K")

    # ------------------------------------------------------- L: the `lp_disabled` REDUCE
    # Gate 2, at its OWN gas: `_single_design` is built on `Gas.reacting_equilibrium()`, not on
    # either CPG one. Bit-for-bit against a bare rung-34 `SpoolTransient`, asserted on BOTH sides
    # here so the dump itself would fail if the dispatch ever stopped being exact.
    single = build_turbojet(Gas.reacting_equilibrium(), PI_HPC, TT4, FLIGHT.p0,
                            pi_d=REAL["pi_d"], eta_c=REAL["eta_hpc"], eta_b=REAL["eta_b"],
                            pi_b=REAL["pi_b"], eta_t=REAL["eta_hpt"], eta_m=REAL["eta_m"],
                            pi_n=REAL["pi_n"], nozzle_convergent=True)
    deg = TwoSpoolTransient(single, FLIGHT, 1.0, map_hp=HP_SHAPED, lp_disabled=True)
    ref = SpoolTransient(single, FLIGHT, 1.0, comp_map=HP_SHAPED)
    for Tt4 in (1500.0, 1200.0):
        a = deg._degenerate.equilibrium(FLIGHT, Tt4)
        b = ref.equilibrium(FLIGHT, Tt4)
        for k in ("nu", "pi_c", "tau_c", "tau_t", "mdot_air", "f", "Phi", "sp_thrust"):
            assert a[k] == b[k], (Tt4, k, a[k], b[k])
            put(f"L/{Tt4:.0f}/{k}", a[k])
    emit_census("L")


def equil_arm():
    """The REACTING-gas cells. ~10 s per `equilibrium` call on PyPy, which is why they are here
    and not folded into `main`."""
    gas = Gas.reacting_equilibrium()
    d = design(gas)

    # ------------------------------------------------------- P: probe 4's TWELVE cells, exactly
    # Two map pairs x six throttles on ONE design object — `probe_r4.py`'s grid, taken from that
    # FILE rather than reconstructed from § 5.15's prose table, because the arm's whole value is
    # that it can reproduce probe 4's 5-of-12 and 10-of-12 under CPython.
    #
    # Probe 3 measured the noise-floor exit as the ORDINARY exit here (6 of 12), against a shipped
    # comment calling it a path "reached ONLY by inputs that previously RAISED". The exit kind and
    # the pass count are DISCRETE keys for exactly that reason.
    for name, ml, mh in (("shaped", LP_SHAPED, HP_SHAPED), ("flat", FLAT, FLAT)):
        t = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh)
        for Tt4 in (1500.0, 1450.0, 1400.0, 1300.0, 1200.0, 1100.0):
            eq = t.equilibrium(FLIGHT, Tt4)
            put_instant(f"P/{name}/{Tt4:.0f}", eq)
            kind, iters = eq_kind_and_iters(EQ_POWERS[-1])
            putd(f"P/{name}/{Tt4:.0f}/exit_noise", kind)
            putd(f"P/{name}/{Tt4:.0f}/passes", iters)
            putd(f"P/{name}/{Tt4:.0f}/powers_calls", EQ_POWERS[-1])
        emit_census(f"P/{name}")

    # ------------------------------------------------------- Q: gate 1's reacting REDUCE
    # `equilibrium` must land on rung 39's `match` — the non-circular direction, since the Newton
    # uses the FORWARD closure only and never calls the matcher.
    t = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED)
    for Tt4 in (1500.0, 1200.0):
        od = t.match(FLIGHT, Tt4)
        put(f"Q/{Tt4:.0f}/nu_lp", od.N_lp_ratio)
        put(f"Q/{Tt4:.0f}/nu_hp", od.N_hp_ratio)
        put(f"Q/{Tt4:.0f}/pi_lpc", od.pi_lpc)
        put(f"Q/{Tt4:.0f}/pi_hpc", od.pi_hpc)
        put(f"Q/{Tt4:.0f}/mdot_air", od.mdot_air)
        put(f"Q/{Tt4:.0f}/slip", od.slip)
    emit_census("Q")

    # ------------------------------------------------------- R: gate 5's REACTING Jacobians
    # The parametrized half of gate 5 — the reacting arm, all seven pairs, three throttles.
    for name, ml, mh in SHAPES:
        t = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        for Tt4 in (1500.0, 1200.0, 950.0):
            od = t.match(FLIGHT, Tt4)
            nu = (od.N_lp_ratio, od.N_hp_ratio)
            t.rho = 1.0
            J = t.jacobian(FLIGHT, Tt4, nu=nu)
            p = f"R/{name}/{Tt4:.0f}"
            for r in range(2):
                for cc in range(2):
                    put(f"{p}/J/{r}{cc}", J[r][cc])
            put(f"{p}/bc", J[0][1] * J[1][0])
            for ir, rho in enumerate(RHO_EIG):
                Jr = [[J[0][0] / rho, J[0][1] / rho], [J[1][0], J[1][1]]]
                lo, hi = t.eigenvalues(Jr)
                put(f"{p}/eig/{ir}/lo", lo)
                put(f"{p}/eig/{ir}/hi", hi)
    emit_census("R")

    # ------------------------------------------------------- S: the reacting forward closure
    # `_close` and the full instant on the reacting gas at off-matched speeds — the path sections
    # P/Q/R only ever reach through a solver.
    t = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED)
    Tt2, pt2, V0 = t._inlet(FLIGHT)
    put("S/tt2", Tt2)
    put("S/pt2", pt2)
    put("S/v0", V0)
    for inu, (a, b) in enumerate(((1.0, 1.0), (0.92, 0.96))):
        put_close(f"S/close/{inu}", t._close(a, b, 1350.0, Tt2, pt2))
        put_instant(f"S/inst/{inu}", t._instant(FLIGHT, a, b, 1350.0))
    emit_census("S")


if ARM in ("main", "cpython"):
    main_arm()
if ARM in ("equil", "cpython"):
    equil_arm()

out = open(OUT, "w", encoding="utf-8", newline="\n") if OUT else sys.stdout
out.write(f"# slice R oracle ({ARM}) — rungs 40+44 TwoSpoolTransient — "
          "key\tu64 bits (or an integer)\trepr\n")
for key, bits, text in ROWS:
    out.write(f"{key}\t{bits}\t{text}\n")
if OUT:
    out.flush()
    os.fsync(out.fileno())
    out.close()
sys.stderr.write(f"[dump_two_spool_transient/{ARM}] {len(ROWS)} values\n")
