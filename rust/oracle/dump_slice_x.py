"""SLICE X step 4 — THE ORACLE for rung 64 (`BleedLimiter` + `LimitedBleedTransient`).

Step 2's smoke is a structural pre-check on a deliberately coarse grid. **This runs the SUITE's
own grid** and both of its map shapes, which is P9's promise, and the header states the grid
rather than implying it:

    ds        0.005  — `tests/test_rung64.py`'s own DS, plus 0.01 and 0.0025 where the suite
                       itself refines (its `the_tautology_is_exact_at_every_grid` and
                       `the_hp_debit_survives_grid_refinement` both sweep those three)
    shapes    BOTH — `shaped` (LP a=.20 b=.05 l=.7 / HP a=.08 b=.15 l=1.0) and `tilted`
                     (a=.14 b=.10 c=.06 sigma=.2 l=.85 on both spools), because the rung's
                     headline rests on a RATIO and the suite runs its two bill gates on both
    r         0.5, `s_settle` 1.2, `Tt4` 1000 -> 1400, the suite's throughout

**WHAT IS DELIBERATELY COARSER THAN THE SUITE, AND WHY.** Section B walks EVERY point of a floored
march and re-solves the valve at each, so it runs at `ds = 0.01` rather than 0.005 — at 0.005 that
one section would be ~700 outer solves for a reading whose content is the SHAPE of `b(s)`, which
0.01 already resolves. Sections F and G sweep set points and authorities at `ds = 0.01` for the
same reason: they exist to reach all THREE regimes by value, not to refine any one of them.
Probe 9's measurements are what these choices were made from — one floored `_bill_cell` is 478
outer solves / 2 068 closure evaluations at `ds = 0.02` and 1 753 / 7 385 at 0.005.

Every float is emitted as its IEEE-754 bit pattern. Regenerate BOTH arms:
    .venv\\Scripts\\python.exe rust\\oracle\\dump_slice_x.py > rust\\oracle\\slice_x_pypy.tsv
    C:\\Python314\\python.exe  rust\\oracle\\dump_slice_x.py > rust\\oracle\\slice_x_cpython.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap, LimitedBleedTransient,
    BleedLimiter, BleedSchedule,
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
LO, HI, SETTLE, R = 1000.0, 1400.0, 1.2, 0.5
DS = 0.005                       # the suite's own
DS_GRID = (0.01, 0.005, 0.0025)  # the suite's own refinement sweep
DS_WALK = 0.01                   # sections B/F/G -- see the module docstring
N_LO, B, PHI = 0.65, 0.10, 0.80
SM = PHI / FLOOR - 1.0

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
TILT = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
SHAPES = (("shaped", LP, HP), ("tilted", TILT, TILT))


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def lt(lp=LP, hp=HP, **kw):
    return LimitedBleedTransient(DESIGN, FLIGHT, 1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


BILL = ("nu_at_min_lp", "s_at_min_lp", "b_at_min_lp", "plateau_span", "min_phi_lp",
        "min_phi_hp", "m_i_lp", "m_i_hp", "b_int", "b_peak", "b_end", "thrust_int",
        "thrust_end", "nu_lp_end", "nu_hp_end", "Tt4_peak", "nu0_lp", "nu0_hp")


def laws(lp, hp):
    return (("shut", dict()),
            ("constant", dict(bleed=B)),
            ("schedule", dict(bleed_sched=BleedSchedule(B, N_LO))),
            ("floor", dict(bleed_lim=BleedLimiter(PHI, B))))


# ============================================================ A -- `_bill_cell`, 4 laws x 2 shapes x 3 ds
for shape, lp, hp in SHAPES:
    for name, kw in laws(lp, hp):
        for ds in DS_GRID:
            c = lt(lp, hp, **kw)._bill_cell(FLIGHT, LO, HI, R, SETTLE, ds)
            tag = "A/%s/%s/%g" % (shape, name, ds)
            for k in BILL:
                f("%s/%s" % (tag, k), c[k])
            d("%s/plateau_pts" % tag, c["plateau_pts"])
            d("%s/npts" % tag, c["npts"])

# ============================================================ B -- `b_at_point` ALONG A WHOLE MARCH
# THE CELL SLICE X CREATES, dumped densely: a reconstruction agrees with the re-solve NOWHERE
# except where the valve is shut, so a per-point walk is the strongest available reading of it.
for shape, lp, hp in SHAPES:
    m = lt(lp, hp, bleed_lim=BleedLimiter(PHI, B))
    traj = m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS_WALK)[0]
    d("B/%s/npts" % shape, len(traj))
    for i, p in enumerate(traj):
        f("B/%s/%d/b" % (shape, i), m.b_at_point(FLIGHT, p))
        f("B/%s/%d/phi_lp" % (shape, i), p["phi_lp"])

# ============================================================ C -- `authority_ceiling`
for shape, lp, hp in SHAPES:
    ac = lt(lp, hp).authority_ceiling(FLIGHT, LO, HI, b_max=B, n_lo=N_LO, r=R,
                                      s_settle=SETTLE, ds=DS)
    for k in ("r", "ds", "b_max", "phi_surge", "ceiling", "phi_lim_over", "gap_schedule",
              "b_at_sched_min", "over_deficit", "over_vs_full"):
        f("C/%s/%s" % (shape, k), ac[k])
    for k in ("sched_saturated", "violated", "bounded_by_full"):
        b("C/%s/%s" % (shape, k), ac[k])
    for name in ("shut", "schedule", "full", "over"):
        for k in BILL:
            f("C/%s/cells/%s/%s" % (shape, name, k), ac["cells"][name][k])
        d("C/%s/cells/%s/plateau_pts" % (shape, name), ac["cells"][name]["plateau_pts"])
        d("C/%s/cells/%s/npts" % (shape, name), ac["cells"][name]["npts"])

# ============================================================ D -- `matched_bill`, THE RUNG
for shape, lp, hp in SHAPES:
    mb = lt(lp, hp).matched_bill(FLIGHT, LO, HI, phi_target=PHI, b_cap=B, n_lo=N_LO,
                                 r=R, s_settle=SETTLE, ds=DS)
    for k in ("r", "ds", "phi_target", "b_cap", "n_lo", "b_star", "bmax_star", "matched",
              "b_ratio_const", "b_ratio_sched"):
        f("D/%s/%s" % (shape, k), mb[k])
    b("D/%s/saturated" % shape, mb["saturated"])
    for name in ("constant", "schedule", "floor"):
        for k in ("d_nu_lp_end", "d_nu_hp_end", "d_thrust_end", "thrust_end_pct",
                  "thrust_int_pct", "d_min_phi_hp", "b_int", "b_peak"):
            f("D/%s/bill/%s/%s" % (shape, name, k), mb["bill"][name][k])
    for name in ("shut", "constant", "schedule", "floor"):
        for k in BILL:
            f("D/%s/cells/%s/%s" % (shape, name, k), mb["cells"][name][k])
        d("D/%s/cells/%s/plateau_pts" % (shape, name), mb["cells"][name]["plateau_pts"])
        d("D/%s/cells/%s/npts" % (shape, name), mb["cells"][name]["npts"])

# ============================================================ E -- `floor_refusal`
for shape, lp, hp in SHAPES:
    fr = lt(lp, hp).floor_refusal(FLIGHT, LO, HI, sm=SM, b_cap=B, d_sm=0.01, r=R,
                                  s_settle=SETTLE, ds=DS)
    for k in ("sm", "d_sm", "phi_lim", "phi_lim_below", "r", "ds", "b_cap", "removed_alone",
              "removed_together", "credit", "removed_below_bare", "removed_below_armed"):
        f("E/%s/%s" % (shape, k), fr[k])
    for k in ("inert", "control_dormant"):
        b("E/%s/%s" % (shape, k), fr[k])
    for name in ("neither", "fuel", "valve", "both", "below_bare", "below_armed"):
        for k in ("m_i", "min_phi", "fuel_removed", "nu_lp_end", "nu_hp_end", "Tt4_peak",
                  "m_phi", "s"):
            f("E/%s/cells/%s/%s" % (shape, name, k), fr["cells"][name][k])

# ============================================================ F -- THE SET-POINT SWEEP: all three regimes
# By VALUE, never by the regime label -- Python returns the label from `_solve_b` and no reader
# reads it, so the dump reads what a reader CAN: b_peak == 0 (dormant throughout),
# 0 < b_peak < b_max (rides), b_peak == b_max (saturates), and whether the floor is delivered.
PHI_GRID = (0.30, 0.70, 0.7354, 0.76, 0.80, 0.8095, 0.95)
for shape, lp, hp in SHAPES:
    for phi in PHI_GRID:
        c = lt(lp, hp, bleed_lim=BleedLimiter(phi, B))._bill_cell(FLIGHT, LO, HI, R,
                                                                  SETTLE, DS_WALK)
        tag = "F/%s/%g" % (shape, phi)
        for k in ("b_int", "b_peak", "b_end", "min_phi_lp", "min_phi_hp", "nu_lp_end",
                  "thrust_int", "m_i_lp"):
            f("%s/%s" % (tag, k), c[k])
        d("%s/plateau_pts" % tag, c["plateau_pts"])
        b("%s/dormant" % tag, c["b_peak"] == 0.0)
        b("%s/saturated" % tag, c["b_peak"] >= B)
        b("%s/delivered" % tag, c["min_phi_lp"] >= phi * (1.0 - 1e-9))

# ============================================================ G -- THE AUTHORITY SWEEP
# The ceiling belongs to `b_max`, so the same set point against four valve sizes is the rung's
# claim read as a monotone family rather than as a single pair.
for bmax in (0.02, 0.05, 0.10, 0.20):
    c = lt(bleed_lim=BleedLimiter(PHI, bmax))._bill_cell(FLIGHT, LO, HI, R, SETTLE, DS_WALK)
    tag = "G/%g" % bmax
    for k in ("b_int", "b_peak", "min_phi_lp", "nu_lp_end", "thrust_int"):
        f("%s/%s" % (tag, k), c[k])
    b("%s/saturated" % tag, c["b_peak"] >= bmax)
    b("%s/delivered" % tag, c["min_phi_lp"] >= PHI * (1.0 - 1e-9))
    d("%s/plateau_pts" % tag, c["plateau_pts"])

# ---------------------------------------------------------------------------- emit
print("# slice X step 4 -- rung 64 ORACLE, the SUITE's grid. key<TAB>u64 (floats are IEEE-754 "
      "bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
