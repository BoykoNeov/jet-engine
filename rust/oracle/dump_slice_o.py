"""SLICE O's value oracle — rung 61, on the grid § 5.11 PRE-REGISTERED and no other.

    fast  : 2 gases (cpg, tpg) x 5 shapes x 4 throttles x 4 settings x 2 spools x 2 targets
            = 640 `compensating_bleed` cells, PLUS the row/headline/seam/price objects
    equil : the TWO-AXIS REDUCE ONLY — 3 corners x 2 throttles x 17 fields

**THE cpg ARM'S GRID IS `probe_o1.py`'s, TO THE VALUE.** § 5.11 (i)/(ii)'s census — 10 613
`_feasible` calls, 0 refusals, 196 bisections all exiting on `_B_TOL`, 124 `valve authority
exhausted` — was measured on that grid and nowhere else. Slice N's lesson (§ 5.10 step 4) was
that a census measured on a probe's grid and restated over a dump's is not a measurement; here
the two grids are the same object, and the counts are EMITTED per cell so Rust compares rather
than restates.

**THE CENSUS IS COUNTED WITHOUT COPYING THE BODY.** `_feasible` is wrapped in a pure counting
shim that calls the original — never a re-implementation. Per cell that gives
`feasible = 2 + walk_steps + bisect_passes` exactly (the bare row, the `v` row at `b = 0`, then
the walk and the bisection), so agreeing on the TOTAL plus the exit reason plus `b*` pins both
halves without this script ever re-deriving the bisection. `probe_o1.py` DID copy the body, and
that copy is a probe, not a gate.

**THE EQUILIBRIUM ARM CANNOT SWEEP.** Measured at § 5.11 (vii): ONE `compensating_bleed` on the
reacting gas costs 54.5 s, so the 320-cell grid would be ~5 hours. It dumps the two-axis reduce
and nothing that root-finds — which is what gate 1 needs and all it needs.

Usage:
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_o.py fast  rust/oracle/slice_o_pypy.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_o.py equil rust/oracle/slice_o_eq_pypy.tsv
    py -3                     rust/oracle/dump_slice_o.py fast  rust/oracle/slice_o_cpython.tsv
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.engine import (  # noqa: E402
    ComponentMap, FlightCondition, StatorBleedMatcher, TwoSpoolBleedMatcher,
    TwoSpoolMapMatcher, VariableStatorMatcher, build_two_spool_turbojet,
)
from turbojet.gas import Gas  # noqa: E402

ARM = sys.argv[1] if len(sys.argv) > 1 else "fast"
OUT = sys.argv[2] if len(sys.argv) > 2 else None

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55

# Rungs 53/54/55/61's OWN five disclosed shapes, verbatim from tests/test_rung61.py::SHAPES.
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
TARGETS = ("phi", "m_phi")
THROTTLE_FAST = (1100.0, 1300.0, 1500.0, 1700.0)   # probe_o1's, to the value
SETTINGS = (0.05, 0.10, 0.20, 0.30)                 # probe_o1's, to the value
THROTTLE_EQ = (1500.0, 1200.0)

# Python's three `reason` strings, as codes. 0 is the SOLVED branch.
REASON_CODE = {
    None: 0.0,
    "valve authority exhausted (b >= cap)": 1.0,
    "choked envelope closed before the target": 2.0,
    "stator setting infeasible with the valve shut": 3.0,
}


def cpg_gas():
    """EXACTLY tests/test_rung61.py::_cpg_gas.

    **NOT `R_c = .4/1.4*cp`** — step 1 measured that `1.4 - 1.0` is not the literal `0.4`, and
    the 1-ULP gas that results moves `v0`, hence every thrust. An oracle inherits every constant
    its dump script types.
    """
    gc, cc, gt, ct = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cc, R_c=(gc - 1.0) / gc * cc,
               gamma_t=gt, cp_t=ct, R_t=(gt - 1.0) / gt * ct, hPR=42.8e6)


GASES = {"cpg": cpg_gas, "tpg": Gas.thermally_perfect, "eq": Gas.reacting_equilibrium}
ARMS = {"fast": ["cpg", "tpg"], "equil": ["eq"]}[ARM]

# ---- the counting shim: wraps, never re-implements ------------------------------------
_ORIG_FEASIBLE = StatorBleedMatcher._feasible
FEAS = [0]


def _counting_feasible(self, flight, Tt4, v, spool, b):
    FEAS[0] += 1
    return _ORIG_FEASIBLE(self, flight, Tt4, v, spool, b)


StatorBleedMatcher._feasible = _counting_feasible

out = []


def put(k, v):
    out.append((k, float(v)))


def flag(k, b):
    out.append((k, 1.0 if b else 0.0))


def matcher(gname, ml, mh, vl=0.0, vh=0.0, b=0.0, design=None):
    a_l, a_h = ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR)
    d = design if design is not None else build_two_spool_turbojet(
        GASES[gname](), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    return StatorBleedMatcher(d, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h,
                              vsv_lp=vl, vsv_hp=vh, bleed=b)


# =======================================================================================
# THE EQUILIBRIUM ARM — the two-axis reduce, and nothing that root-finds
# =======================================================================================
FIELDS = ("pi_lpc", "pi_hpc", "n_lp", "n_hp", "phi_lp", "phi_hp", "slip",
          "eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt", "tau_lpc", "tau_hpc",
          "tau_hpt", "tau_lpt", "mdot_air", "thrust")

if ARM == "equil":
    ml, mh = SHAPES[0][1], SHAPES[0][2]
    a_l, a_h = ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR)
    for tt4 in THROTTLE_EQ:
        for corner, vl, b in (("v0b0", 0.0, 0.0), ("vb0", 0.15, 0.0), ("v0b", 0.0, 0.08)):
            print(f"[eq] Tt4={tt4:.0f} {corner}", file=sys.stderr, flush=True)
            d = build_two_spool_turbojet(GASES["eq"](), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                         nozzle_convergent=True, **REAL)
            sb = StatorBleedMatcher(d, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h,
                                    vsv_lp=vl, bleed=b).match(FLIGHT, tt4)
            for k in FIELDS:
                put(f"eq/{tt4:.0f}/{corner}/61/{k}", getattr(sb, k))
            # …and the PARENT the corner must equal, from its own class.
            if corner == "v0b0":
                par = TwoSpoolMapMatcher(d, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h)
            elif corner == "vb0":
                par = VariableStatorMatcher(d, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h, vsv_lp=vl)
            else:
                par = TwoSpoolBleedMatcher(d, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h, bleed=b)
            pr = par.match(FLIGHT, tt4)
            for k in FIELDS:
                put(f"eq/{tt4:.0f}/{corner}/parent/{k}", getattr(pr, k))

# =======================================================================================
# THE FAST ARM
# =======================================================================================
CB_KEYS = ("b_star", "goal", "resid", "bare_phi", "bare_m_phi", "bare_m_i")
CP_KEYS = ("phi_bare", "phi_stator", "m_i_bare", "m_i_stator", "m_phi_bare", "m_phi_stator",
           "n_bare", "n_stator", "thrust_bare", "thrust_stator", "phi_other_bare",
           "d_phi_other_stator")
CP_COMP = ("phi_comp", "m_i_comp", "m_phi_comp", "n_comp", "thrust_comp",
           "d_m_i", "d_m_i_pred", "d_m_phi", "d_m_phi_pred", "d_m_i_resid", "d_m_phi_resid",
           "dn_stator", "dn_comp", "dF_stator", "dF_comp",
           "phi_other_comp", "d_phi_other_comp")

n_cell = 0
if ARM == "fast":
    for gname in ARMS:
        for sname, ml, mh in SHAPES:
            design = build_two_spool_turbojet(GASES[gname](), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                              nozzle_convergent=True, **REAL)
            m = matcher(gname, ml, mh, design=design)
            base = f"{gname}/{sname}"

            # ---- the ROOT-FINDER, on probe_o1's grid to the value ---------------------
            for tt4 in THROTTLE_FAST:
                for v in SETTINGS:
                    for spool in SPOOLS:
                        for target in TARGETS:
                            n_cell += 1
                            cell = f"{base}/{tt4:.0f}/{v:.2f}/{spool}/{target}"
                            print(f"[cell {n_cell}] {cell}", file=sys.stderr, flush=True)
                            FEAS[0] = 0
                            c = m.compensating_bleed(FLIGHT, tt4, v, spool, target)
                            # THE CENSUS, per cell: feasible = 2 + walk + bisect, exactly.
                            put(f"cb/{cell}/feasible", FEAS[0])
                            put(f"cb/{cell}/reason", REASON_CODE[c.get("reason")])
                            put(f"cb/{cell}/goal", c["goal"])
                            if c["b_star"] is None:
                                # b_last/resid_last exist on TWO of the three None branches.
                                flag(f"cb/{cell}/has_last", "b_last" in c)
                                if "b_last" in c:
                                    put(f"cb/{cell}/b_last", c["b_last"])
                                    put(f"cb/{cell}/resid_last", c["resid_last"])
                            else:
                                flag(f"cb/{cell}/has_last", False)
                                for k in CB_KEYS:
                                    put(f"cb/{cell}/{k}", c[k])

            # ---- THE ROW, at the shipped setting --------------------------------------
            for tt4 in THROTTLE_FAST:
                for spool in SPOOLS:
                    cell = f"{base}/{tt4:.0f}/{spool}"
                    print(f"[row] {cell}", file=sys.stderr, flush=True)
                    r = m.compensated_point(FLIGHT, tt4, 0.20, spool)
                    put(f"cp/{cell}/reason", REASON_CODE[r.get("reason")])
                    for k in CP_KEYS:
                        put(f"cp/{cell}/{k}", r[k])
                    flag(f"cp/{cell}/compensated", r["b_star"] is not None)
                    if r["b_star"] is not None:
                        put(f"cp/{cell}/b_star", r["b_star"])
                        for k in CP_COMP:
                            put(f"cp/{cell}/{k}", r[k])

            # ---- THE HEADLINE, and the truthiness of `ratio` --------------------------
            print(f"[headline] {base}", file=sys.stderr, flush=True)
            for i, row in enumerate(m.compensability(FLIGHT, THROTTLE_FAST, v=0.20)):
                p = f"comp/{base}/{i}"
                put(f"{p}/Tt4", row["Tt4"])
                put(f"{p}/pi_hpc", row["pi_hpc"])
                put(f"{p}/pi_lpc", row["pi_lpc"])
                for sp in SPOOLS:
                    flag(f"{p}/{sp}/present", row[f"b_{sp}"] is not None)
                    if row[f"b_{sp}"] is not None:
                        put(f"{p}/{sp}/b", row[f"b_{sp}"])
                    put(f"{p}/{sp}/why", REASON_CODE[row[f"why_{sp}"]])
                    flag(f"{p}/{sp}/has_resid", row[f"resid_{sp}"] is not None)
                    if row[f"resid_{sp}"] is not None:
                        put(f"{p}/{sp}/resid", row[f"resid_{sp}"])
                flag(f"{p}/ratio_present", row["ratio"] is not None)
                if row["ratio"] is not None:
                    put(f"{p}/ratio", row["ratio"])
            put(f"comp/{base}/n_rows", len(m.compensability(FLIGHT, THROTTLE_FAST, v=0.20)))

            # ---- THE SEAM AS POSED ----------------------------------------------------
            print(f"[seam] {base}", file=sys.stderr, flush=True)
            for spool in SPOOLS:
                for row in m.authority_with_bleed(FLIGHT, 1500.0, (0.0, 0.05, 0.10), spool):
                    p = f"auth/{base}/{spool}/{row['bleed']:.2f}"
                    for k in ("v_edge", "v_peak", "m_i_0", "m_i_peak", "m_i_edge", "span"):
                        put(f"{p}/{k}", row[k])
                    flag(f"{p}/peak_interior", row["peak_interior"])
                    put(f"{p}/n_scan", row["n_scan"])

            # ---- P4's TWO LOCI --------------------------------------------------------
            print(f"[price] {base}", file=sys.stderr, flush=True)
            for spool in SPOOLS:
                for row in m.price_split(FLIGHT, 1500.0, (0.10, 0.20, 0.30), spool):
                    p = f"price/{base}/{spool}/{row['vsv']:.2f}"
                    put(f"{p}/floor_motion", row["floor_motion"])
                    flag(f"{p}/gap_present", row["gap"] is not None)
                    for k in ("b_phi", "b_m_phi", "gap"):
                        flag(f"{p}/{k}_present", row[k] is not None)
                        if row[k] is not None:
                            put(f"{p}/{k}", row[k])
                    put(f"{p}/why_phi", REASON_CODE[row["why_phi"]])
                    put(f"{p}/why_m_phi", REASON_CODE[row["why_m_phi"]])

if OUT:
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        for k, v in out:
            fh.write(f"{k}\t{v.hex()}\n")
else:
    for k, v in out:
        print(f"{k}\t{v.hex()}")
print(f"# {len(out)} keys over {n_cell} root-finder cells", file=sys.stderr, flush=True)
