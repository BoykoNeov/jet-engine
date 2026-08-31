"""SLICE AC step 6 -- THE ORACLE for rungs 70 + 71 (`CrossSplitTransient`, `FullSplitTransient`).

**THE GRID IS THE SUITES' OWN AND NOTHING IS COARSENED**, and the header states it rather than
implying it -- slice S step 4's lesson, *a probe's HEADER claimed the suites' grids and its code
ran another*. Every argument in sections A-M below is COPIED from the calling gate in
`tests/test_rung70.py` / `tests/test_rung71.py`, never chosen:

    flight      T0 250 K, p0 50 kPa, M0 0.85 -- both suites', identical
    LO/HI       1000 -> 1400 . r 0.5 . s_settle 1.2 . FLOOR 0.55 -- both suites' throughout
    ds          0.005 EXCEPT `split_modes` (0.002, the gate passes it), `full_gains` and
                `full_modes` (0.002, their OWN defaults, which the fixtures take wholesale)
    phi/b/v     PHI 0.80, B 0.10, V_MAX 0.20, SM = PHI/FLOOR - 1 -- both suites'
    clocks      TAU 0.05 (valve), TAU_S 0.05 (stator), TAU_GOV 0.05 (governor)
    Tt4_max     1200 K -- RUNG 67's imposed redline, verbatim
    every       10 (`split_gains`), 20 (`split_modes`), 2 (`full_gains`), 4 (`full_modes`)
    clock grid  `split_modes`' four and `full_modes`' six -- each reader's OWN default, which
                its gate takes wholesale
    floor grid  `split_floor`'s own nine
    sweeps      `window_law`'s five `tau_qs` x four `tau_ss`; `ic_contraction`'s six orders x
                four fractions -- each the reader's own default
    maps        `shaped` ONLY (LP a=.20 b=.05 l=.7 / HP a=.08 b=.15 l=1.0) -- neither suite
                builds a second shape, so a second one here would be a grid they do not have

**SECTION N IS A DECLARED EXTRA GRID AND SAYS SO IN ITS OWN HEADER.** Mixing it into A-M would be
exactly the defect the first paragraph guards against, so it is lettered apart.

# SECTION N IS THE STEP's REASON FOR EXISTING, AND IT EXISTS BECAUSE THE READERS CANNOT REACH IT

s 5.27 (iv) registered P6 as a **gated condition**: rung 70's `_zeta_pair` computes
`cmath.sqrt(nz[0]*nz[1])`, probe 10 measured `p` positive-real on **18 of 18** reader calls, and
the port shipped that as an `assert!`. **Step 5 falsified it from the shipped test suite** --
`test_rung71.py`'s damping gate drives the same function on a CONSTRUCTED spectrum where
`p = 4462 + 4947j`.

So re-reading P6 off the readers' own grid would re-publish the measurement that was already
wrong: a readers-only sample is not evidence about a function the SUITE drives elsewhere.
Section N therefore carries two things the readers cannot:

  * **N/const** -- the three constructed spectra of `test_rung71.py:549-561`, verbatim, through
    BOTH damping readers. This is the arm where `p` is genuinely complex, so `cmath.sqrt`'s
    complex branch and CPython's Smith-algorithm complex division are covered by VALUE KEYS and
    not only by an `assert!`.
  * **N/pair, N/ring** -- every `_zeta_pair` and `_zeta_ring` call sections A-M make, INTERCEPTED
    at the function boundary and never reconstructed (slice Z's leading finding). The roots go
    into the golden as INPUTS; the Rust replays the shipped reader on them with the plant taken
    out of the loop, exactly as slice AB's section I replays the cubic solver.

`N/pair/ncalls` and `N/ring/ncalls` are checkable on the Rust side rather than decorative:
`split_modes` and `split_floor` are the only `_zeta_pair` callers and `full_modes` the only
`_zeta_ring` caller, so each count must equal the rows sections C, E and K emit between them.

# EVERY SAMPLE-SHAPED READER EMITS ITS SAMPLE'S SIZE, BECAUSE THAT IS THE MEASURED BREAK SHAPE

s 5.27 (ii): with rung 68's `_triple_laws` in rung 70's slot the governor is simply absent, every
sampled point comes back `interior = False`, and `split_gains` **returns successfully with an
EMPTY table** -- `len(rows)` 2 -> 0, every aggregate `None`, no gain differing because there are
no gains. A dump that emits only values would be blind to that by construction while being
advertised as the value-side backstop. So every reader here emits its row count, its skipped
count and a PRESENCE FLAG beside every `Option`, and `opt()` is used wherever Python can return
`None` -- which it does for real (`_zeta_ring` returns `None` on a fully real spectrum).

# ARM ORDER IS THE GRID's OWN ORDER, AND THE KEYS SAY SO

Step 4 measured that reordering a clock grid shifts 25 of 38 printed lines and is caught by
NEITHER language's gates at rung 70. An oracle only sees it if the keys are bound to the grid's
own index: `C/arm/0/...`, `C/arm/1/...`. Nothing here is sorted, deduplicated or aggregated over
arms without ALSO being emitted per arm.

# THE CROSS-INTERPRETER EXEMPTION IS A SET OF **NAMES**, MEASURED FROM THIS DUMP

s 5.27 (iv) measured two `sum()` sites diverging under CPython 3.14: `_invariants`' `c1`
(inherited from rung 69, 5 of 37 instances) and `cross_identity`'s 13-element sum, which belongs
to rung 67 and is pulled in because `rung67_control` calls it as the built-in negative control.
P8 names those two subtrees. The exempt set in `tests/slice_ac_oracle.rs` is nevertheless READ
OFF THE DIFF rather than predicted: [[rust-port-slice-z-step4]] is a pre-registered exemption of
TWO keys that measured EIGHT, because it counted quantities where a dump emits names. **The port
is held to PyPy**, where nothing is exempt.

# WHAT IS DELIBERATELY NOT EMITTED, so the Rust's missing-key half stays honest

  * `split_bill`'s `phi_lim_source` and `full_bill`'s `own_currency` -- tables of CONSTANT
    strings that cannot differ between two runs. The port drops them for the same reason and
    says so in its own doc comments; recorded here as a DECISION, not an omission.
  * `_triple_gains_at`'s `s` on an INTERIOR return. Python's interior dict does not carry one
    (only the off-regime early return does), so a key for it would assert against a field that
    does not exist. Every row emits its OWN `s` from the trajectory point, which is what the
    callers read; the off-regime arms emit theirs.
  * `band_containment`'s per-point `rows`, which Python builds as a LOCAL and does not return.
    Its four counts and three aggregates are returned and are emitted.

Every float is emitted as its IEEE-754 bit pattern. Regenerate BOTH arms:

    .venv/Scripts/python.exe rust/oracle/dump_slice_ac.py > rust/oracle/slice_ac_pypy.tsv
    C:/Python314/python.exe  rust/oracle/dump_slice_ac.py > rust/oracle/slice_ac_cpython.tsv

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
    CrossSplitTransient, FullSplitTransient,
    BleedLimiter, StatorLimiter, StatorIncidenceLimiter,
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
    sentinel float would conflate a missing value with a real one. Both damping readers return
    `None` for real, and every `max(..., default=None)` aggregate here does so on an EMPTY
    sample, which is s 5.27 (ii)'s measured break shape."""
    b(key + "?", x is not None)
    if x is not None:
        f(key, x)


def opt_d(key, n):
    b(key + "?", n is not None)
    if n is not None:
        d(key, n)


def opt_b(key, flag):
    b(key + "?", flag is not None)
    if flag is not None:
        b(key, flag)


def s(key, text):
    """A STRING key, as an FNV-1a 64-bit hash -- the off-regime arm names, `silenced`, the `ic`
    order labels and the two ledgers' cell names are the non-floats a rung-70/71 reading
    carries."""
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


def c(key, z):
    """A COMPLEX root -- re, im and `abs`. **`abs` is a KEY and not a convenience**: it is
    `hypot`, and `sorted(..., key=abs)` selects WHICH root is which in both damping readers, in
    `n_zero`, `worst_zero`, `min_root` and `max_root`."""
    f(key + "/re", z.real)
    f(key + "/im", z.imag)
    f(key + "/abs", abs(z))


def off(key, names):
    d(key + "/n_off", len(names))
    for i, n in enumerate(names):
        s("%s/off/%d" % (key, i), n)


def gains(key, gg):
    """One `_triple_gains_at` return. The six central differences, the base setting, the cyclic
    product and the three pairwise products -- and the REGIME, which no float can witness."""
    b(key + "/interior", gg["interior"])
    off(key, gg["off_regime"])
    if not gg["interior"]:
        f(key + "/s", gg["s"])
        f(key + "/v_base", gg["v_base"])
        return
    for k in ("R_q", "R_v", "C_g", "C_v", "V_g", "V_q", "v_base",
              "cyclic", "pair_RC", "pair_RV", "pair_CV"):
        f("%s/%s" % (key, k), gg[k])


def span(key, sp):
    """A `(lo, hi, n)` window. **THE COUNT IS A KEY IN ITS OWN RIGHT** -- step 4 measured a
    widened joint predicate taking rung 70's window 61 -> 341 points and `joint_fraction`
    0.179 -> exactly 1.0 while every ported gate stayed green, because their bars were one-sided
    lower bounds. Emitting the count and the fraction discharges that booking."""
    opt(key + "/lo", sp[0])
    opt(key + "/hi", sp[1])
    d(key + "/n", sp[2])


def boundary(key, rows):
    """`_assert_state_boundary`'s returns. The LIVE cross-gains are both non-zero and the DEAD
    control is identically zero -- rung 67's gate, and the instrument that says the
    `_b_state`/`_v_state` boundary around `required` is still there."""
    d(key + "/n", len(rows))
    for i, x in enumerate(rows):
        f("%s/%d/s" % (key, i), x["s"])
        for arm in ("live", "dead"):
            for g in ("R_q", "R_v"):
                f("%s/%d/%s/%s" % (key, i, arm, g), x[arm][g])


def skipped(key, rows):
    """The DISCLOSED off-regime points, never a silent truncation."""
    d(key + "/n", len(rows))
    for i, x in enumerate(rows):
        f("%s/%d/s" % (key, i), x["s"])
        off("%s/%d" % (key, i), x["off_regime"])


def flist(key, xs):
    d(key + "/n", len(xs))
    for i, x in enumerate(xs):
        f("%s/%d" % (key, i), x)


# ---------------------------------------------------------------------------- the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU, TAU_S, TAU_GOV = 0.05, 0.05, 0.05
TT4_MAX = 1200.0                 # RUNG 67's imposed redline, VERBATIM

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """Both suites' `_cpg`, character for character. `R_c` is DERIVED from `(gamma - 1)/gamma`;
    re-spelling it `0.4/1.4` builds a gas ONE ULP away, which presents exactly as a port defect
    (slice Y's own false alarm)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def valve(tau=TAU):
    return BleedLimiter.from_margin(LP, B, SM, tau=tau)


def phi_stator(tau=TAU_S, v_max=V_MAX):
    return StatorLimiter.from_margin(LP, v_max, SM, tau=tau)


def inc(tau=TAU_S, v_max=V_MAX):
    return StatorIncidenceLimiter.from_margin(LP, v_max, SM, tau=tau)


# THE two machines, built exactly as the suites' module fixtures build them.
CROSS = CrossSplitTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                            bleed_lim=valve(), stator_lim=phi_stator())
FULL = FullSplitTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                          bleed_lim=valve(), stator_inc=inc())
KW = dict(r=R, s_settle=SETTLE, tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX)


# ------------------------------------------------- THE INTERCEPT (section N's second instrument)
#
# Re-installed as `staticmethod`s, because both damping readers ARE static and are called as
# `self._zeta_pair(...)`: a plain function assigned onto the class would bind through the instance
# and hand the body a second argument. Pure pass-through -- it records the roots it was handed and
# the value the SHIPPED body returned, so no number below moves because the recorder is on.
#
# `_zeta_pair` is patched on `CrossSplitTransient`, which is where it is DEFINED, so rung 71's
# machine routes through the same recorder by inheritance.
ZP_CALLS = []
ZR_CALLS = []
_ZP = CrossSplitTransient._zeta_pair
_ZR = FullSplitTransient._zeta_ring


def _zp(roots):
    got = _ZP(roots)
    ZP_CALLS.append((list(roots), got))
    return got


def _zr(roots):
    got = _ZR(roots)
    ZR_CALLS.append((list(roots), got))
    return got


CrossSplitTransient._zeta_pair = staticmethod(_zp)
FullSplitTransient._zeta_ring = staticmethod(_zr)


# ============================================================================ A -- split_gains
# `tests/test_rung70.py:114`, the module fixture, verbatim.
A = CROSS.split_gains(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                      tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX, every=10)
d("A/n_riding", A["n_riding"])
d("A/n_sampled", A["n_sampled"])
d("A/n_rows", len(A["rows"]))
b("A/s_window?", A["s_window"] is not None)
if A["s_window"] is not None:
    f("A/s_window/lo", A["s_window"][0])
    f("A/s_window/hi", A["s_window"][1])
for i, x in enumerate(A["rows"]):
    f("A/row/%d/s" % i, x["s"])
    gains("A/row/%d/gov" % i, x["gov"])
    gains("A/row/%d/fuel" % i, x["fuel"])
    f("A/row/%d/pair_gap" % i, x["pair_gap"])
    f("A/row/%d/cyclic_is_RC" % i, x["cyclic_is_RC"])
skipped("A/skipped", A["skipped"])
boundary("A/boundary", A["boundary"])
for _k in ("worst_CV", "worst_RC_is_1", "worst_RV_is_1", "min_pair_gap", "max_pair_gap",
           "worst_cyclic_is_RC", "worst_RC_fuel", "worse_pair"):
    opt("A/" + _k, A[_k])
flist("A/pair_RC", A["pair_RC"])
flist("A/pair_RV", A["pair_RV"])

# ============================================================================ B -- rung67_control
# `tests/test_rung70.py:298`, verbatim -- `every` comes from the signature, as it does there.
Bc = CROSS.rung67_control(FLIGHT, LO, HI, TT4_MAX, SM, tau=TAU, tau_gov=TAU_GOV,
                          tau_s=TAU_S, v_max=V_MAX, r=R, s_settle=SETTLE, ds=DS)
d("B/n", Bc["n"])
opt("B/P70_lo", Bc["P70_lo"])
opt("B/P70_hi", Bc["P70_hi"])
f("B/P67_lo", Bc["P67_lo"])
f("B/P67_hi", Bc["P67_hi"])
opt_b("B/both_negative", Bc["both_negative"])
opt("B/ratio", Bc["ratio"])

# ============================================================================ C -- split_modes
# `tests/test_rung70.py:313/329`, verbatim -- `ds = 0.002` and `every = 20` are the GATE's, and
# `clocks` is the reader's own four-arm default which the gate takes wholesale.
C = CROSS.split_modes(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=0.002,
                      v_max=V_MAX, every=20)
f("C/ds", C["ds"])
d("C/n_clocks", len(C["clocks"]))
d("C/n_arms", len(C["arms"]))
for i, _cl in enumerate(C["clocks"]):
    for j, _t in enumerate(_cl):
        f("C/clock/%d/%d" % (i, j), _t)
for i, arm in enumerate(C["arms"]):
    k = "C/arm/%d" % i
    for j, _t in enumerate(arm["taus"]):
        f("%s/taus/%d" % (k, j), _t)
    f(k + "/rate_sum", arm["rate_sum"])
    d(k + "/n", arm["n"])
    d(k + "/n_sampled", arm["n_sampled"])
    d(k + "/skipped", arm["skipped"])
    d(k + "/n_rows", len(arm["rows"]))
    d(k + "/n_zeros", len(arm["zeros"]))
    for j, _z in enumerate(arm["zeros"]):
        d("%s/zeros/%d" % (k, j), _z)
    for _n in ("max_c0_rel", "min_c1_rel", "max_c1_err"):
        opt("%s/%s" % (k, _n), arm[_n])
    opt_b(k + "/any_complex", arm["any_complex"])
    opt(k + "/zeta_lo", arm["zeta_range"][0])
    opt(k + "/zeta_hi", arm["zeta_range"][1])
    for j, x in enumerate(arm["rows"]):
        rk = "%s/row/%d" % (k, j)
        for _n in ("s", "c2", "c1", "c0", "c1_pred", "pair_RC", "pair_RV", "pair_CV",
                   "cyclic", "worst_zero", "c1_rel", "c0_rel"):
            f("%s/%s" % (rk, _n), x[_n])
        opt(rk + "/c1_err", x["c1_err"])
        opt(rk + "/zeta", x["zeta"])
        b(rk + "/complex_pair", x["complex_pair"])
        d(rk + "/n_zero", x["n_zero"])
        for t, _z in enumerate(x["roots"]):
            c("%s/root/%d" % (rk, t), _z)

# ============================================================================ D -- c1_clock_swap
# `tests/test_rung70.py:352` (the reader's own fast/slow default) and `:381` (the CONTROL, all
# three clocks equal), both verbatim.
for gi, _kw in enumerate((dict(r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX),
                          dict(tau_g=0.05, fast=0.05, slow=0.05, r=R, s_settle=SETTLE,
                               ds=DS, v_max=V_MAX))):
    D = CROSS.c1_clock_swap(FLIGHT, LO, HI, TT4_MAX, SM, **_kw)
    k = "D/%d" % gi
    for _n in ("fast_valve", "fast_stator"):
        ak = "%s/%s" % (k, _n)
        arm = D["arms"][_n]
        for j, _t in enumerate(arm["taus"]):
            f("%s/taus/%d" % (ak, j), _t)
        f(ak + "/s", arm["s"])
        f(ak + "/c1_marched", arm["c1_marched"])
        f(ak + "/pair_RC", arm["pair_RC"])
        f(ak + "/pair_RV", arm["pair_RV"])
        gains(ak + "/gains", arm["gains"])
    for _n in ("held_gains", "one_scalar_null"):
        for _g in ("c1_fast_valve", "c1_fast_stator", "ratio"):
            f("%s/%s/%s" % (k, _n, _g), D[_n][_g])
    for _n in ("k_null", "marched_ratio", "predicted_delta", "measured_delta", "null_delta"):
        f("%s/%s" % (k, _n), D[_n])

# ============================================================================ E -- split_floor
# `tests/test_rung70.py:410/437/472/484`, verbatim -- `grid` is the reader's own nine.
E = CROSS.split_floor(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS, v_max=V_MAX)
d("E/n_rows", len(E["rows"]))
d("E/n_live", sum(1 for x in E["rows"] if "zeta" in x))
b("E/holds", E["holds"])
b("E/strict", E["strict"])
b("E/any_complex", E["any_complex"])
opt("E/floor_lo", E["floor_range"][0])
opt("E/floor_hi", E["floor_range"][1])
opt("E/worst_pred_err", E["worst_pred_err"])
f("E/max_ds_lambda", E["max_ds_lambda"])
opt("E/max_mod_ratio", E["max_mod_ratio"])
for i, x in enumerate(E["rows"]):
    k = "E/row/%d" % i
    for j, _t in enumerate(x["taus"]):
        f("%s/taus/%d" % (k, j), _t)
    d(k + "/n", x["n"])
    off(k, x.get("off_regime", []))
    b(k + "/live", "zeta" in x)
    if "zeta" not in x:
        continue
    for _n in ("s", "pair_RC", "pair_RV", "u", "w", "quiet_share", "a_over_loud",
               "det2", "zeta_pred", "floor", "mod", "mod_pred", "rate_sum"):
        f("%s/%s" % (k, _n), x[_n])
    s(k + "/silenced", x["silenced"])
    opt(k + "/zeta", x["zeta"])
    b(k + "/complex_pair", x["complex_pair"])
b("E/tightest?", E["tightest"] is not None)
if E["tightest"] is not None:
    for _n in ("s", "zeta_pred", "floor", "mod", "rate_sum"):
        f("E/tightest/" + _n, E["tightest"][_n])
    opt("E/tightest/zeta", E["tightest"]["zeta"])
    s("E/tightest/silenced", E["tightest"]["silenced"])

# ============================================================================ F -- window_overlap
# `tests/test_rung70.py:213`, verbatim.
F = CROSS.window_overlap(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                         tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX)
for _n in ("gov", "valve", "stator", "joint"):
    span("F/" + _n, F[_n])
d("F/n", F["n"])
b("F/overlaps", F["overlaps"])
f("F/joint_fraction", F["joint_fraction"])

# ============================================================================ G -- split_bill
# `tests/test_rung70.py:496`, the module fixture, verbatim.
CELLS8 = ("bare", "G", "V", "S", "GV", "GS", "VS", "GVS")
G = CROSS.split_bill(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, ds=DS,
                     tau=TAU, tau_gov=TAU_GOV, tau_s=TAU_S, v_max=V_MAX)
f("G/Tt4_max", G["Tt4_max"])
d("G/n_cells", len(G["cells"]))
for i, _n in enumerate(CELLS8):
    cell = G["cells"][_n]
    k = "G/cell/%d" % i
    s(k + "/name", _n)
    for _g in ("I", "E", "min_phi", "max_Tt4"):
        f("%s/%s" % (k, _g), cell[_g])
    d(k + "/n", cell["n"])
    opt(k + "/credit_phi", cell["credit_phi"])
    opt(k + "/credit_Tt4", cell["credit_Tt4"])
for _n in ("marginal_phi", "marginal_Tt4"):
    for _leg in ("gov", "valve", "stator"):
        f("G/%s/%s" % (_n, _leg), G[_n][_leg])
opt("G/delivered_phi", G["delivered_phi"])
opt("G/delivered_Tt4", G["delivered_Tt4"])

# ============================================================================ H -- window_law
# `tests/test_rung71.py:293` (the reader's own five x four sweep) and `:319` (the single-point
# sweep the second gate passes), both verbatim.


def arm_keys(k, a):
    for j, _t in enumerate(a["taus"]):
        f("%s/taus/%d" % (k, j), _t)
    d(k + "/n", a["n"])
    f(k + "/phi_lim", a["phi_lim"])
    opt(k + "/phi_at_stator_off", a["phi_at_stator_off"])
    opt(k + "/v_at_stator_off", a["v_at_stator_off"])
    for _n in ("gov", "valve", "stator", "joint"):
        span("%s/%s" % (k, _n), a[_n])
    d(k + "/n_interior", a["n_interior"])
    f(k + "/v_hi", a["v_hi"])
    f(k + "/min_phi", a["min_phi"])
    opt(k + "/stator_off", a["stator_off"])
    opt(k + "/phi_recovers_marched", a["phi_recovers_marched"])


for gi, _extra in enumerate((dict(), dict(tau_qs=(TAU,), tau_ss=(TAU_S,)))):
    H = FULL.window_law(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **dict(KW, **_extra))
    k = "H/%d" % gi
    arm_keys(k + "/base", H["base"])
    d(k + "/n_tau_qs", len(H["tau_qs"]))
    d(k + "/n_tau_ss", len(H["tau_ss"]))
    for j, _t in enumerate(H["tau_qs"]):
        f("%s/tau_q/%d" % (k, j), _t)
    for j, _t in enumerate(H["tau_ss"]):
        f("%s/tau_s/%d" % (k, j), _t)
    for j, arm in enumerate(H["by_tau_q"]):
        arm_keys("%s/by_q/%d" % (k, j), arm)
    for j, arm in enumerate(H["by_tau_s"]):
        arm_keys("%s/by_s/%d" % (k, j), arm)
    for _n, _xs in (("edge_q", H["edge_q"]), ("edge_s", H["edge_s"])):
        d("%s/%s/n" % (k, _n), len(_xs))
        for j, x in enumerate(_xs):
            opt("%s/%s/%d" % (k, _n, j), x)
    b(k + "/q_monotone", H["q_monotone"])
    opt(k + "/q_span", H["q_span"])
    opt(k + "/s_span", H["s_span"])
    f(k + "/joint_fraction", H["joint_fraction"])
    opt(k + "/phi_short_at_off", H["phi_short_at_off"])
    opt(k + "/v_at_off", H["v_at_off"])

# ======================================================================= I -- band_containment
# `tests/test_rung71.py:275`, verbatim.
Ib = FULL.band_containment(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **KW)
d("I/n", Ib["n"])
d("I/n_delivering", Ib["n_delivering"])
d("I/riding_while_delivering", Ib["riding_while_delivering"])
d("I/n_riding", Ib["n_riding"])
opt("I/min_slack_delivering", Ib["min_slack_delivering"])
opt("I/worst_slack_minus_v", Ib["worst_slack_minus_v"])
f("I/min_slack_all", Ib["min_slack_all"])

# ============================================================================ J -- full_gains
# `tests/test_rung71.py:123`, the module fixture -- `ds = 0.002` and `every = 2` come from the
# reader's own signature, which is what the fixture takes.
J = FULL.full_gains(FLIGHT, LO, HI, TT4_MAX, SM, **KW)
f("J/ds", J["ds"])
d("J/n_riding", J["n_riding"])
d("J/n_sampled", J["n_sampled"])
d("J/n_rows", len(J["rows"]))
b("J/s_window?", J["s_window"] is not None)
if J["s_window"] is not None:
    f("J/s_window/lo", J["s_window"][0])
    f("J/s_window/hi", J["s_window"][1])
for i, x in enumerate(J["rows"]):
    k = "J/row/%d" % i
    f(k + "/s", x["s"])
    gains(k + "/gains", x["gains"])
    gains(k + "/phi_rig", x["phi_rig"])
    for _n in ("x", "y", "det", "det_pred", "y_is_RV", "x_is_product", "det_err"):
        f("%s/%s" % (k, _n), x[_n])
    opt(k + "/cross_rung", x["cross_rung"])
skipped("J/skipped", J["skipped"])
boundary("J/boundary", J["boundary"])
for _n in ("closest_to_1", "worst_y_is_RV", "worst_x_is_product", "worst_det_err",
           "det_scale", "worst_cross_rung"):
    opt("J/" + _n, J[_n])
flist("J/pair_RC", J["pair_RC"])
flist("J/pair_RV", J["pair_RV"])
flist("J/pair_CV", J["pair_CV"])

# ============================================================================ K -- full_modes
# `tests/test_rung71.py:128`, the module fixture -- `clocks`, `ds = 0.002` and `every = 4` are
# the reader's own six-arm defaults, which the fixture takes wholesale.
K = FULL.full_modes(FLIGHT, LO, HI, TT4_MAX, SM, r=R, s_settle=SETTLE, v_max=V_MAX)
f("K/ds", K["ds"])
d("K/n_clocks", len(K["clocks"]))
d("K/n_arms", len(K["arms"]))
for i, _cl in enumerate(K["clocks"]):
    for j, _t in enumerate(_cl):
        f("K/clock/%d/%d" % (i, j), _t)
d("K/n_zeros_everywhere", len(K["zeros_everywhere"]))
for j, _z in enumerate(K["zeros_everywhere"]):
    d("K/zeros_everywhere/%d" % j, _z)
d("K/arms_with_ring", K["arms_with_ring"])
d("K/arms_real", K["arms_real"])
d("K/arms_below_r69", K["arms_below_r69"])
opt("K/max_c0_err", K["max_c0_err"])
opt("K/min_routh", K["min_routh"])
opt("K/max_mod_ratio", K["max_mod_ratio"])
b("K/all_stable", K["all_stable"])
for i, arm in enumerate(K["arms"]):
    k = "K/arm/%d" % i
    for j, _t in enumerate(arm["taus"]):
        f("%s/taus/%d" % (k, j), _t)
    f(k + "/rate_sum", arm["rate_sum"])
    d(k + "/n", arm["n"])
    d(k + "/n_sampled", arm["n_sampled"])
    d(k + "/skipped", arm["skipped"])
    d(k + "/n_rows", len(arm["rows"]))
    d(k + "/n_zeros", len(arm["zeros"]))
    for j, _z in enumerate(arm["zeros"]):
        d("%s/zeros/%d" % (k, j), _z)
    for _n in ("min_root_rel", "max_c0_err", "min_routh", "max_mod_ratio"):
        opt("%s/%s" % (k, _n), arm[_n])
    for _n in ("all_stable", "any_complex", "any_below_r69"):
        opt_b("%s/%s" % (k, _n), arm[_n])
    opt(k + "/zeta_lo", arm["zeta_range"][0])
    opt(k + "/zeta_hi", arm["zeta_range"][1])
    for j, x in enumerate(arm["rows"]):
        rk = "%s/row/%d" % (k, j)
        for _n in ("s", "c2", "c1", "c0", "c0_pred", "u", "w", "z", "routh",
                   "pair_RC", "pair_RV", "pair_CV", "min_root", "max_root",
                   "ds_lambda", "mod_ratio"):
            f("%s/%s" % (rk, _n), x[_n])
        opt(rk + "/c0_err", x["c0_err"])
        opt(rk + "/zeta", x["zeta"])
        opt(rk + "/r69_floor", x["r69_floor"])
        b(rk + "/below_r69", x["below_r69"])
        b(rk + "/complex_pair", x["complex_pair"])
        b(rk + "/stable", x["stable"])
        d(rk + "/n_zero", x["n_zero"])
        for t, _z in enumerate(x["roots"]):
            c("%s/root/%d" % (rk, t), _z)

# ========================================================================= L -- ic_contraction
# `tests/test_rung71.py:581`, verbatim -- `orders` and `fracs` are the reader's own.
L = FULL.ic_contraction(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **KW)
for _n in ("full", "shared"):
    rig = L[_n]
    k = "L/" + _n
    d(k + "/n", rig["n"])
    d(k + "/n_rows", len(rig["rows"]))
    d(k + "/n_converged", rig["n_converged"])
    d(k + "/members", rig["members"])
    b(k + "/spread?", rig["spread"] is not None)
    if rig["spread"] is not None:
        for _g in ("g", "q", "v"):
            f("%s/spread/%s" % (k, _g), rig["spread"][_g])
    for j, _g in enumerate(("g", "b", "v")):
        f("%s/marched/%d" % (k, j), rig["marched"][j])
    opt_d(k + "/max_iters", rig["max_iters"])
    for j, x in enumerate(rig["rows"]):
        rk = "%s/row/%d" % (k, j)
        s(rk + "/order", x["order"])
        for t, _st in enumerate(x["start"]):
            f("%s/start/%d" % (rk, t), _st)
        for _g in ("band", "g", "q", "v", "res"):
            f("%s/%s" % (rk, _g), x[_g])
        d(rk + "/iters", x["iters"])

# ============================================================================ M -- full_bill
# `tests/test_rung71.py:133`, the module fixture, verbatim.
Mb = FULL.full_bill(FLIGHT, LO, HI, TT4_MAX, SM, ds=DS, **KW)
f("M/Tt4_max", Mb["Tt4_max"])
d("M/n_cells", len(Mb["cells"]))
for i, _n in enumerate(CELLS8):
    cell = Mb["cells"][_n]
    k = "M/cell/%d" % i
    s(k + "/name", _n)
    for _g in ("I", "E", "M", "min_phi", "max_Tt4", "v_hi"):
        f("%s/%s" % (k, _g), cell[_g])
    d(k + "/n", cell["n"])
    for _g in ("credit_phi", "credit_Tt4", "credit_inc"):
        opt("%s/%s" % (k, _g), cell[_g])
for i, _n in enumerate(CELLS8[1:]):
    k = "M/degrades/%d" % i
    s(k + "/cell", _n)
    got = Mb["degrades"][_n]
    d(k + "/n", len(got))
    for j, _g in enumerate(got):
        s("%s/%d" % (k, j), _g)
opt("M/inc_credit_valve_alone", Mb["inc_credit_valve_alone"])
opt("M/inc_credit_stator_alone", Mb["inc_credit_stator_alone"])
for _n in ("marginal", "alone", "marginal_phi", "marginal_Tt4", "marginal_inc"):
    for _leg in ("gov", "valve", "stator"):
        f("M/%s/%s" % (_n, _leg), Mb[_n][_leg])
for _leg in ("gov", "valve", "stator"):
    opt("M/kept/" + _leg, Mb["kept"][_leg])
for _g in ("phi", "Tt4", "inc"):
    opt("M/delivered/" + _g, Mb["delivered"][_g])

# ================================================================ N -- A DECLARED EXTRA GRID
#
# **N/const IS NOT THE READERS' GRID AND DOES NOT PRETEND TO BE.** It is `test_rung71.py`'s own
# damping gate (`:549-561`), whose three spectra are CONSTRUCTED so that the reader is driven
# where no plant on this ladder takes it. The middle one is the case s 5.27 (iv)'s P6 declared
# unreachable and step 5 found in the shipped suite: `nz[0]*nz[1] = 4462 + 4947j`, so
# `cmath.sqrt` takes its COMPLEX branch and `-s / (2*rt)` is a genuine complex division --
# CPython's Smith algorithm, the one operation probe 10 priced at 13 of 18 against a schoolbook
# spelling. The third is a fully REAL spectrum, where `_zeta_ring` returns `None`.
#
# The two readers are called through the SAVED originals, not the recorders, so these three do
# not enter the intercepted stream below and the counts stay reconcilable against A-M's rows.
CONST = (
    ("ok", [complex(-18.0, 0.0), complex(-21.0, 28.0), complex(-21.0, -28.0)]),
    ("bad", [complex(-194.0, 0.0), complex(-23.0, 25.5), complex(-23.0, -25.5)]),
    ("real", [complex(-20.0, 0.0), complex(-82.0, 0.0), complex(-138.0, 0.0)]),
)
d("N/const/n", len(CONST))
for i, (_n, _roots) in enumerate(CONST):
    k = "N/const/%d" % i
    s(k + "/name", _n)
    for t, _z in enumerate(_roots):
        c("%s/root/%d" % (k, t), _z)
    # The two readers on the SAME spectrum -- their difference is the gate's whole subject.
    opt(k + "/pair", _ZP(_roots))
    opt(k + "/ring", _ZR(_roots))
    # `p` AND `s` THEMSELVES, emitted so the complex branch is a KEY and not an inference. `p` is
    # the quantity the port asserts on, and `bad` is the arm where P6's `p.im == 0` is FALSE.
    _nz = sorted(_roots, key=abs)[1:]
    c(k + "/p", _nz[0] * _nz[1])
    c(k + "/s_sum", _nz[0] + _nz[1])

# N/pair, N/ring -- the INTERCEPTED call stream of sections A-M. The roots are INPUTS on the Rust
# side (never re-asserted against themselves) and the returned value is the assertion.
d("N/pair/ncalls", len(ZP_CALLS))
for i, (_roots, _got) in enumerate(ZP_CALLS):
    k = "N/pair/%d" % i
    for t, _z in enumerate(_roots):
        c("%s/in/%d" % (k, t), _z)
    opt(k + "/out", _got)
d("N/ring/ncalls", len(ZR_CALLS))
for i, (_roots, _got) in enumerate(ZR_CALLS):
    k = "N/ring/%d" % i
    for t, _z in enumerate(_roots):
        c("%s/in/%d" % (k, t), _z)
    opt(k + "/out", _got)

# ---------------------------------------------------------------------------- emit
print("# slice AC step 6 -- rungs 70+71 ORACLE, the SUITES' grid (A-M), uncoarsened, plus ONE "
      "declared extra grid (N). key<TAB>u64 (floats are IEEE-754 bits).")
_seen = set()
for _key, _val in OUT:
    assert _key not in _seen, f"duplicate key {_key}"
    _seen.add(_key)
    print(f"{_key}\t{_val}")
print(f"# {len(OUT)} keys, {len(ZP_CALLS)} intercepted _zeta_pair calls, "
      f"{len(ZR_CALLS)} intercepted _zeta_ring calls", file=sys.stderr)

# COVERAGE DOCUMENTATION, NOT A KEY -- how many of the intercepted `_zeta_pair` calls carry a
# genuinely COMPLEX `p`, which is the branch P6 was written about, and how many carry `p.re < 0`,
# which is where CPython's `copysign(s, z.imag)` would flip a non-zero component. A counted copy
# of the body can only gate the copy, so this is printed to stderr and nothing reads it; what it
# buys is the right to SAY whether sections A-M reach the branch, rather than assuming either way.
_cx = _neg = 0
for _roots, _ in ZP_CALLS:
    _nz = sorted(_roots, key=abs)[1:]
    _p = _nz[0] * _nz[1]
    _cx += _p.imag != 0.0
    _neg += _p.real < 0.0
print(f"# COVERAGE: {_cx} of {len(ZP_CALLS)} intercepted `p` are complex, {_neg} have p.re < 0 "
      f"-- section N/const carries the complex branch the readers do not reach",
      file=sys.stderr)
