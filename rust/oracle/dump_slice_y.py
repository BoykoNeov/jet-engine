"""SLICE Y step 4 -- THE ORACLE for rung 65 (`LaggedBleedTransient`).

Step 2's `slice_y_smoke.rs` is a first-contact check against four hand-transcribed anchors on a
deliberately coarse grid. **This runs the SUITE's own grid, and it does not coarsen anywhere** --
which is P8's promise discharged rather than negotiated, and the header states the grid rather
than implying it (slice S step 4's lesson: *a probe's HEADER claimed the suites' grids and its
code ran another*).

    ds        0.005 in A/B/C/D/E -- `tests/test_rung65.py`'s own `DS`, and
              0.01 in F/G/H, which is ALSO the suite's own: its reduce gates
              (`_reduce_no_lag`, `_dormant_floor`, `_b0_none`), its `b_at_point` gate and its
              continuum-edge gate every one of them march at 0.01. **NOTHING HERE IS COARSER
              THAN THE GATE IT MIRRORS.**
    taus      (0.4, 0.2, 0.1, 0.05, 0.02, 0.01) -- the suite's `TAUS`, in the suite's order
    phi/b     PHI 0.80, B 0.10, SM = PHI/FLOOR - 1, TAU 0.05 -- the suite's throughout
    r 0.5 . s_settle 1.2 . Tt4 1000 -> 1400 . FLOOR 0.55 -- the suite's throughout
    maps      `shaped` ONLY (LP a=.20 b=.05 l=.7 / HP a=.08 b=.15 l=1.0). Slice X ran BOTH its
              shapes because rung 64's headline is a RATIO between two spools' bills. Rung 65's
              is a BANDWIDTH sweep on ONE spool's floor and `test_rung65.py` never builds a
              second shape -- so a `tilted` arm here would be a grid the suite does not have.

**WHY IT COSTS NOTHING TO REFUSE TO COARSEN, MEASURED BEFORE THE SECTIONS WERE CHOSEN.** On PyPy:
`bandwidth_ceiling` at ds=0.005 is 3.4 s, `marginal_mode` 6.2 s, `fuel_authority` 0.2 s and one
lagged march 0.2 s. Slice X coarsened three sections because one floored `_bill_cell` there was
1 753 outer solves; rung 65's readers are marches, not nested root sweeps, so the whole dump is
seconds. The number was measured first and the grid chosen from it --
[[rust-port-guessed-census-bars]], *five typed count bars, five wrong*.

**THE ONE THING THAT WAS MEASURED RATHER THAN ASSUMED, AND IT IS THE ADVISOR'S QUESTION.**
`marginal_mode`'s `laws_held` is `float("nan")` when a cell has NO riding points, and Python's
`max()` propagates that NaN while Rust's `f64::max` DISCARDS it -- two different functions, and
a divergence no value key would show if the NaN path is never reached. Measured on this grid:
`n_ride` is 340 / 251 / 214 on natural/lo/hi and 340 on both taucells, so **the NaN path never
fires and the two spellings cannot be told apart here.** That is a zero, so it is not left to a
value key: step 5 gates it by hand ([[rust-port-slice-w-step3]]).

Every float is emitted as its IEEE-754 bit pattern. Regenerate BOTH arms:
    .venv\\Scripts\\python.exe rust\\oracle\\dump_slice_y.py > rust\\oracle\\slice_y_pypy.tsv
    C:\\Python314\\python.exe  rust\\oracle\\dump_slice_y.py > rust\\oracle\\slice_y_cpython.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap, LaggedBleedTransient,
    LimitedBleedTransient, BleedLimiter, BleedSchedule, SurgeLimiter,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


# ---------------------------------------------------------------------------- the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
DS_C = 0.01                      # the SUITE's own for its reduce / b_at_point / edge gates
N_LO, B = 0.65, 0.10
PHI = 0.80
SM = PHI / FLOOR - 1.0
TAU = 0.05
TAUS = (0.4, 0.2, 0.1, 0.05, 0.02, 0.01)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """The suites' `_cpg`, character for character. `R_c` is DERIVED from `(gamma - 1)/gamma`
    and re-spelling it `0.4/1.4` builds a gas ONE ULP away: `1.4 - 1.0` is
    `0.3999999999999999`, not the double nearest `0.4`. Six slice-Y probe files carried that
    misspelling and it moved `nu0_lp` by seven ulps -- presenting exactly as a port defect."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def gt(**kw):
    """The suite's `_gt` -- a rung-65 machine on the shaped maps."""
    return LaggedBleedTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def lt64(**kw):
    """A rung-64 machine on the SAME hardware -- the reduce's reference."""
    return LimitedBleedTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def valve(tau=None, phi_lim=PHI, b_max=B):
    return BleedLimiter(phi_lim=phi_lim, b_max=b_max, tau=tau)


BILL = ("nu_at_min_lp", "s_at_min_lp", "b_at_min_lp", "plateau_span", "min_phi_lp",
        "min_phi_hp", "m_i_lp", "m_i_hp", "b_int", "b_peak", "b_end", "thrust_int",
        "thrust_end", "nu_lp_end", "nu_hp_end", "Tt4_peak", "nu0_lp", "nu0_hp")

# The 16 keys `_integrate_fuel_valve_lag` records. The first 14 are rung 64's, BYTE-UNCHANGED
# (the rung's own claim); `b` and `b_cmd` are the two rung 65 adds. `branch` is a string and is
# emitted as a discrete, `dump_fuel_transient.py`'s idiom.
PT_F = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
        "mdot_air", "sp_thrust", "mf", "mf_sched", "b", "b_cmd")
# The 7 the suite's own `_march_keys` compares -- the reduce's currency.
PT_R = ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf")


def cell(tag, c):
    for k in BILL:
        f("%s/%s" % (tag, k), c[k])
    d("%s/plateau_pts" % tag, c["plateau_pts"])
    d("%s/npts" % tag, c["npts"])


def points(tag, traj, keys=PT_F, branch=True):
    d("%s/npts" % tag, len(traj))
    for i, p in enumerate(traj):
        for k in keys:
            f("%s/%d/%s" % (tag, i, k), p[k])
        if branch:
            d("%s/%d/branch_choked" % (tag, i), 1 if p["branch"] == "choked" else 0)


# ====================================================== A -- `bandwidth_ceiling`, HALF ONE
# The suite's own call, argument for argument, from `test_bandwidth_is_pure_loss_on_both_axes`
# and three siblings. Rows are emitted by INDEX over the caller's `taus` order, never by
# iterating a dict: Python keys `cells` by float and the order is an artefact of insertion.
bc = gt().bandwidth_ceiling(FLIGHT, LO, HI, PHI, B, taus=TAUS, r=R, s_settle=SETTLE, ds=DS)
for k in ("phi_lim", "b_cap", "r", "ds", "inst_min_phi", "inst_b_int", "inst_d_min_phi_hp"):
    f("A/%s" % k, bc[k])
d("A/inst_plateau_pts", bc["inst_plateau_pts"])
d("A/n_taus", len(bc["taus"]))
for k in ("under_monotone", "bint_monotone", "dev_shrinks"):
    b("A/%s" % k, bc[k])
for i, tau in enumerate(TAUS):
    f("A/taus/%d" % i, bc["taus"][i])
    row = bc["rows"][i]
    for k in ("tau", "min_phi_lp", "undershoot", "b_int", "b_peak", "b_end", "plateau_span",
              "s_at_min_lp", "b_at_min_lp", "dev", "d_nu_lp_end", "thrust_end_pct",
              "thrust_int_pct", "d_min_phi_hp", "max_track"):
        f("A/rows/%d/%s" % (i, k), row[k])
    d("A/rows/%d/plateau_pts" % i, row["plateau_pts"])
    b("A/rows/%d/saturated" % i, row["saturated"])
    cell("A/cells/tau%d" % i, bc["cells"][tau])
cell("A/cells/shut", bc["cells"]["shut"])
cell("A/cells/inst", bc["cells"]["inst"])

# ====================================================== B -- `marginal_mode`, HALF TWO, THE RUNG
mm = gt().marginal_mode(FLIGHT, LO, HI, SM, B, tau=TAU, taus=(0.2, 0.01), d_b0=0.01,
                        r=R, s_settle=SETTLE, ds=DS)
for k in ("sm", "tau", "b_cap", "d_b0", "r", "ds", "phi_lim", "b_natural", "frozen",
          "db_db0", "dremoved", "laws_held", "tau_span", "tau_span_rel"):
    f("B/%s" % k, mm[k])
b("B/interior", mm["interior"])
for i, t in enumerate(mm["taus"]):
    f("B/taus/%d" % i, t)


def mcell(tag, c):
    for k in ("b0", "b_end", "drift", "dbds", "removed", "min_phi_lp", "laws_held"):
        f("%s/%s" % (tag, k), c[k])
    b("%s/interior" % tag, c["interior"])
    d("%s/n_ride" % tag, c["n_ride"])
    d("%s/npts" % tag, c["npts"])


mcell("B/natural", mm["natural"])
mcell("B/moved/lo", mm["moved"]["lo"])
mcell("B/moved/hi", mm["moved"]["hi"])
for i, t in enumerate(mm["taus"]):
    mcell("B/taucells/%d" % i, mm["taucells"][t])

# ====================================================== C -- `fuel_authority`, THE DISCRIMINATOR
fa = gt().fuel_authority(FLIGHT, LO, HI, SM, B, tau=TAU, r=R, s_settle=SETTLE, ds=DS)
for k in ("sm", "tau", "b_cap", "phi_lim", "ratio"):
    f("C/%s" % k, fa[k])
for k in ("deleted", "restored"):
    b("C/%s" % k, fa[k])
for i, x in enumerate(fa["fracs"]):
    f("C/fracs/%d" % i, x)
for k in ("s", "nu_lp", "nu_hp", "mf", "b", "phi_lp"):
    f("C/at/%s" % k, fa["at"][k])
for name in ("inst", "lagged"):
    side = fa[name]
    for k in ("span", "max_abs_G"):
        f("C/%s/%s" % (name, k), side[k])
    for k in ("monotone", "sign_change"):
        b("C/%s/%s" % (name, k), side[k])
    for i in range(len(fa["fracs"])):
        f("C/%s/phis/%d" % (name, i), side["phis"][i])
        f("C/%s/G/%d" % (name, i), side["G"][i])

# ====================================================== D -- THE LAGGED MARCH, PER POINT
# The cell rung 65 CREATES, at the suite's own DS. `b`/`b_cmd` make the TRACKING ERROR readable
# straight off a trajectory, so the state itself -- not a summary of it -- is what gets pinned.
# `b_at_point` is walked beside it: on a lagged machine the rung-65 override RETURNS THE RECORDED
# POSITION, and a port that re-solved would hand back `b_cmd`, which is a DIFFERENT number at
# every point where the valve is behind. That difference is dumped as `track` so the two cannot
# silently agree.
m_lag = gt(bleed_lim=valve(TAU))
traj = m_lag._stator_march(FLIGHT, LO, HI, R, SETTLE, DS)[0]
points("D", traj)
for i, p in enumerate(traj):
    f("D/%d/b_at_point" % i, m_lag.b_at_point(FLIGHT, p))
    f("D/%d/track" % i, p["b"] - p["b_cmd"])

# ====================================================== E -- THE SATURATED CASE, gate 4's own
# `test_at_the_stop_bandwidth_buys_nothing_confirming_rung64`, argument for argument. A floor
# ABOVE the fully-open march's own minimum commands `b_max` throughout, so under a lag it is a
# bare exponential approach with NO feedback content -- Python's docstring is explicit that this
# must not be read together with the riding case, and `saturated` is dumped per cell so a reader
# here cannot mix them either.
m_e = gt()
ARGS_E = (FLIGHT, LO, HI, R, SETTLE, DS)
over = m_e.at_lever(bleed=B)._bill_cell(*ARGS_E)["min_phi_lp"] * 1.10
f("E/over", over)
ref = m_e.at_lever(bleed_lim=valve(phi_lim=over))._bill_cell(*ARGS_E)
cell("E/ref", ref)
b("E/ref_violated", ref["min_phi_lp"] < over)
for i, tau in enumerate((0.01, 0.05, 0.2)):
    c = m_e.at_lever(bleed_lim=valve(tau, phi_lim=over))._bill_cell(*ARGS_E)
    cell("E/tau%d" % i, c)
    f("E/tau%d/tau" % i, tau)
    b("E/tau%d/saturated" % i, c["b_peak"] >= B * (1.0 - 1e-12))
    f("E/tau%d/d_min_phi_lp" % i, c["min_phi_lp"] - ref["min_phi_lp"])

# ====================================================== F -- THE `b0` CONTINUUM AND ITS EDGE
# Gate 6's own construction at the suite's own 0.01, and THE `MarchScope` CELL slice Y step 1
# opened: `b0` is a per-march argument threaded through `_stator_march`, so a port that dropped
# it from the scope would march the natural condition three times and report three identical
# drifts. All three trajectories are dumped, not just the drifts.
m_f = gt().at_lever(bleed_lim=BleedLimiter.from_margin(LP, B, SM, tau=TAU))
fuel_f = SurgeLimiter.from_margin(LP, "lp", SM)
f("F/valve/phi_lim", m_f.bleed_lim.phi_lim)
f("F/valve/b_max", m_f.bleed_lim.b_max)
f("F/valve/tau", m_f.bleed_lim.tau)
f("F/fuel/phi_lim", fuel_f.phi_lim)
nat_f = m_f._stator_march(FLIGHT, LO, HI, R, SETTLE, DS_C, surge=fuel_f)[0]
edge = nat_f[0]["b"]
f("F/edge", edge)
points("F/nat", nat_f, keys=("s", "nu_lp", "nu_hp", "phi_lp", "mf", "mf_sched", "b", "b_cmd"),
       branch=False)
f("F/nat/drift", max(abs(p["b"] - nat_f[0]["b"]) for p in nat_f))
f("F/nat/removed", m_f._removed(nat_f))
for lbl, x in (("in", 0.99 * edge), ("on", edge), ("out", 1.01 * edge)):
    t = m_f._stator_march(FLIGHT, LO, HI, R, SETTLE, DS_C, surge=fuel_f, b0=x)[0]
    f("F/%s/b0" % lbl, x)
    f("F/%s/drift" % lbl, max(abs(p["b"] - t[0]["b"]) for p in t))
    f("F/%s/b_end" % lbl, t[-1]["b"])
    f("F/%s/removed" % lbl, m_f._removed(t))
    d("F/%s/npts_x" % lbl, len(t))
    points("F/%s" % lbl, t, keys=("s", "phi_lp", "mf", "mf_sched", "b", "b_cmd"), branch=False)
# THE GUARD RESTORES THE PREVIOUS VALUE, and the only value key that could ever see it is one
# taken AFTER the march: `_b0` must be back to `None` and nothing may be left behind.
b("F/b0_restored", m_f._b0 is None)
b("F/b_state_restored", m_f._b_state is None)
b("F/b_forced_restored", m_f._b_forced is None)

# ====================================================== G -- THE REDUCE ARMS, AS VALUES
# Arm one of P2: an UNLAGGED rung-65 machine is rung 64 bit-for-bit at every arming mode. The
# suite asserts the two marches equal; the oracle dumps BOTH SIDES, because an equality between
# two Rust marches is satisfied by two identically-wrong ports.
for name, kw in (("shut", dict()), ("constant", dict(bleed=B)),
                 ("schedule", dict(bleed_sched=BleedSchedule(B, N_LO))),
                 ("floor", dict(bleed_lim=valve()))):
    a65 = gt(**kw)._stator_march(FLIGHT, LO, HI, R, SETTLE, DS_C)[0]
    a64 = lt64(**kw)._stator_march(FLIGHT, LO, HI, R, SETTLE, DS_C)[0]
    points("G/r65/%s" % name, a65, keys=PT_R, branch=False)
    points("G/r64/%s" % name, a64, keys=PT_R, branch=False)
    b("G/equal/%s" % name,
      [tuple(p[k] for k in PT_R) for p in a65] == [tuple(p[k] for k in PT_R) for p in a64])
# A DORMANT floor must reach the rung-63 grandparent at every state, not merely agree closely.
g_dorm = gt().at_lever(bleed_lim=valve(phi_lim=0.30))._stator_march(
    FLIGHT, LO, HI, R, SETTLE, DS_C)[0]
points("G/dormant", g_dorm, keys=PT_R, branch=False)
# Arm two: `b0` passed explicitly AT the value the march would have chosen is bit-for-bit.
m_g = gt(bleed_lim=valve(TAU))
g_a = m_g._stator_march(FLIGHT, LO, HI, R, SETTLE, DS_C)[0]
g_b = m_g._stator_march(FLIGHT, LO, HI, R, SETTLE, DS_C, b0=g_a[0]["b"])[0]
points("G/b0/auto", g_a, keys=("s", "phi_lp", "mf", "b", "b_cmd"), branch=False)
points("G/b0/given", g_b, keys=("s", "phi_lp", "mf", "b", "b_cmd"), branch=False)
b("G/b0/equal",
  [tuple(p[k] for k in PT_R) for p in g_a] == [tuple(p[k] for k in PT_R) for p in g_b])
b("G/b0/is_command", g_a[0]["b"] == g_a[0]["b_cmd"])
b("G/b0/rides_at_zero", g_a[0]["b"] > 0.0)

# ====================================================== H -- `integrate_fuel` AS A CELL
# Slice Y step 1 made `integrate_fuel` a cell (its first overrider being this rung, 10 in all)
# and typed it on `&dyn Fn` because a fn-pointer table cannot hold a generic. `_stator_march`
# reaches it through one fixed schedule shape, so the cell is exercised here DIRECTLY, with each
# leg of `der`'s min-select armed in turn -- otherwise the `accel` and `Tt4_max` arms of that
# select are a branch no key in this dump reaches.
NU0 = (0.75, 0.79)
S_END = 1.0


def sched_h(s):
    """A hard-coded ramp inside the march's own measured fuel band (0.0094 -> 0.0234 kg/s).
    Spelled with the SAME associativity on both sides: `0.0095 + 0.014 * s`."""
    return 0.0095 + 0.014 * s


surge_h = SurgeLimiter.from_margin(LP, "lp", SM)
accel_h = gt().accel_schedule(FLIGHT, LO, HI, 0.10)
d("H/accel/n", len(accel_h.n_H))
f("H/accel/margin", accel_h.margin)
for i in range(len(accel_h.n_H)):
    f("H/accel/n_H/%d" % i, accel_h.n_H[i])
    f("H/accel/kappa/%d" % i, accel_h.kappa[i])
for name, kw in (("bare", dict()),
                 ("surge", dict(surge=surge_h)),
                 ("topping", dict(Tt4_max=1450.0)),
                 ("accel", dict(accel=accel_h)),
                 ("all_three", dict(surge=surge_h, Tt4_max=1450.0, accel=accel_h)),
                 ("freeze_lp", dict(freeze="lp", surge=surge_h)),
                 ("freeze_hp", dict(freeze="hp", surge=surge_h))):
    pts = gt(bleed_lim=valve(TAU)).integrate_fuel(FLIGHT, sched_h, NU0, S_END, DS_C, **kw)
    points("H/%s" % name, pts)
# THE UNLAGGED PATH THROUGH THE SAME CELL: `integrate_fuel` on a rung-65 machine with no lag
# must land on rung 43's body via `super()`, so the same call on a rung-64 machine agrees.
h65 = gt().integrate_fuel(FLIGHT, sched_h, NU0, S_END, DS_C, surge=surge_h)
h64 = lt64().integrate_fuel(FLIGHT, sched_h, NU0, S_END, DS_C, surge=surge_h)
points("H/nolag/r65", h65, keys=PT_R, branch=False)
points("H/nolag/r64", h64, keys=PT_R, branch=False)
b("H/nolag/equal",
  [tuple(p[k] for k in PT_R) for p in h65] == [tuple(p[k] for k in PT_R) for p in h64])

# ====================================================== I -- `_lagged`, `_removed`, `at_lever`
# The three plain cells no value above reaches on its own, and the sibling constructor whose
# dropped-lever trap rungs 61/62/63/64 each hit once.
for name, m in (("bare", gt()), ("floor", gt(bleed_lim=valve())),
                ("lagged", gt(bleed_lim=valve(TAU))), ("const", gt(bleed=B))):
    b("I/lagged/%s" % name, m._lagged())
sib = gt(bleed_lim=valve(TAU))
b("I/at_lever/keeps_lag", sib.at_lever(bleed_lim=valve(TAU)).bleed_lim.tau == TAU)
b("I/at_lever/isolates", sib.at_lever().bleed_lim is None)
b("I/at_stator/keeps_lag", sib.at_stator().bleed_lim.tau == TAU)
f("I/removed/lagged_march", sib._removed(traj))
f("I/removed/nat_f", m_f._removed(nat_f))

# ---------------------------------------------------------------------------- emit
print("# slice Y step 4 -- rung 65 ORACLE, the SUITE's grid, uncoarsened. key<TAB>u64 "
      "(floats are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
