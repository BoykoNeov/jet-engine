"""SLICE V step 4 -- **THE ORACLE** for rungs 57-60 (`ScheduledStatorTransient`), over the FOUR
SUITES' OWN grids plus the two arms no suite reaches.

Step 3 ported 59 gates, one to one, and then measured what they can see. Every one of them is
RELATIONAL -- it asserts a relation among values THIS interpreter computed -- so a Rust/Python
arithmetic divergence moves both sides and leaves all 59 green. **This file is the instrument
that establishes agreement with Python**, and the four suite headers say so.

# THE TWO GRIDS, AND THE THIRD

Step 3 finding 1 measured the four suites marching on DIFFERENT steps, and section 5.20's own
probes on a THIRD spelling of the schedule knee. Both are carried here deliberately; neither is a
typo, and a later reader must not "fix" one to match the other:

| section | `n_lo` | `ds` | provenance |
|---|---|---|---|
| A / A' | **0.75574** | 0.01 | `probe_p7o.py` -- the run section 5.20 (ii)'s table came off |
| B      | **0.75574** | 0.01 | ADDED (see below); rung 57's grid, since it is rung 57's lever |
| C      | **0.75574** | 0.01 | `test_rung57.py:62` |
| D      | **0.7557**  | 0.01 | `test_rung58.py:51` |
| E      | **0.7557**  | 0.01 | `test_rung59.py:45` |
| F      | **0.7557**  | **0.005** | `test_rung60.py:50` |

# SECTION A IS A SEQUENCE, NOT A SET -- THIS IS LOAD-BEARING

`_arm` mutates `self.map_lp` / `self.map_hp` PERMANENTLY (section 5.20 (i)), so every reader
below leaves the map at whatever ITS last sub-step commanded, and the next reader on the same
object starts from there. The order in section A is `probe_p7o.py`'s exactly and must not be
reshuffled:

    construct -> PRE identity -> the fuel ramp -> POST identity
      -> transient_surge_margin -> transient_surge_margin_fuel -> surge_margin
      -> v_of(lp) -> v_of(hp) -> match -> equilibrium -> fuel_for_Tt4
      -> stator_transient_margin

**Measured, not assumed:** dropping `transient_surge_margin_fuel` from that chain moves
`both/sm/SM_lp` from `3faf2ad9c5223ee0` to `3fadb071a9e7f9a0` -- the design value -- because the
fuel reader's own march re-arms the map before `surge_margin` reads it. A fresh object per reader
would silently produce a different golden.

`probe_p7o.py`'s slot 13 was `stator_margin`, which RAISED `AttributeError` there (it is rung
53's STEADY reader and lives on a different class). It is dropped rather than ported as a hole.

# SECTION A' AND SECTION B ARE **ADDED** -- NO SUITE RUNS THEM

A superset must never be able to pass as a port, so both are labelled at their own section:

* **A' -- the CORRECTED `dTt4`.** `transient_surge_margin(flight, Tt4_lo, dTt4, ...)`'s third
  argument is a DELTA (`test_rung44.py` passes `300.0` / `400.0`). `probe_p7o.py` passed `HI`
  = 1400.0, i.e. it marched Tt4 from 1000 K to **2400 K**. Section A reproduces that call
  verbatim because section 5.20 (ii)'s twelve numbers came off it and P2 is checked against them;
  section A' re-runs the same armings with `dTt4 = HI - LO` on FRESH objects, so the corrected
  reading is in the repo beside the one the plan quotes.
* **B -- the HP-SCHEDULED machine.** Section 5.20 P4 measured **0 of 920 262** closes leaving
  `map_hp` mutated: no suite ever arms an HP schedule that returns non-zero, so the whole HP arm
  of `_arm` is ungated in Python. Section B is the promise P4 booked as step-4 checklist item (a).

# THE TWELVE NUMBERS (step-4 checklist item (b))

Section 5.20 (ii)'s table is **six (arming, key) pairs x two modes** = twelve values. The
BASELINE mode is shipped behaviour and all six of its values are keys here:

    A/lp_only/sm/SM_lp            A/lp_only/tsm/margin_min_lp
    A/hp_only/tsm/margin_min_lp   A/hp_only/sm/SM_hp
    A/both/sm/SM_lp               A/both/tsm/margin_min_lp

The other six are the SCOPED (locally-armed-core) mode -- they exist only under the injected
carrier bug, which is step 5's gate, not a dump of shipped code. The dump additionally carries
the full 4 armings x {SM_lp, SM_hp, margin_min_lp, margin_min_hp} = **16-key grid**, so the
`const_lp` row's *"every key: 0 -- no difference at all"* is checkable too.

Every float is emitted as its IEEE-754 bit pattern, so the comparison is bit-equality and not a
tolerance. Regenerate with:
    .venv\\Scripts\\python.exe rust\\oracle\\dump_slice_v.py > rust\\oracle\\slice_v_pypy.tsv
    <cpython>\\python.exe      rust\\oracle\\dump_slice_v.py > rust\\oracle\\slice_v_cpython.tsv
"""
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


def tag(key):
    """A key whose PRESENCE is the value -- a discrete label (`binding/lp`, `regime/mixed`).
    Asking the golden for a label the run did not take is a missing-key failure, which IS the
    assertion; no string ever has to be compared."""
    OUT.append((key, 1))


# ============================================================================== the shared grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR, V = 0.55, 0.20
LO, HI, SETTLE = 1000.0, 1400.0, 1.2
MARGIN = 0.25
V_HP = 0.10                      # `test_rung59.py:47` -- the HP branch's constant setting
# THE TWO KNEES, and the two steps. See the table in this file's docstring.
N_LO_57 = 0.75574
N_LO_589 = 0.7557
DS_01 = 0.01
DS_005 = 0.005

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
TILT_LP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
TILT_HP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
FLAT_LP = ComponentMap(sigma=0.1, l=0.7).with_phi_surge(FLOOR)
FLAT_HP = ComponentMap(sigma=0.1, l=1.0).with_phi_surge(FLOOR)
T_C = LP.tan_beta1_crit()
RATES = (0.1, 0.25, 0.5, 1.0, 2.0)
# `test_rung60.py:59` -- the three admissible (v, m_lim) pairs.
ADMISSIBLE = ((0.05, 0.500), (0.10, 0.509), (0.15, 0.518))


def _cpg():
    """`(gamma-1)/gamma*cp`, NOT `0.4/1.4` -- `1.4 - 1.0` is `0.3999999999999999`, and the two
    spellings put `R_c` two ULPs apart, which moves every number in this file."""
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=1.3, cp_t=1239.0, R_t=(1.3 - 1.0) / 1.3 * 1239.0, hPR=42.8e6)


def _design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _st(lp=LP, hp=HP, design=None, **kw):
    return ScheduledStatorTransient(design if design is not None else _design(), FLIGHT, 1.0,
                                    map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _ramp(ft, r=0.5, ds=DS_01):
    """`test_rung57.py:80` / `probe_p7o.py`'s `_ramp`, verbatim."""
    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, LO), ft.fuel_for_Tt4(FLIGHT, HI)
    eq = ft.equilibrium(FLIGHT, LO)
    return ft.integrate_fuel(FLIGHT, lambda s: mf0 + (mf1 - mf0) * min(1.0, s / r),
                             (eq["nu_lp"], eq["nu_hp"]), r + SETTLE, ds)


# ------------------------------------------------------------------------------- put_* helpers
PT_KEYS = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
           "mdot_air", "sp_thrust", "mf", "mf_sched")


def put_traj(p, traj, stride):
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


CELL_KEYS = ("m_i", "m_i_grid", "m_phi", "s", "v", "s_grid", "min_phi", "nu0", "nu_lp_end",
             "nu_hp_end", "Tt4_peak", "fuel_removed", "s_eng")


def put_cell(p, c):
    for k in CELL_KEYS:
        f(f"{p}/{k}", c[k])
    d(f"{p}/npts", c["npts"])
    d(f"{p}/prof_len", len(c["prof"]))
    for i in (0, len(c["prof"]) // 2, len(c["prof"]) - 1):
        f(f"{p}/prof/{i}/s", c["prof"][i][0])
        f(f"{p}/prof/{i}/m", c["prof"][i][1])


def put_accel(p, a):
    d(f"{p}/n", len(a.n_H))
    f(f"{p}/margin", a.margin)
    for i in range(len(a.n_H)):
        f(f"{p}/n_H/{i}", a.n_H[i])
        f(f"{p}/kappa/{i}", a.kappa[i])


def put_audit(p, au):
    for k in ("lo", "hi", "n_min", "n_max", "cut_lo", "cut_hi"):
        f(f"{p}/{k}", au[k])
    d(f"{p}/n_cuts", au["n_cuts"])
    d(f"{p}/clamped", au["clamped"])


def put_pin(p, au):
    for k in ("m_set", "m_min", "residual", "s_eng", "removed"):
        f(f"{p}/{k}", au[k])
    for k in ("pinned", "dormant", "from_zero", "admissible"):
        b(f"{p}/{k}", au[k])


TSM_F = ("margin_min_lp", "margin_min_hp", "steady_min_lp", "steady_min_hp",
         "phi_surge_lp", "phi_surge_hp")


def put_tsm(p, r, fuel=False):
    for k in TSM_F + (("min_phi_lp", "min_phi_hp") if fuel else ()):
        f(f"{p}/{k}", r[k])
    b(f"{p}/crossed_lp", r["crossed_lp"])
    b(f"{p}/crossed_hp", r["crossed_hp"])
    d(f"{p}/npts", r["npts"])


SM_F = ("SM_lp", "SM_hp", "x_lp", "x_hp", "phi_lp", "phi_hp", "n_lp", "n_hp", "pi_lpc",
        "pi_hpc", "slip", "Tt4")
EQ_F = ("Tt2", "Tt25", "Tt3", "Tt4", "Tt45", "Tt5", "f", "mdot_air", "mdot4", "nu_lp", "nu_hp",
        "n_lp", "n_hp", "phi_lp", "phi_hp", "pi_lpc", "pi_hpc", "pi_hpt", "pi_lpt", "slip",
        "sp_thrust", "pt4", "M9", "eta_lpc", "eta_hpc", "m_lp", "m_hp")
MATCH_F = ("n_lp", "n_hp", "slip", "phi_lp", "phi_hp", "eta_lpc", "eta_hpc", "eta_hpt",
           "eta_lpt", "nu_hpt", "nu_lpt")


def put_sm(p, r):
    for k in SM_F:
        f(f"{p}/{k}", r[k])
    tag(f"{p}/binding/{r['binding']}")


def put_eq(p, e):
    for k in EQ_F:
        f(f"{p}/{k}", e[k])
    tag(f"{p}/branch/{e['branch']}")


def put_match(p, m):
    for k in MATCH_F:
        f(f"{p}/{k}", getattr(m, k))


# =============================================================================================
# A -- THE POST-MARCH READERS.  `probe_p7o.py`'s grid and, crucially, its ORDER.
#      n_lo = 0.75574, ds = 0.01, r = 0.5.  This is checklist item (b).
# =============================================================================================
SCHED_57 = StatorSchedule(v_max=V, n_lo=N_LO_57)
ARMINGS = (("lp_only", dict(vsv_sched_lp=SCHED_57)),
           ("hp_only", dict(vsv_sched_hp=SCHED_57)),
           ("both", dict(vsv_sched_lp=SCHED_57, vsv_sched_hp=SCHED_57)),
           ("const_lp", dict(vsv_lp=V)))

for _tag, _kw in ARMINGS:
    p = f"A/{_tag}"
    t = _st(**_kw)
    b(f"{p}/map_lp_is_design_PRE", t.map_lp is t.map_lp_design)
    b(f"{p}/map_hp_is_design_PRE", t.map_hp is t.map_hp_design)
    f(f"{p}/pre_vsv_lp", t.map_lp.vsv)
    f(f"{p}/pre_vsv_hp", t.map_hp.vsv)
    traj = _ramp(t)
    put_traj(f"{p}/ramp", traj, stride=17)
    b(f"{p}/map_lp_is_design_POST", t.map_lp is t.map_lp_design)
    b(f"{p}/map_hp_is_design_POST", t.map_hp is t.map_hp_design)
    f(f"{p}/post_vsv_lp", t.map_lp.vsv)
    f(f"{p}/post_vsv_hp", t.map_hp.vsv)
    # --- THE READERS, IN p7o's ORDER. Each one re-arms; the next reads what it left. --------
    put_tsm(f"{p}/tsm", t.transient_surge_margin(FLIGHT, LO, HI, 0.5))
    f(f"{p}/after_tsm_vsv_lp", t.map_lp.vsv)
    f(f"{p}/after_tsm_vsv_hp", t.map_hp.vsv)
    put_tsm(f"{p}/tsmf", t.transient_surge_margin_fuel(FLIGHT, LO, HI, 0.5), fuel=True)
    f(f"{p}/after_tsmf_vsv_lp", t.map_lp.vsv)
    f(f"{p}/after_tsmf_vsv_hp", t.map_hp.vsv)
    put_sm(f"{p}/sm", t.surge_margin(FLIGHT, LO))
    f(f"{p}/v_of_lp", t.v_of("lp", 0.9, 0.9))
    f(f"{p}/v_of_hp", t.v_of("hp", 0.9, 0.9))
    put_match(f"{p}/match", t.match(FLIGHT, LO))
    put_eq(f"{p}/eq", t.equilibrium(FLIGHT, LO))
    f(f"{p}/fuel_for_Tt4", t.fuel_for_Tt4(FLIGHT, LO))
    stm = t.stator_transient_margin(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01)
    put_read(f"{p}/stm", stm)
    f(f"{p}/stm/nu0_lp", stm["nu0_lp"])
    f(f"{p}/stm/nu0_hp", stm["nu0_hp"])
    f(f"{p}/stm/r", stm["r"])
    f(f"{p}/final_vsv_lp", t.map_lp.vsv)
    f(f"{p}/final_vsv_hp", t.map_hp.vsv)
    # the two rung-53 channels the arming drives, at whatever the chain left
    f(f"{p}/final_phi_surge_at", t.map_lp.phi_surge_at())
    f(f"{p}/final_psi", t.map_lp.psi(0.62))

# =============================================================================================
# A' -- ADDED. NO SUITE RUNS THIS. The same reading with `dTt4` given a DELTA, on FRESH objects.
# =============================================================================================
for _tag, _kw in ARMINGS:
    t = _st(**_kw)
    _ramp(t)
    put_tsm(f"Ax/{_tag}/tsm_delta", t.transient_surge_margin(FLIGHT, LO, HI - LO, 0.5))

# =============================================================================================
# B -- ADDED. THE HP-SCHEDULED MACHINE -- section 5.20 P4's ungated arm, checklist item (a).
#      Rung 57's grid, because the lever is rung 57's.
# =============================================================================================
HPS = dict(vsv_sched_hp=SCHED_57)
m = _st(**HPS)
stm = m.stator_transient_margin(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01)
put_read("B/stm", stm)
f("B/stm/nu0_lp", stm["nu0_lp"])
f("B/stm/nu0_hp", stm["nu0_hp"])
f("B/stale_vsv_hp", m.map_hp.vsv)
f("B/stale_vsv_lp", m.map_lp.vsv)
f("B/stale_phi_surge_at_hp", m.map_hp.phi_surge_at())
f("B/stale_psi_hp", m.map_hp.psi(0.62))

CK = ("bare", "armed", "pointwise", "credit", "credit_pointwise", "erosion", "closed_form",
      "v_at_min", "s_at_min", "s_at_min_bare", "nu0_bare", "nu0_armed", "min_phi_bare",
      "min_phi_armed", "m_phi_bare", "m_phi_armed", "r")
for sp in ("hp", "lp"):
    c = _st(**HPS).stator_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01, spool=sp)
    for k in CK:
        f(f"B/credit_{sp}/{k}", c[k])
    b(f"B/credit_{sp}/pointwise_exact", c["pointwise_exact"])

DK = ("bare", "start", "ramp", "full", "share_start", "share_ramp", "self_cancel",
      "nu0_bare", "nu0_armed")
dc = _st(**HPS).credit_decomposition(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01,
                                     spool="hp")
for k in DK:
    f(f"B/dec/{k}", dc[k])

HK = ("predicted", "profile_bare", "profile_fuel", "credit_bare", "credit_fuel", "interaction",
      "share", "v_bare", "v_fuel", "v_ratio", "relocation", "relocation_bare", "leg_cost_bare",
      "leg_cost_armed", "fuel_removed_bare", "fuel_removed_armed", "r", "ds")
ACC_57 = _st().accel_schedule(FLIGHT, LO, HI, MARGIN)
put_accel("B/accel", ACC_57)
cc = _st(**HPS).composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01, spool="hp",
                                 accel=ACC_57)
for k in HK:
    f(f"B/comp/{k}", cc[k])
tag(f"B/comp/leg/{cc['leg']}")
for _c in ("neither", "stator", "fuel", "both"):
    put_cell(f"B/comp/{_c}", cc["cells"][_c])

inv = _st(**HPS).schedule_invariance(FLIGHT, LO, HI, MARGIN)
b("B/inv/ordinate_identical", inv["ordinate_identical"])
b("B/inv/abscissa_identical", inv["abscissa_identical"])
f("B/inv/d_ordinate", inv["d_ordinate"])
f("B/inv/d_abscissa", inv["d_abscissa"])
d("B/inv/chain_n", len(inv["chain"]))
for i, row in enumerate(inv["chain"]):
    for k in ("Tt4", "d_Tt25", "d_Tt3", "d_f", "d_mfp", "d_ratio", "d_kappa", "d_n_hp",
              "d_nu_lp"):
        f(f"B/inv/chain/{i}/{k}", row[k])

MK = ("credit_bare", "interaction_bare_leg", "interaction_matched", "delta_match",
      "delta_index", "delta_value", "abscissa_share", "ordinate_share", "share_bare_leg",
      "share_matched", "s_eng_bare_leg", "s_eng_matched", "removed_bare_leg",
      "removed_matched", "relocation", "d_ordinate", "d_abscissa", "margin", "r", "ds")
mc = _st(**HPS).matched_credit(FLIGHT, LO, HI, MARGIN, r=0.5, s_settle=SETTLE, ds=DS_01,
                               spool="hp")
for k in MK:
    f(f"B/matched/{k}", mc[k])
b("B/matched/ordinate_identical", mc["ordinate_identical"])
b("B/matched/abscissa_identical", mc["abscissa_identical"])
for _c in ("neither", "stator", "fuel", "both_bare_leg", "both_matched", "both_reindexed",
           "both_revalued"):
    put_cell(f"B/matched/{_c}", mc["cells"][_c])
for _a in ("fuel", "both_bare_leg", "both_matched"):
    put_audit(f"B/matched/audit/{_a}", mc["audits"][_a])

# =============================================================================================
# C -- RUNG 57's readers, on `test_rung57.py`'s grid (n_lo = 0.75574, ds = 0.01)
# =============================================================================================
D57 = _design()


def st57(lp=LP, hp=HP, **kw):
    return _st(lp, hp, design=D57, **kw)


for _s, _kw in (("smooth", dict()), ("linear", dict(shape="linear"))):
    s = StatorSchedule(V, N_LO_57, **_kw)
    for i in range(13):
        f(f"C/sched/{_s}/{i}", s(0.60 + 0.05 * i))
    f(f"C/sched/{_s}/at_n_ref", s(1.0))
    f(f"C/sched/{_s}/at_n_lo", s(N_LO_57))
    f(f"C/sched/{_s}/v_max", s.v_max)
    f(f"C/sched/{_s}/n_lo", s.n_lo)
    f(f"C/sched/{_s}/n_ref", s.n_ref)

# the currency split -- `test_currency_split_replays_on_the_transient`
for _t, _kw in (("bare", dict()), ("shut", dict(vsv_lp=V))):
    r = st57(**_kw).stator_transient_margin(FLIGHT, LO, HI, r=0.5)
    put_read(f"C/currency/{_t}", r)
    f(f"C/currency/{_t}/nu0_lp", r["nu0_lp"])
    f(f"C/currency/{_t}/nu0_hp", r["nu0_hp"])

# P1 / P2 -- the constant-setting credit over five ramp rates, on BOTH map pairs
for _pair, lp, hp in (("primary", LP, HP), ("tilted", TILT_LP, TILT_HP)):
    for r in RATES:
        c = st57(lp, hp, vsv_lp=V).stator_credit(FLIGHT, LO, HI, r=r)
        for k in CK:
            f(f"C/credit/{_pair}/r{r:.2f}/{k}", c[k])
        b(f"C/credit/{_pair}/r{r:.2f}/pointwise_exact", c["pointwise_exact"])

# the SCHEDULED credit, and the constant leg matched at its own minimum
g = st57(vsv_sched_lp=StatorSchedule(V, N_LO_57)).stator_credit(FLIGHT, LO, HI, r=0.5)
for k in CK:
    f(f"C/credit/sched/{k}", g[k])
b("C/credit/sched/pointwise_exact", g["pointwise_exact"])
cm = st57(vsv_lp=g["v_at_min"]).stator_credit(FLIGHT, LO, HI, r=0.5)
for k in CK:
    f(f"C/credit/matched_const/{k}", cm[k])

# P3 / P4 -- the decomposition over the same five rates
for r in RATES:
    dc = st57(vsv_sched_lp=StatorSchedule(V, N_LO_57)).credit_decomposition(
        FLIGHT, LO, HI, r=r)
    for k in DK:
        f(f"C/dec/r{r:.2f}/{k}", dc[k])

# P5 -- the arrow toggle at a FIXED transient state
AK = ("v", "s", "nu_lp", "nu_hp", "d_phi_lp", "d_phi_hp", "d_n_hp", "d_Tt25", "phi_lp",
      "phi_hp")
a0 = st57().arrow_toggle(FLIGHT, LO, HI, V, spool="lp")
for k in AK:
    f(f"C/arrow/seed/{k}", a0[k])
STATE = a0["state"]
for i, v in enumerate(STATE):
    f(f"C/arrow/state/{i}", v)
for _pair, lp, hp in (("shaped", LP, HP), ("flat", FLAT_LP, FLAT_HP)):
    for sp in ("lp", "hp"):
        a = st57(lp, hp).arrow_toggle(FLIGHT, LO, HI, V, spool=sp, state=STATE)
        for k in AK:
            f(f"C/arrow/{_pair}/{sp}/{k}", a[k])

for sp in ("lp", "hp"):
    for _i, (nl, nh) in enumerate(((0.80, 0.90), (0.95, 1.00))):
        f(f"C/v_of/{sp}/{_i}",
          st57(vsv_sched_lp=StatorSchedule(V, N_LO_57)).v_of(sp, nl, nh))

# THE REDUCE -- an unarmed rung-57 march IS rung 43/45's
bare43 = TwoSpoolFuelTransient(D57, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
put_traj("C/reduce/r43", _ramp(bare43), stride=17)
put_traj("C/reduce/r57_unarmed",
         st57()._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_01)[0], stride=17)
Z57 = StatorSchedule(0.0, 0.75)
put_traj("C/reduce/r57_zero_lp",
         st57(vsv_sched_lp=Z57)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_01)[0],
         stride=17)
put_traj("C/reduce/r57_zero_both",
         st57(vsv_sched_lp=Z57, vsv_sched_hp=Z57)._stator_march(
             FLIGHT, LO, HI, 0.5, SETTLE, DS_01)[0], stride=17)

# =============================================================================================
# D -- RUNG 58's readers, on `test_rung58.py`'s grid (n_lo = 0.7557, ds = 0.01)
# =============================================================================================
D58 = _design()


def st58(**kw):
    return _st(design=D58, **kw)


def sched58(v_max=V, n_lo=N_LO_589):
    return StatorSchedule(v_max=v_max, n_lo=n_lo)


ACC = st58().accel_schedule(FLIGHT, LO, HI, MARGIN)
put_accel("D/accel", ACC)
put_accel("D/accel_dormant", st58(vsv_sched_lp=sched58()).accel_schedule(FLIGHT, LO, HI, 0.60))

# the four cells, each arming, at r = 0.5 -- `_composite("sched")` / `_composite("const")`
for _t, _kw in (("sched", dict(vsv_sched_lp=sched58())), ("const", dict(vsv_lp=V))):
    cc = st58(**_kw).composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01,
                                      accel=ACC)
    for k in HK:
        f(f"D/comp/{_t}/{k}", cc[k])
    tag(f"D/comp/{_t}/leg/{cc['leg']}")
    for _c in ("neither", "stator", "fuel", "both"):
        put_cell(f"D/comp/{_t}/{_c}", cc["cells"][_c])

# P4's ramp-rate sweep -- the r = 2.00 row is the DORMANT envelope edge and is emitted with it
for r in (0.15, 0.25, 0.50, 1.00, 2.00):
    cc = st58(vsv_sched_lp=sched58()).composite_credit(FLIGHT, LO, HI, r=r, s_settle=SETTLE,
                                                       ds=DS_01, accel=ACC)
    for k in HK:
        f(f"D/rate/r{r:.2f}/{k}", cc[k])
    for _c in ("neither", "stator", "fuel", "both"):
        put_cell(f"D/rate/r{r:.2f}/{_c}", cc["cells"][_c])

es = st58(vsv_sched_lp=sched58()).engagement_shift(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE,
                                                   ds=DS_01, accel=ACC)
for k in ("bare_limited", "bare_dormant", "armed_limited", "armed_dormant", "d_limited",
          "d_dormant", "rel_limited", "rel_dormant", "r", "ds"):
    f(f"D/eng/{k}", es[k])
tag(f"D/eng/leg/{es['leg']}")

SWEEP_LEGS = [(f"n_lo={x}", dict(vsv_sched_lp=sched58(n_lo=x))) for x in (0.60, N_LO_589, 0.86)]
SWEEP_LEGS.append(("const", dict(vsv_lp=V)))
sw = st58().interaction_sweep(FLIGHT, LO, HI, SWEEP_LEGS, r=0.5, s_settle=SETTLE, ds=DS_01,
                              accel=ACC)
d("D/sweep/n", len(sw))
for _i, row in enumerate(sw):
    tag(f"D/sweep/{_i}/tag/{row['tag']}")
    for k in ("credit_bare", "credit_fuel", "interaction", "share", "v_bare", "v_fuel",
              "v_ratio", "relocation", "leg_cost_bare", "leg_cost_armed"):
        f(f"D/sweep/{_i}/{k}", row[k])
f("D/sweep/why_saturated", sched58(n_lo=0.86)(0.94))

# P5's two phi floors, on both armings -- the pinned identity
for _t, _kw in (("const", dict(vsv_lp=V)), ("sched", dict(vsv_sched_lp=sched58()))):
    for floor in (0.7450, 0.7500):
        cc = st58(**_kw).composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01,
                                          surge=SurgeLimiter(spool="lp", phi_lim=floor))
        for k in HK:
            f(f"D/floor/{_t}/{floor:.4f}/{k}", cc[k])
        tag(f"D/floor/{_t}/{floor:.4f}/leg/{cc['leg']}")
        for _c in ("fuel", "both"):
            put_cell(f"D/floor/{_t}/{floor:.4f}/{_c}", cc["cells"][_c])

# P5's DISJOINT windows
for _t, _kw in (("bare", dict()), ("sched", dict(vsv_sched_lp=sched58())),
                ("const", dict(vsv_lp=V))):
    traj, _ = st58(**_kw)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_01)
    f(f"D/window/{_t}/min_phi", min(p["phi_lp"] for p in traj))
    f(f"D/window/{_t}/phi_0", traj[0]["phi_lp"])

# the DORMANT-leg reduce -- armed but never binding must leave the march alone
_mm = st58(vsv_sched_lp=sched58())
put_traj("D/dormant/base", _mm._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_01)[0], stride=17)
put_traj("D/dormant/accel",
         _mm._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_01,
                           accel=_mm.accel_schedule(FLIGHT, LO, HI, 0.60))[0], stride=17)
put_traj("D/dormant/surge",
         _mm._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_01,
                           surge=SurgeLimiter(spool="lp", phi_lim=0.50))[0], stride=17)

# =============================================================================================
# E -- RUNG 59's readers, on `test_rung59.py`'s grid (n_lo = 0.7557, ds = 0.01)
# =============================================================================================
D59 = _design()


def st59(**kw):
    return _st(design=D59, **kw)


for _t, _kw in (("lp_const", dict(vsv_lp=V)), ("lp_sched", dict(vsv_sched_lp=sched58())),
                ("hp_const", dict(vsv_hp=V_HP))):
    inv = st59(**_kw).schedule_invariance(FLIGHT, LO, HI, MARGIN)
    b(f"E/inv/{_t}/ordinate_identical", inv["ordinate_identical"])
    b(f"E/inv/{_t}/abscissa_identical", inv["abscissa_identical"])
    f(f"E/inv/{_t}/d_ordinate", inv["d_ordinate"])
    f(f"E/inv/{_t}/d_abscissa", inv["d_abscissa"])
    put_accel(f"E/inv/{_t}/bare", inv["bare"])
    put_accel(f"E/inv/{_t}/matched", inv["matched"])
    for i, row in enumerate(inv["chain"]):
        for k in ("Tt4", "d_Tt25", "d_Tt3", "d_f", "d_mfp", "d_ratio", "d_kappa", "d_n_hp",
                  "d_nu_lp"):
            f(f"E/inv/{_t}/chain/{i}/{k}", row[k])

for tt in (1000.0, 1200.0, 1400.0):
    pc = st59()._proof_chain(FLIGHT, tt)
    for k in ("Tt4", "Tt25", "Tt3", "f", "mfp", "ratio", "kappa", "n_hp", "nu_lp"):
        f(f"E/chain/{tt:.0f}/{k}", pc[k])

# the v = 0 tuple-identity reduce, both arming routes
put_accel("E/reduce/bare", st59().accel_schedule(FLIGHT, LO, HI, MARGIN))
put_accel("E/reduce/sched_zero",
          st59(vsv_sched_lp=sched58(v_max=0.0)).accel_schedule(FLIGHT, LO, HI, MARGIN))
put_accel("E/reduce/const_zero",
          st59(vsv_lp=0.0).accel_schedule(FLIGHT, LO, HI, MARGIN))
put_accel("E/reduce/synthetic",
          ScheduledStatorTransient._synthetic_leg(ACC, ACC))

for _t, _kw, _sp in (("sched_lp", dict(vsv_sched_lp=sched58()), "lp"),
                     ("const_lp", dict(vsv_lp=V), "lp"),
                     ("hp_on_lp", dict(vsv_hp=V_HP), "lp"),
                     ("hp_on_hp", dict(vsv_hp=V_HP), "hp")):
    mc = st59(**_kw).matched_credit(FLIGHT, LO, HI, MARGIN, r=0.5, s_settle=SETTLE, ds=DS_01,
                                    spool=_sp)
    for k in MK:
        f(f"E/matched/{_t}/{k}", mc[k])
    b(f"E/matched/{_t}/ordinate_identical", mc["ordinate_identical"])
    b(f"E/matched/{_t}/abscissa_identical", mc["abscissa_identical"])
    for _c in ("neither", "stator", "fuel", "both_bare_leg", "both_matched", "both_reindexed",
               "both_revalued"):
        put_cell(f"E/matched/{_t}/{_c}", mc["cells"][_c])
    for _a in ("fuel", "both_bare_leg", "both_matched"):
        put_audit(f"E/matched/{_t}/audit/{_a}", mc["audits"][_a])

# rung 58's own reader, still reporting its published object on rung 59's machine
_m59 = st59(vsv_sched_lp=sched58())
_L59 = _m59.at_stator().accel_schedule(FLIGHT, LO, HI, MARGIN)
put_accel("E/at_stator_leg", _L59)
es = _m59.engagement_shift(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_01, accel=_L59)
for k in ("bare_limited", "bare_dormant", "armed_limited", "armed_dormant", "rel_limited",
          "rel_dormant"):
    f(f"E/eng58/{k}", es[k])
tag(f"E/eng58/leg/{es['leg']}")

# =============================================================================================
# F -- RUNG 60's readers, on `test_rung60.py`'s grid (n_lo = 0.7557, ds = 0.005 -- HALF the step)
# =============================================================================================
D60 = _design()


def st60(**kw):
    return _st(design=D60, **kw)


f("F/T_c", T_C)
for sm in (0.0, 0.02, 0.05, 0.10, 0.25):
    for v in (0.0, 0.05, 0.20):
        mr = st60(vsv_lp=V).matching_rules(sm, v)
        for k in ("sm", "v", "T_c", "phi_bare", "m_bare", "phi_rel", "phi_inc", "gap",
                  "gap_closed_form", "residual"):
            f(f"F/rules/{sm:.2f}/{v:.2f}/{k}", mr[k])

sb = st60(vsv_lp=0.20).set_point_bands(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_005)
for k in ("gap_phi", "gap_m", "gap_phi_bands", "gap_m_bands", "credit", "excursion",
          "criterion", "identity_residual", "overlap_lo", "overlap_hi", "r", "ds"):
    f(f"F/bands/{k}", sb[k])
b("F/bands/phi_admissible", sb["phi_admissible"])
b("F/bands/m_admissible", sb["m_admissible"])
for side in ("bare", "armed"):
    for k in ("phi_0", "phi_min", "phi_exc", "m_0", "m_min", "m_exc", "T_c", "v_0"):
        f(f"F/bands/{side}/{k}", sb[side][k])

LAD_KEYS = ("r", "credit", "excursion", "criterion", "gap_m", "gap_m_bands", "gap_phi",
            "gap_phi_bands")
legs = [(f"const v={v}", dict(vsv_lp=v)) for v in (0.05, 0.15, 0.20)] + \
       [("sched v_max=0.20", dict(vsv_sched_lp=StatorSchedule(0.20, N_LO_589)))]
lad = st60().composability_ladder(FLIGHT, LO, HI, legs=legs, r=0.5, s_settle=SETTLE, ds=DS_005)
d("F/ladder_legs/n", len(lad))
for i, row in enumerate(lad):
    tag(f"F/ladder_legs/{i}/tag/{row['tag']}")
    for k in LAD_KEYS:
        f(f"F/ladder_legs/{i}/{k}", row[k])
    b(f"F/ladder_legs/{i}/m_admissible", row["m_admissible"])
    b(f"F/ladder_legs/{i}/phi_admissible", row["phi_admissible"])

rates = [(r, dict(vsv_lp=0.20)) for r in (0.15, 0.25, 0.50, 0.75, 1.00)]
lad_r = st60().composability_ladder(FLIGHT, LO, HI, rates=rates, s_settle=SETTLE, ds=DS_005)
d("F/ladder_rates/n", len(lad_r))
for i, row in enumerate(lad_r):
    tag(f"F/ladder_rates/{i}/tag/{row['tag']}")
    for k in LAD_KEYS:
        f(f"F/ladder_rates/{i}/{k}", row[k])
    b(f"F/ladder_rates/{i}/m_admissible", row["m_admissible"])
    b(f"F/ladder_rates/{i}/phi_admissible", row["phi_admissible"])

FK = ("credit_bare", "credit_fuel", "interaction", "pinned_prediction", "pinned_residual",
      "s_eng_bare", "s_eng_armed", "d_s_eng", "removed_bare", "removed_armed", "v_at_min",
      "r", "ds")
FLOORS = [(f"inc_v{v:.2f}", v, IncidenceLimiter(spool="lp", m_lim=ml)) for v, ml in ADMISSIBLE] + \
         [(f"phi_v{v:.2f}", v, SurgeLimiter(spool="lp", phi_lim=0.750)) for v in (0.15, 0.20)] + \
         [("clears", 0.15, IncidenceLimiter(spool="lp", m_lim=0.490))]
for _t, v, floor in FLOORS:
    fc = st60(vsv_lp=v).floor_composite(FLIGHT, LO, HI, floor, r=0.5, s_settle=SETTLE,
                                        ds=DS_005)
    f(f"F/floor/{_t}/v_set", v)
    for k in FK:
        f(f"F/floor/{_t}/{k}", fc[k])
    tag(f"F/floor/{_t}/regime/{fc['regime']}")
    tag(f"F/floor/{_t}/kind/{fc['floor']}")
    b(f"F/floor/{_t}/admissible", fc["admissible"])
    for _c in ("neither", "stator", "fuel", "both"):
        put_cell(f"F/floor/{_t}/{_c}", fc["cells"][_c])
    for _a in ("fuel", "both"):
        put_pin(f"F/floor/{_t}/audit/{_a}", fc["audits"][_a])

# THE REDUCE -- an incidence floor at v = 0 IS rung 49's phi floor, float-identical
_inc = IncidenceLimiter(spool="lp", m_lim=0.500)
_phi = SurgeLimiter(spool="lp", phi_lim=1.0 / (T_C - 0.500))
f("F/reduce/inc_at_zero", _inc.at(T_C, 0.0).phi_lim)
f("F/reduce/phi_lim", _phi.phi_lim)
b("F/reduce/float_identical", _inc.at(T_C, 0.0).phi_lim == _phi.phi_lim)
put_traj("F/reduce/march_inc",
         st60()._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_005, surge=_inc)[0], stride=37)
put_traj("F/reduce/march_phi",
         st60()._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_005, surge=_phi)[0], stride=37)

# the incidence limiter's own value type, over the sm ladder -- rung 60's conversion
for i, sm in enumerate((0.0, 0.05, 0.15, 0.30)):
    il = IncidenceLimiter.from_margin(LP, "lp", sm)
    f(f"F/inc/from_margin/{i}/m_lim", il.m_lim)
    for j, v in enumerate((0.0, 0.05, 0.20)):
        f(f"F/inc/from_margin/{i}/phi_lim_at/{j}", il.phi_lim_at(T_C, v))
il0 = IncidenceLimiter.from_phi(LP, "lp", 0.62)
f("F/inc/from_phi/m_lim", il0.m_lim)
f("F/inc/from_phi/roundtrip", il0.at(T_C, 0.0).phi_lim)
b("F/inc/from_phi/roundtrip_exact", il0.at(T_C, 0.0).phi_lim == 0.62)

# ---------------------------------------------------------------------------------- emit
print("# slice V step 4 -- rungs 57-60 ORACLE. key<TAB>u64 (float keys are IEEE-754 bits).")
# THE INTERPRETER STAMPS ITSELF. The two arms came back BYTE-IDENTICAL, and that is a claim
# about provenance as much as about arithmetic: two files generated by the same interpreter are
# also byte-identical, and nothing in the bytes said which one made them. The comparator skips
# `#` lines, so this costs nothing and makes the identity checkable.
print(f"# generated by {sys.implementation.name} {sys.version.split()[0]}")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
