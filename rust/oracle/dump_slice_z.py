"""SLICE Z step 4 -- THE ORACLE for rungs 66 + 67 (`TwoLagCascadeTransient`,
`CrossLoopCascadeTransient`).

**THE GRID IS THE SUITES' OWN AND NOTHING IS COARSENED** (P8), and the header states it rather
than implying it -- slice S step 4's lesson, *a probe's HEADER claimed the suites' grids and its
code ran another*. Every argument below is copied from the calling gate in `tests/test_rung66.py`
or `tests/test_rung67.py`, never chosen:

    ds          0.005 everywhere EXCEPT section N, which sweeps (0.01, 0.005, 0.0025) --
                and that triple is `test_the_headline_numbers_are_grid_converged`'s own.
    LO/HI       1000 -> 1400 . r 0.5 . s_settle 1.2 . FLOOR 0.55 -- the suites' throughout
    phi/b       PHI 0.80, B 0.10, SM = PHI/FLOOR - 1, TAU 0.05, TMAX 1200 -- the suites'
    tau_atts    (0.005, 0.05, 0.5), rel_mult 3.0, n_sample 12 -- `cascade_identity`'s defaults,
                which `test_the_cross_gains_are_RECIPROCALS` takes wholesale
    tau_govs    (0.005, 0.05, 0.5), n_sample 8 -- passed EXPLICITLY by three rung-67 gates
    rhos        (0.5, 1.0, 2.0) -- `test_the_mode_is_ADMISSIBLE_and_UNOBSERVABLE`'s, not the
                seven-wide default, because that gate narrows it and the oracle mirrors gates
    corners     Tt4_maxes (1150, 1300) x Tt4_los (1000, 1200) -- the suite's 2x2, not the
                default 4x2
    maps        `shaped` ONLY (LP a=.20 b=.05 l=.7 / HP a=.08 b=.15 l=1.0) -- neither suite
                builds a second shape, so a second one here would be a grid they do not have

**IT COSTS NOTHING TO REFUSE TO COARSEN, AND THAT WAS MEASURED BEFORE THE SECTIONS WERE CHOSEN**
(s 5.24 (viii)): the ten readers total 33.31 s on PyPy at these arguments.

# THE ONE DECLARED CROSS-INTERPRETER EXEMPTION, AND THE DELIVERED COUNT IS RE-READ HERE

Rung 67 has exactly ONE float `sum()` -- `cross_identity`'s `P_mid = sum(prods)/len(prods)`.
CPython 3.12+'s `sum()` is Neumaier-COMPENSATED and PyPy's is naive, so a Rust left fold agrees
with PyPy and can differ from CPython in the last bit.

**THE NUMBER OF PRODUCTS SUMMED IS NOT `n_sample`.** `sub = ride[::max(1, len(ride)//n_sample)]`
is a STRIDE, so the delivered count is `len(ride)//(len(ride)//n_sample)` and lands ABOVE the
request -- s 5.24 (i)'s leading finding, which cost a probe its answer by chunking at 8 where the
reader delivers 9. **This dump therefore EMITS `n_ride` and the delivered `n_sample` for every
row of every `cross_identity` call it makes** (sections F and N), at three different `ds`, so the
Rust arm compares a measured count and no reader of this file has to inherit the 9.

Every float is emitted as its IEEE-754 bit pattern. Regenerate BOTH arms:

    .venv/Scripts/python.exe rust/oracle/dump_slice_z.py > rust/oracle/slice_z_pypy.tsv
    C:/Python314/python.exe  rust/oracle/dump_slice_z.py > rust/oracle/slice_z_cpython.tsv

**Redirect through a POSIX shell, not PowerShell 5.1** -- its `1>` writes UTF-8 WITH A BOM and the
BOM lands in front of the `#` on line 1, so the header parses as data.
[[windows-tooling-file-hazards]].
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    LaggedBleedTransient, TwoLagCascadeTransient, CrossLoopCascadeTransient,
    BleedLimiter, BleedSchedule, SurgeLimiter, AsymmetricLag,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


def opt(key, x):
    """A key Python may return as `None` -- emitted as a PRESENCE flag beside the value, because
    `rho_lo`/`reciprocal`/`first_diff` reach `None` by TWO different routes (an empty row, and the
    `P >= 0` branch of a non-empty one) and a sentinel float would conflate them with a value."""
    b(key + "?", x is not None)
    if x is not None:
        f(key, x)


def opt_d(key, n):
    b(key + "?", n is not None)
    if n is not None:
        d(key, n)


# ---------------------------------------------------------------------------- the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, TMAX = 0.10, 0.80, 1200.0
SM = PHI / FLOOR - 1.0
TAU, TAU_GOV = 0.05, 0.05
TAU_ATT, TAU_REL = 0.05, 0.15
TAU_ATTS = (0.005, 0.05, 0.5)
TAU_GOVS = (0.005, 0.05, 0.5)
TAU_RELS = (0.15, 0.30, 0.60)
REL_MULT = 3.0
D_B0 = 0.01
OSC_D_B0 = 0.005
RHOS = (0.5, 1.0, 2.0)
DS_SWEEP = (0.01, 0.005, 0.0025)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """The suites' `_cpg`, character for character. `R_c` is DERIVED from `(gamma - 1)/gamma`;
    re-spelling it `0.4/1.4` builds a gas ONE ULP away, which presents exactly as a port defect
    (slice Y's own week-long false alarm)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def cas(**kw):
    return TwoLagCascadeTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def cross(**kw):
    return CrossLoopCascadeTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def lag65(**kw):
    return LaggedBleedTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


def fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


# The 14 keys EVERY route records. `branch` is a string and rides as a discrete,
# `dump_fuel_transient.py`'s idiom.
PT14 = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
        "mdot_air", "sp_thrust", "mf", "mf_sched")
# The 7 the suites' own `_keys` compares -- the reduce arms' currency.
PT7 = ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf")


def points(tag, traj, keys=PT14, branch=True):
    d("%s/npts" % tag, len(traj))
    for i, p in enumerate(traj):
        for k in keys:
            f("%s/%d/%s" % (tag, i, k), p[k])
        if branch:
            d("%s/%d/branch_choked" % (tag, i), 1 if p["branch"] == "choked" else 0)


def cascade_points(tag, traj, cross_loop):
    """Rung 66's 20 per-point keys, or rung 67's 21. **The count is EMITTED from the live dict**
    rather than typed -- [[rust-port-guessed-census-bars]] is five typed count bars that were
    every one wrong, and `FuelPoint::key_count`'s doc comment says this line is what checks it."""
    points(tag, traj)
    d("%s/key_count" % tag, len(traj[0]))
    for i, p in enumerate(traj):
        for k in ("g", "required", "b", "b_cmd", "ic_res"):
            f("%s/%d/%s" % (tag, i, k), p[k])
        d("%s/%d/ic_iters" % (tag, i), p["ic_iters"])
        if cross_loop:
            f("%s/%d/ic_damp" % (tag, i), p["ic_damp"])


# ===================================================== A -- rung 66 `cascade_identity`, THE RUNG
ci = cas(bleed_lim=valve(TAU)).cascade_identity(
    FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_atts=TAU_ATTS, rel_mult=REL_MULT,
    n_sample=12, r=R, s_settle=SETTLE, ds=DS)
for k in ("sm", "b_cap", "tau", "ds", "r", "phi_lim", "prod_lo", "prod_hi", "rho_err_max"):
    f("A/%s" % k, ci[k])
b("A/all_real", ci["all_real"])
d("A/n_rows", len(ci["rows"]))
for i, row in enumerate(ci["rows"]):
    for k in ("tau_att", "tau_v", "prod_lo", "prod_hi", "rho_max", "rate_closed_form", "rho_err",
              "gain_span_R", "gain_span_C", "R_q_lo", "R_q_hi", "C_g_lo", "C_g_hi", "ds_rho"):
        f("A/rows/%d/%s" % (i, k), row[k])
    # THE DELIVERED SAMPLE COUNT, beside the ride it was strided out of -- s 5.24 (i).
    for k in ("n_ride", "n_sample", "n_real"):
        d("A/rows/%d/%s" % (i, k), row[k])

# ===================================================== B -- rung 66 `cascade_bill`
BILL66 = ("I", "min_phi", "s_at_min", "s_last", "removed", "min_phi_hp", "nu_lp_end",
          "nu_hp_end", "thrust_end")
cb = cas(bleed_lim=valve(TAU)).cascade_bill(
    FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU_ATT, rel_mult=REL_MULT,
    r=R, s_settle=SETTLE, ds=DS)
for k in ("sm", "b_cap", "tau", "tau_att", "ds", "r", "phi_lim", "sum_alone", "delivered",
          "marginal_fuel", "marginal_valve", "erosion_fuel", "erosion_valve"):
    f("B/%s" % k, cb[k])
for k in ("subadditive", "beats_both"):
    b("B/%s" % k, cb[k])
for k in ("fuel", "valve", "both"):
    f("B/credit/%s" % k, cb["credit"][k])
for name in ("bare", "fuel", "valve", "both"):
    c = cb["cells"][name]
    for k in BILL66:
        f("B/cells/%s/%s" % (name, k), c[k])
    d("B/cells/%s/npts" % name, c["npts"])
    b("B/cells/%s/truncated" % name, c["truncated"])

# ===================================================== C -- rung 66 `marginal_mode_cascade`
MM66 = ("b0", "b_end", "g_end", "drift", "removed", "I", "min_phi_lp", "track_b", "track_g",
        "laws_held")
mm = cas(bleed_lim=valve(TAU)).marginal_mode_cascade(
    FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU_ATT, rel_mult=REL_MULT, d_b0=D_B0,
    r=R, s_settle=SETTLE, ds=DS)
for k in ("sm", "tau", "tau_att", "b_cap", "d_b0", "r", "ds", "phi_lim", "b_natural", "frozen",
          "db_db0", "dremoved", "dremoved_rel", "track_b", "track_g", "laws_held"):
    f("C/%s" % k, mm[k])
b("C/washed_out", mm["washed_out"])
for name, cell in (("natural", mm["natural"]), ("lo", mm["moved"]["lo"]),
                   ("hi", mm["moved"]["hi"])):
    for k in MM66:
        f("C/%s/%s" % (name, k), cell[k])
    d("C/%s/n_on" % name, cell["n_on"])
    d("C/%s/npts" % name, cell["npts"])

# ===================================================== D -- rung 66 `merge_identity`
mi = cas(bleed_lim=valve(TAU)).merge_identity(
    FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU_ATT, tau_rels=TAU_RELS,
    r=R, s_settle=SETTLE, ds=DS)
for k in ("sm", "tau", "tau_att", "ds"):
    f("D/%s" % k, mi[k])
b("D/ok", mi["ok"])
opt_d("D/crossing", mi["crossing"])
opt("D/s_crossing", mi["s_crossing"])
d("D/n_rows", len(mi["rows"]))
for i, row in enumerate(mi["rows"]):
    f("D/rows/%d/tau_rel" % i, row["tau_rel"])
    d("D/rows/%d/npts" % i, row["npts"])
    b("D/rows/%d/identical" % i, row["identical"])
    opt_d("D/rows/%d/first_diff" % i, row["first_diff"])
    opt("D/rows/%d/s_first" % i, row["s_first"])

# ===================================================== E -- rung 66's FOUR-STATE march, in full
e_traj, e_nu = cas(bleed_lim=valve(TAU))._stator_march(
    FLIGHT, LO, HI, R, SETTLE, DS, surge=fuel(), lag=lag())
cascade_points("E/cascade", e_traj, cross_loop=False)
f("E/nu_lp", e_nu[0])
f("E/nu_hp", e_nu[1])

# ===================================================== F -- rung 67 `cross_identity`, THE SCALAR
xi = cross(bleed_lim=valve(TAU)).cross_identity(
    FLIGHT, LO, HI, TMAX, tau=TAU, tau_govs=TAU_GOVS, n_sample=8, r=R, s_settle=SETTLE, ds=DS)
for k in ("Tt4_max", "tau", "ds", "r", "phi_lim", "b_max", "prod_lo", "prod_hi", "R_q_min_abs"):
    f("F/%s" % k, xi[k])
for k in ("all_negative", "sum_always_safe"):
    b("F/%s" % k, xi[k])
d("F/n_rows", len(xi["rows"]))
for i, row in enumerate(xi["rows"]):
    for k in ("tau_gov", "tau_v", "rho_clock", "prod_lo", "prod_hi", "P_mid", "R_q_lo", "R_q_hi",
              "C_g_lo", "C_g_hi", "gain_span_R", "gain_span_C", "rho_max", "sum_bound",
              "sum_conservative"):
        f("F/rows/%d/%s" % (i, k), row[k])
    for k in ("n_ride", "n_sample", "n_complex", "n_saturated"):
        d("F/rows/%d/%s" % (i, k), row[k])
    # THE SIX WINDOW KEYS, spread in from a dict that is EMPTY on an empty row and missing
    # `reciprocal` on the `P >= 0` branch of a non-empty one -- two routes to the same `None`.
    for k in ("rho_lo", "rho_hi", "zeta", "T_over_tau", "reciprocal"):
        opt("F/rows/%d/%s" % (i, k), row[k])
    opt_d("F/rows/%d/opens" % i, None if row["opens"] is None else int(row["opens"]))

# ===================================================== G -- rung 67 `oscillation_window`
ow = cross(bleed_lim=valve(TAU)).oscillation_window(
    FLIGHT, LO, HI, TMAX, tau=TAU, rhos=RHOS, d_b0=OSC_D_B0, r=R, s_settle=SETTLE, ds=DS)
for k in ("Tt4_max", "tau", "ds", "r", "d_b0", "P", "survives_max"):
    f("G/%s" % k, ow[k])
for k in ("n_complex", "n_real", "max_sign_changes"):
    d("G/%s" % k, ow[k])
b("G/rings_anywhere", ow["rings_anywhere"])
for k in ("P", "k", "zeta", "T_over_tau"):
    f("G/window/%s" % k, ow["window"][k])
b("G/window/opens", ow["window"]["opens"])
for k in ("rho_lo", "rho_hi", "reciprocal"):
    opt("G/window/%s" % k, ow["window"].get(k))
d("G/n_rows", len(ow["rows"]))
# EMITTED, not assumed: how many rows the inherited rung-66 `ds` floor skipped. On this grid it
# is 0, and a zero left un-emitted is exactly what [[rust-port-slice-w-step3]] is about.
d("G/n_skipped", sum(1 for x in ow["rows"] if "skipped" in x))
for i, row in enumerate(ow["rows"]):
    b("G/rows/%d/skipped" % i, "skipped" in row)
    f("G/rows/%d/rho" % i, row["rho"])
    f("G/rows/%d/tau_gov" % i, row["tau_gov"])
    if "skipped" in row:
        continue
    for k in ("d0", "d_end", "survives", "d_peak"):
        f("G/rows/%d/%s" % (i, k), row[k])
    for k in ("npts", "sign_changes_q", "sign_changes_g"):
        d("G/rows/%d/%s" % (i, k), row[k])
    b("G/rows/%d/complex_predicted" % i, row["complex_predicted"])
    b("G/rows/%d/rings" % i, row["rings"])

# ===================================================== H -- rung 67 `cross_bill`
BILL67 = ("I_T", "I_phi", "s_last", "max_Tt4", "min_phi", "removed", "nu_lp_end", "nu_hp_end",
          "thrust_end")
xb = cross(bleed_lim=valve(TAU)).cross_bill(
    FLIGHT, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, r=R, s_settle=SETTLE, ds=DS)
for k in ("Tt4_max", "tau", "tau_gov", "ds", "r", "phi_lim", "erosion_gov", "erosion_valve",
          "marginal_gov_T", "marginal_valve_phi", "valve_on_T", "gov_on_phi", "sum_alone_T",
          "sum_alone_phi"):
    f("H/%s" % k, xb[k])
for k in ("valve_debits_T", "gov_credits_phi"):
    b("H/%s" % k, xb[k])
for k in ("gov", "valve", "both"):
    f("H/credit_T/%s" % k, xb["credit_T"][k])
    f("H/credit_phi/%s" % k, xb["credit_phi"][k])
for name in ("bare", "gov", "valve", "both"):
    c = xb["cells"][name]
    for k in BILL67:
        f("H/cells/%s/%s" % (name, k), c[k])
    d("H/cells/%s/npts" % name, c["npts"])
    b("H/cells/%s/truncated" % name, c["truncated"])

# ===================================================== I -- rung 67 `marginal_mode_cross`
MM67 = ("b0", "b_end", "g_end", "drift", "removed", "I_phi", "I_T", "min_phi_lp", "track_b",
        "track_g")
mx = cross(bleed_lim=valve(TAU)).marginal_mode_cross(
    FLIGHT, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, d_b0=D_B0, r=R, s_settle=SETTLE, ds=DS)
for k in ("Tt4_max", "tau", "tau_gov", "d_b0", "r", "ds", "phi_lim", "b_natural", "db_db0",
          "dremoved", "dremoved_rel", "dI_phi", "dI_phi_rel", "drift", "track_b", "track_g"):
    f("I/%s" % k, mx[k])
for name, cell in (("natural", mx["natural"]), ("lo", mx["moved"]["lo"]),
                   ("hi", mx["moved"]["hi"])):
    for k in MM67:
        f("I/%s/%s" % (name, k), cell[k])
    for k in ("n_on", "npts", "ic_iters"):
        d("I/%s/%s" % (name, k), cell[k])

# ===================================================== J -- rung 67 `joint_ic_corners`
jc = cross(bleed_lim=valve(TAU)).joint_ic_corners(
    FLIGHT, LO, HI, Tt4_maxes=(1150.0, 1300.0), Tt4_los=(1000.0, 1200.0),
    tau=TAU, tau_gov=TAU_GOV, r=R, s_settle=SETTLE, ds=DS)
f("J/tau", jc["tau"])
f("J/tau_gov", jc["tau_gov"])
f("J/ds", jc["ds"])
d("J/n_live", jc["n_live"])
d("J/max_iters", jc["max_iters"])
b("J/all_converged", jc["all_converged"])
b("J/ever_damped", jc["ever_damped"])
d("J/n_rows", len(jc["rows"]))
# The CAUGHT arm's own count, emitted so `all_converged` cannot pass vacuously over a row set
# that is entirely `failed` -- Python's `all([])` is True.
d("J/n_failed", sum(1 for x in jc["rows"] if "failed" in x))
for i, row in enumerate(jc["rows"]):
    f("J/rows/%d/Tt4_lo" % i, row["Tt4_lo"])
    f("J/rows/%d/Tt4_max" % i, row["Tt4_max"])
    b("J/rows/%d/failed" % i, "failed" in row)
    if "failed" in row:
        d("J/rows/%d/msg_len" % i, len(row["failed"]))
        continue
    for k in ("required0", "b0", "g0", "ic_res", "ic_damp"):
        f("J/rows/%d/%s" % (i, k), row[k])
    d("J/rows/%d/ic_iters" % i, row["ic_iters"])
    d("J/rows/%d/npts" % i, row["npts"])
    b("J/rows/%d/live" % i, row["live"])

# ===================================================== K -- the LEAF STATICS
W = CrossLoopCascadeTransient._window
# The eight `P` the two window gates evaluate, PLUS the plant's own `P_mid` from section F --
# which is where step 3's injection I6 measured a one-ulp re-spelling that no gate could see.
P_VALUES = (1.0, 0.5, -1e-3, -0.02, -0.5, -3.0, -10.0, xi["rows"][1]["P_mid"])
for i, P in enumerate(P_VALUES):
    w = W(P)
    for k in ("P", "k", "zeta", "T_over_tau"):
        f("K/window/%d/%s" % (i, k), w[k])
    b("K/window/%d/opens" % i, w["opens"])
    for k in ("rho_lo", "rho_hi", "reciprocal"):
        opt("K/window/%d/%s" % (i, k), w.get(k))

# `_exceed` on the synthetic ramp `test_the_exceedance_integral_does_not_DROP_its_final_cell`
# builds, at its three limits -- the one place the straddling cell's INTERPOLATION is exact by
# hand rather than measured off a march.
syn = [dict(s=i * 0.1, Tt4=1000.0 + 100.0 * i) for i in range(8)]
for i, s_hi in enumerate((0.5, 0.55, 0.5 * (1.0 + 1e-15))):
    f("K/exceed/%d" % i, CrossLoopCascadeTransient._exceed(syn, 1000.0, s_hi))
# and rung 66's `_violation` on the SAME synthetic ramp, so the two upper limits are on one grid
# and the difference the port must not fold away is a NUMBER here rather than a doc comment.
syn_v = [dict(s=i * 0.1, phi_lp=0.80 - 0.1 * i) for i in range(8)]
for i, s_hi in enumerate((0.5, 0.55, 0.5 * (1.0 + 1e-15))):
    f("K/violation/%d" % i, TwoLagCascadeTransient._violation(syn_v, 0.80, s_hi))

# `_sign_changes`, including the `peak <= 0` early return that fires on NO shipped grid
SC = (([0.0, 0.0, 0.0], "zeros"), ([1.0, -1.0, 1.0, -1.0], "alt"),
      ([1.0, 1e-9, -1.0], "floored"))
for xs, name in SC:
    d("K/sign_changes/%s" % name, CrossLoopCascadeTransient._sign_changes(xs))

# `_joint_fixed_point`'s damping ladder, at the five `P` its own gate sweeps
solve = CrossLoopCascadeTransient._joint_fixed_point
G_STAR, Q_STAR, A_LIN = 3.0e-3, 0.04, 1.0e-3
for i, P in enumerate((-0.02, -0.5, -0.9, -2.0, -5.0)):
    R_of = (lambda q: G_STAR + A_LIN * (q - Q_STAR))
    C_of = (lambda g, P=P: Q_STAR + (P / A_LIN) * (g - G_STAR))
    g, q, res, its, w = solve(R_of, C_of, Q_STAR + 0.01)
    for k, v in (("g", g), ("q", q), ("res", res), ("w", w)):
        f("K/jfp/%d/%s" % (i, k), v)
    d("K/jfp/%d/its" % i, its)
R_of = (lambda q: G_STAR + A_LIN * (q - Q_STAR))
C_of = (lambda g: Q_STAR + (-0.02 / A_LIN) * (g - G_STAR))
g, q, res, its, w = solve(R_of, C_of, 0.055, fix_q=True)
for k, v in (("g", g), ("q", q), ("res", res), ("w", w)):
    f("K/jfp/fixq/%s" % k, v)
d("K/jfp/fixq/its", its)

# `detector_sensitivity`, at its own defaults -- the classmethod `test_the_ringing_detector…`
# calls with no arguments at all.
ds_ = CrossLoopCascadeTransient.detector_sensitivity()
f("K/det/tau", ds_["tau"])
f("K/det/ds", ds_["ds"])
f("K/det/s_end", ds_["s_end"])
b("K/det/fires", ds_["fires"])
b("K/det/quiet_at_weak", ds_["quiet_at_weak"])
d("K/det/n_rows", len(ds_["rows"]))
for i, row in enumerate(ds_["rows"]):
    for k in ("P", "zeta", "T_over_tau", "T", "periods", "decay_per_period"):
        f("K/det/rows/%d/%s" % (i, k), row[k])
    d("K/det/rows/%d/sign_changes" % i, row["sign_changes"])
    b("K/det/rows/%d/rings" % i, row["rings"])
d("K/RINGS", CrossLoopCascadeTransient._RINGS)

# ===================================================== L -- rung 67's CROSS march, in full
l_traj, l_nu = cross(bleed_lim=valve(TAU))._stator_march(
    FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TMAX, tau_gov=TAU_GOV)
cascade_points("L/cross", l_traj, cross_loop=True)
f("L/nu_lp", l_nu[0])
f("L/nu_hp", l_nu[1])

# ===================================================== M -- THE REDUCE ARMS, as VALUES
# The six arms of P2. `slice_z_smoke.rs` gates them as EQUALITIES between two Rust marches, which
# is agreement and not correctness: two marches can agree with each other and both be wrong. Here
# each arm's trajectory is pinned against PYTHON's, on the seven keys the suites compare.
m64, _ = cas()._stator_march(FLIGHT, LO, HI, R, SETTLE, DS)
points("M/r66_to_64", m64, keys=PT7, branch=False)
m65, _ = cas(bleed_lim=valve(TAU))._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, surge=fuel())
points("M/r66_to_65", m65, keys=PT7, branch=False)
m52, _ = cas()._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, surge=fuel(), lag=lag())
points("M/r66_to_52", m52, keys=PT7, branch=False)
x65, _ = cross(bleed_lim=valve(TAU))._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TMAX)
points("M/r67_to_65", x65, keys=PT7, branch=False)
x66, _ = cross(bleed_lim=valve(TAU))._stator_march(
    FLIGHT, LO, HI, R, SETTLE, DS, surge=fuel(), lag=AsymmetricLag(tau_att=TAU, tau_rel=3.0 * TAU))
points("M/r67_to_66", x66, keys=PT7, branch=False)
x47, _ = cross()._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TMAX, tau_gov=TAU_GOV)
points("M/r67_to_47", x47, keys=PT7, branch=False)
# and the rung-64 arm's THREE other arming modes, which `test_reduce_no_lags…` loops over
for i, kw in enumerate((dict(bleed=B), dict(bleed_sched=BleedSchedule(B, 0.65)),
                        dict(bleed_lim=valve()))):
    t, _ = cas(**kw)._stator_march(FLIGHT, LO, HI, R, SETTLE, DS)
    points("M/r66_to_64/arm%d" % i, t, keys=PT7, branch=False)

# ===================================================== N -- THE GRID SWEEP, and the STRIDE
# `test_the_headline_numbers_are_grid_converged`'s own three grids. **This is where s 5.24 (i)'s
# delivered-count finding is re-measured rather than inherited**: the chunk width is
# `len(ride)//n_sample`, so a different `ds` gives a different stride and possibly a different
# width. Every row emits `n_ride` AND the delivered `n_sample`.
mx_n = cross(bleed_lim=valve(TAU))
for i, ds_i in enumerate(DS_SWEEP):
    idt = mx_n.cross_identity(FLIGHT, LO, HI, TMAX, tau=TAU, tau_govs=(TAU,), n_sample=6,
                              r=R, s_settle=SETTLE, ds=ds_i)
    bil = mx_n.cross_bill(FLIGHT, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, r=R, s_settle=SETTLE,
                          ds=ds_i)
    f("N/%d/ds" % i, ds_i)
    f("N/%d/P_mid" % i, idt["rows"][0]["P_mid"])
    d("N/%d/n_ride" % i, idt["rows"][0]["n_ride"])
    d("N/%d/n_sample" % i, idt["rows"][0]["n_sample"])
    d("N/%d/n_requested" % i, 6)
    f("N/%d/I_T" % i, bil["cells"]["both"]["I_T"])
    f("N/%d/I_phi" % i, bil["cells"]["both"]["I_phi"])
    f("N/%d/credit_T_gov" % i, bil["credit_T"]["gov"])

# ---------------------------------------------------------------------------- emit
print("# slice Z step 4 -- rungs 66+67 ORACLE, the SUITES' grid, uncoarsened. key<TAB>u64 "
      "(floats are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
