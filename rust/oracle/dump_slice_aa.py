"""SLICE AA step 4 -- THE ORACLE for rung 68 (`StatorLimiter`, `ThreeLoopCascadeTransient`).

**THE GRID IS THE SUITE'S OWN AND NOTHING IS COARSENED**, and the header states it rather than
implying it -- slice S step 4's lesson, *a probe's HEADER claimed the suites' grids and its code
ran another*. Every argument below is copied from the calling gate in `tests/test_rung68.py`,
never chosen:

    ds          0.005 everywhere EXCEPT section D, whose `triple_modes` default is 0.002 --
                and that is `test_two_zero_eigenvalues_and_the_rates_add`'s own default.
    LO/HI       1000 -> 1400 . r 0.5 . s_settle 1.2 . FLOOR 0.55 -- the suite's throughout
    phi/b/v     PHI 0.80, B 0.10, V_MAX 0.20, SM = PHI/FLOOR - 1 -- the suite's
    clocks       TAU 0.05 (valve), TAU_S 0.05 (stator), TAU_ATT 0.05 / TAU_REL 0.15 (fuel)
    clocks grid  ((.05,.05,.05), (.05,.005,.05), (.05,.5,.05), (.02,.05,.10)) -- `triple_modes`'
                 own default, which gate 3 takes wholesale
    deltas       (0, 1e-4, 1e-3, 1e-2, 3e-2) -- `cyclic_sensitivity`'s default, gate 2's
    v_max_sat    0.02 -- `saturation_counterfeit`'s default, gate 6's
    orders       the six permutations . starts (None, 0.0, 0.02, 0.06) -- `ic_family`'s, gate 9's
    maps         `shaped` ONLY (LP a=.20 b=.05 l=.7 / HP a=.08 b=.15 l=1.0) -- the suite builds
                 no second shape, so a second one here would be a grid it does not have

# THE ONE DECLARED CROSS-INTERPRETER EXEMPTION, AND IT IS A SET OF **NAMES**

Rung 68 has NINE float `sum()` sites against slice Z's one, which reads like nine times the
exposure and is not. Probe 4 intercepted what the readers actually sum and probe 5 re-summed
those lists under both interpreters against a naive left fold:

  * **eight of the nine sum THREE or FOUR numbers** (`triple_modes`' `c1`/`c2`/`rate`,
    `triple_bill`'s `sum_singles`, `saturation_counterfeit`'s two, `cyclic_sensitivity`'s two)
    and agree on both interpreters, because a compensation has nowhere to accumulate;
  * **the ninth -- `ic_family`'s `withheld` -- sums 101 trajectory terms** and differs on CPython
    3.12+ in 2 of 10 instances measured.

So the exemption is one READER, not nine sites. **`EXEMPT` below is a set of KEY NAMES and not a
count**: [[rust-port-slice-z-step4]] is a pre-registered exemption of TWO keys that measured
EIGHT, because it counted quantities where the dump emits names.

Every float is emitted as its IEEE-754 bit pattern. Regenerate BOTH arms:

    .venv/Scripts/python.exe rust/oracle/dump_slice_aa.py > rust/oracle/slice_aa_pypy.tsv
    C:/Python314/python.exe  rust/oracle/dump_slice_aa.py > rust/oracle/slice_aa_cpython.tsv

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
    ThreeLoopCascadeTransient, TwoLagCascadeTransient,
    BleedLimiter, BleedSchedule, StatorLimiter, SurgeLimiter, AsymmetricLag,
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
    a sentinel float would conflate a missing value with a real one."""
    b(key + "?", x is not None)
    if x is not None:
        f(key, x)


def s(key, text):
    """A STRING key, as an FNV-1a 64-bit hash -- `v_regime` and `ic_order` are the two non-floats
    a rung-68 trajectory carries, and the regime is the one thing no float can witness."""
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


# ---------------------------------------------------------------------------- the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU, TAU_S = 0.05, 0.05
TAU_ATT, TAU_REL = 0.05, 0.15
CLOCKS = ((0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05), (0.02, 0.05, 0.10))
DELTAS = (0.0, 1e-4, 1e-3, 1e-2, 3e-2)
V_MAX_SAT = 0.02
ORDERS = ("gqv", "gvq", "qgv", "qvg", "vgq", "vqg")
STARTS = (None, 0.0, 0.02, 0.06)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """The suite's `_cpg`, character for character. `R_c` is DERIVED from `(gamma - 1)/gamma`;
    re-spelling it `0.4/1.4` builds a gas ONE ULP away, which presents exactly as a port defect
    (slice Y's own false alarm)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def three(**kw):
    return ThreeLoopCascadeTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def two(**kw):
    return TwoLagCascadeTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


def stator(tau=TAU_S, v_max=V_MAX):
    return StatorLimiter(phi_lim=PHI, v_max=v_max, tau=tau)


def fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


def march(m, ds=DS, **kw):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, ds, **kw)[0]


def put_point(p, pt):
    """The FOURTEEN keys every route emits, plus whatever else this one carries. Emitted by
    ITERATING the dict rather than by a typed list, so a key the port forgets shows up as a COUNT
    mismatch and not as a silently-unread field."""
    d(p + "/nkeys", len(pt))
    for k in sorted(pt):
        v = pt[k]
        if isinstance(v, str):
            s("%s/%s" % (p, k), v)
        elif isinstance(v, bool):
            b("%s/%s" % (p, k), v)
        elif isinstance(v, int):
            d("%s/%s" % (p, k), v)
        else:
            f("%s/%s" % (p, k), v)


def put_traj(p, traj, stride=1):
    d(p + "/npts", len(traj))
    for i, pt in enumerate(traj):
        if i % stride == 0 or i == len(traj) - 1:
            put_point("%s/%d" % (p, i), pt)


# ===================================================== A -- THE REDUCE, key for key
# Gate 1's four inherited arms plus the rung-66 one. The comparison the SUITE makes is against a
# rung-66 machine; what the ORACLE adds is that the rung-68 side's own values are pinned, so a
# port that broke BOTH sides identically -- the shape a same-run difference cannot see -- still
# goes red here.
for i, (kw, mkw) in enumerate((
        (dict(bleed_lim=valve(TAU)), dict(surge=fuel(), lag=lag())),          # rung 66
        (dict(bleed_lim=valve(TAU)), dict(surge=fuel())),                     # rung 65
        (dict(), dict(surge=fuel(), lag=lag())),                              # rung 52
        (dict(bleed_lim=valve()), dict()),                                    # rung 64
        (dict(bleed_sched=BleedSchedule(B, 0.65)), dict()))):                 # rung 62
    ta = march(three(**kw), **mkw)
    tb = march(two(**kw), **mkw)
    put_traj("A/%d/three" % i, ta, stride=37)
    put_traj("A/%d/two" % i, tb, stride=37)
    b("A/%d/identical" % i, [tuple(sorted(x.items())) for x in ta]
                            == [tuple(sorted(y.items())) for y in tb])

# ===================================================== B -- THE ARMED FIVE-STATE MARCH
# The rung's own machine. EVERY point, all 24 keys -- this is the bulk of the dump and the only
# section that can catch a defect in the RK4 loop itself.
M = three(bleed_lim=valve(TAU), stator_lim=stator())
TRAJ = march(M, surge=fuel(), lag=lag())
put_traj("B/traj", TRAJ)
f("B/violation", M._violation(TRAJ, PHI, R))
f("B/violation_inc", M._violation_inc(
    TRAJ, LP.tan_beta1_crit() - 1.0 / PHI, LP.tan_beta1_crit(), R))
d("B/n_riding", len(M._riding(TRAJ, B)))
for name, reg in (("dormant", "dormant"), ("riding", "riding"), ("saturated", "saturated")):
    d("B/regime/%s" % name, sum(1 for p in TRAJ if p["v_regime"] == reg))

# The SATURATING machine, which is the only one that reaches the third regime.
MS = three(bleed_lim=valve(TAU), stator_lim=stator(v_max=V_MAX_SAT))
TS = march(MS, surge=fuel(), lag=lag())
put_traj("B/sat", TS, stride=17)
for name in ("dormant", "riding", "saturated"):
    d("B/sat/regime/%s" % name, sum(1 for p in TS if p["v_regime"] == name))

# ===================================================== C -- THE SIX CROSS-GAINS
G = M.triple_gains(FLIGHT, LO, HI, sm=SM)
d("C/n_riding", G["n_riding"])
d("C/n_sampled", G["n_sampled"])
d("C/n_rows", len(G["rows"]))
d("C/n_skipped", len(G["skipped"]))
opt("C/worst_on", G["worst_on"])
opt("C/worst_live", G["worst_live"])
if G["s_window"] is not None:
    f("C/s_lo", G["s_window"][0])
    f("C/s_hi", G["s_window"][1])
for i, row in enumerate(G["rows"]):
    f("C/%d/s" % i, row["s"])
    for side in ("on", "live"):
        g = row[side]
        b("C/%d/%s/interior" % (i, side), g["interior"])
        for k in ("R_q", "R_v", "C_g", "C_v", "V_g", "V_q", "v_base", "cyclic",
                  "pair_RC", "pair_RV", "pair_CV"):
            f("C/%d/%s/%s" % (i, side, k), g[k])

# ===================================================== D -- THE SPECTRUM (ds = 0.002, the reader's)
MODES = M.triple_modes(FLIGHT, LO, HI, sm=SM)
d("D/n_arms", len(MODES["arms"]))
f("D/ds", MODES["ds"])
for i, arm in enumerate(MODES["arms"]):
    for j, t in enumerate(arm["taus"]):
        f("D/%d/tau/%d" % (i, j), t)
    f("D/%d/rate_sum" % i, arm["rate_sum"])
    d("D/%d/n" % i, arm["n"])
    d("D/%d/n_sampled" % i, arm["n_sampled"])
    d("D/%d/skipped" % i, arm["skipped"])
    d("D/%d/n_rows" % i, len(arm["rows"]))
    opt("D/%d/worst_zero" % i, arm["worst_zero"])
    opt("D/%d/dom_lo" % i, arm["dom_range"][0])
    opt("D/%d/dom_hi" % i, arm["dom_range"][1])
    for j, x in enumerate(arm["rows"]):
        for k in ("s", "c2", "c1", "c0", "cyclic", "dom"):
            f("D/%d/%d/%s" % (i, j, k), x[k])
        for kk, r in enumerate(x["roots"]):
            f("D/%d/%d/root/%d" % (i, j, kk), r)
        for kk, z in enumerate(x["zeros"]):
            f("D/%d/%d/zero/%d" % (i, j, kk), z)

# ===================================================== E -- THE DETECTOR'S SENSITIVITY
SENS = M.cyclic_sensitivity(FLIGHT, LO, HI, sm=SM)
f("E/s", SENS["s"])
f("E/floor", SENS["floor"])
opt("E/gain", SENS["gain"])
opt("E/resolves", SENS["resolves"])
d("E/n_rows", len(SENS["rows"]))
for i, row in enumerate(SENS["rows"]):
    f("E/%d/delta" % i, row["delta"])
    opt("E/%d/dep" % i, row["dep"])
    d("E/%d/n_off" % i, len(row["off_regime"]))
    if row["dep"] is not None:
        for k in ("cyclic", "pair_RC", "pair_RV", "pair_CV"):
            f("E/%d/%s" % (i, k), row[k])

# ===================================================== F -- THE 8-CELL LEDGER
BILL = M.triple_bill(FLIGHT, LO, HI, sm=SM)
f("F/phi_lim", BILL["phi_lim"])
f("F/m_lim", BILL["m_lim"])
f("F/sum_singles", BILL["sum_singles"])
f("F/delivered", BILL["delivered"])
for name in ("bare", "F", "V", "S", "FV", "FS", "VS", "FVS"):
    c = BILL["cells"][name]
    for k in ("I", "I_inc", "min_phi", "end_s", "v_min", "v_max_used", "b_max_used",
              "credit", "credit_inc"):
        f("F/%s/%s" % (name, k), c[k])
    d("F/%s/npts" % name, c["npts"])
    b("F/%s/v_saturated" % name, c["v_saturated"])
for k in ("fuel", "valve", "stator"):
    f("F/marginal/%s" % k, BILL["marginal"][k])
    f("F/marginal_inc/%s" % k, BILL["marginal_incidence"][k])
    f("F/singles/%s" % k, BILL["singles"][k])
    f("F/erosion/%s" % k, BILL["erosion"][k])

# ===================================================== G -- THE SATURATION COUNTERFEIT
SAT = M.saturation_counterfeit(FLIGHT, LO, HI, sm=SM)
f("G/v_max", SAT["v_max"])
d("G/n_saturated", SAT["n_saturated"])
d("G/n_riding", SAT["n_riding"])
d("G/n_rows", len(SAT["rows"]))
for i, row in enumerate(SAT["rows"]):
    f("G/%d/s" % i, row["s"])
    s("G/%d/regime" % i, row["regime"])
    d("G/%d/n_off" % i, len(row["off_regime"]))
    d("G/%d/n_zero" % i, row["n_zero"])
    for k in ("V_g", "V_q", "pair_RC", "pair_RV", "pair_CV", "c1", "c0"):
        f("G/%d/%s" % (i, k), row[k])
    for j, r in enumerate(row["roots"]):
        f("G/%d/root/%d" % (i, j), r)

# ===================================================== H -- THE IC FAMILY, and the EXEMPTION
FAM = M.ic_family(FLIGHT, LO, HI, sm=SM)
d("H/order_members", FAM["order_members"])
opt("H/start_spread_I", FAM["start_spread_I"])
opt("H/start_spread_withheld", FAM["start_spread_withheld"])
for o in ORDERS:
    x = FAM["by_order"][o]
    for k in ("g0", "b0", "v0", "res", "I", "min_phi", "withheld"):
        f("H/order/%s/%s" % (o, k), x[k])
    d("H/order/%s/iters" % o, x["iters"])
for st in STARTS:
    tag = "none" if st is None else ("%.4f" % st)
    x = FAM["by_start"][st]
    for k in ("g0", "b0", "v0", "res", "I", "min_phi", "withheld"):
        f("H/start/%s/%s" % (tag, k), x[k])
    d("H/start/%s/iters" % tag, x["iters"])

# ===================================================== I -- THE CUBIC SOLVER, DIRECTLY
# `_cubic_roots` is the one body in this slice with no precedent in the crate, and the risk
# registered against it was a cube-root spelling that MEASURED TO ZERO -- there is none, and the
# whole rung has one `**` at exponent 0.5. What survives is the BRANCH: `disc >= 0` picks the real
# arm and `disc < 0` reports Re twice, and the sections above reach only whichever arm the plant
# happens to visit. These call it directly, on both sides of the knife edge and on the degenerate
# `c2 == 0` start.
CUBICS = ((-60.0, 1.0e-8, -1.0e-14),      # the rung's own shape: {0, 0, c2}
          (-60.0, 0.0, 0.0),              # exactly rank one
          (0.0, 1.0, -1.0),               # c2 == 0 -> the `x = 1.0` start
          (-3.0, 3.0, -1.0),              # a triple root at 1
          (-2.0, 5.0, -10.0),             # disc < 0: a complex pair
          (-6.0, 11.0, -6.0),             # roots 1, 2, 3
          (1.0, -1.0, 1.0))
for i, (c2, c1, c0) in enumerate(CUBICS):
    f("I/%d/c2" % i, c2)
    f("I/%d/c1" % i, c1)
    f("I/%d/c0" % i, c0)
    for j, r in enumerate(ThreeLoopCascadeTransient._cubic_roots(c2, c1, c0)):
        f("I/%d/root/%d" % (i, j), r)

# ===================================================== J -- THE READERS OFF A MARCHED POINT
# `v_at_point` RE-READS and never re-solves, and `v_of` outside a march hands back the PARENT's
# answer -- a lagged setting is not a function of the state. Both are single lines that no other
# section's keys can distinguish from their wrong versions.
for i in (0, 60, 170, 340):
    f("J/v_at_point/%d" % i, M.v_at_point(TRAJ[i]))
    f("J/v_of_lp/%d" % i, M.v_of("lp", TRAJ[i]["nu_lp"], TRAJ[i]["nu_hp"]))
    f("J/v_of_hp/%d" % i, M.v_of("hp", TRAJ[i]["nu_lp"], TRAJ[i]["nu_hp"]))
b("J/v_of_is_parent_outside_a_march",
  M.v_of("lp", TRAJ[170]["nu_lp"], TRAJ[170]["nu_hp"]) == 0.0)

# ===================================================== K -- v_max, INERT AND BINDING
# Gate 5's own six runs, whose whole content is a COMPARISON between two machines. The oracle
# pins the six absolute numbers, which the comparison cannot.
for i, vm in enumerate((0.05, 0.10, 0.20, 0.02)):
    for j, with_valve in enumerate((True, False)):
        m = three(bleed_lim=valve(TAU) if with_valve else None,
                  stator_lim=stator(v_max=vm))
        t = march(m, surge=fuel() if with_valve else None,
                  lag=lag() if with_valve else None)
        f("K/%d/%d/I" % (i, j), m._violation(t, PHI, R))
        f("K/%d/%d/v_min" % (i, j), min(p["v"] for p in t))
        b("K/%d/%d/saturated" % (i, j), any(p["v_regime"] == "saturated" for p in t))
        d("K/%d/%d/npts" % (i, j), len(t))

# ===================================================== L -- THE tau_s LIMITS
for i, ts in enumerate((0.02, 0.5, 2.0, 10.0, 500.0)):
    m = three(bleed_lim=valve(TAU), stator_lim=stator(tau=ts))
    t = march(m, surge=fuel(), lag=lag())
    f("L/%d/tau_s" % i, ts)
    f("L/%d/I" % i, m._violation(t, PHI, R))
    f("L/%d/v_min" % i, min(p["v"] for p in t))

# ---------------------------------------------------------------------------- emit
print("# slice AA step 4 -- rung 68 ORACLE, the SUITE's grid, uncoarsened. key<TAB>u64 "
      "(floats are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
