"""SLICE V step 2 — the SMOKE dump for rungs 57-60 (`ScheduledStatorTransient`).

Not the slice's oracle (that is step 4, on all four suites' grids). This exists to catch a
structural mistake BEFORE the 59 Python gates are ported on top of it at step 3 — and
§ 5.20's six probes named the mistakes in advance, each of which the shipped Rust deliberately
does NOT make:

  1. **THE LOCALLY-ARMED CORE.** The port's natural shape for `_arm` is to build the armed maps
     inside the closure and leave the caller's object untouched. § 5.20 (ii) measured that
     costing 15.4 % on `margin_min_lp` with all 59 ported gates green. Section C dumps the LIVE
     map's `vsv` AFTER a march — the value Python's `_arm` leaves at whatever the LAST RK
     sub-step happened to be — so a scoped port shows as a wrong number rather than as nothing.
  2. **`_arm` INLINED into the closure.** Rung 62 calls it from two more sites (slice W), and
     rung 68 overrides it (slice AA). Section K reads the two arms' dispatch counts.
  3. **The `is`-identity reduce ported as written.** `_arm` hands back the SAME map OBJECT at
     `v == 0.0`; `ComponentMap` is `Copy` in Rust and has no identity. Section B carries the
     zero-schedule march bit-for-bit AND section K the `arm_lp_zero` count, which is what the
     identity claim becomes.
  4. **`_read`'s FIRST-STRICT minimum turned into `min_by`.** The row that wins feeds
     `s_at_min` / `v_at_min`, so a last-wins tie-break moves a reported key and no margin does.
  5. **`s_eng` typed as `Option`.** `_pin_audit`'s `from_zero` is a NaN self-inequality test on
     it and it is a dumped key — section J carries both.

Every float is emitted as its IEEE-754 bit pattern, so the comparison is bit-equality and not a
tolerance. Regenerate with:
    .venv\\Scripts\\python.exe rust\\oracle\\dump_slice_v_smoke.py > rust\\oracle\\slice_v_smoke_pypy.tsv
"""
import math
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap, TwoSpoolFuelTransient,
    ScheduledStatorTransient, StatorSchedule, SurgeLimiter, IncidenceLimiter,
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
FLOOR, V = 0.55, 0.20
LO, HI, SETTLE = 1000.0, 1400.0, 1.2
N_LO = 0.7557
MARGIN = 0.25
# The smoke marches COARSE on purpose -- it is a structural check, not the oracle. The four
# suites' own `ds` (0.01 / 0.005) is step 3's, and section B pins the grid LENGTH so a coarser
# march can never be mistaken for the same one.
DS = 0.05

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg():
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=1.3, cp_t=1239.0, R_t=(1.3 - 1.0) / 1.3 * 1239.0, hPR=42.8e6)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def _st(**kw):
    return ScheduledStatorTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _sched(v_max=V, n_lo=N_LO, **kw):
    return StatorSchedule(v_max=v_max, n_lo=n_lo, **kw)


PT_KEYS = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
           "mdot_air", "sp_thrust", "mf", "mf_sched")


def put_traj(p, traj, stride=1):
    d(f"{p}/npts", len(traj))
    for i in range(0, len(traj), stride):
        for k in PT_KEYS:
            f(f"{p}/{i}/{k}", traj[i][k])


def put_read(p, rd):
    d(f"{p}/npts", rd["npts"])
    for sp in ("lp", "hp"):
        r = rd[sp]
        for k in ("m_phi", "m_i", "T_c", "min_phi"):
            f(f"{p}/{sp}/{k}", r[k])
        for k in ("s", "phi", "v", "nu_lp", "nu_hp"):
            f(f"{p}/{sp}/at/{k}", r["at"][k])


def put_cell(p, c):
    for k in ("m_i", "m_i_grid", "m_phi", "s", "v", "s_grid", "min_phi", "nu0", "nu_lp_end",
              "nu_hp_end", "Tt4_peak", "fuel_removed", "s_eng"):
        f(f"{p}/{k}", c[k])
    d(f"{p}/npts", c["npts"])
    d(f"{p}/prof_len", len(c["prof"]))
    for i in (0, len(c["prof"]) // 2, len(c["prof"]) - 1):
        f(f"{p}/prof/{i}/s", c["prof"][i][0])
        f(f"{p}/prof/{i}/m", c["prof"][i][1])


# =============================================================================
# A — the two VALUE TYPES, before any march touches them
# =============================================================================
SHAPES = (("smooth", dict()), ("linear", dict(shape="linear")))
for tag, kw in SHAPES:
    s = _sched(**kw)
    for i in range(13):
        n = 0.60 + 0.05 * i          # spans below n_lo, through the knee, past n_ref
        f(f"A/sched/{tag}/{i}", s(n))
    f(f"A/sched/{tag}/at_n_ref", s(1.0))
    f(f"A/sched/{tag}/v_max", s.v_max)
    f(f"A/sched/{tag}/n_lo", s.n_lo)
    f(f"A/sched/{tag}/n_ref", s.n_ref)

# The zero schedule -- the arm the reduce runs on.
z = _sched(v_max=0.0, n_lo=0.75)
for i in range(5):
    f(f"A/sched/zero/{i}", z(0.70 + 0.08 * i))

T_C = LP.tan_beta1_crit()
f("A/inc/T_c", T_C)
for i, sm in enumerate((0.0, 0.05, 0.15, 0.30)):
    il = IncidenceLimiter.from_margin(LP, "lp", sm)
    f(f"A/inc/from_margin/{i}/m_lim", il.m_lim)
    for j, v in enumerate((0.0, 0.05, 0.20)):
        f(f"A/inc/from_margin/{i}/phi_lim_at/{j}", il.phi_lim_at(T_C, v))
        f(f"A/inc/from_margin/{i}/at/{j}", il.at(T_C, v).phi_lim)
# THE REDUCE, in the value type: at v = 0 the incidence floor IS the rung-49 one, float-identical.
il0 = IncidenceLimiter.from_phi(LP, "lp", 0.62)
f("A/inc/from_phi/m_lim", il0.m_lim)
f("A/inc/from_phi/roundtrip", il0.at(T_C, 0.0).phi_lim)
b("A/inc/from_phi/roundtrip_exact", il0.at(T_C, 0.0).phi_lim == 0.62)

# =============================================================================
# B — THE REDUCE: an unarmed rung-57 march IS rung 43/45's, bit-for-bit
# =============================================================================
bare43 = TwoSpoolFuelTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
mf0, mf1 = bare43.fuel_for_Tt4(FLIGHT, LO), bare43.fuel_for_Tt4(FLIGHT, HI)
eq0 = bare43.equilibrium(FLIGHT, LO)


def _ref_ramp(ft, r=0.5, ds=DS):
    a, c = ft.fuel_for_Tt4(FLIGHT, LO), ft.fuel_for_Tt4(FLIGHT, HI)
    e = ft.equilibrium(FLIGHT, LO)

    def sched(s):
        if s <= 0.0:
            return a
        if s >= r:
            return c
        return a + (c - a) * (s / r)

    return ft.integrate_fuel(FLIGHT, sched, (e["nu_lp"], e["nu_hp"]), r + SETTLE, ds)


put_traj("B/r43", _ref_ramp(bare43), stride=7)
put_traj("B/r57_unarmed", _st()._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)[0], stride=7)
put_traj("B/r57_zero_lp",
         _st(vsv_sched_lp=z)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)[0], stride=7)
put_traj("B/r57_zero_both",
         _st(vsv_sched_lp=z, vsv_sched_hp=z)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE,
                                                           DS)[0], stride=7)

# =============================================================================
# C — THE CARRIER: what `_arm` LEAVES on the object after a march
# =============================================================================
# This is § 5.20 (ii)'s whole subject. A locally-armed-core port leaves the DESIGN map here.
for tag, kw in (("lp_only", dict(vsv_sched_lp=_sched())),
                ("hp_only", dict(vsv_sched_hp=_sched())),
                ("both", dict(vsv_sched_lp=_sched(), vsv_sched_hp=_sched())),
                ("const_lp", dict(vsv_lp=V))):
    m = _st(**kw)
    traj, nu0 = m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)
    f(f"C/{tag}/stale_vsv_lp", m.map_lp.vsv)
    f(f"C/{tag}/stale_vsv_hp", m.map_hp.vsv)
    f(f"C/{tag}/design_vsv_lp", m.map_lp_design.vsv)
    f(f"C/{tag}/nu0_lp", nu0[0])
    f(f"C/{tag}/nu0_hp", nu0[1])
    put_traj(f"C/{tag}", traj, stride=11)
    put_read(f"C/{tag}/read", m._read(traj))
    # The two rung-53 channels the arming actually drives, at the stale setting.
    f(f"C/{tag}/stale_phi_surge_at", m.map_lp.phi_surge_at())
    f(f"C/{tag}/stale_psi", m.map_lp.psi(0.62))
    # ... and the one it provably cannot (§ (iii)): `eta_t_at` reads only `a_t`.
    f(f"C/{tag}/stale_eta_t_at", m.map_lp.eta_t_at(0.92, 1.03))

# =============================================================================
# D — rung 57's reading instrument
# =============================================================================
for tag, kw in (("const", dict(vsv_lp=V)), ("sched", dict(vsv_sched_lp=_sched())),
                ("bare", dict())):
    m = _st(**kw)
    r = m.stator_transient_margin(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS)
    put_read(f"D/{tag}", r)
    f(f"D/{tag}/nu0_lp", r["nu0_lp"])
    f(f"D/{tag}/nu0_hp", r["nu0_hp"])
    f(f"D/{tag}/r", r["r"])
    for sp in ("lp", "hp"):
        for nl, nh in ((0.80, 0.90), (0.95, 1.00)):
            f(f"D/{tag}/v_of/{sp}/{nl}", m.v_of(sp, nl, nh))

# =============================================================================
# E — THE FINDING (rung 57): the credit, its erosion, and the decomposition
# =============================================================================
CK = ("bare", "armed", "pointwise", "credit", "credit_pointwise", "erosion", "closed_form",
      "v_at_min", "s_at_min", "s_at_min_bare", "nu0_bare", "nu0_armed", "min_phi_bare",
      "min_phi_armed", "m_phi_bare", "m_phi_armed", "r")
for tag, kw, sp in (("const_lp", dict(vsv_lp=V), "lp"),
                    ("const_hp", dict(vsv_hp=0.05), "hp"),
                    ("sched_lp", dict(vsv_sched_lp=_sched()), "lp")):
    c = _st(**kw).stator_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS, spool=sp)
    for k in CK:
        f(f"E/{tag}/{k}", c[k])
    b(f"E/{tag}/pointwise_exact", c["pointwise_exact"])

DK = ("bare", "start", "ramp", "full", "share_start", "share_ramp", "self_cancel",
      "nu0_bare", "nu0_armed")
for tag, kw in (("sched", dict(vsv_sched_lp=_sched())), ("const", dict(vsv_lp=V))):
    dc = _st(**kw).credit_decomposition(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS)
    for k in DK:
        f(f"E/dec_{tag}/{k}", dc[k])

# =============================================================================
# F — rung 53's P5 transplanted
# =============================================================================
for sp in ("lp", "hp"):
    a = _st().arrow_toggle(FLIGHT, LO, HI, V, spool=sp, r=0.5, s_settle=SETTLE, ds=DS)
    for k in ("v", "s", "nu_lp", "nu_hp", "d_phi_lp", "d_phi_hp", "d_n_hp", "d_Tt25",
              "phi_lp", "phi_hp"):
        f(f"F/{sp}/{k}", a[k])
    f(f"F/{sp}/state_mf", a["state"][2])
# ... and the FIXED-STATE arm, which is the eta-mediation control's signature.
_tj = _st()._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)[0]
FIXED = (_tj[10]["nu_lp"], _tj[10]["nu_hp"], _tj[10]["mf"])
f("F/fixed/state_nu_lp", FIXED[0])
f("F/fixed/state_nu_hp", FIXED[1])
f("F/fixed/state_mf", FIXED[2])
a = _st().arrow_toggle(FLIGHT, LO, HI, V, spool="lp", r=0.5, s_settle=SETTLE, ds=DS,
                       state=FIXED)
for k in ("d_phi_lp", "d_phi_hp", "d_n_hp", "d_Tt25", "phi_lp", "phi_hp"):
    f(f"F/fixed/{k}", a[k])
b("F/fixed/s_is_nan", a["s"] != a["s"])

# =============================================================================
# G — rung 58's refined cell
# =============================================================================
ACC = _st().accel_schedule(FLIGHT, LO, HI, MARGIN, 13)
d("G/accel/n", len(ACC.n_H))
for i in range(len(ACC.n_H)):
    f(f"G/accel/n_H/{i}", ACC.n_H[i])
    f(f"G/accel/kappa/{i}", ACC.kappa[i])
f("G/accel/margin", ACC.margin)

ARGS = (FLIGHT, LO, HI, 0.5, SETTLE, DS, "lp")
put_cell("G/neither", _st()._cell(*ARGS, None, None, None))
put_cell("G/stator", _st(vsv_sched_lp=_sched())._cell(*ARGS, None, None, None))
put_cell("G/fuel", _st()._cell(*ARGS, ACC, None, None))
put_cell("G/both", _st(vsv_sched_lp=_sched())._cell(*ARGS, ACC, None, None))

# The residual and its sub-grid crossing, on their own -- `s_eng` is the half rung 60 keeps.
tj = _st()._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS, accel=ACC)[0]
res = _st()._leg_residual(FLIGHT, tj, accel=ACC)
d("G/resid/n", len(res))
for i in range(0, len(res), 5):
    f(f"G/resid/{i}/s", res[i][0])
    f(f"G/resid/{i}/g", res[i][1])
f("G/resid/s_eng", _st()._s_eng(res))

# =============================================================================
# H — THE RUNG (58): the composite and its converse reading
# =============================================================================
HK = ("predicted", "profile_bare", "profile_fuel", "credit_bare", "credit_fuel", "interaction",
      "share", "v_bare", "v_fuel", "v_ratio", "relocation", "relocation_bare", "leg_cost_bare",
      "leg_cost_armed", "fuel_removed_bare", "fuel_removed_armed", "r", "ds")
cc = _st(vsv_sched_lp=_sched()).composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS,
                                                 spool="lp", accel=ACC)
for k in HK:
    f(f"H/comp/{k}", cc[k])
for tag in ("neither", "stator", "fuel", "both"):
    put_cell(f"H/comp/{tag}", cc["cells"][tag])

es = _st(vsv_sched_lp=_sched()).engagement_shift(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS,
                                                 accel=ACC)
for k in ("bare_limited", "bare_dormant", "armed_limited", "armed_dormant", "d_limited",
          "d_dormant", "rel_limited", "rel_dormant", "r", "ds"):
    f(f"H/eng/{k}", es[k])

sw = _st().interaction_sweep(FLIGHT, LO, HI,
                             [("c05", dict(vsv_lp=0.05)),
                              ("s_knee", dict(vsv_sched_lp=_sched(n_lo=0.70)))],
                             r=0.5, s_settle=SETTLE, ds=DS, spool="lp", accel=ACC)
d("H/sweep/n", len(sw))
for row in sw:
    for k in ("credit_bare", "credit_fuel", "interaction", "share", "v_bare", "v_fuel",
              "v_ratio", "relocation", "leg_cost_bare", "leg_cost_armed"):
        f(f"H/sweep/{row['tag']}/{k}", row[k])

# =============================================================================
# I — THE RUNG (59): the matched schedule
# =============================================================================
inv = _st(vsv_sched_lp=_sched()).schedule_invariance(FLIGHT, LO, HI, MARGIN, 9)
b("I/inv/ordinate_identical", inv["ordinate_identical"])
b("I/inv/abscissa_identical", inv["abscissa_identical"])
f("I/inv/d_ordinate", inv["d_ordinate"])
f("I/inv/d_abscissa", inv["d_abscissa"])
d("I/inv/chain_n", len(inv["chain"]))
for i, row in enumerate(inv["chain"]):
    for k in ("Tt4", "d_Tt25", "d_Tt3", "d_f", "d_mfp", "d_ratio", "d_kappa", "d_n_hp",
              "d_nu_lp"):
        f(f"I/inv/chain/{i}/{k}", row[k])
# THE HP ARM -- the one the LP arm's two identities do NOT cover.
inv_hp = _st(vsv_hp=0.10).schedule_invariance(FLIGHT, LO, HI, MARGIN, 9)
b("I/inv_hp/ordinate_identical", inv_hp["ordinate_identical"])
b("I/inv_hp/abscissa_identical", inv_hp["abscissa_identical"])
f("I/inv_hp/d_ordinate", inv_hp["d_ordinate"])
f("I/inv_hp/d_abscissa", inv_hp["d_abscissa"])

# THE TUPLE-IDENTITY arm: at v == 0 everywhere the two tables are bit-equal, which is the only
# place the source claims `==` rather than a noise-floor bound.
inv_z = _st(vsv_sched_lp=z).schedule_invariance(FLIGHT, LO, HI, MARGIN, 9)
b("I/inv_zero/ordinate_identical", inv_z["ordinate_identical"])
b("I/inv_zero/abscissa_identical", inv_z["abscissa_identical"])
f("I/inv_zero/d_ordinate", inv_z["d_ordinate"])
f("I/inv_zero/d_abscissa", inv_z["d_abscissa"])

pc = ( \
     _st()._proof_chain(FLIGHT, 1200.0))
for k in ("Tt4", "Tt25", "Tt3", "f", "mfp", "ratio", "kappa", "n_hp", "nu_lp"):
    f(f"I/chain/{k}", pc[k])

mc = _st(vsv_sched_lp=_sched()).matched_credit(FLIGHT, LO, HI, MARGIN, r=0.5, s_settle=SETTLE,
                                               ds=DS, spool="lp", n=9)
for k in ("credit_bare", "interaction_bare_leg", "interaction_matched", "delta_match",
          "delta_index", "delta_value", "abscissa_share", "ordinate_share", "share_bare_leg",
          "share_matched", "s_eng_bare_leg", "s_eng_matched", "removed_bare_leg",
          "removed_matched", "relocation", "d_ordinate", "d_abscissa", "margin", "r", "ds"):
    f(f"I/matched/{k}", mc[k])
b("I/matched/ordinate_identical", mc["ordinate_identical"])
b("I/matched/abscissa_identical", mc["abscissa_identical"])
for tag in ("neither", "stator", "fuel", "both_bare_leg", "both_matched", "both_reindexed",
            "both_revalued"):
    put_cell(f"I/matched/{tag}", mc["cells"][tag])
for tag in ("fuel", "both_bare_leg", "both_matched"):
    au = mc["audits"][tag]
    for k in ("lo", "hi", "n_min", "n_max", "cut_lo", "cut_hi"):
        f(f"I/matched/audit/{tag}/{k}", au[k])
    d(f"I/matched/audit/{tag}/n_cuts", au["n_cuts"])
    d(f"I/matched/audit/{tag}/clamped", au["clamped"])

# =============================================================================
# J — THE RUNG (60): the matched FLOOR
# =============================================================================
for i, (sm, v) in enumerate(((0.0, 0.0), (0.10, 0.20), (0.25, 0.05))):
    mr = _st().matching_rules(sm, v)
    for k in ("sm", "v", "T_c", "phi_bare", "m_bare", "phi_rel", "phi_inc", "gap",
              "gap_closed_form", "residual"):
        f(f"J/rules/{i}/{k}", mr[k])

sb = _st(vsv_lp=0.10).set_point_bands(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS)
for k in ("gap_phi", "gap_m", "gap_phi_bands", "gap_m_bands", "credit", "excursion",
          "criterion", "identity_residual", "overlap_lo", "overlap_hi", "r", "ds"):
    f(f"J/bands/{k}", sb[k])
b("J/bands/phi_admissible", sb["phi_admissible"])
b("J/bands/m_admissible", sb["m_admissible"])
for side in ("bare", "armed"):
    for k in ("phi_0", "phi_min", "phi_exc", "m_0", "m_min", "m_exc", "T_c", "v_0"):
        f(f"J/bands/{side}/{k}", sb[side][k])

lad = _st().composability_ladder(FLIGHT, LO, HI,
                                 legs=[("v05", dict(vsv_lp=0.05)), ("v15", dict(vsv_lp=0.15))],
                                 r=0.5, s_settle=SETTLE, ds=DS)
d("J/ladder_legs/n", len(lad))
for row in lad:
    for k in ("r", "credit", "excursion", "criterion", "gap_m", "gap_m_bands", "gap_phi",
              "gap_phi_bands"):
        f(f"J/ladder_legs/{row['tag']}/{k}", row[k])
    b(f"J/ladder_legs/{row['tag']}/m_admissible", row["m_admissible"])
    b(f"J/ladder_legs/{row['tag']}/phi_admissible", row["phi_admissible"])

lad_r = _st().composability_ladder(FLIGHT, LO, HI,
                                   rates=[(0.25, dict(vsv_lp=0.10)), (1.0, dict(vsv_lp=0.10))],
                                   s_settle=SETTLE, ds=DS)
d("J/ladder_rates/n", len(lad_r))
for row in lad_r:
    OUT.append((f"J/ladder_rates/tag/{row['tag']}", 1))
    for k in ("r", "credit", "excursion", "criterion"):
        f(f"J/ladder_rates/{row['tag']}/{k}", row[k])

FK = ("credit_bare", "credit_fuel", "interaction", "pinned_prediction", "pinned_residual",
      "s_eng_bare", "s_eng_armed", "d_s_eng", "removed_bare", "removed_armed", "v_at_min",
      "r", "ds")
# TWO of them, because ONE pair reaches ONE regime: a floor's three degenerate regimes are the
# whole subject of `_pin_audit`, and a smoke that only ever saw `mixed` would leave rung 60's
# derived `pinned_prediction` (exactly `v`, or exactly `0`) unexercised.
PAIRS = ((0.10, 0.509), (0.05, 0.500))
for pi, (v_set, M_LIM) in enumerate(PAIRS):
  for kind, floor in (("phi", SurgeLimiter(spool="lp", phi_lim=1.0 / (T_C - M_LIM))),
                      ("inc", IncidenceLimiter(spool="lp", m_lim=M_LIM))):
    tag = f"{kind}{pi}"
    fc = _st(vsv_lp=v_set).floor_composite(FLIGHT, LO, HI, floor, r=0.5, s_settle=SETTLE, ds=DS)
    f(f"J/floor_{tag}/v_set", v_set)
    for k in FK:
        f(f"J/floor_{tag}/{k}", fc[k])
    OUT.append((f"J/floor_{tag}/regime/{fc['regime']}", 1))
    OUT.append((f"J/floor_{tag}/kind/{fc['floor']}", 1))
    b(f"J/floor_{tag}/admissible", fc["admissible"])
    for cell_tag in ("neither", "stator", "fuel", "both"):
        put_cell(f"J/floor_{tag}/{cell_tag}", fc["cells"][cell_tag])
    for au_tag in ("fuel", "both"):
        au = fc["audits"][au_tag]
        for k in ("m_set", "m_min", "residual", "s_eng", "removed"):
            f(f"J/floor_{tag}/audit/{au_tag}/{k}", au[k])
        for k in ("pinned", "dormant", "from_zero", "admissible"):
            b(f"J/floor_{tag}/audit/{au_tag}/{k}", au[k])

# =============================================================================
# K — the counts Python CAN see: how many closures a march makes, per arm
# =============================================================================
# `_arm` has no counter in the source, so it is instrumented HERE by wrapping the bound method
# rather than by copying its body -- slice R's rule: an instrument that re-derives the thing it
# measures is measuring itself.
for tag, kw in (("unarmed", dict()), ("lp_only", dict(vsv_sched_lp=_sched())),
                ("zero_lp", dict(vsv_sched_lp=z)),
                ("both", dict(vsv_sched_lp=_sched(), vsv_sched_hp=_sched()))):
    m = _st(**kw)
    tally = dict(calls=0, unarmed=0, lp_zero=0, lp_moved=0, hp_zero=0, hp_moved=0)
    real = m._arm

    def wrapped(nu_lp, nu_hp, Tt2, _m=m, _t=tally, _r=real):
        _t["calls"] += 1
        if not _m._is_armed():
            _t["unarmed"] += 1
        else:
            if _m.vsv_sched_lp is not None:
                v = _m.vsv_sched_lp(nu_lp * (_m.Tt2_d / Tt2) ** 0.5)
                _t["lp_zero" if v == 0.0 else "lp_moved"] += 1
            if _m.vsv_sched_hp is not None:
                v = _m.vsv_sched_hp(nu_hp)
                _t["hp_zero" if v == 0.0 else "hp_moved"] += 1
        return _r(nu_lp, nu_hp, Tt2)

    m._arm = wrapped
    m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)
    for k, n in tally.items():
        d(f"K/{tag}/{k}", n)

# ---------------------------------------------------------------------------- emit
print("# slice V step 2 -- rungs 57-60 SMOKE. key<TAB>u64 (float keys are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
