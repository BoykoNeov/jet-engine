"""SLICE AG step 4 -- THROWAWAY DRIVE for rung 76's cap: `_cap_march`, `_with_cap`, `accel_for`,
`_c_at`.

Step 3's precedent (plan section 5.31.3): a bodies step with no readers of its own proves itself by
DRIVING every method end to end against PyPy, not by a gate file. The ported gates are step 6's.

THE GRID IS `tests/test_rung76.py`'s OWN, copied argument by argument:

    FLIGHT      T0 250, p0 50 000, M0 0.85 . PI_LPC 3.0 / PI_HPC 6.0 / TT4 1500
    REAL        the suite's nine losses, nozzle_convergent=True
    FLOOR 0.55 . B 0.10 . V_MAX 0.20 . TT4_MAX 1200 . TAUS (.05,.05,.05,.05)
    LO/HI       1000 -> 1400 . R 0.5 . SETTLE 1.2 . DS 0.005
    phi         PHI_JAC 0.80 and PHI_BOTH 0.76 -- the suite's two floors
    margin      MARGIN 0.10, plus 0.20 in section A only (the suite's own sweep point)

Every float is an IEEE-754 bit pattern. Run:

    .venv/Scripts/python.exe W:/temp/claude/slice-ag-step4/drive_py.py > .../py.tsv
"""
import os
import struct
import sys

sys.path.insert(0, r"M:\claud_projects\jet engine")

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    SensedCapTransient, BleedLimiter, StatorLimiter, SurgeLimiter, AsymmetricLag,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def opt(key, fn):
    """A key whose Python call may RAISE where Rust returns `Err(Abort)` -- emitted as a presence
    flag beside the value, so an abort is a measurement rather than a crashed dump."""
    try:
        x = fn()
    except AssertionError:
        d(key + "?", 0)
        return
    d(key + "?", 1)
    f(key, x)


def s(key, text):
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, V_MAX, TT4_MAX = 0.10, 0.20, 1200.0
TAUS = (0.05, 0.05, 0.05, 0.05)
TAU, TAU_S = 0.05, 0.05
PHI_JAC, PHI_BOTH = 0.80, 0.76
MARGIN = 0.10

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg():
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=1.3, cp_t=1239.0, R_t=(1.3 - 1.0) / 1.3 * 1239.0, hPR=42.8e6)


DESIGN = build_two_spool_turbojet(_cpg(), 3.0, 6.0, 1500.0, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def _sm(phi_lim):
    return phi_lim / FLOOR - 1.0


def _rig(sm, cap_law="solve"):
    m = SensedCapTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                           bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
                           stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law = "demand", "sched"
    m._windup_law, m._tau_t = "none", None
    m._cap_law = cap_law
    return m


# =============================================================================================
# SECTION A -- `accel_for`: the TABLE it builds, and the caps that table returns
#
# The schedule is an OBJECT, so it is driven from both ends: its own thirteen `n_H`/`kappa` rows
# (the table, which is what `accel_schedule(..., n = 13)` produces and where a wrong `n` shows as
# a different row count) AND seven sampled `cap(n_h, pt3)` readings, which is the only thing any
# caller ever asks it for.
# =============================================================================================
for phi in (PHI_JAC, PHI_BOTH):
    for margin in (MARGIN, 0.20):
        sm = _sm(phi)
        acc = _rig(sm).accel_for(FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, False, margin)
        tag = "A/%g/%g" % (phi, margin)
        d(tag + "/n_rows", len(acc.n_H))
        f(tag + "/margin", acc.margin)
        for k in range(len(acc.n_H)):
            f("%s/n_H/%d" % (tag, k), acc.n_H[k])
            f("%s/kappa/%d" % (tag, k), acc.kappa[k])
        lo_n, hi_n = acc.n_H[0], acc.n_H[-1]
        for j in range(7):
            n_h = lo_n * 0.95 + (hi_n * 1.05 - lo_n * 0.95) * j / 6.0
            f("%s/cap/%d" % (tag, j), acc.cap(n_h, 3.0e5))

# =============================================================================================
# SECTION B -- `_cap_march`: the trajectory under BOTH cap laws, at BOTH floors
#
# THE POSITIVE OBSERVATION THE REDUCE CANNOT MAKE. `_cap_march`'s one structural difference from
# rung 75's `_windup_march` is that it ARMS the accel leg; drop that and `_sensed_cap` is never
# dispatched, so the `sensed` arm silently returns the `solve` arm's floats -- which IS this
# rung's reduce answer. `B/<phi>/n_diff` and `B/<phi>/max_abs_dTt4` are that difference COUNTED:
# the pre-flight measured 0 of 341 at 0.80 and 341 of 341, max 1.025497e+01, at 0.76.
# =============================================================================================
TRAJ = {}
for phi in (PHI_JAC, PHI_BOTH):
    sm = _sm(phi)
    acc = _rig(sm).accel_for(FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, False, MARGIN)
    for law in ("solve", "sensed"):
        m0 = _rig(sm)
        m, surge, lag, traj = m0._cap_march(
            FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, False,
            "demand", "sched", "none", None, law, acc)
        TRAJ[(phi, law)] = (m, acc, traj)
        tag = "B/%g/%s" % (phi, law)
        d(tag + "/n", len(traj))
        s(tag + "/cap_law_on_rig", m._cap_law)
        s(tag + "/coord_on_rig", m._lag_coord)
        s(tag + "/ref_on_rig", m._ref_law)
        s(tag + "/windup_on_rig", m._windup_law)
        for i in range(0, len(traj), 17):
            p = traj[i]
            for key in ("s", "nu_lp", "nu_hp", "Tt4", "mf", "mf_sched", "b", "v",
                        "w_fuel", "w_gov", "phi_lp", "required"):
                if key in p and isinstance(p[key], float):
                    f("%s/%d/%s" % (tag, i, key), p[key])
    a = TRAJ[(phi, "solve")][2]
    b_ = TRAJ[(phi, "sensed")][2]
    d("B/%g/n_diff" % phi, sum(1 for x, y in zip(a, b_) if x["Tt4"] != y["Tt4"]))
    f("B/%g/max_abs_dTt4" % phi, max(abs(x["Tt4"] - y["Tt4"]) for x, y in zip(a, b_)))
    d("B/%g/n_diff_mf" % phi, sum(1 for x, y in zip(a, b_) if x["mf"] != y["mf"]))
    # THE PRE-FLIGHT's OWN POPULATION, so its published 341 is reproduced rather than nearly
    # reproduced: `probe_d_arming.py:179` compares the ELEVEN-KEY TUPLE, where `n_diff` above
    # compares `Tt4` alone. The first eight points differ in `mf`/`w_fuel` while `Tt4` is still
    # bit-equal, which is the whole of the 341-vs-333 gap.
    KS = ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "v", "w_fuel", "w_gov")
    def _t(p):
        return tuple(p[k] for k in KS if k in p)
    d("B/%g/n_diff_tuple" % phi, sum(1 for x, y in zip(a, b_) if _t(x) != _t(y)))

# =============================================================================================
# SECTION C -- `_c_at`: the derivative the rung rests on
#
# Driven on the `solve` arm's OWN marched points, at two fuels each (`mf`, the applied, and
# `mf_sched`, the demanded) -- the two `w` its step-5 callers pass. `C/fold/*` drives the
# EXPRESSION-FIRST `max(w, 1e-9)` on both sides of its own hinge, which is the one spelling
# decision this method carries.
# =============================================================================================
for phi in (PHI_JAC, PHI_BOTH):
    m, acc, traj = TRAJ[(phi, "solve")]
    tag = "C/%g" % phi
    for i in range(0, len(traj), 41):
        p = traj[i]
        a, h, q, v = p["nu_lp"], p["nu_hp"], p["b"], p["v"]
        opt("%s/%d/c_mf" % (tag, i), lambda: m._c_at(FLIGHT, a, h, acc, p["mf"], q, v))
        opt("%s/%d/c_sched" % (tag, i),
            lambda: m._c_at(FLIGHT, a, h, acc, p["mf_sched"], q, v))
        opt("%s/%d/c_rel_1em4" % (tag, i),
            lambda: m._c_at(FLIGHT, a, h, acc, p["mf"], q, v, rel=1e-4))
        # The guard's RESTORE, read on the receiver after the call returns.
        d("%s/%d/b_state_after" % (tag, i), 1 if m._b_state is None else 0)
        d("%s/%d/v_state_after" % (tag, i), 1 if m._v_state is None else 0)

m, acc, traj = TRAJ[(PHI_BOTH, "solve")]
p = traj[len(traj) // 2]
_a, _h, _q, _v = p["nu_lp"], p["nu_hp"], p["b"], p["v"]
for j, w in enumerate((1e-12, 1e-10, 1e-9, 1e-8, p["mf"])):
    # BELOW the hinge the step is `rel * 1e-9`; above it, `rel * w`. A spelling that dropped the
    # fold entirely agrees at the last two and diverges at the first three.
    opt("C/fold/%d" % j, lambda w=w: m._c_at(FLIGHT, _a, _h, acc, w, _q, _v))

# =============================================================================================
# SECTION D -- `_with_cap`: the reload guard, and the LAW it puts in front of a real reader
#
# The guard restores the PREVIOUS value, not `None` -- the opposite policy from `_c_at`'s two
# state guards three lines up, and the two live in one file. D/scope/* reads the field; D/cap/*
# drives `_cap_fuel` under each law at marched states, which is what the guard exists for.
# =============================================================================================
m = _rig(_sm(PHI_BOTH), cap_law="sensed")
s("D/scope/before", m._cap_law)


def _inner():
    s("D/scope/inside_outer", m._cap_law)
    r = m._with_cap("sensed", lambda: (s("D/scope/inside_inner", m._cap_law), 0.0)[1])
    # THE RESTORE-PREVIOUS POLICY, READ WHERE IT MATTERS: after the inner guard closes, the OUTER
    # law must be live again. A `None`-restoring guard leaves the field unset here and the next
    # reader's declared-law refusal fires instead of this one's.
    s("D/scope/inside_outer_again", m._cap_law)
    return r


f("D/scope/nested_ret", m._with_cap("solve", _inner))
s("D/scope/after", m._cap_law)

for phi in (PHI_JAC, PHI_BOTH):
    mm, acc, traj = TRAJ[(phi, "solve")]
    surge = SurgeLimiter.from_margin(LP, "lp", _sm(phi))
    tag = "D/cap/%g" % phi
    for i in range(0, len(traj), 41):
        p = traj[i]
        a, h, ms = p["nu_lp"], p["nu_hp"], p["mf_sched"]
        mm._b_state, mm._v_state = p["b"], p["v"]
        try:
            opt("%s/%d/solve" % (tag, i), lambda: mm._with_cap(
                "solve", mm._cap_fuel, FLIGHT, a, h, ms, acc, surge, mf_app=p["mf"]))
            opt("%s/%d/sensed" % (tag, i), lambda: mm._with_cap(
                "sensed", mm._cap_fuel, FLIGHT, a, h, ms, acc, surge, mf_app=p["mf"]))
        finally:
            mm._b_state, mm._v_state = None, None
        s("%s/%d/law_after" % (tag, i), mm._cap_law)

seen = set()
for k, v in OUT:
    assert k not in seen, "duplicate key %s" % k
    seen.add(k)
    print("%s\t%d" % (k, v))
print("# %d keys" % len(OUT), file=sys.stderr)
