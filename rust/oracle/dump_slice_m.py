"""SLICE M's value oracle — rungs 53 + 54, on the grid § 5.9 PRE-REGISTERED and no other.

    2 gases (cpg, tpg) x 5 disclosed shapes x 4 throttles x 2 spools = 80 cells
    equilibrium: the SAME shapes and spools at 2 throttles = 20 cells

THE GRID IS THE PRE-REGISTERED ONE, NOT A NEIGHBOURING ONE (§ 5.7 (e)). The pass-count sets in
§ 5.9 (iv), the `binds` census in (vii) and the three field-set splits in (vi) were all measured
HERE; a dump that swept a slightly different grid would leave every one of those bars asserting
over unmeasured cells.

THE EQUILIBRIUM ARM IS A SEPARATE PROCESS WRITING ITS OWN FILE. Measured at ~44 s per `_scan`,
it is ~15 min even sampled at 2 throttles, and the FIRST attempt at that measurement died at a
15-minute cap with its output still buffered, having recorded nothing. So: unbuffered, per-cell
progress to stderr, and its own TSV — if it dies it must not take the cpg/tpg arm's output with
it.

Usage:
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_m.py fast  rust/oracle/slice_m_pypy.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_m.py equil rust/oracle/slice_m_eq_pypy.tsv
    py -3                     rust/oracle/dump_slice_m.py fast  rust/oracle/slice_m_cpython.tsv
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.engine import (  # noqa: E402
    ComponentMap, FlightCondition, VariableStatorMatcher, build_two_spool_turbojet,
)
from turbojet.gas import Gas  # noqa: E402

ARM = sys.argv[1] if len(sys.argv) > 1 else "fast"
OUT = sys.argv[2] if len(sys.argv) > 2 else None

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55

# Rung 53's OWN five disclosed shapes, verbatim from tests/test_rung53.py::SHAPES.
SHAPES = [
    ("flow_press", ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                   ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)),
    ("press_flow", ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    ("tilted",     ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                   ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
    ("steep",      ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2),
                   ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)),
    ("flat_eta",   ComponentMap(sigma=0.1, l=0.7), ComponentMap(sigma=0.1, l=1.0)),
]
SPOOLS = ("lp", "hp")
THROTTLE_FAST = (1500.0, 1200.0, 1000.0, 800.0)
THROTTLE_EQ = (1500.0, 1200.0)
# The three capacities the `binds` census is measured over (§ 5.9 (vii)).
CAPACITIES = (0.00, 0.80, 0.90)
BINDS_CODE = {"throat": 0.0, "peak": 1.0, "edge": 2.0}


def cpg_gas():
    gc, cc, gt, ct = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cc, R_c=(gc - 1.0) / gc * cc,
               gamma_t=gt, cp_t=ct, R_t=(gt - 1.0) / gt * ct, hPR=42.8e6)


GASES = {"cpg": cpg_gas, "tpg": Gas.thermally_perfect, "eq": Gas.reacting_equilibrium}
ARMS = {"fast": ["cpg", "tpg"], "equil": ["eq"]}[ARM]
THROTTLES = THROTTLE_EQ if ARM == "equil" else THROTTLE_FAST

# THE EQUILIBRIUM ARM IS LEAN, AND THE RE-SIZING IS RECORDED RATHER THAN QUIET. § 5.9 sized it
# at "887 s for 20 cells, ~44 s per `_scan`" -- a figure that assumes ONE scan per cell. The full
# per-cell body here runs FIVE (the bare scan, one inside each of three `authority_ceiling`
# calls, and one inside `schedule_throat`), which is ~75 min rather than ~15 and would put a
# quarter-hour into the Rust gate for a claim two scans already carry. So the equilibrium arm
# dumps the margin rows, ONE scan and the throat rows, and nothing that re-scans.
#
# What it is FOR survives the trim intact: P1's claim is that the caught scope reaches `solve_n`
# on ALL THREE gases, and `_scan` IS the caught scope -- its LENGTH is the witness. What it
# CANNOT carry is the `binds` census and the schedule split; those are cpg/tpg columns and say so.
LEAN = ARM == "equil"

out = []


def put(k, v):
    out.append((k, float(v)))


def flag(k, b):
    out.append((k, 1.0 if b else 0.0))


def matcher(gname, ml, mh, cap=0.0, vl=0.0, vh=0.0):
    """The capture-then-move constructor, with the maps ARMED (floor always; throat if cap>0)."""
    a_l = ml.with_phi_surge(FLOOR)
    a_h = mh.with_phi_surge(FLOOR)
    if cap > 0.0:
        a_l, a_h = a_l.with_capacity(cap), a_h.with_capacity(cap)
    design = build_two_spool_turbojet(GASES[gname](), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                      nozzle_convergent=True, **REAL)
    return VariableStatorMatcher(design, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h,
                                 vsv_lp=vl, vsv_hp=vh)


MARGIN_KEYS = ("vsv", "phi_op", "n", "m", "phi_surge", "phi_surge_design", "m_phi",
               "tan_b1", "tan_b1_crit", "m_i", "pi_op", "sm_n")

n_cell = 0
for gname in ARMS:
    for sname, ml, mh in SHAPES:
        for tt4 in THROTTLES:
            for spool in SPOOLS:
                n_cell += 1
                cell = f"{gname}/{sname}/{tt4:.0f}/{spool}"
                print(f"[cell {n_cell}] {cell}", file=sys.stderr, flush=True)

                # ---- rung 53's row, at DESIGN and at a MOVED stator ---------------------
                for vtag, v in (("v0", 0.0), ("vmv", 0.12)):
                    m = matcher(gname, ml, mh, vl=v if spool == "lp" else 0.0,
                                vh=v if spool == "hp" else 0.0)
                    row = m.stator_margin(FLIGHT, tt4)[spool]
                    for k in MARGIN_KEYS:
                        put(f"margin/{cell}/{vtag}/{k}", row[k])

                m0 = matcher(gname, ml, mh)

                # ---- the headline, and the v=0 control ---------------------------------
                cs = None if LEAN else m0.currency_split(FLIGHT, tt4, spool=spool)
                if cs is not None:
                    for k in ("phi_op", "d_phi_op", "d_m", "d_n", "flow_vs_speed",
                              "d_phi_op_closed", "d_m_phi", "d_m_i", "d_sm_n",
                              "d_m_i_closed_design", "ratio", "floor_boundary"):
                        put(f"split/{cell}/{k}", cs[k])
                    flag(f"split/{cell}/is_split", cs["split"])
                    flag(f"split/{cell}/in_interval", cs["in_interval"])

                # ---- rung 54's scan: THE ABORT CENSUS + the V_MAX instrument -----------
                mc = matcher(gname, ml, mh, cap=0.80)
                scan = mc._scan(FLIGHT, tt4, spool)
                put(f"scan/{cell}/n", float(len(scan)))
                put(f"scan/{cell}/v_edge", scan[-1]["vsv"])
                put(f"scan/{cell}/x_edge", scan[-1]["throat_loading"])
                put(f"scan/{cell}/m_i_0", scan[0]["m_i"])
                put(f"scan/{cell}/m_i_edge", scan[-1]["m_i"])

                # ---- the throat row on BOTH branches of the capacity split -------------
                for ctag, mm_ in (("noC", m0), ("C80", mc)):
                    row = mm_.throat_margin(FLIGHT, tt4)[spool]
                    for k in ("area", "throat_loading", "c_min", "capacity"):
                        put(f"throat/{cell}/{ctag}/{k}", row[k])
                    flag(f"throat/{cell}/{ctag}/has_choke", "m_c" in row)
                    if "m_c" in row:
                        put(f"throat/{cell}/{ctag}/m_c", row["m_c"])
                        flag(f"throat/{cell}/{ctag}/choked", row["choked"])
                        put(f"throat/{cell}/{ctag}/throat_mach_design",
                            row["throat_mach_design"])

                if not LEAN:
                    # ---- authority_ceiling at THREE capacities: the binds census ----------
                    for cap in CAPACITIES:
                        a = mc.authority_ceiling(FLIGHT, tt4, spool=spool, capacity=cap)
                        p = f"ceil/{cell}/{cap:.2f}"
                        for k in ("capacity", "v_edge", "x_edge", "c_edge", "v_peak", "m_i_peak",
                                  "m_i_0", "m_i_edge", "m_i_usable", "retained", "setting_cut"):
                            put(f"{p}/{k}", a[k])
                        put(f"{p}/binds", BINDS_CODE[a["binds"]])
                        put(f"{p}/n_scan", float(a["n_scan"]))
                        flag(f"{p}/peak_interior", a["peak_interior"])
                        flag(f"{p}/throat_before_edge", a["throat_before_edge"])
                        flag(f"{p}/has_v_ch", a["v_ch"] is not None)
                        if a["v_ch"] is not None:
                            put(f"{p}/v_ch", a["v_ch"])
                        flag(f"{p}/has_m_i_at_throat", a["m_i_at_throat"] is not None)
                        if a["m_i_at_throat"] is not None:
                            put(f"{p}/m_i_at_throat", a["m_i_at_throat"])

                    # ---- rung 54's schedule: the exists split + THE RACE -------------------
                    srow = mc.schedule_throat(FLIGHT, [tt4], spool=spool)[0]
                    p = f"sthroat/{cell}"
                    flag(f"{p}/exists", srow["exists"])
                    put(f"{p}/tan_b1_min", srow["tan_b1_min"])
                    put(f"{p}/tan_b1_design", srow["tan_b1_design"])
                    put(f"{p}/v_edge", srow["v_edge"])
                    if srow["exists"]:
                        for k in ("vsv_star", "tan_b1", "m", "phi_op", "n", "m_i", "m_phi",
                                  "throat_loading", "c_min", "m_c"):
                            put(f"{p}/{k}", srow[k])
                        flag(f"{p}/feasible", srow["feasible"])

                    # ---- rung 53's ladder, at the SHIPPED default cap ---------------------
                    # It ASSERTS on 18 of the 80 cells (§ 5.9 (v)) and that is a finding, so the
                    # raise is recorded as a flag rather than allowed to kill the sweep.
                    p = f"sched/{cell}"
                    try:
                        r = m0.incidence_schedule(FLIGHT, [tt4], spool=spool)[0]
                        flag(f"{p}/bracketed", True)
                        for k in ("vsv_star", "residual", "tan_b1", "tan_b1_design", "phi_op",
                                  "phi_op_bare", "phi_surge", "m_i", "m_i_bare", "m_phi",
                                  "m_phi_bare", "sm_n", "sm_n_bare", "n"):
                            put(f"{p}/{k}", r[k])
                    except AssertionError:
                        flag(f"{p}/bracketed", False)

if OUT:
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        for k, v in out:
            fh.write(f"{k}\t{v.hex()}\n")
else:
    for k, v in out:
        print(f"{k}\t{v.hex()}")
print(f"# {len(out)} keys over {n_cell} cells", file=sys.stderr, flush=True)
