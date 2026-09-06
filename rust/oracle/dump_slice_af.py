"""SLICE AF step 5 — **THE ORACLE for rung 74**, run once on PyPy and once on CPython.

Emits `key<TAB>u64` to a file, sorted, with the tally on stderr. Every float is its IEEE-754 bit
pattern, every string an FNV-1a 64-bit hash, every `None` a PRESENCE FLAG beside the value it
stands in for, and every sample-shaped reading carries its ROW COUNT.

    pypy   rust/oracle/dump_slice_af.py rust/oracle/slice_af_pypy.tsv
    python rust/oracle/dump_slice_af.py rust/oracle/slice_af_cpython.tsv

# THE GRIDS ARE THE SHIPPED CALLERS', AND THEY ARE READ OFF `engine.py`'s `def` LINES

§ 5.27.6 (i) burned a slice on transcribing one reader's stride into another. Rung 74's four
called readers pass at most `FLIGHT, LO, HI, TT4_MAX, phi_lim=`, so everything below the fifth
argument is the reader's OWN default:

    reader                    ds       every   caller
    demand_law                0.005    --      `main.py`, per (inc, phi) with `floors=(phi,)`
    demand_gains              0.002    4       `tests/test_rung74.py`, phi = 0.80
    latch_discriminator       0.005    --      NOTHING IN THE TREE  -> its own defaults
    windup_law                0.005    --      `tests/test_rung74.py`, phi = 0.76
    flat_schedule_identity    0.005    --      `tests/test_rung74.py`, Tt4_flat = 1150
    forcing_openloop          0.005    --      `tests/test_rung74.py`, phi = 0.76

**There is no overridden default anywhere in `tests/test_rung74.py`** — measured by reading all
four call sites, and recorded because rung 73's file had exactly one.

# THREE THINGS SECTIONS A-F CANNOT DO, AND THE TWO SECTIONS THAT DO THEM

**AN AGGREGATE IS LOSSY, SO SECTION G EMITS THE MARCH ITSELF.** A-F are the readers' folds over
trajectories; AD step 5 found its six drifting keys at 2 points of 1 302 only because the
equivalent section existed, and AE step 4's `n_pos_zero` deficit of exactly 2 resolved by NAME
only against per-point keys. G walks `_coord_march` at THREE coordinates x TWO floors and emits
every fifth point WHOLE, plus the `min`, `max` and LAST of every float column over ALL points.

**That backstop is real and it is weaker than "nothing can hide"** — AD step 5's close-out
measured a hidden-point defect moving 0 of 54 116 keys against a control that moved 10. What G
pins is one point in five plus both extremes and the endpoint. No more, and it is written that
way round deliberately.

**A CELL CAN BREAK BY EMPTYING THE SAMPLE**, so every list emits its length before its members,
and every `Option` a presence flag. Rung 74 has FOUR keys that are legitimately `None` on every
arm (§ 5.30.4 (a) measured them: the `first_gov` of the two ARREST arms' two demand tags, where
the plant never accelerates so the governor never takes the actuator). `None == None` agrees
perfectly and measures nothing — so their fourteen siblings are `Some`, the flag DISCRIMINATES,
and section Z emits the COUNT of `None`s as a key of its own.

**SECTION H IS A DECLARED EXTRA GRID**, not a shipped caller's: `demand_law` swept one floor at
a time the way `main.py` calls it, which reaches `arrested`/`redline_flips` rows the single
three-floor call in A folds together.

# THE SIGNS, THE ZEROS AND THE REFUSAL TEXT — the three hazards this rung supplies

* **`g_fuel`/`required_*` GO NEGATIVE.** They are unfloored projections here (`mf_sched - w`),
  and `cap > mf_sched` is reachable: 21 of 341 on `demand`, 0 of 341 on `demand-latched`. So no
  key below is normalised by a bare relative form and no positivity is assumed.
* **SIGNED ZERO.** Bit patterns are emitted, so `-0.0` and `+0.0` are DIFFERENT keys' values;
  section Z counts them, because AE step 4 found 63 keys flipping `+0.0`-ness between goldens.
* **`D/cell1/why` IS THE ONLY KEY THAT WITNESSES A MESSAGE.** One of `windup_law`'s four cells
  raises (`demand x applied` has no interior equilibrium — § 4's finding), and its text is
  hashed. Step 4 § (c) measured the port's own version of that message FOUR formatting
  divergences wide against Python's, found only because a reader read one as a value. An oracle
  that drops this key loses that entirely.

# P2, PRE-REGISTERED HERE BEFORE THE CPYTHON ARM HAS EVER RUN

§ 5.30 (iii) attributes **two of rung 74's four `sum()` calls to `forcing_openloop`** — the
largest share, and the only reader whose published quantity is an average over the ramp. CPython
3.12+'s `sum` is Neumaier-compensated; PyPy's and Rust's are naive left folds. **P2 names the
exemption in advance: `F/mean_delta_late`, `F/ratio_late`, `F/worst_rel_late`.** The other two
`sum()` calls are `demand_gains`'s (one) and `flat_schedule_identity`'s (one).

**THE FALSIFIER, stated so the prediction can lose**: if the CPython arm differs on a key OUTSIDE
that set and outside section G's plant keys, P2 is wrong and the cause is not `sum()`.
"""
import os
import struct
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, REPO)

from turbojet.gas import Gas                                              # noqa: E402
from turbojet.engine import (                                             # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    DemandCoordinateTransient,
    BleedLimiter, StatorIncidenceLimiter, StatorLimiter,
)

# ---------------------------------------------------------------- `tests/test_rung74.py`'s grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI = 1000.0, 1400.0
R, SETTLE = 0.5, 1.2
B, PHI, V_MAX = 0.10, 0.80, 0.20
TAU, TAU_S = 0.05, 0.05
TT4_MAX = 1200.0
PHI_ARREST, PHI_BOTH, PHI_GOV = 0.80, 0.76, 0.70
TAUS = (0.05, 0.05, 0.05, 0.05)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

COORDS3 = ("clip", "demand-latched", "demand")

#: Section G's stride. Every fifth point WHOLE — see the header on what that does and does not pin.
G_STRIDE = 5

OUT = []
#: Section Z's censuses, accumulated as the emitters run rather than recomputed afterwards.
N_NONE = [0]
N_NEG_ZERO = [0]
N_POS_ZERO = [0]


def f(key, x):
    v = float(x)
    bits = struct.unpack("<Q", struct.pack("<d", v))[0]
    if v == 0.0:
        (N_NEG_ZERO if bits else N_POS_ZERO)[0] += 1
    OUT.append((key, bits))


def d(key, n):
    OUT.append((key, int(n) & 0xFFFFFFFFFFFFFFFF))


def b(key, x):
    OUT.append((key, 1 if x else 0))


def s(key, text):
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


def optf(key, x):
    b(key + "?", x is not None)
    if x is None:
        N_NONE[0] += 1
    else:
        f(key, x)


def optd(key, x):
    b(key + "?", x is not None)
    if x is None:
        N_NONE[0] += 1
    else:
        d(key, x)


def optb(key, x):
    b(key + "?", x is not None)
    if x is None:
        N_NONE[0] += 1
    else:
        b(key, x)


def taus4(key, t):
    for i, x in enumerate(t):
        f("%s/%d" % (key, i), x)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def rig(phi_lim, inc=False):
    """`tests/test_rung74.py`'s `_demand(design, sm=_sm(phi_lim), inc=inc)` — the RECEIVER.

    The readers build their own rig from `sm`; this is the machine they are called ON, and
    `forcing_openloop` reads its caps off `self` rather than off the marched sibling, so the
    receiver's arming is load-bearing for exactly one of the six.
    """
    sm = phi_lim / FLOOR - 1.0
    m = DemandCoordinateTransient(
        DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
        stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                    else None),
        stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law = "demand", "sched"
    return m


def emit_demand_law(tag, A):
    taus4(tag + "/taus", A["taus"])
    f(tag + "/ds", A["ds"])
    d(tag + "/floors/n", len(A["floors"]))
    for i, x in enumerate(A["floors"]):
        f("%s/floors/%d" % (tag, i), x)
    f(tag + "/Tt4_max", A["Tt4_max"])
    d(tag + "/n_arms", len(A["arms"]))
    for nm in ("redline_flips", "arrested"):
        d("%s/%s/n" % (tag, nm), len(A[nm]))
        for i, (inc, phi) in enumerate(A[nm]):
            b("%s/%s/%d/inc" % (tag, nm, i), inc)
            f("%s/%s/%d/phi" % (tag, nm, i), phi)
    for k, arm in enumerate(A["arms"]):
        p = "%s/arm%d" % (tag, k)
        b(p + "/inc", arm["inc"])
        f(p + "/phi_lim", arm["phi_lim"])
        f(p + "/sm", arm["sm"])
        for c in COORDS3:
            cp = "%s/%s" % (p, c)
            v = arm["coords"][c]
            b(cp + "/failed?", "failed" in v)
            if "failed" in v:
                s(cp + "/failed", v["failed"])
                d(cp + "/failed/len", len(v["failed"]))
                continue
            d(cp + "/n", v["n"])
            f(cp + "/max_Tt4", v["max_Tt4"])
            f(cp + "/min_phi", v["min_phi"])
            f(cp + "/overshoot", v["overshoot"])
            f(cp + "/breach", v["breach"])
            d(cp + "/handovers/n", len(v["handovers"]))
            for i, x in enumerate(v["handovers"]):
                f("%s/handovers/%d" % (cp, i), x)
            optf(cp + "/first_gov", v["first_gov"])
            b(cp + "/arrested", v["arrested"])
            f(cp + "/max_clip", v["max_clip"])
            d(cp + "/ic_iters", v["ic_iters"])
        optf(p + "/dTt4_coord", arm.get("dTt4_coord"))
        optf(p + "/dphi_coord", arm.get("dphi_coord"))
        optb(p + "/holds_redline", arm.get("holds_redline"))
        optf(p + "/dTt4_floor", arm.get("dTt4_floor"))


# =====================================================================================
# A -- demand_law.  `main.py` drives it per (inc, phi) with `floors=(phi,)`; the reader
# sweeps `inc` itself, so ONE call with all three floors covers the same six arms.
# =====================================================================================
emit_demand_law("A", rig(PHI_ARREST).demand_law(FLIGHT, LO, HI, TT4_MAX, 0.0))

# =====================================================================================
# B -- demand_gains, the suite's own arm (phi = 0.80, ds = 0.002, every = 4).
# =====================================================================================
Bg = rig(PHI_ARREST).demand_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_ARREST)
b("B/inc", Bg["inc"])
f("B/phi_lim", Bg["phi_lim"])
taus4("B/taus", Bg["taus"])
f("B/ds", Bg["ds"])
d("B/n", Bg["n"])
d("B/skipped/regime", Bg["skipped"]["regime"])
d("B/skipped/switch", Bg["skipped"]["switch"])
for nm in ("worst_poly_gap", "worst_poly_rel", "worst_flip", "worst_keep",
           "worst_pairs_gap", "worst_mask_leak", "biggest_moved"):
    optf("B/" + nm, Bg[nm])
optd("B/min_sign_changed", Bg["min_sign_changed"])
d("B/rows/n", len(Bg["rows"]))
for i, r in enumerate(Bg["rows"]):
    p = "B/row%d" % i
    f(p + "/s", r["s"])
    s(p + "/authority", r["authority"])
    f(p + "/poly_gap", r["poly_gap"])
    f(p + "/poly_scale", r["poly_scale"])
    f(p + "/worst_flip", r["worst_flip"])
    f(p + "/worst_keep", r["worst_keep"])
    d(p + "/n_sign_changed", r["n_sign_changed"])
    f(p + "/biggest_moved", r["biggest_moved"])
    optf(p + "/mask_leak_w", r["mask_leak_w"])
    optf(p + "/mask_leak_g", r["mask_leak_g"])
    f(p + "/pairs_gap", r["pairs_gap"])

# =====================================================================================
# C -- latch_discriminator.  NO caller in the shipped tree, so: its own defaults.  This is
# the reader `docs/rung74-spec.md` s 3's `65.2 K` / 332-of-341 is published from.
# =====================================================================================
C = rig(PHI_BOTH).latch_discriminator(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH)
b("C/inc", C["inc"])
f("C/phi_lim", C["phi_lim"])
taus4("C/taus", C["taus"])
f("C/ds", C["ds"])
d("C/n", C["n"])
f("C/slope", C["slope"])
f("C/forcing", C["forcing"])
f("C/coord_dTt4", C["coord_dTt4"])
optf("C/coord_dg_ramp", C["coord_dg_ramp"])
optf("C/coord_dg_post", C["coord_dg_post"])
optf("C/coord_dg_at_mid", C["coord_dg_at_mid"])
optf("C/forcing_ratio", C["forcing_ratio"])
f("C/floor_dTt4", C["floor_dTt4"])
optf("C/floor_dg_riding", C["floor_dg_riding"])
d("C/n_both_riding", C["n_both_riding"])
for c in COORDS3:
    f("C/max_Tt4/%s" % c, C["max_Tt4"][c])
    f("C/min_phi/%s" % c, C["min_phi"][c])

# =====================================================================================
# D -- windup_law, the suite's arm.  ONE of the four cells RAISES, and its message is the
# FIRST shipped refusal text this port compares as a value (`D/cell1/why`).
# =====================================================================================
D = rig(PHI_BOTH).windup_law(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH)
b("D/inc", D["inc"])
f("D/phi_lim", D["phi_lim"])
taus4("D/taus", D["taus"])
f("D/ds", D["ds"])
b("D/no_equilibrium_without_a_stop", D["no_equilibrium_without_a_stop"])
b("D/both_sched_exist", D["both_sched_exist"])
for i, k in enumerate(("demand|sched", "demand|applied",
                       "demand-latched|sched", "demand-latched|applied")):
    p = "D/cell%d" % i
    v = D["cells"][k]
    s(p + "/name", k)
    b(p + "/exists", v["exists"])
    if not v["exists"]:
        s(p + "/why", v["why"])
        d(p + "/why/len", len(v["why"]))
        continue
    d(p + "/n", v["n"])
    d(p + "/ic_iters", v["ic_iters"])
    f(p + "/ic_res", v["ic_res"])
    optf(p + "/max_masked_w", v["max_masked_w"])
    optf(p + "/max_masked_over_sched", v["max_masked_over_sched"])
    f(p + "/max_Tt4", v["max_Tt4"])

# =====================================================================================
# E -- flat_schedule_identity, the suite's arm.  THE REDUCE THAT RUNS.
# =====================================================================================
E = rig(PHI_BOTH).flat_schedule_identity(FLIGHT, 1150.0, phi_lim=PHI_BOTH)
b("E/inc", E["inc"])
f("E/phi_lim", E["phi_lim"])
d("E/n", E["n"])
f("E/nu0/0", E["nu0"][0])
f("E/nu0/1", E["nu0"][1])
for i, k in enumerate(("nu_lp", "nu_hp", "Tt4", "phi_lp", "mf", "b", "v", "g_fuel", "g_gov")):
    f("E/worst/%d" % i, E["worst"][k])
    s("E/worst/%d/name" % i, k)
f("E/worst_any", E["worst_any"])
b("E/bit_identical", E["bit_identical"])
d("E/riding", E["riding"])
b("E/non_vacuous", E["non_vacuous"])
f("E/span_Tt4/0", E["span_Tt4"][0])
f("E/span_Tt4/1", E["span_Tt4"][1])

# =====================================================================================
# F -- forcing_openloop, the suite's arm.  OWNS TWO OF RUNG 74's FOUR `sum()` CALLS, which
# is why P2 names it in advance rather than reading the exemption off the diff.
# =====================================================================================
F = rig(PHI_BOTH).forcing_openloop(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH)
b("F/inc", F["inc"])
f("F/phi_lim", F["phi_lim"])
taus4("F/taus", F["taus"])
f("F/ds", F["ds"])
d("F/n", F["n"])
f("F/slope", F["slope"])
f("F/predicted", F["predicted"])
d("F/n_on_ramp", F["n_on_ramp"])
d("F/n_post", F["n_post"])
optf("F/mean_delta_late", F["mean_delta_late"])
optf("F/ratio_late", F["ratio_late"])
optf("F/worst_rel_late", F["worst_rel_late"])
optf("F/delta_post_first", F["delta_post_first"])
optf("F/delta_post_last", F["delta_post_last"])
optb("F/decayed", F["decayed"])
d("F/rows/n", len(F["rows"]))
for i, r in enumerate(F["rows"]):
    p = "F/row%d" % i
    f(p + "/s", r["s"])
    b(p + "/on_ramp", r["on_ramp"])
    f(p + "/cap", r["cap"])
    f(p + "/req", r["req"])
    f(p + "/g_clip", r["g_clip"])
    f(p + "/g_dem", r["g_dem"])
    f(p + "/delta", r["delta"])
    b(p + "/riding", r["riding"])

# =====================================================================================
# G -- THE PLANT ITSELF.  `_coord_march` at THREE coordinates x TWO floors: every fifth
# point WHOLE, plus min/max/last of every float column over ALL points.
#
# A-F are folds; a difference at one point of several hundred survives every one of them.
# This is the section AD step 5 found its six drifting keys in.
# =====================================================================================
#: The float columns emitted per point. `mf_sched` and the two projections are here BECAUSE
#: they go negative on the `demand` tag (21 of 341) — a fact no aggregate above records.
G_FLOATS = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
            "mdot_air", "sp_thrust", "mf", "mf_sched", "g", "required", "b", "b_cmd",
            "v", "v_cmd", "ic_res", "g_fuel", "g_gov", "required_fuel", "required_gov")
#: Present only on a rung-74 trajectory — the five keys the coordinate ADDS.
G_FLOATS_74 = ("w_fuel", "w_gov", "cap_fuel", "cap_gov")
G_INTS = ("ic_iters",)
G_STRS = ("branch", "authority", "share_law", "ic_order", "lag_coord")

for gi, phi in enumerate((PHI_ARREST, PHI_BOTH)):
    sm = phi / FLOOR - 1.0
    for coord in COORDS3:
        tag = "G/%d/%s" % (gi, coord)
        f(tag + "/phi_lim", phi)
        s(tag + "/coord", coord)
        try:
            traj = rig(phi)._coord_march(FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE,
                                         0.005, V_MAX, False, coord)[3]
        except AssertionError as exc:
            b(tag + "/marched", False)
            s(tag + "/why", str(exc)[:240])
            d(tag + "/why/len", len(str(exc)[:240]))
            continue
        b(tag + "/marched", True)
        d(tag + "/n", len(traj))
        # A MARCH THAT RETURNED AN EMPTY LIST WOULD AGREE WITH ITSELF ON EVERY KEY BELOW, which
        # is § 5.27 (ii)'s emptied-sample defect. Refused here rather than folded over.
        assert traj, "an empty march is not a measurement: %s" % tag
        d(tag + "/keys", len(traj[0]))
        # every fifth point, WHOLE
        for i in range(0, len(traj), G_STRIDE):
            p, q = traj[i], "%s/p%d" % (tag, i)
            for k in G_FLOATS:
                if k in p:
                    f("%s/%s" % (q, k), p[k])
            for k in G_FLOATS_74:
                b("%s/%s?" % (q, k), k in p)
                if k in p:
                    f("%s/%s" % (q, k), p[k])
            for k in G_INTS:
                if k in p:
                    d("%s/%s" % (q, k), p[k])
            for k in G_STRS:
                if k in p:
                    s("%s/%s" % (q, k), str(p[k]))
            b("%s/v_regime?" % q, p.get("v_regime") is not None)
            if p.get("v_regime") is not None:
                s("%s/v_regime" % q, str(p["v_regime"]))
        # and the three extremes over ALL points, which the stride cannot see
        for k in G_FLOATS + G_FLOATS_74:
            col = [p[k] for p in traj if k in p]
            d("%s/col/%s/n" % (tag, k), len(col))
            if not col:
                continue
            f("%s/col/%s/min" % (tag, k), min(col))
            f("%s/col/%s/max" % (tag, k), max(col))
            f("%s/col/%s/last" % (tag, k), col[-1])
        # the SIGN census this rung exists to make visible
        for k in ("g_fuel", "g_gov", "required_fuel", "required_gov"):
            col = [p[k] for p in traj if k in p]
            d("%s/neg/%s" % (tag, k), sum(1 for x in col if x < 0.0))

# =====================================================================================
# H -- A DECLARED EXTRA GRID, not a shipped caller's: `demand_law` swept ONE floor at a
# time, which is the shape `main.py:5746` calls it in (`floors=(phi,)`).  A's single
# three-floor call folds `arrested`/`redline_flips` across the arms; this splits them.
# =====================================================================================
for hi_, phi in enumerate((PHI_ARREST, PHI_BOTH, PHI_GOV)):
    emit_demand_law("H/%d" % hi_,
                    rig(phi).demand_law(FLIGHT, LO, HI, TT4_MAX, 0.0, floors=(phi,)))

# =====================================================================================
# Z -- THE CENSUSES.  An `Option` that is `None` on both sides agrees perfectly and
# measures nothing, and a `+0.0` that should be `-0.0` is invisible to every relative bar.
# Both are counted rather than hoped about.
# =====================================================================================
d("Z/n_none", N_NONE[0])
d("Z/n_neg_zero", N_NEG_ZERO[0])
d("Z/n_pos_zero", N_POS_ZERO[0])

seen = {}
for k, v in OUT:
    assert k not in seen, "DUPLICATE KEY %s" % k
    seen[k] = v
d_total = len(seen)
path = sys.argv[1] if len(sys.argv) > 1 else None
assert path, "usage: dump_slice_af.py <out.tsv>"
with open(path, "w", encoding="utf-8", newline="\n") as fh:
    for k in sorted(seen):
        fh.write("%s\t%d\n" % (k, seen[k]))
sys.stderr.write("keys %d -> %s\n" % (d_total, path))
sys.stderr.write("  none %d   neg_zero %d   pos_zero %d\n"
                 % (N_NONE[0], N_NEG_ZERO[0], N_POS_ZERO[0]))
