"""SLICE AE step 4 -- THE ORACLE for rung 73 (`AppliedReferenceTransient`).

`rung73.rs` ports the suite's own 27 gates, and step 3 measured what they can and cannot see.
Two of its readings set this file's job, and neither is the one the pre-flight expected:

  * **P5 was FALSIFIED at 6 of 27.** Folding path 2's float-identity branch away IS caught by the
    ported gates -- and by **the same 6 in Python, name for name**, a bijection between the two
    languages' catch sets. So this file is NOT the only seat for that injection, and it does not
    claim to be. What it adds is the LEDGER: the exact key count, per section, for an injection
    whose gate answer is already known.
  * **Step 2's M22 was measured UNOBSERVABLE and BOOKED HERE.** 101 keys were exactly `-0.0`
    (every one a `*/g/pair_FR`), 925 exactly `+0.0`, and inside the four `sorted({...})` sets the
    defence re-keys, `+0.0` appeared twice and `-0.0` not at all -- so on step 2's grid the two
    zeros never met in one set. This dump is the wider grid, and **section M measures it again
    rather than carrying the old number forward**.

# THE GRID IS THE READERS' OWN, AND IT IS FIVE DIFFERENT GRIDS

Read off `turbojet/engine.py`'s own `def` lines -- **never off this module's constants**, which is
[[rust-port-slice-ac-step6]]'s `every = 40`-vs-`10` defect and which step 3 § (h) already had to
avoid once inside this slice (the suite's `DS` agrees with only two of the five):

    reader              def line   ds       every   clock argument
    handover_law        17064      0.005    --      `clocks`, the THREE-arm tuple
    applied_gains       17133      0.002    2       `taus`, one 4-tuple   + an `inc` arm
    applied_cells       17202      0.002    2       `clocks`, the THREE-arm tuple
    ref_discriminator   17340      0.002    4       `taus`                + an `inc` arm
    applied_bill        17432      0.005    --      `taus`                + an `inc` arm

Every reader below is called with the FIVE positional arguments the suite passes, plus its own
clock argument and, where it has one, both `inc` arms -- and nothing else. **Exactly one call in
`tests/test_rung73.py` overrides a default** (the broken-instrument probe at `:237`, `ds = 0.01,
every = 8`); it is a DELIBERATELY BROKEN reader and is not a grid this dump copies.

The flight condition, the maps, the floor, `LO`/`HI`, `PHI`/`B`/`V_MAX`, `TAU`, `TT4_MAX` and
`CLOCKS` are `tests/test_rung73.py`'s own module constants, copied verbatim.

Sections **A** `handover_law` / **B,C** `applied_gains` / **D** `applied_cells` /
**E,F** `ref_discriminator` / **G,H** `applied_bill` are the five readers. **J, K, L** are
DECLARED EXTRA GRIDS and each says at its own head why the readers cannot reach it; **M** is a
census over this dump's own emitted keys.

# EVERY INTERCEPTED SECTION EMITS ITS OWN CALL COUNT, EVEN WHEN THAT COUNT IS ZERO

AD probe F's bar, which earned its keep twice inside THIS slice's pre-flight (probes E and H both
raised `TypeError` on both arms and were caught only by their own controls) and once more in this
step's pre-measurement, which read the trajectory out of the wrong tuple slot and printed
`march_pts=0` beside a live call counter. **A confident zero from a section that reached nothing
is this slice's most-repeated failure mode**, so no section below is allowed to be silent about
whether it ran, and sections J and K carry an `assert` that fires on zero.

# WHAT IS DELIBERATELY NOT EMITTED, so the Rust's missing-key half stays honest

  * **`at_lever`.** It is a BUILDER, and slice AC step 7 measured it as the LAUNDERER -- every
    reader rebuilds its machine through it and installs the shipped tables, so no value key can
    witness which function pointer sat in the slot. That is step 5's subject, not this one's.
  * **`shared_bill`'s `own_currency`** -- a table of CONSTANT strings that cannot differ between
    two runs. AD dropped it for that reason and so does this file. A DECISION, not an omission.
  * **`_quad_gains_at`'s `s` key.** Python's INTERIOR return does not carry one and its two
    non-interior early returns do; **all 540 calls on this grid return the interior dict**
    (measured), so `s` is absent from every one. The port's `QuadGains` carries `s`
    unconditionally, which is a REPRESENTATION difference and not a value difference -- gating it
    would fail on a distinction Python never makes here. `interior` IS emitted, so the premise of
    this paragraph is itself a key.
  * **`_quad_gains_at`'s POINTER.** § 5.29 (iv) measured its cell observable (32 keys move, 70
    vanish under the parent's body) and P4 assigns that seat to **step 5**, on a DECLARED EXTRA
    GRID no shipped test sits in. Its VALUES are covered here: every gains row in B, C, E and F
    comes out of it. No pointer swap appears in this file.

Every float is emitted as its IEEE-754 bit pattern; every string as an FNV-1a 64-bit hash.
Regenerate BOTH arms:

    .venv/Scripts/python.exe rust/oracle/dump_slice_ae.py > rust/oracle/slice_ae_pypy.tsv
    C:/Python314/python.exe  rust/oracle/dump_slice_ae.py > rust/oracle/slice_ae_cpython.tsv

**Redirect through a POSIX shell, not PowerShell 5.1** -- its `1>` writes UTF-8 WITH A BOM and the
BOM lands in front of the `#` on line 1, so the header parses as data.
[[windows-tooling-file-hazards]].
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                       # noqa: E402
from turbojet.engine import (                                                      # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    AppliedReferenceTransient, SharedActuatorTransient, BleedLimiter,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    """An INTEGER key, as TWO'S COMPLEMENT. **The mask is not decoration**: `dzeros_B` and
    `dzeros_C` are DIFFERENCES of zero counts, and rung 73's own docstring says reading C
    *"REMOVES one wherever the live leg's droop restores full rank"* -- so a `-1` is expected
    there. Printed raw it would emit `-1`, and the Rust golden parser is `bits.parse::<u64>()`,
    which REFUSES it. The port reads these back with `as i64`, and the range assert below is what
    stops a genuinely out-of-range value from being silently folded into the mask."""
    n = int(n)
    assert -(2 ** 63) <= n < 2 ** 64, "%s: %d does not fit a 64-bit slot" % (key, n)
    OUT.append((key, n & 0xFFFFFFFFFFFFFFFF))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


def s(key, text):
    """A STRING key, as an FNV-1a 64-bit hash. **A REGIME IS THE ONE THING NO FLOAT WITNESSES** --
    AD step 2 measured a wrong authority label inside a FILTER, which dropped a point and then
    reported perfect tracking over an empty set."""
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


def opt(key, x):
    """A key Python may return as `None` -- emitted as a PRESENCE FLAG beside the value. Rung 73's
    `max(..., default=None)` aggregates return `None` on an EMPTY sample, which is § 5.27 (ii)'s
    measured break shape: a cell that empties the table and passes every value diff."""
    b(key + "?", x is not None)
    if x is not None:
        f(key, x)


def opt_d(key, n):
    b(key + "?", n is not None)
    if n is not None:
        d(key, n)


def opt_s(key, text):
    b(key + "?", text is not None)
    if text is not None:
        s(key, text)


def opt_span(key, pair):
    b(key + "?", pair is not None)
    if pair is not None:
        f(key + "/lo", pair[0])
        f(key + "/hi", pair[1])


def flist(key, xs):
    d(key + "/n", len(xs))
    for i, x in enumerate(xs):
        f("%s/%d" % (key, i), x)


def dlist(key, xs):
    d(key + "/n", len(xs))
    for i, x in enumerate(xs):
        d("%s/%d" % (key, i), x)


def slist(key, xs):
    d(key + "/n", len(xs))
    for i, x in enumerate(xs):
        s("%s/%d" % (key, i), x)


def taus4(key, t):
    for i, x in enumerate(t):
        f("%s/%d" % (key, i), x)


# THE MARCH POINT'S THIRTY FIELDS, split by type. Spelled out rather than derived from a live
# point, because the PORT's side is a hand-written list and a generic loop facing a hand-written
# list is the pair that drifts silently. `v_regime` is the one that can be `None` -- section J
# asserts this set against a live point, so the spelling cannot drift from the plant either.
PT_FLOAT = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
            "mdot_air", "sp_thrust", "mf", "mf_sched", "g", "required", "g_fuel", "g_gov",
            "required_fuel", "required_gov", "b", "b_cmd", "v", "v_cmd", "ic_res")
PT_STR = ("branch", "authority", "ic_order", "share_law")
PT_OPT_STR = ("v_regime",)
PT_INT = ("ic_iters",)
assert len(PT_FLOAT) + len(PT_STR) + len(PT_OPT_STR) + len(PT_INT) == 30


# ==============================================================================================
# THE SUITE'S OWN CONSTANTS -- `tests/test_rung73.py` lines 42-66, copied
# ==============================================================================================
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI = 1000.0, 1400.0
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU = 0.05
TT4_MAX = 1200.0
CLOCKS = ((0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05), (0.20, 0.005, 0.80, 0.05))

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg():
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=1.3, cp_t=1239.0, R_t=(1.3 - 1.0) / 1.3 * 1239.0, hPR=42.8e6)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def machine():
    """A FRESH machine per reader, which is what the suite does. AD step 5 measured that the plant
    carries no state between reader calls (full-trajectory equality on four repeated signatures);
    building one machine here and reusing it would make that a HOPE rather than the suite's own
    shape."""
    return AppliedReferenceTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                     bleed_lim=BleedLimiter.from_margin(LP, B, SM, tau=TAU))


# ==============================================================================================
# THE INTERCEPTORS -- installed BEFORE section A, removed after section H
#
# Every extra grid is INTERCEPTED at its own function boundary and never reconstructed by hand
# (slice Z's leading finding: a counted copy of a body can only gate the copy). So what they
# capture is exactly what sections A-H drove, and a reader that changes its grid changes these
# sections with it.
#
# THE STRIDES. `J_STRIDE` is coprime to BOTH reader samplings (`every` = 2 and 4), so section J's
# points interleave with the readers' rather than coinciding with them. `K_STRIDE` is applied PER
# PATH and not per call. `L_STRIDE` is per call, and both its functions are dense enough on this
# grid for that to reach each of them -- 490 and 389 calls, MEASURED, and emitted below as keys so
# the claim is checkable rather than asserted here.
# ==============================================================================================
J_STRIDE = 5
K_STRIDE = 250
L_STRIDE = 5

_REF = AppliedReferenceTransient._reference
_MARCH = SharedActuatorTransient._shared_march
_CP4 = SharedActuatorTransient.__dict__["_charpoly4"].__func__
_QR = SharedActuatorTransient.__dict__["_quartic_roots_c"].__func__

REF_N = [0]
REF_PATH = [0, 0, 0, 0]      # slots 1/2/3 used; slot 0 unused so the path number IS the index
REF_BIT = [0, 0, 0, 0]       # per path: returned `req` BITWISE
REF_ZERO = [0]               # `req == 0.0` exactly -- where a RELATIVE gap is undefined
REF_GAP = []                 # path 3 only: |out - req|
REF_TUPLES = []

MARCH_N = [0]
MARCH_SIGS = []
MARCH_SEEN = {}

CP4_N = [0]
CP4_PAIRS = []
QR_N = [0]
QR_PAIRS = []


def _ref_tap(self, req, g_own, gf, gr):
    REF_N[0] += 1
    out = _REF(self, req, g_own, gf, gr)
    # THE PATH IS REPLICATED FROM THE SHIPPED CONDITION, NEVER INFERRED FROM THE VALUE. Asking
    # `out == req` to name the branch would beg the very question this section asks of path 3
    # (does it EVER return `req` bitwise -- § 5.29 (iii) measured 0 of 109 307 on the suite's own
    # grid), and a fold of path 2 would then silently RE-LABEL itself instead of showing up.
    if self._ref_law != "applied":
        path = 1
    elif self._applied_clip(gf, gr) == g_own:
        path = 2
    else:
        path = 3
        REF_GAP.append(abs(out - req))
    REF_PATH[path] += 1
    if out == req:
        REF_BIT[path] += 1
    if req == 0.0:
        REF_ZERO[0] += 1
    if REF_PATH[path] % K_STRIDE == 0:
        REF_TUPLES.append((self._share_law, self._ref_law, req, g_own, gf, gr, out, path))
    return out


def _march_tap(self, flight, Tt4_lo, Tt4_hi, Tt4_max, sm, taus, r, s_settle, ds, v_max, inc):
    MARCH_N[0] += 1
    out = _MARCH(self, flight, Tt4_lo, Tt4_hi, Tt4_max, sm, taus, r, s_settle, ds, v_max, inc)
    # THE SIGNATURE CARRIES `_ref_law`, AND THAT IS NOT DECORATION. `handover_law` runs the SAME
    # eleven arguments under BOTH laws (`_with_ref('sched', self._shared_march, ...)` and then
    # `'applied'`) and `_reference` is live inside the march, so the trajectory is a function of
    # the law as well. Keying on the arguments alone would collapse two DIFFERENT marches onto one
    # signature and emit whichever ran last.
    #
    # `flight` is NOT in the key: it is this module's single constant and every call passes it.
    # The pre-measurement probe put it in and died on `unhashable type: 'FlightCondition'` -- after
    # printing every count, so the failure was loud and cost nothing.
    sig = (self._ref_law, Tt4_lo, Tt4_hi, Tt4_max, sm, tuple(taus), r, s_settle, ds, v_max, inc)
    if sig not in MARCH_SEEN:
        MARCH_SEEN[sig] = out[3]
        MARCH_SIGS.append(sig)
    return out


def _cp4_tap(A):
    CP4_N[0] += 1
    out = _CP4(A)
    if CP4_N[0] % L_STRIDE == 0:
        CP4_PAIRS.append(([row[:] for row in A], list(out)))
    return out


def _qr_tap(coef):
    QR_N[0] += 1
    out = _QR(coef)
    if QR_N[0] % L_STRIDE == 0:
        QR_PAIRS.append((list(coef), list(out)))
    return out


AppliedReferenceTransient._reference = _ref_tap
SharedActuatorTransient._shared_march = _march_tap
SharedActuatorTransient._charpoly4 = staticmethod(_cp4_tap)
SharedActuatorTransient._quartic_roots_c = staticmethod(_qr_tap)


def emit_gains(p, g):
    """`_quad_gains_at`'s return, whole. See the header for why `s` is not among these keys."""
    b(p + "/interior", g["interior"])
    b(p + "/near_switch", g["near_switch"])
    slist(p + "/off_regime", g["off_regime"])
    f(p + "/v_base", g["v_base"])
    for nm in ("F_f", "F_r", "F_q", "F_v", "R_f", "R_r", "R_q", "R_v",
               "C_f", "C_r", "C_v", "V_f", "V_r", "V_q",
               "pair_FR", "pair_RC", "pair_CV", "pair_RV"):
        f(p + "/" + nm, g[nm])
    opt_s(p + "/authority", g["authority"])
    opt_s(p + "/masked", g["masked"])
    opt(p + "/mask_leak", g["mask_leak"])
    opt(p + "/self_masked", g["self_masked"])
    opt(p + "/cross_masked", g["cross_masked"])
    opt(p + "/self_live", g["self_live"])


# ==============================================================================================
# A -- `handover_law`, at `ds = 0.005` over the THREE-arm `CLOCKS`
# ==============================================================================================
h = machine().handover_law(FLIGHT, LO, HI, TT4_MAX, SM, clocks=CLOCKS)
b("A/always_later", h["always_later"])
b("A/never_back", h["never_back"])
b("A/one_handover", h["one_handover"])
b("A/full_march", h["full_march"])
f("A/worst_dTt4", h["worst_dTt4"])
f("A/worst_dphi", h["worst_dphi"])
f("A/worst_delay", h["worst_delay"])
f("A/ds", h["ds"])
d("A/n_clocks", len(h["clocks"]))
for i, c in enumerate(h["clocks"]):
    taus4("A/clock/%d" % i, c)
d("A/n_arms", len(h["arms"]))
for i, a in enumerate(h["arms"]):
    k = "A/arm/%d" % i
    b(k + "/inc", a["inc"])
    taus4(k + "/taus", a["taus"])
    b(k + "/later", a["later"])
    opt(k + "/delay", a["delay"])
    f(k + "/dTt4", a["dTt4"])
    f(k + "/dphi", a["dphi"])
    for law in ("sched", "applied"):
        p, kk = a["laws"][law], "%s/%s" % (k, law)
        d(kk + "/n", p["n"])
        flist(kk + "/handovers", p["handovers"])
        flist(kk + "/hands_back", p["hands_back"])
        opt(kk + "/first_gov", p["first_gov"])
        opt(kk + "/max_masked", p["max_masked"])
        f(kk + "/final_g_fuel", p["final_g_fuel"])
        f(kk + "/final_g_gov", p["final_g_gov"])
        f(kk + "/max_Tt4", p["max_Tt4"])
        f(kk + "/min_phi", p["min_phi"])
        d(kk + "/ic_iters", p["ic_iters"])
        f(kk + "/ic_res", p["ic_res"])

# ==============================================================================================
# B, C -- `applied_gains`, at `ds = 0.002`, `every = 2`, on BOTH `inc` arms
# ==============================================================================================
for tag, inc in (("B", False), ("C", True)):
    g = machine().applied_gains(FLIGHT, LO, HI, TT4_MAX, SM, taus=CLOCKS[0], inc=inc)
    b(tag + "/inc", g["inc"])
    taus4(tag + "/taus", g["taus"])
    f(tag + "/ds", g["ds"])
    d(tag + "/n_riding", g["n_riding"])
    d(tag + "/n_sampled", g["n_sampled"])
    d(tag + "/skipped/switch", g["skipped"]["switch"])
    d(tag + "/skipped/regime", g["skipped"]["regime"])
    d(tag + "/by_authority/fuel", g["by_authority"]["fuel"])
    d(tag + "/by_authority/gov", g["by_authority"]["gov"])
    flist(tag + "/self_masked", g["self_masked"])
    flist(tag + "/cross_masked", g["cross_masked"])
    flist(tag + "/self_live", g["self_live"])
    flist(tag + "/moved_scaled", g["moved_scaled"])
    opt(tag + "/worst_mask_leak", g["worst_mask_leak"])
    opt(tag + "/worst_delta_rest", g["worst_delta_rest"])
    opt(tag + "/min_live_gain", g["min_live_gain"])
    opt_span(tag + "/det_range", g["det_range"])
    d(tag + "/n_boundary", len(g["boundary"]))
    for i, x in enumerate(g["boundary"]):
        k = "%s/bnd/%d" % (tag, i)
        f(k + "/s", x["s"])
        for side in ("live", "dead"):
            for nm in ("F_q", "F_v", "R_q", "R_v"):
                f("%s/%s/%s" % (k, side, nm), x[side][nm])
    d(tag + "/n_rows", len(g["rows"]))
    for i, x in enumerate(g["rows"]):
        k = "%s/row/%d" % (tag, i)
        f(k + "/s", x["s"])
        opt_s(k + "/authority", x["authority"])
        opt_s(k + "/masked", x["masked"])
        opt(k + "/self_masked", x["self_masked"])
        opt(k + "/cross_masked", x["cross_masked"])
        opt(k + "/self_live", x["self_live"])
        opt(k + "/mask_leak", x["mask_leak"])
        f(k + "/delta_moved/0", x["delta_moved"][0])
        f(k + "/delta_moved/1", x["delta_moved"][1])
        f(k + "/delta_rest", x["delta_rest"])
        f(k + "/det", x["det"])
        taus4(k + "/taus", x["taus"])
        emit_gains(k + "/g", x["gains"])

# ==============================================================================================
# D -- `applied_cells`, at `ds = 0.002`, `every = 2`, over the THREE-arm `CLOCKS`
# ==============================================================================================
c = machine().applied_cells(FLIGHT, LO, HI, TT4_MAX, SM, clocks=CLOCKS)
b("D/law_holds", c["law_holds"])
b("D/all_four_cells", c["all_four_cells"])
f("D/ds", c["ds"])
f("D/worst_parent_gap", c["worst_parent_gap"])
f("D/worst_parent_gap_hi", c["worst_parent_gap_hi"])
f("D/worst_v_gap", c["worst_v_gap"])
f("D/worst_null", c["worst_null"])
f("D/worst_det", c["worst_det"])
f("D/worst_lam", c["worst_lam"])
f("D/pole_at_origin", c["pole_at_origin"])
d("D/n_clocks", len(c["clocks"]))
for i, cl in enumerate(c["clocks"]):
    taus4("D/clock/%d" % i, cl)
for nm in ("predicted", "rung72"):
    d("D/%s/n" % nm, len(c[nm]))
    for key in sorted(c[nm], key=lambda t: (t[0], t[1])):
        d("D/%s/%d_%s" % (nm, int(key[0]), key[1]), c[nm][key])
d("D/n_cells", len(c["cells"]))
for key in sorted(c["cells"], key=lambda t: (t[0], t[1])):
    v, k = c["cells"][key], "D/cell/%d_%s" % (int(key[0]), key[1])
    s(k + "/parent", v["parent"])
    dlist(k + "/zeros", v["zeros"])
    f(k + "/gap", v["gap"])
    f(k + "/gap_hi", v["gap_hi"])
    f(k + "/vgap", v["vgap"])
    f(k + "/pole", v["pole"])
    f(k + "/null", v["null"])
    f(k + "/lam_max", v["lam_max"])
    f(k + "/det", v["det"])
    d(k + "/n", v["n"])
    d(k + "/n_parent", v["n_parent"])
d("D/n_arms", len(c["arms"]))
for i, a in enumerate(c["arms"]):
    k = "D/arm/%d" % i
    b(k + "/inc", a["inc"])
    taus4(k + "/taus", a["taus"])
    d(k + "/n_riding", a["n_riding"])
    d(k + "/n_sampled", a["n_sampled"])
    d(k + "/skipped/switch", a["skipped"]["switch"])
    d(k + "/skipped/regime", a["skipped"]["regime"])
    d(k + "/skipped/parent", a["skipped"]["parent"])
    d(k + "/n_cells", len(a["cells"]))
    for auth in sorted(a["cells"]):
        cc, kk = a["cells"][auth], "%s/cell/%s" % (k, auth)
        d(kk + "/n", cc["n"])
        d(kk + "/n_parent", cc["n_parent"])
        dlist(kk + "/zeros", cc["zeros"])
        f(kk + "/gap", cc["gap"])
        f(kk + "/gap_hi", cc["gap_hi"])
        f(kk + "/vgap", cc["vgap"])
        f(kk + "/pole", cc["pole"])
        f(kk + "/null", cc["null"])
        f(kk + "/lam_max", cc["lam_max"])
        f(kk + "/det/lo", cc["det"][0])
        f(kk + "/det/hi", cc["det"][1])
        f(kk + "/s/lo", cc["s"][0])
        f(kk + "/s/hi", cc["s"][1])
        s(kk + "/parent", cc["parent"])

# ==============================================================================================
# E, F -- `ref_discriminator`, at `ds = 0.002`, `every = 4`, on BOTH `inc` arms
# ==============================================================================================
for tag, inc in (("E", False), ("F", True)):
    dd = machine().ref_discriminator(FLIGHT, LO, HI, TT4_MAX, SM, taus=CLOCKS[0], inc=inc)
    b(tag + "/inc", dd["inc"])
    taus4(tag + "/taus", dd["taus"])
    f(tag + "/ds", dd["ds"])
    d(tag + "/n", dd["n"])
    opt(tag + "/worst_origin_B", dd["worst_origin_B"])
    opt(tag + "/best_origin_C", dd["best_origin_C"])
    opt(tag + "/best_origin_72", dd["best_origin_72"])
    opt(tag + "/worst_pole_C", dd["worst_pole_C"])
    opt(tag + "/worst_pole_72", dd["worst_pole_72"])
    opt(tag + "/best_pole_B", dd["best_pole_B"])
    flist(tag + "/live_diag_B", dd["live_diag_B"])
    flist(tag + "/live_diag_C", dd["live_diag_C"])
    for k2 in ("B", "C", "72"):
        dlist(tag + "/zeros/" + k2, dd["zeros"][k2])
    dlist(tag + "/dzeros_B", dd["dzeros_B"])
    dlist(tag + "/dzeros_C", dd["dzeros_C"])
    d(tag + "/n_rows", len(dd["rows"]))
    for i, x in enumerate(dd["rows"]):
        k = "%s/row/%d" % (tag, i)
        f(k + "/s", x["s"])
        opt_s(k + "/authority", x["authority"])
        opt_s(k + "/masked", x["masked"])
        taus4(k + "/taus", x["taus"])
        f(k + "/tau_live", x["tau_live"])
        for nm in ("origin_B", "origin_C", "origin_72", "pole_B", "pole_C", "pole_72",
                   "live_diag_B", "live_diag_C"):
            f(k + "/" + nm, x[nm])
        for nm in ("zeros_B", "zeros_C", "zeros_72"):
            d(k + "/" + nm, x[nm])

# ==============================================================================================
# G, H -- `applied_bill`, at `ds = 0.005`, on BOTH `inc` arms
# ==============================================================================================
for tag, inc in (("G", False), ("H", True)):
    bl = machine().applied_bill(FLIGHT, LO, HI, TT4_MAX, SM, taus=CLOCKS[0], inc=inc)
    b(tag + "/inc", bl["inc"])
    taus4(tag + "/taus", bl["taus"])
    f(tag + "/ds", bl["ds"])
    for nm in ("debit_sched", "debit_applied", "phi_marginal_sched", "phi_marginal_applied",
               "phi_full_sched", "phi_full_applied", "Tt4_integral_sched",
               "Tt4_integral_applied"):
        f(tag + "/" + nm, bl[nm])
    for nm in ("debit_ratio", "kept_sched", "kept_applied", "handover_sched",
               "handover_applied"):
        opt(tag + "/" + nm, bl[nm])
    for law in ("sched", "applied"):
        sb, kk = bl[law], "%s/%s" % (tag, law)
        b(kk + "/inc", sb["inc"])
        taus4(kk + "/taus", sb["taus"])
        f(kk + "/Tt4_full", sb["Tt4_full"])
        f(kk + "/Tt4_no_fuel", sb["Tt4_no_fuel"])
        f(kk + "/phi_full", sb["phi_full"])
        f(kk + "/phi_no_fuel", sb["phi_no_fuel"])
        f(kk + "/fuel_marginal_phi", sb["fuel_marginal_phi"])
        f(kk + "/fuel_marginal_Tt4", sb["fuel_marginal_Tt4"])
        opt(kk + "/handover", sb["handover"])
        for nm in ("phi", "Tt4", "inc"):
            opt(kk + "/delivered/" + nm, sb["delivered"][nm])
        for leg in ("F", "G", "V", "S"):
            f(kk + "/marginal/" + leg, sb["marginal"][leg])
            f(kk + "/alone/" + leg, sb["alone"][leg])
            opt(kk + "/kept/" + leg, sb["kept"][leg])
        d(kk + "/n_cells", len(sb["cells"]))
        for cell in sorted(sb["cells"]):
            c2, k3 = sb["cells"][cell], "%s/cell/%s" % (kk, cell)
            for leg in ("F", "G", "V", "S"):
                b("%s/on/%s" % (k3, leg), c2["on"][leg])
            for nm in ("I", "E", "M", "min_phi", "max_Tt4"):
                f(k3 + "/" + nm, c2[nm])
            d(k3 + "/n", c2["n"])
            d(k3 + "/auth_fuel", c2["auth_fuel"])
            opt(k3 + "/handover", c2["handover"])
            for nm in ("credit_phi", "credit_Tt4", "credit_inc"):
                opt(k3 + "/" + nm, c2[nm])

# --- THE INTERCEPTORS COME OFF HERE. Sections J-M read only what A-H drove. -------------------
AppliedReferenceTransient._reference = _REF
SharedActuatorTransient._shared_march = _MARCH
SharedActuatorTransient._charpoly4 = staticmethod(_CP4)
SharedActuatorTransient._quartic_roots_c = staticmethod(_QR)

# ==============================================================================================
# J -- THE SIX-STATE MARCH, PER POINT.  A DECLARED EXTRA GRID.
#
# Sections A-H are the readers' AGGREGATES over this march: a `min`, a count, a window and a
# worst-of can all sit still while the points under them move. And the suite's own reduce spine
# (`_keys`) compares NINE of the march's thirty recorded fields, so twenty-one are read by nothing
# in it.
#
# **P3's clause (b) cannot be scored without this section.** It predicts the CPython arm drifts on
# march values, first in the stator state `v`, at 1-11 ULPs -- a PER-POINT claim no reader
# aggregate can confirm or refute. AD step 5 found its six drifting keys at 2 points of 1 302 only
# because the equivalent section existed.
#
# **THE BACKSTOP IS CARRIED AS AD's CORRECTION, NOT AS AD's CLAIM.** `agg/*/{min,max,last}` runs
# over EVERY point, not the strided ones, so a defect moving a column's EXTREME or its FINAL value
# is caught wherever it sits. AD step 5's close-out measured the REST of that sentence FALSE: a
# hidden-point defect does NOT still have to move a key (index 137 moved 0 of 54 116, against a
# control at 135 that moved 10). So what this section pins is every field at one point in five,
# plus both extremes and the endpoint of every float column -- and nothing more.
# ==============================================================================================
assert MARCH_N[0] > 0, "section J intercepted NOTHING -- a silent zero from a live grid"
d("J/n_calls", MARCH_N[0])
d("J/n_sigs", len(MARCH_SIGS))
d("J/stride", J_STRIDE)
d("J/n_fields", len(PT_FLOAT) + len(PT_STR) + len(PT_OPT_STR) + len(PT_INT))
d("J/n_float_fields", len(PT_FLOAT))
d("J/n_str_fields", len(PT_STR))
d("J/n_opt_str_fields", len(PT_OPT_STR))
d("J/n_int_fields", len(PT_INT))
for i, k in enumerate(PT_FLOAT + PT_STR + PT_OPT_STR + PT_INT):
    s("J/field/%d" % i, k)
# THE FIELD SET IS ASSERTED AGAINST THE POINT, NOT ASSUMED FROM IT -- a hand-written list on one
# side and a generic loop on the other is the pair that silently drifts, and a hand-written list
# on BOTH sides drifts silently from the PLANT. So the names are spelled out (they have to be, to
# match the port's) and then checked against a live point.
assert set(PT_FLOAT + PT_STR + PT_OPT_STR + PT_INT) == set(MARCH_SEEN[MARCH_SIGS[0]][0]), (
    sorted(set(MARCH_SEEN[MARCH_SIGS[0]][0]) ^ set(PT_FLOAT + PT_STR + PT_OPT_STR + PT_INT)))
for i, sig in enumerate(MARCH_SIGS):
    law, lo, hi, tmax, smv, taus, r, s_settle, ds, v_max, inc = sig
    traj, p = MARCH_SEEN[sig], "J/sig/%d" % i
    s(p + "/in/law", law)
    f(p + "/in/Tt4_lo", lo)
    f(p + "/in/Tt4_hi", hi)
    f(p + "/in/Tt4_max", tmax)
    f(p + "/in/sm", smv)
    taus4(p + "/in/tau", taus)
    f(p + "/in/r", r)
    f(p + "/in/s_settle", s_settle)
    f(p + "/in/ds", ds)
    f(p + "/in/v_max", v_max)
    b(p + "/in/inc", inc)
    d(p + "/n_points", len(traj))
    d(p + "/n_emitted", (len(traj) + J_STRIDE - 1) // J_STRIDE)
    for k in PT_FLOAT:
        col = [pt[k] for pt in traj]
        f("%s/agg/%s/min" % (p, k), min(col))
        f("%s/agg/%s/max" % (p, k), max(col))
        f("%s/agg/%s/last" % (p, k), col[-1])
    for j in range(0, len(traj), J_STRIDE):
        q = "%s/pt/%d" % (p, j)
        for k in PT_FLOAT:
            f("%s/%s" % (q, k), traj[j][k])
        for k in PT_STR:
            s("%s/%s" % (q, k), traj[j][k])
        for k in PT_OPT_STR:
            opt_s("%s/%s" % (q, k), traj[j][k])
        for k in PT_INT:
            d("%s/%s" % (q, k), traj[j][k])

# ==============================================================================================
# K -- `_reference`, THE RUNG'S OWN LAW, PER CALL.  A DECLARED EXTRA GRID.
#
# **THIS SECTION MEASURES THE FUNCTION, NOT THE PLANT, AND SAYS SO.** The replay tuples are read
# back as INPUTS, so a difference here is the law's own; the PLANT is measured by A-H and J, which
# recompute everything. AD step 5 shipped a 5 022-name exemption because its section H read
# coefficients as inputs and nobody had written that sentence down first.
#
# What it buys: the three paths are emitted as a CENSUS, and the replay is STRATIFIED -- every
# `K_STRIDE`-th call OF EACH PATH -- so a path that fires rarely is still present in known
# proportion. A plain call-index stride samples the three paths in their population ratio and can
# return zero tuples for a rare one. The Rust RE-DERIVES each replayed tuple's path from its own
# `applied_clip`, and the per-path counts OF THE REPLAY SET with it, so the census is not merely
# an input it trusts.
#
# § 5.29 (iii) measured on the whole shipped suite: path 1 41 346 / path 2 109 537 / path 3
# 109 307 (42.01 %), with **0 of path 3's calls returning `req` bitwise**. Those are the SUITE's
# numbers on the SUITE's grid; every count below is THIS grid's, re-measured and never quoted
# ([[rust-port-slice-ac-step6]]).
# ==============================================================================================
assert REF_N[0] > 0, "section K intercepted NOTHING -- a silent zero from a live grid"
d("K/n_calls", REF_N[0])
for pth in (1, 2, 3):
    d("K/path/%d" % pth, REF_PATH[pth])
    d("K/returns_req_bitwise/%d" % pth, REF_BIT[pth])
d("K/n_req_exactly_zero", REF_ZERO[0])
d("K/gap3/n", len(REF_GAP))
opt("K/gap3/min", min(REF_GAP) if REF_GAP else None)
opt("K/gap3/max", max(REF_GAP) if REF_GAP else None)
d("K/stride", K_STRIDE)
d("K/n_replay", len(REF_TUPLES))
for i, (sl, rl, req, g_own, gf, gr, out, pth) in enumerate(REF_TUPLES):
    p = "K/replay/%d" % i
    s(p + "/share_law", sl)
    s(p + "/ref_law", rl)
    f(p + "/req", req)
    f(p + "/g_own", g_own)
    f(p + "/gf", gf)
    f(p + "/gr", gr)
    f(p + "/out", out)
    d(p + "/path", pth)

# ==============================================================================================
# L -- THE INHERITED ARITHMETIC: `_charpoly4` AND `_quartic_roots_c`.  A DECLARED EXTRA GRID.
#
# § 5.29 (vi) measured that rung 73 defines NO solver -- the quartic chain is rung 72's, inherited
# entire -- and P3's clause (a) predicts the CPython exemption is dominated by the `sum()`-built
# polynomial inside `_charpoly4`. **"Inherited" is not "driven"**, so this section emits the CALL
# COUNT of each whether or not it is reached: a clause predicted to dominate a grid that never
# enters the function would be VACUOUS, and that is a finding rather than a disappointment (AD's
# j06 at 0 of 3 216 and then 0 of 54 116).
#
# LIKE SECTION K, THIS IS A REPLAY AND MEASURES THE FUNCTION -- which is exactly what makes it an
# ATTRIBUTION instrument. Fed the SAME 4x4 matrix, a coefficient that differs between two
# interpreters is `sum()`'s difference and nothing downstream of it; fed the same coefficients, a
# root that differs is the root finder's. AD step 5 could only reach that by a cross-feed run
# after the fact.
# ==============================================================================================
d("L/cp4/n_calls", CP4_N[0])
d("L/qr/n_calls", QR_N[0])
d("L/stride", L_STRIDE)
d("L/cp4/n_replay", len(CP4_PAIRS))
for i, (A, out) in enumerate(CP4_PAIRS):
    p = "L/cp4/%d" % i
    for rr in range(4):
        for cc in range(4):
            f("%s/in/%d/%d" % (p, rr, cc), A[rr][cc])
    for k, x in enumerate(out):
        f("%s/out/%d" % (p, k), x)
d("L/qr/n_replay", len(QR_PAIRS))
for i, (coef, out) in enumerate(QR_PAIRS):
    p = "L/qr/%d" % i
    for k, x in enumerate(coef):
        f("%s/in/%d" % (p, k), x)
    for k, z in enumerate(out):
        f("%s/out/%d/re" % (p, k), z.real)
        f("%s/out/%d/im" % (p, k), z.imag)
        f("%s/out/%d/abs" % (p, k), abs(z))

# ==============================================================================================
# M -- THE SIGNED-ZERO CENSUS OVER THIS DUMP'S OWN KEYS.  Step 2's M22, RE-MEASURED HERE.
#
# Step 2 § (e) mutated the four `sorted({...})` sets in `applied_gains` to key by BITS and got
# **0 of 5 066** -- then measured WHY rather than shrugging: 101 keys were exactly `-0.0` (every
# one a `*/g/pair_FR`), 925 exactly `+0.0`, and inside those four sets `+0.0` appeared twice and
# `-0.0` not at all. So the hazard the defence guards against is real IN THE DUMP and simply never
# enters one of the sets it guards. It was BOOKED HERE, to a wider grid.
#
# **The CONDITION is emitted, never the verdict.** `M/set/*/n_neg` and `M/set/*/n_pos` say whether
# the two zeros ever meet in ONE set, and the Rust re-derives every count below from its own
# values, so this section is a re-derivation and not an input.
# ==============================================================================================
_NEG0, _POS0 = 0x8000000000000000, 0
d("M/n_keys_before_M", len(OUT))
d("M/n_neg_zero", sum(1 for _, v in OUT if v == _NEG0))
d("M/n_pos_zero", sum(1 for _, v in OUT if v == _POS0))
for nm in ("self_masked", "cross_masked", "self_live", "moved_scaled"):
    sel = [(k, v) for k, v in OUT
           if k.startswith(("B/" + nm + "/", "C/" + nm + "/")) and not k.endswith("/n")]
    d("M/set/%s/n" % nm, len(sel))
    d("M/set/%s/n_neg" % nm, sum(1 for _, v in sel if v == _NEG0))
    d("M/set/%s/n_pos" % nm, sum(1 for _, v in sel if v == _POS0))

# ---------------------------------------------------------------------------------------------
print("# slice AE step 4 -- rung 73 ORACLE, the READERS' own grid (A-H), uncoarsened, plus THREE "
      "declared extra grids (J march, K reference, L inherited arithmetic) and a signed-zero "
      "census (M). key<TAB>u64 (floats are IEEE-754 bits, strings FNV-1a).")
_seen = set()
for _key, _val in OUT:
    assert _key not in _seen, "duplicate key %s" % _key
    _seen.add(_key)
    print("%s\t%d" % (_key, _val))
print("# %d keys; %d march calls over %d distinct signatures; %d `_reference` calls "
      "(%d/%d/%d by path); %d charpoly4 and %d quartic calls"
      % (len(OUT), MARCH_N[0], len(MARCH_SIGS), REF_N[0], REF_PATH[1], REF_PATH[2], REF_PATH[3],
         CP4_N[0], QR_N[0]), file=sys.stderr)
