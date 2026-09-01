"""SLICE AD step 5 -- THE ORACLE for rung 72 (`SharedActuatorTransient`).

`rung72.rs` ports the suite's own 28 gates, and step 4 measured what they cannot see: deleting
the `|a3|` term from Durand-Kerner's start scale moves **26 of step 3's 3 216 values** -- eight of
them inside `charpoly_selftest`, the rung's OWN instrument-gated-against-itself -- and **all 28
gates pass, in Rust AND in Python**. Their bars are one-sided, so a different start converging to
the same roots moves the residuals in their last digits and never across anything. **This file is
the seat that can see it**: every value compared as its IEEE-754 bit pattern.

# THE GRID IS THE READERS' OWN AND NOTHING IS COARSENED, AND IT IS FIVE DIFFERENT GRIDS

Step 4 § (e) measured that rung 72's five readers default to three `ds`, two `every` and three
clock grids, and that the shipped suite passes NONE of them -- every call site spells
`FLIGHT, LO, HI, TT4_MAX, SM` plus at most `inc=` or `clocks=CLOCKS`, and `CLOCKS` is
bit-identical to the default tuple, so those two call sites substitute nothing. That table is
copied here VERBATIM rather than re-derived, because transcribing this module's own `DS` into the
other three would move every number in sections B-F without failing anything --
[[rust-port-slice-ac-step6]]'s `every = 40`-vs-`10` defect, which is the reason the table exists:

    reader                ds       every   clock grid
    authority_law         0.005    --      the two-arm `CLOCKS`
    shared_gains          0.002    2       matched `taus`
    shared_cells          0.002    2       the two-arm `CLOCKS`
    mask_discriminator    0.002    4       its OWN THREE-arm grid
    shared_bill           0.005    --      matched `taus`

So every reader below is called with the FIVE arguments the suite passes and nothing else. The
flight condition, the maps, the floor, `LO`/`HI`, `PHI`/`B`/`V_MAX`, `TAU` and `TT4_MAX` are
`tests/test_rung72.py`'s own module constants, copied.

# SECTIONS G, H AND J ARE DECLARED EXTRA GRIDS, AND EACH SAYS WHY THE READERS CANNOT REACH IT

Folding one into A-F would be exactly the defect the paragraph above guards against, so they are
lettered apart -- slice AC's section N, three times over. **All three are INTERCEPTED at the
function boundary and never reconstructed** (slice Z's leading finding): a counted copy of a body
can only gate the copy. The interceptors are installed BEFORE section A runs and removed after
section F, so what they capture is exactly what sections A-F drove.

  * **G -- THE SIX-STATE MARCH, PER POINT.** Step 2 landed a 209-line integrator behind THIRTEEN
    gates that are all RELATIONS, and step 3's 3 216 keys are the readers' AGGREGATES over its
    trajectory. An aggregate is lossy: a `min`, a count and a window can all sit still while the
    points under them move. **And the suite's own reduce spine compares 9 of the march's 30
    recorded fields** (`_keys`'s tuple), so 21 of them are read by nothing in it. THE SIGNATURES
    ARE INTERCEPTED, not enumerated by hand, so the arms are the readers' own by construction and
    a reader that changes its grid changes this section with it.

    **THE STRIDE IS 5 AND IT IS CHOSEN TO BE COPRIME TO THE READERS' OWN.** `shared_gains` and
    `shared_cells` sample at `every = 2` and `mask_discriminator` at `every = 4`, so a stride of 2
    or 4 here would emit exactly the points their gain rows already cover and nothing else -- an
    extra grid that is not extra. 5 shares no factor with either, so section G's points and the
    readers' samples interleave. **And the stride is backstopped**: `G/sig/i/agg/...` carries the
    `min`, `max` and LAST value of every float field over ALL points, not the strided ones, so a
    defect isolated to a hidden point still has to move a key.

  * **H -- `_quartic_roots_c`, PER DISTINCT COEFFICIENT VECTOR.** § 5.28 (iii) measured the solver
    at **1 068 calls over 375 distinct vectors, 167 of them near-double, ON THE WHOLE SUITE**.
    This dump's grid is smaller and both counts are RE-MEASURED here and emitted as keys, because
    quoting the pre-flight's would be a number about a different population -- the same error
    § 5.27.6 (i) records for a row measured at `every = 40` and quoted against a fixture passing
    10. Each vector carries a `near_double` flag (min pairwise root separation `< 1e-6`,
    § 5.28 (iii)'s own definition) and the INDEX of the term that won `scale`'s max, because P3 is
    TWO claims and the second one is unscoreable without the flag.

  * **J -- `_authority`, AND ITS MARGIN.** P4 predicts that writing `gf == gr` for
    `abs(gf - gr) <= tol` changes no oracle key. **That prediction is VACUOUS if this dump's grid
    never reaches the function**, and four instruments in this slice's own history printed a
    confident zero from a run that reached nothing ([[rust-port-slice-ad-preflight]]). So the call
    count, the distinct-pair count, the exact-zero count, the OPEN-INTERVAL count, the label
    histogram and `min_nonzero_gap` are all emitted as keys. **`J/n_open` is the key P4 lives or
    dies on**, and `J/min_nonzero_gap` is the MARGIN -- a number, never the word "unreachable"
    (§ 5.28.3 (f)'s discipline). The replayed pairs are not all 7 450: they are every EXACT-ZERO
    pair plus the twenty smallest non-zero gaps, which is where a tolerance change would first
    bite. The selection rule is stated so it can be checked, and the Rust re-derives it from the
    emitted count rather than trusting the file.

# WHAT IS DELIBERATELY NOT EMITTED, so the Rust's missing-key half stays honest

  * **`at_lever`.** It is a BUILDER, and slice AC step 7 measured it as the LAUNDERER -- every
    reader rebuilds its machine through it and installs the shipped tables, so no value key can
    witness which function pointer sat in the slot. That is step 6's subject, not this one's.
  * **The raises.** `_assert_fuel_boundary`'s two bars, `_rk4_floor_shared`'s `ds * rate <= 2.0`,
    the four arming asserts and the joint-IC residual assert are CONTROL FLOW, not values. Nothing
    below passes a `ds` that trips the floor, so no key here can see it -- stated rather than
    implied, because a silent absence reads as coverage. `rung72.rs` gates them by their RUNG TAG,
    which § 5.28 (v) measured to be the only discriminating token (the shipped Python needle
    `"FOUR actuator states"` is in rungs 72, 73 AND 74's character-identical messages).
  * **`shared_bill`'s `own_currency`** -- a table of CONSTANT strings that cannot differ between
    two runs. The port drops it for the same reason and says so in its own doc comment. A
    DECISION, not an omission.
  * **`_quad_gains_at`'s DEFERRAL.** § 5.28.3 (a) measured it a FOURTH cell -- two definers,
    identical signature, ZERO call sites of any kind, PASSED as a bound method at eleven lines
    over six rungs. It is not installed as a hook (§ 5.28.3 (b)) and is booked to slice AE, so
    nothing here is a dispatch key for it. Its VALUES are covered: every row of sections C, D and
    E comes out of it.
  * **The three dead roots and the `tie` branch.** `|a3|` wins `scale`'s max on every shipped call
    (§ 5.28 (iii)), so the cube root and both even roots are unreachable -- section H emits the
    winning index per vector, which makes that a MEASURED key here and not a claim. And section J
    measures ZERO `tie` labels on this grid where the whole suite has one, so no key below can see
    the `tie` branch either.

Every float is emitted as its IEEE-754 bit pattern. Regenerate BOTH arms:

    .venv/Scripts/python.exe rust/oracle/dump_slice_ad.py > rust/oracle/slice_ad_pypy.tsv
    C:/Python314/python.exe  rust/oracle/dump_slice_ad.py > rust/oracle/slice_ad_cpython.tsv

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
    SharedActuatorTransient, BleedLimiter,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


def s(key, text):
    """A STRING key, as an FNV-1a 64-bit hash -- the authority labels, the nozzle branch, the
    stator regime, the `ic_order` and the ledger's cell names are the non-floats a rung-72 reading
    carries. **A REGIME IS THE ONE THING NO FLOAT WITNESSES**, and § 5.28.2 (a) measured that a
    wrong label inside a FILTER drops a point and then reports perfect tracking over an empty
    set."""
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


def opt(key, x):
    """A key Python may return as `None` -- emitted as a PRESENCE FLAG beside the value. Rung 72's
    `max(..., default=None)` aggregates return `None` on an EMPTY sample, which is § 5.27 (ii)'s
    measured break shape (a cell that empties the table and passes every value diff), and
    `handover`, `credit_*` and `mask_leak` are `None` for real."""
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
    """A `(lo, hi)` window Python returns as `None` on an empty sample."""
    b(key + "?", pair is not None)
    if pair is not None:
        f(key + "/lo", pair[0])
        f(key + "/hi", pair[1])


def c(key, z):
    """A COMPLEX root -- `re`, `im` and `abs`. **`abs` is a KEY and not a convenience**: it is
    `hypot`, and § 5.28 (iii) measured Durand-Kerner leaving an ASYMMETRIC last-bit imaginary
    residue (one member of a conjugate pair at exactly `0.0` and the other not), which is what
    makes bit-exactness the only achievable bar here -- a port agreeing to 1e-14 would move 259 of
    the suite's complex-root counts."""
    f(key + "/re", z.real)
    f(key + "/im", z.imag)
    f(key + "/abs", abs(z))


# =============================================================================================
# THE SUITE'S OWN CONSTANTS -- `tests/test_rung72.py` lines 40-63, copied
# =============================================================================================
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI = 1000.0, 1400.0
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU = 0.05
TT4_MAX = 1200.0                     # RUNG 67's imposed redline, verbatim through rungs 70/71
LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

AUTH_CODE = {"dormant": 1, "tie": 2, "fuel": 3, "gov": 4}
PARENT_CODE = {"rung 68": 68, "rung 69": 69, "rung 70": 70, "rung 71": 71}

# The march point's THIRTY fields, in Python's own dict-construction order. Spelled out rather
# than read off the live dict, because the Rust must emit the same names in the same shapes and a
# generic loop on one side and a hand-written list on the other is the pair that silently drifts.
PT_FLOAT = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
            "mdot_air", "sp_thrust", "mf", "mf_sched", "g", "required", "g_fuel", "g_gov",
            "required_fuel", "required_gov", "b", "b_cmd", "v", "v_cmd", "ic_res")
PT_STR = ("branch", "authority", "ic_order", "share_law")
PT_OPT_STR = ("v_regime",)
PT_INT = ("ic_iters",)
assert len(PT_FLOAT) + len(PT_STR) + len(PT_OPT_STR) + len(PT_INT) == 30

STRIDE = 5                           # coprime to the readers' `every` of 2 and 4 -- see header


def _cpg():
    gc, cc, gt, ct = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cc, R_c=(gc - 1.0) / gc * cc,
               gamma_t=gt, cp_t=ct, R_t=(gt - 1.0) / gt * ct, hPR=42.8e6)


# =============================================================================================
# THE THREE INTERCEPTORS -- installed BEFORE section A, removed after section F
# =============================================================================================
MARCHES, QUART, AUTH = [], [], []
_orig_march = SharedActuatorTransient._shared_march
_orig_quart = SharedActuatorTransient._quartic_roots_c
_orig_auth = SharedActuatorTransient._authority


def _spy_march(self, flight, Tt4_lo, Tt4_hi, Tt4_max, sm, taus, r, s_settle, ds, v_max, inc):
    out = _orig_march(self, flight, Tt4_lo, Tt4_hi, Tt4_max, sm, taus, r, s_settle, ds, v_max,
                      inc)
    MARCHES.append(((tuple(float(t) for t in taus), float(r), float(s_settle), float(ds),
                     float(v_max), bool(inc)), out[3]))
    return out


def _spy_quart(coef):
    out = _orig_quart(coef)
    QUART.append((tuple(float(x) for x in coef), tuple(out)))
    return out


def _spy_auth(gf, gr, tol=1e-12):
    out = _orig_auth(gf, gr, tol)
    AUTH.append((float(gf), float(gr), out))
    return out


SharedActuatorTransient._shared_march = _spy_march
SharedActuatorTransient._quartic_roots_c = staticmethod(_spy_quart)
SharedActuatorTransient._authority = staticmethod(_spy_auth)

design = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)
m = SharedActuatorTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                            bleed_lim=BleedLimiter.from_margin(LP, B, SM, tau=TAU))

# =============================================================================================
# SECTION A -- `charpoly_selftest`, THE ONE READER WITH NO PLANT
# =============================================================================================
# A classmethod with no arguments, no march and no rig, exercising `_charpoly4`,
# `_quartic_roots_c` and every complex operation this slice adds on two fixed matrices. It is
# also the reader step 4's j05 moved EIGHT of TEN keys inside while all 28 gates passed.
_cp = SharedActuatorTransient.charpoly_selftest()
for _name in ("general", "triangular"):
    for _k in sorted(_cp[_name]):
        f("A/%s/%s" % (_name, _k), _cp[_name][_k])
d("A/n_arms", len(_cp))

# =============================================================================================
# SECTION B -- `authority_law` (ds 0.005, the two-arm `CLOCKS`, its OWN defaults)
# =============================================================================================
_al = m.authority_law(FLIGHT, LO, HI, TT4_MAX, SM)
b("B/both_cells_everywhere", _al["both_cells_everywhere"])
b("B/one_handover", _al["one_handover"])
f("B/ds", _al["ds"])
d("B/n_arms", len(_al["arms"]))
for _i, _a in enumerate(_al["arms"]):
    _p = "B/arm/%d" % _i
    b(_p + "/inc", _a["inc"])
    d(_p + "/n", _a["n"])
    for _lab in ("dormant", "tie", "fuel", "gov"):
        d("%s/census/%s" % (_p, _lab), _a["census"].get(_lab, 0))
    d(_p + "/handovers/n", len(_a["handovers"]))
    for _j, _h in enumerate(_a["handovers"]):
        f("%s/handovers/%d" % (_p, _j), _h)
    for _w in ("fuel", "gov", "valve", "stator", "joint"):
        _lo, _hi, _n = _a[_w]
        opt("%s/%s/lo" % (_p, _w), _lo)
        opt("%s/%s/hi" % (_p, _w), _hi)
        d("%s/%s/n" % (_p, _w), _n)
    f(_p + "/joint_fraction", _a["joint_fraction"])
    d(_p + "/both_want", _a["both_want"])
    d(_p + "/in_joint/fuel", _a["in_joint"]["fuel"])
    d(_p + "/in_joint/gov", _a["in_joint"]["gov"])
    b(_p + "/handover_inside", _a["handover_inside"])
    f(_p + "/min_phi", _a["min_phi"])
    f(_p + "/max_Tt4", _a["max_Tt4"])

# =============================================================================================
# SECTION C -- `shared_gains`, BOTH `inc` ARMS (ds 0.002, every 2, matched `taus`)
# =============================================================================================
# Every sample-shaped reading emits its ROW COUNT and its SKIPPED counts, because § 5.27 (ii)'s
# measured break shape is a cell that returns SUCCESSFULLY with an empty table: a value diff over
# two empty tables passes.
for _inc in (False, True):
    _g = m.shared_gains(FLIGHT, LO, HI, TT4_MAX, SM, inc=_inc)
    _p = "C/%d" % int(_inc)
    for _k in ("worst_F_r", "worst_R_f", "worst_pair_FR", "worst_mask_leak", "min_live_gain"):
        opt("%s/%s" % (_p, _k), _g[_k])
    d(_p + "/n_riding", _g["n_riding"])
    d(_p + "/n_sampled", _g["n_sampled"])
    d(_p + "/n_rows", len(_g["rows"]))
    d(_p + "/skipped/switch", _g["skipped"]["switch"])
    d(_p + "/skipped/regime", _g["skipped"]["regime"])
    d(_p + "/by_authority/fuel", _g["by_authority"]["fuel"])
    d(_p + "/by_authority/gov", _g["by_authority"]["gov"])
    opt_span(_p + "/s_window", _g["s_window"])
    opt_span(_p + "/det_range", _g["det_range"])
    d(_p + "/n_boundary", len(_g["boundary"]))
    for _j, _bd in enumerate(_g["boundary"]):
        _q = "%s/boundary/%d" % (_p, _j)
        f(_q + "/s", _bd["s"])
        # `live` and `dead` are two nested dicts, not eight flat keys -- and the DEAD half is
        # emitted even though `_assert_fuel_boundary` already asserts it is identically zero.
        # An asserted zero and an emitted zero are different evidence: the assert lives inside
        # the function under test, so a port that lost the boundary in BOTH the live and the
        # blind path would satisfy its own assert. Four exact zeros in the golden cannot.
        for _half in ("live", "dead"):
            for _k in ("F_q", "F_v", "R_q", "R_v"):
                f("%s/%s/%s" % (_q, _half, _k), _bd[_half][_k])
    for _j, _row in enumerate(_g["rows"]):
        _q = "%s/row/%d" % (_p, _j)
        f(_q + "/s", _row["s"])
        f(_q + "/det", _row["det"])
        opt_s(_q + "/authority", _row["authority"])
        opt_s(_q + "/masked", _row.get("masked"))
        opt(_q + "/mask_leak", _row["mask_leak"])
        for _gk in ("F_r", "F_q", "F_v", "R_f", "R_q", "R_v", "C_f", "C_r", "C_v",
                    "V_f", "V_r", "V_q", "pair_FR", "pair_RC", "pair_CV", "pair_RV", "v_base"):
            f("%s/%s" % (_q, _gk), _row["gains"][_gk])

# =============================================================================================
# SECTION D -- `shared_cells` (ds 0.002, every 2, the two-arm `CLOCKS`, BOTH `inc`)
# =============================================================================================
_c = m.shared_cells(FLIGHT, LO, HI, TT4_MAX, SM)
b("D/law_holds", _c["law_holds"])
b("D/all_four_cells", _c["all_four_cells"])
f("D/worst_parent_gap", _c["worst_parent_gap"])
f("D/worst_v_gap", _c["worst_v_gap"])
f("D/worst_pole", _c["worst_pole"])
d("D/n_cells", len(_c["cells"]))
d("D/n_arms", len(_c["arms"]))
for _key in sorted(_c["cells"], key=lambda k: (k[0], k[1])):
    _dd = _c["cells"][_key]
    _p = "D/cell/%d/%s" % (int(_key[0]), _key[1])
    d(_p + "/parent", PARENT_CODE[_dd["parent"]])
    d(_p + "/zeros/n", len(_dd["zeros"]))
    for _j, _z in enumerate(_dd["zeros"]):
        d("%s/zeros/%d" % (_p, _j), _z)
    f(_p + "/gap", _dd["gap"])
    f(_p + "/vgap", _dd["vgap"])
    f(_p + "/pole", _dd["pole"])
    d(_p + "/n", _dd["n"])
for _i, _a in enumerate(_c["arms"]):
    _p = "D/arm/%d" % _i
    b(_p + "/inc", _a["inc"])
    d(_p + "/n_riding", _a["n_riding"])
    d(_p + "/n_sampled", _a["n_sampled"])
    for _kk in ("switch", "regime", "parent"):
        d("%s/skipped/%s" % (_p, _kk), _a["skipped"][_kk])
    d(_p + "/n_cells", len(_a["cells"]))
    for _auth in sorted(_a["cells"]):
        _dd = _a["cells"][_auth]
        _q = "%s/%s" % (_p, _auth)
        d(_q + "/n", _dd["n"])
        d(_q + "/n_parent", _dd["n_parent"])
        f(_q + "/gap", _dd["gap"])
        f(_q + "/vgap", _dd["vgap"])
        f(_q + "/pole", _dd["pole"])
        opt_span(_q + "/det", _dd["det"])
        opt_span(_q + "/s", _dd["s"])
        d(_q + "/zeros/n", len(_dd["zeros"]))
        for _j, _z in enumerate(_dd["zeros"]):
            d("%s/zeros/%d" % (_q, _j), _z)

# =============================================================================================
# SECTION E -- `mask_discriminator` (ds 0.002, every 4, its OWN THREE-arm clock grid)
# =============================================================================================
_md = m.mask_discriminator(FLIGHT, LO, HI, TT4_MAX, SM)
for _k in ("max_pole_unmatched", "sum_pole_unmatched", "sum_pole_matched"):
    opt("E/%s" % _k, _md[_k])
f("E/sum_worst_re", _md["sum_worst_re"])
f("E/max_worst_re", _md["max_worst_re"])
d("E/n_arms", len(_md["arms"]))
for _i, _a in enumerate(_md["arms"]):
    _p = "E/arm/%d" % _i
    b(_p + "/matched", _a["matched"])
    d(_p + "/n", _a["n"])
    for _law in ("max", "sum"):
        _L = _a["laws"][_law]
        _q = "%s/%s" % (_p, _law)
        opt(_q + "/worst_pole", _L["worst_pole"])
        f(_q + "/worst_re", _L["worst_re"])
        d(_q + "/n_auth", len(_L["authority"]))
        for _j, _lab in enumerate(_L["authority"]):
            s("%s/authority/%d" % (_q, _j), _lab)
        d(_q + "/n_zerokeys", len(_L["zeros"]))
        for _auth in sorted(_L["zeros"]):
            d("%s/zeros/%s/n" % (_q, _auth), len(_L["zeros"][_auth]))
            for _j, _z in enumerate(_L["zeros"][_auth]):
                d("%s/zeros/%s/%d" % (_q, _auth, _j), _z)

# =============================================================================================
# SECTION F -- `shared_bill`, BOTH `inc` ARMS (ds 0.005, matched `taus`, 16 CELLS EACH)
# =============================================================================================
for _inc in (False, True):
    _bl = m.shared_bill(FLIGHT, LO, HI, TT4_MAX, SM, inc=_inc)
    _p = "F/%d" % int(_inc)
    for _k in ("fuel_marginal_phi", "fuel_marginal_Tt4", "Tt4_full", "Tt4_no_fuel",
               "phi_full", "phi_no_fuel"):
        f("%s/%s" % (_p, _k), _bl[_k])
    opt(_p + "/handover", _bl["handover"])
    for _k in ("phi", "Tt4", "inc"):
        opt("%s/delivered/%s" % (_p, _k), _bl["delivered"][_k])
    for _leg in ("F", "G", "V", "S"):
        f("%s/marginal/%s" % (_p, _leg), _bl["marginal"][_leg])
        f("%s/alone/%s" % (_p, _leg), _bl["alone"][_leg])
        opt("%s/kept/%s" % (_p, _leg), _bl["kept"][_leg])
    d(_p + "/n_cells", len(_bl["cells"]))
    for _key in sorted(_bl["cells"]):
        _dd = _bl["cells"][_key]
        _q = "%s/cell/%s" % (_p, _key)
        for _k in ("I", "E", "M", "min_phi", "max_Tt4"):
            f("%s/%s" % (_q, _k), _dd[_k])
        d(_q + "/n", _dd["n"])
        d(_q + "/auth_fuel", _dd["auth_fuel"])
        opt(_q + "/handover", _dd["handover"])
        for _k in ("credit_phi", "credit_Tt4", "credit_inc"):
            opt("%s/%s" % (_q, _k), _dd[_k])

# --------------------------------------------------------------------------------------------
# THE INTERCEPTORS COME OFF HERE. Everything below is a DECLARED EXTRA GRID built from what
# sections A-F drove; nothing below adds a call to the plant.
# --------------------------------------------------------------------------------------------
SharedActuatorTransient._shared_march = _orig_march
SharedActuatorTransient._quartic_roots_c = staticmethod(_orig_quart)
SharedActuatorTransient._authority = staticmethod(_orig_auth)

# =============================================================================================
# SECTION G -- THE SIX-STATE MARCH, PER POINT, over the DISTINCT intercepted signatures
# =============================================================================================
_sigs = {}
for _sig, _traj in MARCHES:
    if _sig in _sigs:
        # A repeated signature MUST give a repeated trajectory -- the march is deterministic and
        # a reader calling it twice is not a second grid. Asserted rather than assumed, and on the
        # POINTS rather than on the length, because a plant that carried state between calls would
        # give the same number of different points. Four of the ten signatures are driven more
        # than once (14 calls, 10 signatures), so this assertion is live rather than decorative.
        assert _sigs[_sig] == _traj, ("a repeated march signature gave a DIFFERENT trajectory -- "
                                      "the plant is carrying state between reader calls", _sig)
        continue
    _sigs[_sig] = _traj
d("G/n_calls", len(MARCHES))
d("G/n_sigs", len(_sigs))
d("G/stride", STRIDE)
for _i, _sig in enumerate(sorted(_sigs)):
    _traj = _sigs[_sig]
    _p = "G/sig/%d" % _i
    # the signature, as INPUTS the Rust replays on
    for _j, _t in enumerate(_sig[0]):
        f("%s/in/tau/%d" % (_p, _j), _t)
    f(_p + "/in/r", _sig[1])
    f(_p + "/in/s_settle", _sig[2])
    f(_p + "/in/ds", _sig[3])
    f(_p + "/in/v_max", _sig[4])
    b(_p + "/in/inc", _sig[5])
    d(_p + "/n_points", len(_traj))
    d(_p + "/n_emitted", len(range(0, len(_traj), STRIDE)))
    # THE AGGREGATES ARE OVER EVERY POINT, NOT THE STRIDED ONES -- the stride's backstop.
    for _fk in PT_FLOAT:
        _col = [p[_fk] for p in _traj]
        f("%s/agg/%s/min" % (_p, _fk), min(_col))
        f("%s/agg/%s/max" % (_p, _fk), max(_col))
        f("%s/agg/%s/last" % (_p, _fk), _col[-1])
    for _j in range(0, len(_traj), STRIDE):
        _pt, _q = _traj[_j], "%s/pt/%d" % (_p, _j)
        for _fk in PT_FLOAT:
            f("%s/%s" % (_q, _fk), _pt[_fk])
        for _fk in PT_STR:
            s("%s/%s" % (_q, _fk), _pt[_fk])
        for _fk in PT_OPT_STR:
            opt_s("%s/%s" % (_q, _fk), _pt[_fk])
        for _fk in PT_INT:
            d("%s/%s" % (_q, _fk), _pt[_fk])

# =============================================================================================
# SECTION H -- `_quartic_roots_c`, PER DISTINCT COEFFICIENT VECTOR
# =============================================================================================
def _bits(x):
    return struct.unpack("<Q", struct.pack("<d", float(x)))[0]


def _winner(coef):
    """WHICH term wins `scale = max(1.0, |a3|, |a2|**0.5, |a1|**(1/3.), |a0|**0.25)`, by Python's
    own `max` semantics -- FIRST of equal arguments, so ties go to the lower index."""
    _a3, _a2, _a1, _a0 = coef[1], coef[2], coef[3], coef[4]
    cand = [1.0, abs(_a3), abs(_a2) ** 0.5, abs(_a1) ** (1 / 3.), abs(_a0) ** 0.25]
    best = 0
    for _k in range(1, 5):
        if cand[_k] > cand[best]:
            best = _k
    return best


_seen_q, _distinct = set(), []
_near_calls = 0
for _coef, _roots in QUART:
    _sep = min(abs(_roots[_i] - _roots[_j]) for _i in range(4) for _j in range(_i + 1, 4))
    _near_calls += _sep < 1e-6
    _kb = tuple(_bits(x) for x in _coef)
    if _kb not in _seen_q:
        _seen_q.add(_kb)
        _distinct.append((_coef, _roots, _sep))
d("H/n_calls", len(QUART))
d("H/n_distinct", len(_distinct))
d("H/n_near_double_calls", _near_calls)
d("H/n_near_double_distinct", sum(1 for _, _, _sp in _distinct if _sp < 1e-6))
for _k in range(5):
    d("H/scale_winner/%d" % _k, sum(1 for _cf, _, _ in _distinct if _winner(_cf) == _k))
for _i, (_coef, _roots, _sep) in enumerate(_distinct):
    _p = "H/v/%d" % _i
    for _j, _x in enumerate(_coef):
        f("%s/in/coef/%d" % (_p, _j), _x)
    for _j, _z in enumerate(_roots):
        c("%s/root/%d" % (_p, _j), _z)
    f(_p + "/min_sep", _sep)
    b(_p + "/near_double", _sep < 1e-6)
    d(_p + "/scale_winner", _winner(_coef))
    d(_p + "/n_complex", sum(1 for _z in _roots if _z.imag != 0.0))

# =============================================================================================
# SECTION J -- `_authority`, ITS CENSUS AND ITS MARGIN
# =============================================================================================
d("J/n_calls", len(AUTH))
_pairs = {}
for _gf, _gr, _lab in AUTH:
    _pairs.setdefault((_bits(_gf), _bits(_gr)), (_gf, _gr, _lab))
d("J/n_distinct", len(_pairs))
for _lab in ("dormant", "tie", "fuel", "gov"):
    d("J/label/%s" % _lab, sum(1 for _, _, _l in AUTH if _l == _lab))
_gaps = [(abs(_gf - _gr), _gf, _gr, _lab) for _gf, _gr, _lab in _pairs.values()]
_zeros = [g for g in _gaps if g[0] == 0.0]
_tol = [g for g in _gaps if g[0] <= 1e-12]
d("J/n_zero", len(_zeros))
d("J/n_within_tol", len(_tol))
# THE KEY P4 LIVES OR DIES ON.
d("J/n_open", len(_tol) - len(_zeros))
_nonzero = sorted(g for g in _gaps if g[0] > 0.0)
# THE MARGIN -- a number, never the word "unreachable" (§ 5.28.3 (f)).
f("J/min_nonzero_gap", _nonzero[0][0])
f("J/min_nonzero_gap_over_tol", _nonzero[0][0] / 1e-12)
# THE REPLAYED PAIRS: every exact zero, plus the twenty smallest non-zero gaps -- where a
# tolerance change would first bite. The rule is stated so the Rust can re-derive the count.
_replay = sorted(_zeros) + _nonzero[:20]
d("J/n_replay", len(_replay))
d("J/n_replay_zero", len(_zeros))
for _i, (_gap, _gf, _gr, _lab) in enumerate(_replay):
    _p = "J/pair/%d" % _i
    f(_p + "/in/gf", _gf)
    f(_p + "/in/gr", _gr)
    f(_p + "/gap", _gap)
    s(_p + "/label", _lab)

# ---------------------------------------------------------------------------------- emit
print("# slice AD step 5 -- rung 72 ORACLE, the READERS' own grid (A-F), uncoarsened, plus THREE "
      "declared extra grids (G march, H quartic, J authority). key<TAB>u64 (floats are IEEE-754 "
      "bits).")
_seen = set()
for _key, _val in OUT:
    assert _key not in _seen, "duplicate key %s" % _key
    _seen.add(_key)
    print("%s\t%d" % (_key, _val))
print("# %d keys; %d march calls over %d distinct signatures; %d quartic calls over %d distinct "
      "vectors; %d authority calls over %d distinct pairs"
      % (len(OUT), len(MARCHES), len(_sigs), len(QUART), len(_distinct), len(AUTH), len(_pairs)),
      file=sys.stderr)
