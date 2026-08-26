"""SLICE X step 2 — the SMOKE dump for rung 64 (`BleedLimiter` + `LimitedBleedTransient`).

Not the slice's oracle (that is step 4, on the suite's own grid). This exists to catch a
structural mistake BEFORE the 23 Python gates are ported on top of it at step 3 — and
§ 5.22's probes named the mistakes in advance, each of which the shipped Rust deliberately
does NOT make:

  1. **`b_at_point` RECONSTRUCTING INSTEAD OF RE-SOLVING.** § 5.22 (ii) measured that this
     drives a floored march's `b_int` and `b_peak` to EXACTLY 0 and both published ratios to
     0 — with all 111 rung-62/63/64 Python gates still green, because the only assertion
     reading them is an ordering that zeroing the smallest term satisfies. Sections D/F carry
     `b_int`, `b_peak`, `b_end`, `b_at_min_lp` and both ratios.
  2. **`R62`'s `b_at_point` SLOT DEFAULTED TO `b_of`.** Right on a rung-62 machine, wrong on
     a floored one — a claim no value gate could see. The Rust points that slot at a PANIC;
     section B reads `b_at_point` on a machine with no floor, which is the leg that must
     still answer.
  3. **`at_stator` LEFT AS RUNG 62's**, which sets `bleed_lim=None` deliberately. Section I
     reads the sibling's own arming and its `b_at_point`, so a dropped floor is a wrong
     number rather than a missing method.
  4. **`_isolating`'s `want` LEFT TWO-WAY.** Rung 64's override IS that one term; the
     assert's other side is dispatched and already gains the floor, so a two-way `want` fires
     the assert on a FLOORED NEIGHBOUR. Section H isolates a stator against exactly that
     neighbour — the case rung 63's body cannot express.
  5. **`_b_forced` LEAKING PAST THE TRIAL.** A leaked trial position makes the closure report
     a state the plant never visited. Section B reads `b_of` immediately after a completed
     solve, and section C marches, where a leak would move every downstream point.

**THE GRID IS NOT THE SUITE'S, AND THAT IS DELIBERATE — P9.** Every marched reader here runs
at `ds = 0.02` **except section G, which runs at 0.01 and says why in place**; `tests/test_rung64.py` runs at `ds = 0.005`. Probe 9 measured one floored
`_bill_cell` at 1 753 outer solves / 7 385 closure evaluations at `ds = 0.005` against 478 /
2 068 at `ds = 0.02`, and the three top-level readers at 0.13 s / 0.50 s / 0.76 s at `ds =
0.02` on PyPy. Step 4's oracle runs the suite's own `ds`; this file trades resolution for a
fast pre-check and says so rather than implying otherwise.

Every float is emitted as its IEEE-754 bit pattern, so the comparison is bit-equality and not
a tolerance. Regenerate with:
    .venv\\Scripts\\python.exe rust\\oracle\\dump_slice_x_smoke.py > rust\\oracle\\slice_x_smoke_pypy.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap, LimitedBleedTransient,
    BleedLimiter, BleedSchedule, SurgeLimiter,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


# ---------------------------------------------------------------------------- the grid
# tests/test_rung64.py's OWN module-level constants, copied so this file is self-contained and
# a change to the suite shows up as a golden mismatch rather than moving silently underneath.
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, SETTLE = 1000.0, 1400.0, 1.2
DS = 0.02                        # <- NOT the suite's 0.005; see the module docstring
N_LO, B, R = 0.65, 0.10, 0.5
PHI = 0.80                       # strictly inside [0.7354 shut, 0.8095 fully open]
SM = PHI / FLOOR - 1.0
V = 0.20

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def lt(**kw):
    return LimitedBleedTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


VALVE = BleedLimiter(phi_lim=PHI, b_max=B)

# ============================================================ A -- THE DEVICE
f("A/new/phi_lim", VALVE.phi_lim)
f("A/new/b_max", VALVE.b_max)
b("A/new/tau_is_none", VALVE.tau is None)

FM = BleedLimiter.from_margin(LP, B, SM)
f("A/from_margin/phi_lim", FM.phi_lim)
f("A/from_margin/b_max", FM.b_max)
b("A/from_margin/tau_is_none", FM.tau is None)
# from_margin off the map's OWN imposed surge line, so rung 49's floor and this one are set in
# identical units -- the equality is the whole reason rung 63 s 3's band edges are comparable.
b("A/from_margin/matches_rung49_units",
  FM.phi_lim == SurgeLimiter.from_margin(LP, "lp", SM).phi_lim)

LG = VALVE.lagged(0.05)
f("A/lagged/phi_lim", LG.phi_lim)
f("A/lagged/b_max", LG.b_max)
f("A/lagged/tau", LG.tau)

# ============================================================ B -- `b_of`, `b_at_point`, THE REDUCE
FLOORED = lt(bleed_lim=VALVE)
BARE = lt()
CONST = lt(bleed=B)

# `b_of` on a FLOORED machine with no solve in flight falls THROUGH to rung 62's constant,
# which is 0.0. That is the dead branch of s 5.22 (vi) read directly -- and it is exactly why
# reconstructing `b_at_point` from `b_of` zeroes the bleed integral.
for i, nu in enumerate((0.70, 0.85, 1.00)):
    f("B/b_of/floored/%d" % i, FLOORED.b_of(nu))
    f("B/b_of/const/%d" % i, CONST.b_of(nu))
    f("B/b_of/bare/%d" % i, BARE.b_of(nu))

traj_f, nu0_f = FLOORED._stator_march(FLIGHT, LO, HI, R, SETTLE, DS)
d("B/march/npts", len(traj_f))
IDX = [0, len(traj_f) // 3, 2 * len(traj_f) // 3, len(traj_f) - 1]
for j, i in enumerate(IDX):
    f("B/b_at_point/floored/%d" % j, FLOORED.b_at_point(FLIGHT, traj_f[i]))
    # the `bleed_lim is None` leg -- the one R62's slot would answer WRONGLY if defaulted
    f("B/b_at_point/const/%d" % j, CONST.b_at_point(FLIGHT, traj_f[i]))
    f("B/b_at_point/bare/%d" % j, BARE.b_at_point(FLIGHT, traj_f[i]))
# A LEAKED `_b_forced` would show here: after a completed solve the carrier must be clear
# again, so `b_of` must still read rung 62's constant.
f("B/b_of/after_solve", FLOORED.b_of(0.85))
b("B/armed/floored", FLOORED._armed_bleed())
b("B/armed/bare", BARE._armed_bleed())
b("B/armed/const", CONST._armed_bleed())

# ============================================================ C -- ONE FLOORED MARCH
for j, i in enumerate(IDX):
    p = traj_f[i]
    for k in ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "pi_lpc", "sp_thrust"):
        f("C/floored/%d/%s" % (j, k), p[k])
f("C/floored/nu0_lp", nu0_f[0])
f("C/floored/nu0_hp", nu0_f[1])

# ============================================================ D -- `_bill_cell`, FOUR LAWS
SCHED = BleedSchedule(B, N_LO)
LAWS = {
    "shut": BARE,
    "constant": CONST,
    "schedule": lt(bleed_sched=SCHED),
    "floor": FLOORED,
}
BILL_KEYS = ("nu_at_min_lp", "s_at_min_lp", "b_at_min_lp", "plateau_span",
             "min_phi_lp", "min_phi_hp", "m_i_lp", "m_i_hp", "b_int", "b_peak", "b_end",
             "thrust_int", "thrust_end", "nu_lp_end", "nu_hp_end", "Tt4_peak",
             "nu0_lp", "nu0_hp")
for name, m in LAWS.items():
    c = m._bill_cell(FLIGHT, LO, HI, R, SETTLE, DS)
    for k in BILL_KEYS:
        f("D/%s/%s" % (name, k), c[k])
    d("D/%s/plateau_pts" % name, c["plateau_pts"])
    d("D/%s/npts" % name, c["npts"])
    b("D/%s/has_traj" % name, "traj" in c)
# the rung-65 key, ADDED and not defaulted -- an un-asking caller must get the SAME key set
ckt = FLOORED._bill_cell(FLIGHT, LO, HI, R, SETTLE, DS, keep_traj=True)
b("D/keep_traj/has_traj", "traj" in ckt)
d("D/keep_traj/traj_len", len(ckt["traj"]))
d("D/keep_traj/extra_keys", len(set(ckt) - set(FLOORED._bill_cell(
    FLIGHT, LO, HI, R, SETTLE, DS))))

# ============================================================ E -- `authority_ceiling`
ac = BARE.authority_ceiling(FLIGHT, LO, HI, b_max=B, n_lo=N_LO, r=R, s_settle=SETTLE, ds=DS)
for k in ("r", "ds", "b_max", "phi_surge", "ceiling", "phi_lim_over", "gap_schedule",
          "b_at_sched_min", "over_deficit", "over_vs_full"):
    f("E/%s" % k, ac[k])
b("E/sched_saturated", ac["sched_saturated"])
b("E/violated", ac["violated"])
b("E/bounded_by_full", ac["bounded_by_full"])
for name in ("shut", "schedule", "full", "over"):
    for k in ("min_phi_lp", "b_int", "b_peak", "b_at_min_lp", "nu_lp_end", "thrust_end"):
        f("E/cells/%s/%s" % (name, k), ac["cells"][name][k])
    d("E/cells/%s/plateau_pts" % name, ac["cells"][name]["plateau_pts"])

# ============================================================ F -- `matched_bill`, THE RUNG
mb = BARE.matched_bill(FLIGHT, LO, HI, phi_target=PHI, b_cap=B, n_lo=N_LO, r=R,
                       s_settle=SETTLE, ds=DS)
for k in ("r", "ds", "phi_target", "b_cap", "n_lo", "b_star", "bmax_star", "matched",
          "b_ratio_const", "b_ratio_sched"):
    f("F/%s" % k, mb[k])
b("F/saturated", mb["saturated"])
for name in ("constant", "schedule", "floor"):
    for k in ("d_nu_lp_end", "d_nu_hp_end", "d_thrust_end", "thrust_end_pct",
              "thrust_int_pct", "d_min_phi_hp", "b_int", "b_peak"):
        f("F/bill/%s/%s" % (name, k), mb["bill"][name][k])
for name in ("shut", "constant", "schedule", "floor"):
    for k in ("min_phi_lp", "b_int", "nu_lp_end", "thrust_end"):
        f("F/cells/%s/%s" % (name, k), mb["cells"][name][k])
    d("F/cells/%s/plateau_pts" % name, mb["cells"][name]["plateau_pts"])

# ============================================================ G -- `floor_refusal`
# **SECTION G RUNS AT ds = 0.01, NOT AT THIS FILE'S ds = 0.02, AND THAT IS A MEASUREMENT.**
# `inert` -- rung 64's own claim (i), that the composite IS the valve-alone march -- is True at
# ds = 0.005 and at 0.01 and **False at 0.02**: the coarse march moves `m_i` by 2.894e-04, four
# orders above the 1e-14 bar, while `min_phi` still agrees to 1.1e-16. So the flip is the
# PARABOLA-REFINED minimum moving on a coarser grid, not the physics changing. Left at 0.02 this
# section would have published `G/inert = 0` as a bit-exact golden and read as a refutation of
# the rung. Both grids are emitted so the flip is GATED rather than avoided.
DS_G = 0.01
fr = BARE.floor_refusal(FLIGHT, LO, HI, sm=SM, b_cap=B, d_sm=0.01, r=R, s_settle=SETTLE,
                        ds=DS_G)
for k in ("sm", "d_sm", "phi_lim", "phi_lim_below", "r", "ds", "b_cap", "removed_alone",
          "removed_together", "credit", "removed_below_bare", "removed_below_armed"):
    f("G/%s" % k, fr[k])
b("G/inert", fr["inert"])
b("G/control_dormant", fr["control_dormant"])
for name in ("neither", "fuel", "valve", "both", "below_bare", "below_armed"):
    for k in ("m_i", "min_phi", "fuel_removed", "nu_lp_end", "nu_hp_end"):
        f("G/cells/%s/%s" % (name, k), fr["cells"][name][k])
# THE FLIP ITSELF, gated: same reader, this file's own coarse ds.
frc = BARE.floor_refusal(FLIGHT, LO, HI, sm=SM, b_cap=B, d_sm=0.01, r=R, s_settle=SETTLE,
                         ds=DS)
b("G/coarse/inert", frc["inert"])
b("G/coarse/control_dormant", frc["control_dormant"])
f("G/coarse/credit", frc["credit"])
f("G/coarse/d_m_i", abs(frc["cells"]["both"]["m_i"] - frc["cells"]["valve"]["m_i"]))
f("G/coarse/d_min_phi",
  abs(frc["cells"]["both"]["min_phi"] - frc["cells"]["valve"]["min_phi"]))

# ============================================================ H -- `_isolating`, THE THREE-WAY `want`
# THE CASE RUNG 63's BODY CANNOT EXPRESS: a FLOORED neighbour. Its `want` omits `bleed_lim`
# while the dispatched `ref._armed_bleed()` includes it, so the assert fires -- which is the
# entire content of rung 64's override.
ref, armed = FLOORED._isolating(dict(vsv_lp=V), neighbour=dict(bleed_lim=VALVE))
b("H/floored_neighbour/ref_armed", ref._armed_bleed())
b("H/floored_neighbour/armed_armed", armed._armed_bleed())
b("H/floored_neighbour/ref_is_armed_stator", ref._is_armed())
b("H/floored_neighbour/armed_is_armed_stator", armed._is_armed())
f("H/floored_neighbour/ref_bill_b_int",
  ref._bill_cell(FLIGHT, LO, HI, R, SETTLE, DS)["b_int"])
f("H/floored_neighbour/armed_bill_b_int",
  armed._bill_cell(FLIGHT, LO, HI, R, SETTLE, DS)["b_int"])
# and the plain leg, where 63's body and 64's agree
ref2, armed2 = BARE._isolating(dict(bleed_lim=VALVE))
b("H/plain/ref_armed", ref2._armed_bleed())
b("H/plain/armed_armed", armed2._armed_bleed())

# ============================================================ I -- `at_stator` CARRIES THE FLOOR
sib = FLOORED.at_stator(vsv_lp=V)
b("I/sibling_armed", sib._armed_bleed())
f("I/sibling_b_at_point", sib.b_at_point(FLIGHT, traj_f[IDX[2]]))
f("I/sibling_b_int", sib._bill_cell(FLIGHT, LO, HI, R, SETTLE, DS)["b_int"])
# rung 62's body would return a machine with NO floor, whose b_int is exactly 0.0
b("I/sibling_b_int_is_zero",
  sib._bill_cell(FLIGHT, LO, HI, R, SETTLE, DS)["b_int"] == 0.0)

# ---------------------------------------------------------------------------- emit
print("# slice X step 2 -- rung 64 SMOKE. key<TAB>u64 (float keys are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
