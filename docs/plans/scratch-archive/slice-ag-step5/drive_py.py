"""SLICE AG step 5 -- THROWAWAY DRIVE for rung 76's readers: `_cap_rows`, `cap_gains`,
`cap_bill`, `solve_gain`.

Steps 3 and 4's precedent (plan sections 5.31.3 / 5.31.4): a readers step proves itself by DRIVING
every reader end to end against a PyPy golden, not by a gate file of its own. The ported gates are
step 6's.

THE GRID IS `tests/test_rung76.py`'s OWN, copied argument by argument -- the same grid step 4's
drive used, extended with the reader arguments the suite passes:

    FLIGHT      T0 250, p0 50 000, M0 0.85 . PI_LPC 3.0 / PI_HPC 6.0 / TT4 1500
    REAL        the suite's nine losses, nozzle_convergent=True
    FLOOR 0.55 . B 0.10 . V_MAX 0.20 . TT4_MAX 1200 . TAUS (.05,.05,.05,.05) . TAU_T 0.05
    LO/HI       1000 -> 1400 . R 0.5 . SETTLE 1.2 . DS 0.005 . every 8 . dq 1e-5
    phi         PHI_JAC 0.80 (every JACOBIAN) and PHI_BOTH 0.76 (every TRAJECTORY)
    margin      MARGIN 0.10, plus 0.05 and 0.40 in section H -- the suite's own solve_gain sweep

Every float is an IEEE-754 bit pattern. Run:

    .venv/Scripts/python.exe W:/temp/claude/slice-ag-step5/drive_py.py > .../py.tsv
"""
import struct
import sys

sys.path.insert(0, r"M:\claud_projects\jet engine")

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    SensedCapTransient, BleedLimiter, StatorLimiter, StatorIncidenceLimiter,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


def s(key, text):
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


def fopt(key, x):
    """A key Python spells `None` on one arm and a float on the other -- emitted as a presence
    flag beside the value, so `absent` and `zero` stay distinguishable (slice AE's rule)."""
    if x is None:
        d(key + "?", 0)
        return
    d(key + "?", 1)
    f(key, x)


def pair(key, t):
    f(key + "/0", t[0])
    f(key + "/1", t[1])


def pair_opt(key, t):
    if t is None:
        d(key + "?", 0)
        return
    d(key + "?", 1)
    pair(key, t)


FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, V_MAX, TT4_MAX = 0.10, 0.20, 1200.0
TAUS = (0.05, 0.05, 0.05, 0.05)
TAU, TAU_S = 0.05, 0.05
TAU_T = 0.05
PHI_JAC, PHI_BOTH = 0.80, 0.76
MARGIN = 0.10
EVERY = 8
DQ = 1e-5
TAIL = 3.0

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg():
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=1.3, cp_t=1239.0, R_t=(1.3 - 1.0) / 1.3 * 1239.0, hPR=42.8e6)


DESIGN = build_two_spool_turbojet(_cpg(), 3.0, 6.0, 1500.0, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def _sm(phi_lim):
    return phi_lim / FLOOR - 1.0


def _rig(sm, inc=False):
    m = SensedCapTransient(
        DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
        stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                    else None),
        stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law = "demand", "sched"
    m._windup_law, m._tau_t = "none", None
    m._cap_law = "solve"
    return m


ROW_F = ("s", "c", "cap_accel", "cap_phi", "tau_auth", "tau_masked", "auth_diag", "auth_diag0",
         "masked_diag", "masked_diag0", "row_auth", "row_auth0", "mask_leak", "mask_leak0",
         "det", "det0", "gov_row")

# =============================================================================================
# SECTION E -- `_cap_rows` PER ROW, which is the only place the row detail is reachable
#
# `cap_gains`'s cells carry AGGREGATES and, unlike rung 75's `windup_gains`, no `rows` key -- so
# every per-point field (`accel_binds`, the three caps, both `tau`s, `gov_row`) can only be
# bit-checked by calling the private reader directly. ONE cell is driven here (`sched | none`,
# the default) and section F then re-drives all four through the public one.
# =============================================================================================
sm_j = _sm(PHI_JAC)
for inc in (False, True):
  for margin in (MARGIN, 0.20):
    acc_j = _rig(sm_j, inc).accel_for(FLIGHT, LO, HI, sm_j, TT4_MAX, TAUS, V_MAX, inc, margin)
    rows, n_riding = _rig(sm_j, inc)._cap_rows(
        FLIGHT, sm_j, "sched", "none", None, TAUS, inc, LO, HI, TT4_MAX, R, SETTLE, DS,
        V_MAX, acc_j, EVERY)
    E = "E/i%d/%g" % (inc, margin)
    d(E + "/n_tau_split", sum(1 for x in rows if x["tau_auth"] != x["tau_masked"]))
    d(E + "/n_rows", len(rows))
    d(E + "/n_riding", n_riding)
    d(E + "/n_binding", sum(1 for x in rows if x["accel_binds"]))
    for i, x in enumerate(rows):
        for k in ROW_F:
            f("%s/%d/%s" % (E, i, k), x[k])
        s("%s/%d/auth" % (E, i), x["auth"])
        s("%s/%d/masked" % (E, i), x["masked"])
        b("%s/%d/accel_binds" % (E, i), x["accel_binds"])
        d("%s/%d/zeros" % (E, i), x["zeros"])
        d("%s/%d/zeros0" % (E, i), x["zeros0"])

# =============================================================================================
# SECTION F -- `cap_gains`: the 2x2x2 of (ref, law, auth), and BOTH cell arms
#
# At `margin = 0.10` the pre-flight measured 4 of 8 cells live, so this single call exercises the
# SIX-key empty dict and the TWENTY-SIX-key reading in one run -- which is the shape the port had
# to guess at and is why the enum is driven rather than reasoned about.
#
# AND `margin = 0.20` IS DRIVEN BESIDE IT BECAUSE THE FIRST GRID CANNOT SEE TWO BRANCHES. At
# `0.10` this drive measured `n_inert = 0` in every cell (so the whole three-cap min-select guard
# is inert: 10 of 10 rows bind) and `auth == "fuel"` in every live one (so `row_err`'s `gov`-only
# expression, and the `sched`-vs-`applied` target split inside it, are never evaluated). The
# pre-flight's own margin sweep says `0.20` is where the accel leg stops winning everywhere --
# 548 of 612 -- so the second margin is what makes those branches READABLE. Slice W step 3's rule:
# make the instrument prove it can SEE.
# =============================================================================================
for inc in (False, True):
  for margin in (MARGIN, 0.20):
    g = _rig(sm_j, inc).cap_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=margin,
                                  taus=TAUS, tau_t=TAU_T, inc=inc, r=R, s_settle=SETTLE, ds=DS,
                                  v_max=V_MAX, every=EVERY)
    F = "F/i%d/%g" % (inc, margin)
    f(F + "/phi_lim", g["phi_lim"])
    f(F + "/margin", g["margin"])
    f(F + "/tau_t", g["tau_t"])
    f(F + "/ds", g["ds"])
    b(F + "/inc", g["inc"])
    d(F + "/n_cells", len(g["cells"]))
    for key in sorted(g["cells"]):
        c = g["cells"][key]
        t = F + "/" + key
        s(t + "/key", key)
        d(t + "/n", c["n"])
        d(t + "/n_riding", c["n_riding"])
        d(t + "/n_inert", c["n_inert"])
        s(t + "/ref", c["ref"])
        s(t + "/law", c["law"])
        s(t + "/auth", c["auth"])
        b(t + "/live", bool(c["n"]))
        d(t + "/n_keys", len(c))
        if not c["n"]:
            continue
        for k in ("c", "auth_diag", "auth_diag0", "masked_diag", "row_auth", "row_auth0",
                  "det", "det0"):
            pair(t + "/" + k, c[k])
        for k in ("auth_moved", "auth_err", "masked_moved", "mask_leak", "mask_leak0",
                  "gov_row"):
            f(t + "/" + k, c[k])
        fopt(t + "/row_err", c["row_err"])
        pair_opt(t + "/det_ratio", c["det_ratio"])
        fopt(t + "/det_err", c["det_err"])
        for k in ("zeros", "zeros0"):
            d(t + "/" + k + "/0", c[k][0])
            d(t + "/" + k + "/1", c[k][1])
        d(t + "/zeros_moved", c["zeros_moved"])

# =============================================================================================
# SECTION G -- `cap_bill`: the two arms marched at PHI_BOTH, and the UNTAGGED grid assert
#
# Two cells: the suite's own `sched | none`, and `applied | track` -- which is the only place in
# this drive that exercises `tau_t if law == "track" else None` on the BILL's march, and the only
# reader arm where the reference is the applied one.
# =============================================================================================
sm_b = _sm(PHI_BOTH)
for inc, ref, law in ((False, "sched", "none"), (False, "applied", "track"),
                      (True, "sched", "none")):
    bill = _rig(sm_b, inc).cap_bill(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, margin=MARGIN,
                                    taus=TAUS, tau_t=TAU_T, ref=ref, law=law, inc=inc, r=R,
                                    s_settle=SETTLE, ds=DS, v_max=V_MAX, tail=TAIL)
    t = "G/i%d/%s|%s" % (inc, ref, law)
    d(t + "/n", bill["n"])
    f(t + "/s_tail", bill["s_tail"])
    pair(t + "/max_Tt4", bill["max_Tt4"])
    pair(t + "/min_phi", bill["min_phi"])
    pair(t + "/fuel_int", bill["fuel_int"])
    fopt(t + "/wf_tail", bill["wf_tail"])
    fopt(t + "/wf_ramp", bill["wf_ramp"])
    b(t + "/cuts_harder", bill["cuts_harder"])
    for arm in ("solve", "sensed"):
        traj = bill["traj"][arm]
        for i in range(0, len(traj), 23):
            p = traj[i]
            for k in ("s", "Tt4", "mf", "mf_sched", "phi_lp", "w_fuel", "w_gov", "b", "v"):
                if k in p and isinstance(p[k], float):
                    f("%s/%s/%d/%s" % (t, arm, i, k), p[k])

# =============================================================================================
# SECTION H -- `solve_gain`: the two identities, over the suite's OWN margin sweep
#
# `0.05 / 0.10 / 0.40` is `test_c_is_strictly_inside_the_unit_interval`'s grid. `H/*/n_dS_zero`
# COUNTS the exact-zero `dS` denominators rather than reasoning about them: the `nan` branch of
# `gain` is the one NaN reachable by construction in this slice, and whether it is reachable ON
# THIS PLANT decides whether the port's Python-faithful `min`/`max` folds are gate-able at all.
# =============================================================================================
for inc in (False, True):
  for margin in (0.05, MARGIN, 0.40):
    sg = _rig(sm_j, inc).solve_gain(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=margin,
                                    taus=TAUS, inc=inc, r=R, s_settle=SETTLE, ds=DS,
                                    v_max=V_MAX, dq=DQ, every=EVERY)
    t = "H/i%d/%g" % (inc, margin)
    d(t + "/n_fp_zero", sum(1 for x in sg["rows"] if x["fixed_point"] == 0.0))
    d(t + "/n", sg["n"])
    f(t + "/margin", sg["margin"])
    s(t + "/ref", sg["ref"])
    fopt(t + "/fixed_point", sg["fixed_point"])
    pair_opt(t + "/gain", sg["gain"])
    fopt(t + "/gain_err", sg["gain_err"])
    d(t + "/n_dS_zero", sum(1 for x in sg["rows"] if x["dS"] == 0.0))
    d(t + "/n_gain_nan", sum(1 for x in sg["rows"] if x["gain"] != x["gain"]))
    for i, x in enumerate(sg["rows"]):
        for k in ("s", "cap_solve", "c", "fixed_point", "dS", "dD", "gain", "predicted"):
            f("%s/%d/%s" % (t, i, k), x[k])

seen = set()
for k, v in OUT:
    assert k not in seen, "duplicate key %s" % k
    seen.add(k)
    print("%s\t%d" % (k, v))
print("# %d keys" % len(OUT), file=sys.stderr)
