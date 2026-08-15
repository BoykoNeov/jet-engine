"""Slice M step 2 smoke check — rung 53's four reading methods, one cell each.

Deliberately NOT one method: slice L step 3's lesson is that a smoke check witnessing
1 of the 3 methods a slice's headline names has measured almost nothing.
Prints `key<TAB>float.hex()` so the Rust side can compare BITS, not decimals.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.gas import Gas
from turbojet.engine import (FlightCondition, build_two_spool_turbojet, ComponentMap,
                             VariableStatorMatcher)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def gas():
    gc, cc, gt, ct = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cc, R_c=(gc - 1.0) / gc * cc,
               gamma_t=gt, cp_t=ct, R_t=(gt - 1.0) / gt * ct, hPR=42.8e6)


def design(g):
    return build_two_spool_turbojet(g, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def vm(vl=0.0, vh=0.0):
    return VariableStatorMatcher(design(gas()), FLIGHT, 1.0, map_lp=LP, map_hp=HP,
                                 vsv_lp=vl, vsv_hp=vh)


out = []


def put(k, v):
    out.append((k, float(v)))


# --- 1. stator_margin, at a MOVED stator (v != 0 is what exercises the new psi term) ------
for tag, (vl, vh) in (("v0", (0.0, 0.0)), ("vlp", (0.15, 0.0)), ("vhp", (0.0, 0.15))):
    r = vm(vl, vh).stator_margin(FLIGHT, 1200.0)
    for spool in ("lp", "hp"):
        row = r[spool]
        for key in ("vsv", "phi_op", "n", "m", "phi_surge", "phi_surge_design", "m_phi",
                    "tan_b1", "tan_b1_crit", "m_i", "pi_op", "sm_n"):
            put(f"margin/{tag}/{spool}/{key}", row[key])

# --- 2. stator_sweep — two-sided, LP swept -----------------------------------------------
for row in vm().stator_sweep(FLIGHT, 1200.0, [-0.10, 0.0, 0.10], spool="lp"):
    v = row["vsv"]
    put(f"sweep/{v:+.2f}/lp/m_i", row["lp"]["m_i"])
    put(f"sweep/{v:+.2f}/lp/m_phi", row["lp"]["m_phi"])
    put(f"sweep/{v:+.2f}/hp/m_i", row["hp"]["m_i"])

# --- 3. currency_split — THE HEADLINE ------------------------------------------------------
for spool in ("lp", "hp"):
    cs = vm().currency_split(FLIGHT, 1200.0, spool=spool)
    for key in ("phi_op", "phi_surge", "d_phi_op", "d_m", "d_n", "flow_vs_speed",
                "d_phi_op_closed", "d_m_phi", "d_m_i", "d_sm_n", "d_m_i_closed_design",
                "ratio", "floor_boundary"):
        put(f"split/{spool}/{key}", cs[key])
    put(f"split/{spool}/is_split", 1.0 if cs["split"] else 0.0)
    put(f"split/{spool}/in_interval", 1.0 if cs["in_interval"] else 0.0)

# --- 3b. currency_split on a MOVED matcher — the arm that can SEE the unswept spool ------
# At v=0 "the other spool holds at self's setting" and "the other spool is pinned to 0" are the
# same instruction, so arm 3 cannot discriminate them. This one can: vsv_hp != 0 while LP is
# swept, so a leg built with the wrong sibling constructor lands on a different machine.
for spool in ("lp", "hp"):
    cs = vm(0.15, 0.10).currency_split(FLIGHT, 1200.0, spool=spool)
    for key in ("phi_op", "phi_surge", "d_phi_op", "d_m", "d_n", "d_m_phi", "d_m_i", "ratio"):
        put(f"splitmv/{spool}/{key}", cs[key])

# --- 4. throttle_currency — the v=0 control ------------------------------------------------
for row in vm().throttle_currency(FLIGHT, [1500.0, 1300.0, 1100.0], spool="lp"):
    t = row["Tt4"]
    for key in ("d_m_phi", "d_m_i", "d_sm_n", "ratio", "jacobian", "phi_mid"):
        put(f"throt/{t:.0f}/{key}", row[key])
    put(f"throt/{t:.0f}/signs_agree", 1.0 if row["signs_agree"] else 0.0)

# --- 5. incidence_schedule — the payoff object, at the SHIPPED default cap -----------------
for row in vm().incidence_schedule(FLIGHT, [1400.0, 1200.0], spool="lp"):
    t = row["Tt4"]
    for key in ("vsv_star", "residual", "tan_b1", "tan_b1_design", "phi_op", "phi_op_bare",
                "phi_surge", "m_i", "m_i_bare", "m_phi", "m_phi_bare", "sm_n", "sm_n_bare",
                "n"):
        put(f"sched/{t:.0f}/{key}", row[key])


# =========================================================================================
# RUNG 54 — the throat. Five methods again, and the two field-set splits BOTH exercised.
# =========================================================================================

CAP = 0.80

def vmc(vl=0.0, vh=0.0, cap=CAP):
    """A matcher whose maps carry a THROAT MODEL, built the same capture-then-move way."""
    return VariableStatorMatcher(design(gas()), FLIGHT, 1.0,
                                 map_lp=LP.with_capacity(cap), map_hp=HP.with_capacity(cap),
                                 vsv_lp=vl, vsv_hp=vh)


# --- 6. throat_margin on BOTH branches of the capacity split (16 keys vs 19) --------------
for tag, m in (("noC", vm(0.10, 0.0)), ("C80", vmc(0.10, 0.0))):
    r = m.throat_margin(FLIGHT, 1200.0)
    for spool in ("lp", "hp"):
        row = r[spool]
        for key in ("area", "throat_loading", "c_min", "capacity"):
            put(f"throat/{tag}/{spool}/{key}", row[key])
        put(f"throat/{tag}/{spool}/has_choke", 1.0 if "m_c" in row else 0.0)
        if "m_c" in row:
            put(f"throat/{tag}/{spool}/m_c", row["m_c"])
            put(f"throat/{tag}/{spool}/choked", 1.0 if row["choked"] else 0.0)
            put(f"throat/{tag}/{spool}/throat_mach_design", row["throat_mach_design"])

# --- 7. throat_sweep — two-sided, and EVEN in v on the geometric channel -------------------
for row in vmc().throat_sweep(FLIGHT, 1200.0, [-0.10, 0.0, 0.10], spool="lp"):
    v = row["vsv"]
    put(f"tsweep/{v:+.2f}/area", row["area"])
    put(f"tsweep/{v:+.2f}/throat_loading", row["throat_loading"])
    put(f"tsweep/{v:+.2f}/m_c", row["m_c"])

# --- 8. _scan — the walk that ends on solve_n's bracket; its LENGTH is the V_MAX instrument
for spool in ("lp", "hp"):
    rows = vmc()._scan(FLIGHT, 1200.0, spool)
    put(f"scan/{spool}/n", float(len(rows)))
    put(f"scan/{spool}/v_edge", rows[-1]["vsv"])
    put(f"scan/{spool}/m_i_0", rows[0]["m_i"])
    put(f"scan/{spool}/m_i_edge", rows[-1]["m_i"])
    put(f"scan/{spool}/x_edge", rows[-1]["throat_loading"])

# --- 9. authority_ceiling — the headline object, at THREE capacities incl. C=0 -------------
_BINDS = {"throat": 0.0, "peak": 1.0, "edge": 2.0}
for cap in (0.0, 0.80, 0.90):
    for spool in ("lp", "hp"):
        a = vmc().authority_ceiling(FLIGHT, 1200.0, spool=spool, capacity=cap)
        p = f"ceil/{cap:.2f}/{spool}"
        for key in ("capacity", "v_edge", "x_edge", "c_edge", "v_peak", "m_i_peak",
                    "m_i_0", "m_i_edge", "m_i_usable", "retained", "setting_cut"):
            put(f"{p}/{key}", a[key])
        put(f"{p}/binds", _BINDS[a["binds"]])
        put(f"{p}/peak_interior", 1.0 if a["peak_interior"] else 0.0)
        put(f"{p}/n_scan", float(a["n_scan"]))
        put(f"{p}/throat_before_edge", 1.0 if a["throat_before_edge"] else 0.0)
        # THE SPLIT a float dump cannot see: v_ch and m_i_at_throat are None on one branch.
        put(f"{p}/has_v_ch", 1.0 if a["v_ch"] is not None else 0.0)
        if a["v_ch"] is not None:
            put(f"{p}/v_ch", a["v_ch"])
        put(f"{p}/has_m_i_at_throat", 1.0 if a["m_i_at_throat"] is not None else 0.0)
        if a["m_i_at_throat"] is not None:
            put(f"{p}/m_i_at_throat", a["m_i_at_throat"])

# --- 10. schedule_throat — THE RACE, and the exists split ----------------------------------
for row in vmc().schedule_throat(FLIGHT, [1400.0, 1200.0, 1000.0], spool="lp"):
    t = row["Tt4"]
    p = f"sthroat/{t:.0f}"
    put(f"{p}/exists", 1.0 if row["exists"] else 0.0)
    put(f"{p}/tan_b1_min", row["tan_b1_min"])
    put(f"{p}/tan_b1_design", row["tan_b1_design"])
    put(f"{p}/v_edge", row["v_edge"])
    if row["exists"]:
        for key in ("vsv_star", "tan_b1", "m", "phi_op", "n", "m_i", "m_phi",
                    "throat_loading", "c_min", "m_c"):
            put(f"{p}/{key}", row[key])
        put(f"{p}/feasible", 1.0 if row["feasible"] else 0.0)


# --- 11. THE STEEP SHAPE — the three branches the LP/HP pair above never reaches ----------
# Probed, not guessed: on the default maps `peak_interior` is False, every schedule EXISTS and
# `v_ch` is present at C>0, so sections 9-10 leave the parabolic refinement, the `found: None`
# branch and the `v_ch: None`-WITH-a-throat-model branch entirely unmeasured. `steep` reaches
# all three -- LP at 1000/800 has no schedule, HP at 1200 never crosses 1/C.
STEEP = ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2).with_phi_surge(FLOOR)


def vsteep(cap=CAP):
    return VariableStatorMatcher(design(gas()), FLIGHT, 1.0,
                                 map_lp=STEEP.with_capacity(cap),
                                 map_hp=STEEP.with_capacity(cap))


for spool, T in (("lp", 1200.0), ("lp", 1000.0), ("hp", 1200.0)):
    a = vsteep().authority_ceiling(FLIGHT, T, spool=spool)
    p = f"steep/{spool}/{T:.0f}"
    for key in ("v_edge", "v_peak", "m_i_peak", "m_i_0", "m_i_usable", "retained",
                "setting_cut"):
        put(f"{p}/{key}", a[key])
    put(f"{p}/binds", _BINDS[a["binds"]])
    put(f"{p}/peak_interior", 1.0 if a["peak_interior"] else 0.0)
    put(f"{p}/n_scan", float(a["n_scan"]))
    put(f"{p}/has_v_ch", 1.0 if a["v_ch"] is not None else 0.0)
    if a["v_ch"] is not None:
        put(f"{p}/v_ch", a["v_ch"])
    put(f"{p}/has_m_i_at_throat", 1.0 if a["m_i_at_throat"] is not None else 0.0)
    if a["m_i_at_throat"] is not None:
        put(f"{p}/m_i_at_throat", a["m_i_at_throat"])

for row in vsteep().schedule_throat(FLIGHT, [1200.0, 1000.0], spool="lp"):
    T = row["Tt4"]
    p = f"steepsched/{T:.0f}"
    put(f"{p}/exists", 1.0 if row["exists"] else 0.0)
    put(f"{p}/tan_b1_min", row["tan_b1_min"])
    put(f"{p}/tan_b1_design", row["tan_b1_design"])
    put(f"{p}/v_edge", row["v_edge"])
    if row["exists"]:
        for key in ("vsv_star", "throat_loading", "c_min", "m_c"):
            put(f"{p}/{key}", row[key])

for k, v in out:
    print(f"{k}\t{v.hex()}")
print(f"# {len(out)} keys", file=sys.stderr)
