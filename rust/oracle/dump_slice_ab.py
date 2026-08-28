"""SLICE AB step 4 -- THE ORACLE for rung 69 (`StatorIncidenceLimiter`, `ReferenceSplitTransient`).

**THE GRID IS THE SUITE'S OWN AND NOTHING IS COARSENED**, and the header states it rather than
implying it -- slice S step 4's lesson, *a probe's HEADER claimed the suites' grids and its code
ran another*. Every argument in sections A-I below is copied from the calling gate in
`tests/test_rung69.py`, never chosen:

    ds          0.005 everywhere EXCEPT `reference_modes` and `ring_visibility`, whose own
                defaults are 0.002 -- and those are the defaults the suite's gates take
    LO/HI       1000 -> 1400 . r 0.5 . s_settle 1.2 . FLOOR 0.55 -- the suite's throughout
    phi/b/v     PHI 0.80, B 0.10, V_MAX 0.20, SM = PHI/FLOOR - 1 -- the suite's
    clocks      TAU 0.05 (valve), TAU_S 0.05 (stator), TAU_ATT 0.05 / TAU_REL 0.15 (fuel)
    clock grid  ((.05,.05,.05), (.05,.005,.05), (.05,.5,.05), (.02,.05,.10)) -- `reference_modes`'
                own default, which the suite takes wholesale
    damp grid   the six of `damping_floor`'s own default
    disp        0.05 -- `ring_visibility`'s, the suite's
    maps        `shaped` ONLY (LP a=.20 b=.05 l=.7 / HP a=.08 b=.15 l=1.0) -- the suite builds no
                second shape, so a second one here would be a grid it does not have

**SECTIONS J AND K ARE A DECLARED EXTRA GRID AND SAY SO IN THEIR OWN HEADERS.** J calls the root
finder directly on triples the plant never visits (knife edges, the degenerate start, and the
EXHAUSTED shape); K runs two grid points the suite does not have, in order to reach two of the four
degenerate branches step 2 s (h) disclosed. Mixing those into A-I would be exactly the defect the
first paragraph guards against, so they are numbered apart.

# SECTION I IS THE STEP's REASON FOR EXISTING

Step 3 measured that **no value gate in this slice can see `_cubic_roots_c`'s Newton budget**:
cutting 80 to 20 leaves every slice-AB binary green while moving 56 of 243 root components. The
instrument is this dump. Two independent constructions carry it:

  * sections D/E/F emit **every root the readers compute, as re/im bit patterns** -- the plant's
    own call stream; and
  * section I **INTERCEPTS** every `_cubic_roots_c` call those sections make (never reconstructs
    them -- slice Z's leading finding) and emits the coefficient triple beside its three roots, so
    the Rust replays the SHIPPED solver on the SHIPPED inputs with the plant taken out of the loop.

`I/ncalls` is checkable on the Rust side rather than decorative: `reference_modes`, `damping_floor`
and `rk4_margin` are the only three callers, so the count must equal the number of root-carrying
rows sections D, E and F emit between them.

**AND THE COUNT IS 94, NOT s 5.26 (iii)'s 256.** That is not a coverage gap and it is not the same
question: 256 is what the whole `pytest` session makes, where several gates call the same reader
again; this dump calls each reader ONCE. Saying "256" here would be the defect slice S step 4
recorded -- a header claiming a grid its code does not run. What matters is that the EXHAUSTED arm
is covered, and it is: **24 of the 94 intercepted triples spend all 80 Newton steps without
converging** (measured, printed to stderr by the coverage block at the foot of this file).

# THE CROSS-INTERPRETER EXEMPTION IS A SET OF **NAMES**, MEASURED FROM THIS DUMP

s 5.26 (i) measured a **three-element** `sum()` diverging between interpreters -- which slice AA's
own explanation says cannot happen -- and refuted the obvious replacement (cancellation) with the
same probe. `_invariants`' `c1` differs on 23 of 256 instances under CPython 3.14 while `c2`, built
the same way at the same site, agrees on all 256. So the exempt set is whatever this dump emits
downstream of `c1`, and it is READ OFF THE DIFF rather than predicted: [[rust-port-slice-z-step4]]
is a pre-registered exemption of TWO keys that measured EIGHT, because it counted quantities where
a dump emits names. **The port is held to PyPy**, where nothing is exempt.

Every float is emitted as its IEEE-754 bit pattern. Regenerate BOTH arms:

    .venv/Scripts/python.exe rust/oracle/dump_slice_ab.py > rust/oracle/slice_ab_pypy.tsv
    C:/Python314/python.exe  rust/oracle/dump_slice_ab.py > rust/oracle/slice_ab_cpython.tsv

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
    ReferenceSplitTransient, ThreeLoopCascadeTransient,
    BleedLimiter, BleedSchedule, StatorLimiter, StatorIncidenceLimiter,
    SurgeLimiter, AsymmetricLag,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


def opt(key, x):
    """A key Python may return as `None` -- emitted as a PRESENCE flag beside the value, because a
    sentinel float would conflate a missing value with a real one."""
    b(key + "?", x is not None)
    if x is not None:
        f(key, x)


def s(key, text):
    """A STRING key, as an FNV-1a 64-bit hash -- `v_regime`, `ic_order` and the off-regime arm
    names are the non-floats a rung-69 reading carries."""
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


def c(key, z):
    """A COMPLEX root -- re, im and `abs`. **`abs` is a KEY and not a convenience**: it is
    `hypot`, this slice's one remaining platform-library exposure (s 5.26.2 (e)), and
    `sorted(..., key=abs)`, `zeta`, `n_zero` and `worst_zero` all read it. Every root with
    `im == 0` reduces to `|re|` on any conforming `hypot`, so only the genuinely complex pairs put
    the library at risk -- which is why `I/n_complex` below counts them."""
    f(key + "/re", z.real)
    f(key + "/im", z.imag)
    f(key + "/abs", abs(z))


def off(key, names):
    d(key + "/n_off", len(names))
    for i, n in enumerate(names):
        s("%s/off/%d" % (key, i), n)


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
DISP = 0.05

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
T_C = LP.tan_beta1_crit()
M_LIM = T_C - 1.0 / PHI                      # THE SAME WALL, read at the design setting


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """The suite's `_cpg`, character for character. `R_c` is DERIVED from `(gamma - 1)/gamma`;
    re-spelling it `0.4/1.4` builds a gas ONE ULP away, which presents exactly as a port defect
    (slice Y's own false alarm)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def split(**kw):
    return ReferenceSplitTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def three(**kw):
    return ThreeLoopCascadeTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


def valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


def inc(tau=TAU_S, v_max=V_MAX):
    return StatorIncidenceLimiter.from_margin(LP, v_max, SM, tau=tau)


def phi_stator(tau=TAU_S, v_max=V_MAX):
    return StatorLimiter.from_margin(LP, v_max, SM, tau=tau)


def fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


def march(m, ds=DS, **kw):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, ds, **kw)[0]


# ------------------------------------------------------- THE INTERCEPT (section I's instrument)
#
# Installed as a `staticmethod`, because `_cubic_roots_c` IS one and is called as
# `self._cubic_roots_c(...)`: a plain function assigned onto the class would bind through the
# instance and hand the body a fourth argument. Pure pass-through -- it records, it does not
# compute, so no value below moves because the recorder is on.
CUBIC_CALLS = []
_ORIG_CUBIC = ReferenceSplitTransient.__dict__["_cubic_roots_c"].__func__


def _recording_cubic(c2, c1, c0):
    out = _ORIG_CUBIC(c2, c1, c0)
    CUBIC_CALLS.append((c2, c1, c0, out))
    return out


ReferenceSplitTransient._cubic_roots_c = staticmethod(_recording_cubic)


def put_point(p, pt):
    """Every key the point carries, emitted by ITERATING the dict rather than from a typed list,
    so a key the port forgets shows up as a COUNT mismatch and not as a silently-unread field."""
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


def put_gains(p, gg):
    """A gains dict, WHOSE KEY SET DEPENDS ON `interior` -- and the two branches are not nested.

    The non-interior early return carries `interior`, `off_regime`, `s` and `v_base`; the interior
    one carries `interior`, `off_regime`, `v_base`, the six gains and the four products -- and
    **NO `s`**. The Rust struct carries `s` on BOTH branches and fills the gains with NaN on the
    off one, so an unconditional emit here would invent a golden key on one branch and demand a
    NaN on the other. Only what Python actually has is emitted."""
    b(p + "/interior", gg["interior"])
    off(p, gg["off_regime"])
    f(p + "/v_base", gg["v_base"])
    if gg["interior"]:
        for k in ("R_q", "R_v", "C_g", "C_v", "V_g", "V_q",
                  "cyclic", "pair_RC", "pair_RV", "pair_CV"):
            f("%s/%s" % (p, k), gg[k])
    else:
        f(p + "/s", gg["s"])


def put_cell(p, cell):
    for k in ("I", "I_inc", "min_phi", "end_s", "v_min", "v_max_used", "b_max_used",
              "credit", "credit_inc"):
        f("%s/%s" % (p, k), cell[k])
    d(p + "/npts", cell["npts"])
    b(p + "/v_saturated", cell["v_saturated"])


def put_bill(p, bill):
    f(p + "/phi_lim", bill["phi_lim"])
    f(p + "/m_lim", bill["m_lim"])
    f(p + "/sum_singles", bill["sum_singles"])
    f(p + "/delivered", bill["delivered"])
    for name in ("bare", "F", "V", "S", "FV", "FS", "VS", "FVS"):
        put_cell("%s/%s" % (p, name), bill["cells"][name])
    for k in ("fuel", "valve", "stator"):
        f("%s/marginal/%s" % (p, k), bill["marginal"][k])
        f("%s/marginal_inc/%s" % (p, k), bill["marginal_incidence"][k])
        f("%s/singles/%s" % (p, k), bill["singles"][k])
        f("%s/erosion/%s" % (p, k), bill["erosion"][k])


# ============================================== A -- THE REDUCE, and BOTH SIDES' OWN ABSOLUTES
# The suite's two reduce gates compare two machines in ONE run, which is blind to a defect that
# moves both. Here each side's values are pinned against Python as well, so a port that broke the
# rung-68 body and the rung-69 dispatch to it identically still goes red.
#
# Arm 0 is `stator_lim` armed (rung 68's phi stator on a rung-69 object); arms 1-5 are the
# inherited ones, rung 66 / 65 / 52 / 64 / 62.
for i, (kw, mkw) in enumerate((
        (dict(bleed_lim=valve(TAU), stator_lim=phi_stator()),
         dict(surge=fuel(), lag=lag())),                                    # rung 68
        (dict(bleed_lim=valve(TAU)), dict(surge=fuel(), lag=lag())),        # rung 66
        (dict(bleed_lim=valve(TAU)), dict(surge=fuel())),                   # rung 65
        (dict(), dict(surge=fuel(), lag=lag())),                            # rung 52
        (dict(bleed_lim=valve()), dict()),                                  # rung 64
        (dict(bleed_sched=BleedSchedule(B, 0.65)), dict()))):               # rung 62
    ta = march(split(**kw), **mkw)
    tb = march(three(**kw), **mkw)
    put_traj("A/%d/split" % i, ta, stride=37)
    put_traj("A/%d/three" % i, tb, stride=37)
    b("A/%d/identical" % i, [tuple(sorted(x.items())) for x in ta]
                            == [tuple(sorted(y.items())) for y in tb])
    b("A/%d/carries_v" % i, "v" in ta[0])

# ============================================== B -- THE ARMED FIVE-STATE MARCH, and THE BAND
# The rung's own machine, EVERY point -- the only section that can catch a defect in the RK4 loop
# itself, and the one that pins the band flip pointwise rather than through a `min`.
M = split(bleed_lim=valve(TAU), stator_inc=inc())
TRAJ = march(M, surge=fuel(), lag=lag())
put_traj("B/traj", TRAJ)
f("B/violation", M._violation(TRAJ, PHI, R))
f("B/violation_inc", M._violation_inc(TRAJ, M_LIM, T_C, R))
d("B/n_riding", len(M._riding(TRAJ, B)))
f("B/v_min", min(p["v"] for p in TRAJ))
f("B/v_max_seen", max(p["v"] for p in TRAJ))
for name in ("dormant", "riding", "saturated"):
    d("B/regime/%s" % name, sum(1 for p in TRAJ if p["v_regime"] == name))

# The DISPLACED start the band gate admits (`v0 = +0.05`); its mirror `-0.05` is refused, which is
# a raise and therefore not a value key.
put_traj("B/v0", march(M, surge=fuel(), lag=lag(), v0=DISP), stride=37)

# The SATURATING machine -- `v_max = 0.02`, no valve, no fuel leg: the only one reaching the third
# regime, and the suite's own `test_a_float_comparison_against_the_stop_is_not_the_regime`.
MS = split(bleed_lim=None, stator_inc=inc(v_max=0.02))
TS = march(MS)
put_traj("B/sat", TS, stride=17)
for name in ("dormant", "riding", "saturated"):
    d("B/sat/regime/%s" % name, sum(1 for p in TS if p["v_regime"] == name))

# `at_lever` -- the SEVENTH instance of the trap, and the second where the signature GROWS. A
# sibling that silently swapped the reference would march the OTHER band.
SIB = M.at_lever(bleed_lim=valve(TAU), stator_inc=inc())
put_traj("B/at_lever", march(SIB, surge=fuel(), lag=lag()), stride=37)

# The limiter's OWN numbers -- `m_lim` is a physical wall and `phi_lim_at` its inverse.
f("B/lim/m_lim", inc().m_lim)
f("B/lim/v_max", inc().v_max)
f("B/lim/tau", inc().tau)
f("B/lim/phi_lim_at", inc().phi_lim_at(LP))
f("B/lim/from_phi_m_lim", StatorIncidenceLimiter.from_phi(LP, V_MAX, PHI).m_lim)
for j, (phi, v) in enumerate(((0.80, 0.0), (0.80, 0.20), (0.60, 0.05), (1.20, -0.05))):
    f("B/lim/margin/%d" % j, StatorIncidenceLimiter.margin(T_C, phi, v))

# ============================================== C -- s 1, THE PAIRWISE SPLIT
G = M.reference_gains(FLIGHT, LO, HI, sm=SM)
d("C/n_riding", G["n_riding"])
d("C/n_sampled", G["n_sampled"])
d("C/n_rows", len(G["rows"]))
d("C/n_skipped", len(G["skipped"]))
opt("C/worst_RC_inc", G["worst_RC_inc"])
opt("C/worst_RC_phi", G["worst_RC_phi"])
opt("C/worst_pair_gap", G["worst_pair_gap"])
opt("C/worst_RC_own", G["worst_RC_own"])
opt("C/k_lo", G["k_range"][0])
opt("C/k_hi", G["k_range"][1])
if G["s_window"] is not None:
    f("C/s_lo", G["s_window"][0])
    f("C/s_hi", G["s_window"][1])
for i, row in enumerate(G["rows"]):
    f("C/%d/s" % i, row["s"])
    f("C/%d/k" % i, row["k"])
    f("C/%d/pair_gap" % i, row["pair_gap"])
    f("C/%d/v_base" % i, row["v_base"])
    for side in ("inc", "phi", "own"):
        put_gains("C/%d/%s" % (i, side), row[side])
for i, sk in enumerate(G["skipped"]):
    f("C/skip/%d/s" % i, sk["s"])
    off("C/skip/%d/inc" % i, sk["inc"])
    off("C/skip/%d/phi" % i, sk["phi"])

# ============================================ D -- s 1/3, THE SPECTRUM (ds = 0.002, the reader's)
# **EVERY ROOT, as re/im/abs.** Step 3 measured that the Newton budget moves 56 of 243 root
# components and NOTHING a gate reads, so this section is where the budget becomes observable.
MODES = M.reference_modes(FLIGHT, LO, HI, sm=SM)
d("D/n_arms", len(MODES["arms"]))
f("D/ds", MODES["ds"])
for i, arm in enumerate(MODES["arms"]):
    for j, t in enumerate(arm["taus"]):
        f("D/%d/tau/%d" % (i, j), t)
    for ref in ("inc", "phi"):
        x = arm["refs"][ref]
        p = "D/%d/%s" % (i, ref)
        f(p + "/rate_sum", x["rate_sum"])
        d(p + "/n", x["n"])
        d(p + "/n_sampled", x["n_sampled"])
        d(p + "/skipped", x["skipped"])
        d(p + "/n_rows", len(x["rows"]))
        d(p + "/n_zeros", len(x["zeros"]))
        for j, z in enumerate(x["zeros"]):
            d("%s/zeros/%d" % (p, j), z)
        opt(p + "/max_c0_rel", x["max_c0_rel"])
        opt(p + "/min_c1_rel", x["min_c1_rel"])
        b(p + "/all_complex?", x["all_complex"] is not None)
        if x["all_complex"] is not None:
            b(p + "/all_complex", x["all_complex"])
        opt(p + "/zeta_lo", x["zeta_range"][0])
        opt(p + "/zeta_hi", x["zeta_range"][1])
        for j, row in enumerate(x["rows"]):
            q = "%s/%d" % (p, j)
            for k in ("s", "c1", "c0", "c2", "k", "pair_RC", "cyclic",
                      "worst_zero", "c1_rel", "c0_rel"):
                f("%s/%s" % (q, k), row[k])
            d(q + "/n_zero", row["n_zero"])
            b(q + "/complex_pair", row["complex_pair"])
            opt(q + "/zeta", row["zeta"])
            for kk, rt in enumerate(row["roots"]):
                c("%s/root/%d" % (q, kk), rt)

# ============================================== E -- s 3, THE DAMPING FLOOR
DF = M.damping_floor(FLIGHT, LO, HI, sm=SM)
d("E/n_rows", len(DF["rows"]))
b("E/holds", DF["holds"])
opt("E/worst_pred_err", DF["worst_pred_err"])
b("E/tightest?", DF["tightest"] is not None)
if DF["tightest"] is not None:
    f("E/tightest/s", DF["tightest"]["s"])
    f("E/tightest/zeta", DF["tightest"]["zeta"])
    f("E/tightest/floor", DF["tightest"]["floor"])
for i, row in enumerate(DF["rows"]):
    p = "E/%d" % i
    for j, t in enumerate(row["taus"]):
        f("%s/tau/%d" % (p, j), t)
    d(p + "/n", row["n"])
    b(p + "/live", "zeta" in row)
    off(p, row.get("off_regime", []))
    if "zeta" in row:
        for k in ("s", "k", "A", "z", "A_over_z", "det2", "zeta_pred", "zeta", "floor",
                  "mod", "mod_pred", "rate_sum"):
            f("%s/%s" % (p, k), row[k])
        b(p + "/complex_pair", row["complex_pair"])

# ============================================== F -- THE RK4 GUARD, MEASURED
RK = M.rk4_margin(FLIGHT, LO, HI, sm=SM)
f("F/rate_sum", RK["rate_sum"])
d("F/n", RK["n"])
d("F/n_rows", len(RK["rows"]))
opt("F/max_mod", RK["max_mod"])
opt("F/max_ratio", RK["max_ratio"])
opt("F/max_bound", RK["max_bound"])
f("F/ds_lambda", RK["ds_lambda"])
for i, row in enumerate(RK["rows"]):
    for k in ("s", "mod", "k", "ratio", "bound"):
        f("F/%d/%s" % (i, k), row[k])

# ============================================== G -- s 4, THE LEDGER UNDER BOTH REFERENCES
BILL = M.reference_bill(FLIGHT, LO, HI, sm=SM)
for ref in ("inc", "phi"):
    put_bill("G/%s" % ref, BILL[ref])
    cr = BILL["stator_credit"][ref]
    for k in ("alone", "alone_inc", "marginal", "marginal_inc"):
        f("G/credit/%s/%s" % (ref, k), cr[k])
    f("G/delivered/%s" % ref, BILL["delivered"][ref])
    f("G/delivered_inc/%s" % ref, BILL["delivered_inc"][ref])
f("G/common_max_rel", BILL["common_max_rel"])
for name in ("bare", "F", "V", "FV"):
    f("G/common/%s/inc" % name, BILL["common"][name][0])
    f("G/common/%s/phi" % name, BILL["common"][name][1])

# ============================================== H -- IS THE MODE OBSERVABLE?
RV = M.ring_visibility(FLIGHT, LO, HI, sm=SM, disp=DISP)
for ref in ("inc", "phi"):
    for name in ("base", "displaced"):
        a = RV[ref][name]
        p = "H/%s/%s" % (ref, name)
        d(p + "/n", a["n"])
        d(p + "/n_riding", a["n_riding"])
        d(p + "/crossings", a["crossings"])
        f(p + "/e0", a["e0"])
        opt(p + "/survives", a["survives"])
        opt(p + "/counter", a["counter"])
        f(p + "/v_lo", a["v_range"][0])
        f(p + "/v_hi", a["v_range"][1])

# ============================================== I -- THE ROOT FINDER, INTERCEPTED
# Everything sections D, E and F asked of `_cubic_roots_c`, captured at the call and emitted with
# its answer. The Rust replays the SHIPPED solver on these exact coefficient bits, which is the
# only construction in the slice where a wrong Newton budget is breakable with the plant taken out
# of the loop. `ncalls` is checkable: those three readers are the ONLY callers.
N_SUITE = len(CUBIC_CALLS)
d("I/ncalls", N_SUITE)
d("I/n_complex", sum(1 for _, _, _, rs in CUBIC_CALLS if any(r.imag != 0.0 for r in rs)))
for i, (c2, c1, c0, roots) in enumerate(CUBIC_CALLS):
    f("I/%d/c2" % i, c2)
    f("I/%d/c1" % i, c1)
    f("I/%d/c0" % i, c0)
    for j, rt in enumerate(roots):
        c("I/%d/root/%d" % (i, j), rt)

# ============================================== J -- THE ROOT FINDER, ON A DECLARED EXTRA TABLE
# **NOT THE SUITE'S GRID, AND SAID SO.** Section I covers the triples the plant visits; these are
# the ones it does not -- the two branch knife edges, the degenerate `c2 = 0` start, and the
# EXHAUSTED shape s 5.26 (iii) measured (`c2 = -60`, `c1 ~ 1e-8`, `c0 ~ -1e-12`), where Newton
# spends all 80 steps wandering because the near-zero pair is COMPLEX and there is no real root to
# find. That last row is the one a cut budget moves.
CUBICS = ((-60.0, 9.7e-08, -1.7e-12),     # the EXHAUSTED shape: 80 steps, no convergence
          (-60.0, 1.0e-08, -1.0e-14),     # rung 68's own {0, 0, c2} shape
          (-60.0, 0.0, 0.0),              # exactly rank one
          (0.0, 1.0, -1.0),               # c2 == 0 -> the x = 0 start, and a complex pair
          (-3.0, 3.0, -1.0),              # a triple root at 1
          (-2.0, 5.0, -10.0),             # a complex pair, comfortably off the axis
          (-6.0, 11.0, -6.0),             # roots 1, 2, 3
          (1.0, -1.0, 1.0),
          (-240.0, 1.0e-04, -1.0e-09))    # the (0.05, 0.005, 0.05) clock arm's own scale
for i, (c2, c1, c0) in enumerate(CUBICS):
    f("J/%d/c2" % i, c2)
    f("J/%d/c1" % i, c1)
    f("J/%d/c0" % i, c0)
    for j, rt in enumerate(_ORIG_CUBIC(c2, c1, c0)):
        c("J/%d/root/%d" % (i, j), rt)

# ============================================== K -- THE DEGENERATE BRANCHES, ON AN EXTRA GRID
# **NOT THE SUITE'S GRID EITHER.** Step 2 s (h) disclosed four branches nothing reaches, and step
# 3 left them a standing hole. Two of them are reachable, and it is the RAMP that reaches them and
# not the clocks (probe_ab12/13/14: six clock grids spanning a 1000x range change nothing, because
# a slower loop rides for LONGER):
#
#   K/0  a FLAT ramp (1000 -> 1000): no accel, so no loop ever engages -- `n = 0`, `tightest`
#        None, and `reference_modes`' arm has NO rows, i.e. `all_complex` is None.
#   K/1  a 10 K ramp at the (0.005, 0.05, 0.05) clock: the MID riding point is not interior, so
#        the row carries `off_regime` and no `zeta`.
#
# The fourth branch, `zeta = None`, is **UNREACHABLE BY CONSTRUCTION** and is not chased: the
# roots sum to `c2 = -(1/tau_g + 1/tau_q + 1/tau_s)`, which is non-zero for every finite positive
# clock, so they cannot all be zero and the dominant one has non-zero modulus.
KD0 = M.damping_floor(FLIGHT, LO, LO, sm=SM, grid=((0.05, 0.05, 0.05),))
d("K/0/n_rows", len(KD0["rows"]))
d("K/0/n", KD0["rows"][0]["n"])
b("K/0/live", "zeta" in KD0["rows"][0])
b("K/0/holds", KD0["holds"])
b("K/0/tightest?", KD0["tightest"] is not None)
b("K/0/worst_pred_err?", KD0["worst_pred_err"] is not None)
KM0 = M.reference_modes(FLIGHT, LO, LO, sm=SM, clocks=((0.05, 0.05, 0.05),))
for ref in ("inc", "phi"):
    x = KM0["arms"][0]["refs"][ref]
    d("K/0/%s/n" % ref, x["n"])
    d("K/0/%s/n_rows" % ref, len(x["rows"]))
    b("K/0/%s/all_complex?" % ref, x["all_complex"] is not None)
    b("K/0/%s/zeta_lo?" % ref, x["zeta_range"][0] is not None)
    b("K/0/%s/max_c0_rel?" % ref, x["max_c0_rel"] is not None)
KD1 = M.damping_floor(FLIGHT, LO, 1010.0, sm=SM, grid=((0.005, 0.05, 0.05),))
d("K/1/n", KD1["rows"][0]["n"])
b("K/1/live", "zeta" in KD1["rows"][0])
off("K/1", KD1["rows"][0].get("off_regime", []))
b("K/1/holds", KD1["holds"])
b("K/1/tightest?", KD1["tightest"] is not None)

# ---------------------------------------------------------------------------- emit
print("# slice AB step 4 -- rung 69 ORACLE, the SUITE's grid (A-I), uncoarsened, plus TWO "
      "declared extra grids (J, K). key<TAB>u64 (floats are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys, {N_SUITE} intercepted cubic calls", file=sys.stderr)

# COVERAGE DOCUMENTATION, NOT A KEY -- how many of the intercepted triples exhaust the 80-step
# budget. A counted copy of the loop can only gate the copy, so this is printed to stderr and
# nothing reads it; what it buys is the right to say section I covers the exhausted arm.
_exh = 0
for c2, c1, c0, _ in CUBIC_CALLS:
    x, n = 0.0, 0
    for _ in range(80):
        dd = (3.0 * x - 2.0 * c2) * x + c1
        if dd == 0.0:
            break
        step = (((x - c2) * x + c1) * x - c0) / dd
        x -= step
        n += 1
        if abs(step) <= 1e-15 * max(abs(c2), abs(x), 1.0):
            break
    _exh += n == 80
print(f"# COVERAGE: {_exh} of {N_SUITE} intercepted triples exhaust the 80-step budget",
      file=sys.stderr)
