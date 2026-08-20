"""SLICE U step 5 — THE ORACLE for rungs 49 + 50 + 51 + 52's readers, over the FOUR suites' grids.

Steps 1-4 ported 63 gates and then measured what those gates can and cannot see. This dump exists
to hold exactly what they cannot. The list is DERIVED from the four injection batteries, not
guessed:

  * **`deficit_at_release`** — the rung's OWN named quantity. Gate 8 of `test_rung50.py` is
    entirely about "the debit is monotone in the deficit at release" and reads `fuel_removed` as
    a proxy; the key itself has **no reader in either language**. Both its VALUE and its
    `eng[-1]`-versus-`eng[0]` choice pass all 15 gates when broken (step 2, injections C and J).
  * **`relief_watched` / `relief_other` out of `release_relief`** — swapping them moves 80 keys
    and no rung-50 gate reads either (step 2, injection D).
  * **`nu_hp_end_bare`** read off the LIMITED march — invisible, while its twin `nu_hp_end` off
    the BARE march IS caught, by contract 1b alone. The only difference between them is that
    Python's ten-key list in that contract names one and not the other (step 2, G vs H).
  * **`fuel_removed`'s SCALE.** Contract 1b holds it as a DIFFERENCE between two copies; break
    all THREE identical trapezoids in the module and rungs 48, 49 and 50 are all green (step 2,
    B and B'). Slice T step 3's finding, one rung on and sharpened.
  * **the march coordinate's SPELLING on the knife-edge cells.** `k * ds` moves `n_engaged` from
    8 to 7 and `s_rel` by a WHOLE GRID CELL on the `s_off = 0.26` rows and all 15 gates pass
    (step 2, injection A). Held here as the `s_rel` / `n_engaged` BITS of those exact cells.
  * **`rate_sweep` with a LIVE `tau_rel`.** Of the four `rate_sweep` rows the whole rung-51 suite
    produces, exactly two carry a non-`None` `tau_rel` and both are contract 4's, whose claim is
    that `tau_rel` is INERT there — so dropping the forwarding entirely moves 2 of 972 keys, both
    of them the record echoing its own argument back (step 3, injection I1). Section F adds a
    `rate_sweep` cell INSIDE the window.
  * **`g_peak`, and `required_at_cross`** out of `lag_relief` — measured ungated at step 4.

THREE CELLS ARE **ADDED** RATHER THAN PORTED, and each is named so that a superset cannot pass as
a port:

  * section E's **no-engagement** cells. `surge_relief` and `release_relief` both return
    `s_eng`/`s_rel` = `NaN` when nothing engages, and § 5.18 finding 4 measured the minimum
    `n_engaged` over every rung-49 floor cell at **10** and over every rung-50 `s_off` cell at
    **2** — never zero. So both NaN arms are unreachable from the suites and get a cell here.
    The companion ambiguity is emitted beside them: with nothing engaged `release_relief` returns
    `s_eng = nan` but `deficit_at_release = 0.0`, **two sentinels for one condition in one
    record**, and `0.0` is a legitimate deficit.
  * section F's `rate_sweep` cell inside the window (above).
  * section G's **knife-edge** cells at both `ds`, whose `s_rel` bits are the only instrument that
    can hold the march coordinate's spelling.

The MANUFACTURED `armed`-seed trajectory (§ 5.18 finding 2) is NOT here: it needs a hand-built
`FuelPoint` sequence with `required < g` at the first clipped point, which no marched cell
produces and Python has no constructor for. It lives on the Rust side, on `first_raw_min`'s
tie-gate template.

TWO ARMS:

  main      everything, under PyPy — the golden.
  cpython   the same under CPython 3.14, read as a DETECTOR with a measured sensitivity and never
            as coverage. Every cell here is CPG (all four suites build `_cpg_gas()`), so nothing
            is expected to move; NO count is registered — five typed count bars in this port,
            five wrong.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_release.py main    rust/oracle/release_pypy.tsv
    C:\\Python314\\python.exe  rust/oracle/dump_release.py cpython rust/oracle/release_cpython.tsv
"""
import os
import struct
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(_HERE, "..", ".."))
sys.path.insert(0, os.path.join(_HERE, "..", "..", "tests"))

import test_rung49 as R49                                                 # noqa: E402
import test_rung50 as R50                                                 # noqa: E402
import test_rung51 as R51                                                 # noqa: E402
import test_rung52 as R52                                                 # noqa: E402
from turbojet.engine import AsymmetricLag, SurgeLimiter                   # noqa: E402

ARM = sys.argv[1] if len(sys.argv) > 1 else "main"
OUT = sys.argv[2] if len(sys.argv) > 2 else None
assert ARM in ("main", "cpython"), ARM

ROWS = []


def put(key, value):
    """A finite float key."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), "%s is not finite: %r" % (key, v)
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putn(key, value):
    """A float key ALLOWED to be NaN — the no-engagement arms of section E, and nothing else.

    Separate from `put` on purpose: the finiteness assert there is a real guard on every other
    key and widening it would disarm it everywhere to reach one arm. An INFINITY is still
    refused. PyPy's `float("nan")` and Rust's `f64::NAN` are both `7ff8000000000000`, measured in
    slice T, so nothing NaN-aware happens on either side of the comparison."""
    v = float(value)
    assert abs(v) != float("inf"), "%s is infinite: %r" % (key, v)
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    """A discrete key — counts and booleans."""
    n = int(n)
    ROWS.append((key, n if n >= 0 else n + (1 << 64), str(n)))


def puts(key, s):
    """A STRING key — `spool`. Emitted as a byte-length-plus-hash-free literal in the text column
    and as a small integer code in the bits column, so the Rust compares an enum rather than
    re-deriving a string."""
    code = {None: 0, "lp": 1, "hp": 2}[s]
    ROWS.append((key, code, repr(s)))


def emit_row(prefix, row, floats, opt_floats=(), discretes=(), strings=(), nanable=()):
    for k in floats:
        put("%s/%s" % (prefix, k), row[k])
    for k in opt_floats:
        if row[k] is None:
            putd("%s/%s/is_none" % (prefix, k), 1)
        else:
            putd("%s/%s/is_none" % (prefix, k), 0)
            put("%s/%s" % (prefix, k), row[k])
    for k in discretes:
        putd("%s/%s" % (prefix, k), row[k])
    for k in strings:
        puts("%s/%s" % (prefix, k), row[k])
    for k in nanable:
        putn("%s/%s" % (prefix, k), row[k])


# The three records' field lists, spelled out EXHAUSTIVELY so a key Python adds and the port
# forgets shows up as a missing golden rather than as silence. The counts are § 5.18 finding 5's
# measured 25 / 27 / 34 and are asserted against `len(row)` at every call.
_R49_F = ("phi_lim", "r", "rho", "hold_err", "s_lp_bare", "s_hp_bare", "relief_lp", "relief_hp",
          "relief_watched", "relief_other", "s_min_other", "min_phi_lp_bare", "min_phi_lp_lim",
          "min_phi_hp_bare", "min_phi_hp_lim", "fuel_removed", "Tt4_peak_bare", "Tt4_peak_lim",
          "nu_hp_end", "nu_hp_end_bare")
_R49_D = ("n_engaged", "both_edges_inside_ramp")
_R49_S = ("spool",)
_R49_N = ("s_eng", "s_rel")

_R50_F = ("r", "rho", "ds", "deficit_at_release", "s_lp_bare", "s_hp_bare", "relief_lp",
          "relief_hp", "s_min_lp", "s_min_hp", "min_phi_lp_bare", "min_phi_lp_lim",
          "min_phi_hp_bare", "min_phi_hp_lim", "fuel_removed", "nu_hp_end", "nu_hp_end_bare")
_R50_O = ("s_off", "tau_rel", "phi_lim", "margin", "relief_watched", "relief_other")
_R50_D = ("n_engaged",)
_R50_S = ("spool",)
_R50_N = ("s_eng", "s_rel")

_R52_F = ("tau_att", "tau_rel", "r", "rho", "ds", "g_peak", "s_lp_bare", "s_hp_bare",
          "relief_lp", "relief_hp", "s_min_lp", "s_min_hp", "min_phi_lp_bare", "min_phi_lp_lag",
          "min_phi_hp_bare", "min_phi_hp_lag", "fuel_removed", "Tt4_peak_bare", "Tt4_peak_lag",
          "nu_hp_end", "nu_hp_end_bare")
_R52_O = ("phi_lim", "margin", "relief_watched", "relief_other")
_R52_D = ("n_recross",)
_R52_S = ("spool",)
_R52_N = ("s_cross", "g_at_cross", "required_at_cross",
          "s_eng_0.05", "s_rel_0.05", "s_eng_0.01", "s_rel_0.01")


def put49(prefix, row):
    assert len(row) == 25, (len(row), sorted(row))
    emit_row(prefix, row, _R49_F, (), _R49_D, _R49_S, _R49_N)


def put50(prefix, row):
    assert len(row) == 27, (len(row), sorted(row))
    emit_row(prefix, row, _R50_F, _R50_O, _R50_D, _R50_S, _R50_N)


def put52(prefix, row):
    assert len(row) == 34, (len(row), sorted(row))
    emit_row(prefix, row, _R52_F, _R52_O, _R52_D, _R52_S, _R52_N)


# ============================================================ A — rung 49's two shared sweeps
_ft49 = R49._ft()
for _name, _floors, _spool in (("lp", R49.LP_FLOORS, "lp"), ("hp", R49.HP_FLOORS, "hp")):
    for _i, _row in enumerate(_ft49.floor_sweep(R49.FLIGHT, R49.LO, R49.HI, _floors,
                                                spool=_spool, r=R49.R, s_settle=R49.SETTLE,
                                                ds=R49.DS)):
        put49("A/%s/%d" % (_name, _i), _row)

# ============================================================ B — rung 50's eleven sweeps
_B = (
    ("g3g4_r2", R50.R2_OFFS, R50.PHI_LIM_2, None, 2.0, 2.0, 0.02, 1.0),
    ("g5_early", (0.16, 0.20, 0.26, 0.30, 0.36, 0.44, 0.60), R50.PHI_LIM, None,
     0.5, 2.0, 0.02, 1.0),
    ("g6_m025", (0.30, 0.44, 0.50, 9.90), None, 0.25, 0.5, 2.0, 0.02, 1.0),
    ("g7_m015_r2", (0.66, 1.10, 1.80, 9.90), None, 0.15, 2.0, 2.0, 0.02, 1.0),
    ("g9_settle4", R50.R2_OFFS, R50.PHI_LIM_2, None, 2.0, 4.0, 0.02, 1.0),
    ("g10_r05_ds02", (0.30, 0.40, 0.44), R50.PHI_LIM, None, 0.5, 2.0, 0.02, 1.0),
    ("g10_r05_ds01", (0.30, 0.40, 0.44), R50.PHI_LIM, None, 0.5, 2.0, 0.01, 1.0),
    ("g10_r2_ds02", (1.10, 1.56), R50.PHI_LIM_2, None, 2.0, 2.0, 0.02, 1.0),
    ("g10_r2_ds01", (1.10, 1.56), R50.PHI_LIM_2, None, 2.0, 2.0, 0.01, 1.0),
    ("g10b_rho025", (0.26, 0.30, 0.36), R50.PHI_LIM, None, 0.5, 2.0, 0.02, 0.25),
    ("g10b_rho4", (0.26, 0.30, 0.36), R50.PHI_LIM, None, 0.5, 2.0, 0.02, 4.0),
)
for _tag, _offs, _phi, _m, _r, _settle, _ds, _rho in _B:
    for _i, _row in enumerate(R50._sweep(_offs, phi_lim=_phi, margin=_m, r=_r, settle=_settle,
                                         ds=_ds, rho=_rho)):
        put50("B/%s/%d" % (_tag, _i), _row)

# rung 50's three matched-release cells and its unforced one
_ft50 = R50._ft()
_acc50 = _ft50.accel_schedule(R50.FLIGHT, R50.LO, R50.HI, 0.25)
for _i, _kw in enumerate((dict(surge=SurgeLimiter(spool="lp", phi_lim=0.7450)),
                          dict(surge=SurgeLimiter(spool="lp", phi_lim=0.7500)),
                          dict(accel=_acc50))):
    put50("B/g8_matched/%d" % _i,
          _ft50.release_relief(R50.FLIGHT, R50.LO, R50.HI, 0.44, r=R50.R, s_settle=R50.SETTLE,
                               ds=R50.DS, **_kw))
put50("B/c1b_unforced/0",
      _ft50.release_relief(R50.FLIGHT, R50.LO, R50.HI, None,
                           surge=SurgeLimiter(spool="lp", phi_lim=R50.PHI_LIM),
                           r=R50.R, s_settle=R50.SETTLE, ds=R50.DS))

# ============================================================ C — rung 51's memo cells
# The cell list is READ OFF the suite: run its sixteen gates, then dump `_ROWS`.
for _fn in [getattr(R51, _n) for _n in dir(R51) if _n.startswith("test_")]:
    _fn()
for _k in sorted(R51._ROWS, key=lambda t: tuple((v is None, v) for v in t)):
    _so, _tr, _phi, _m, _r, _rho, _ds = _k
    put50("C/so=%s,tr=%s,phi=%s,m=%s,r=%s,rho=%s,ds=%s" % _k, R51._ROWS[_k])

# rung 51's two `rate_sweep` calls and its never-called `deficit_curve`
_ft51 = R51._ft()
_leg51 = SurgeLimiter(spool="lp", phi_lim=R51.PHI_LIM)
for _tag, _s_off, _taus in (("c1b", 0.30, [None]), ("c4", 0.60, [None, 0.04, 0.32])):
    for _i, _row in enumerate(_ft51.rate_sweep(R51.FLIGHT, R51.LO, R51.HI, _s_off, _taus,
                                               surge=_leg51, r=R51.R, s_settle=R51.SETTLE,
                                               ds=R51.DS)):
        put50("C/rate_sweep/%s/%d" % (_tag, _i), _row)
for _i, _row in enumerate(_ft51.deficit_curve(R51.FLIGHT, R51.LO, R51.HI, 0.44,
                                              (0.7550, 0.7500, 0.7450), spool="lp",
                                              r=R51.R, s_settle=R51.SETTLE, ds=R51.DS)):
    put50("C/deficit_curve/%d" % _i, _row)

# ============================================================ D — rung 52's memo cells + grids
for _fn in [getattr(R52, _n) for _n in dir(R52) if _n.startswith("test_")]:
    _fn()
for _k in sorted(R52._ROWS):
    put52("D/ta=%s,tr=%s,phi=%s,r=%s,rho=%s,ds=%s" % _k, R52._ROWS[_k])

_ft52 = R52._ft()
for _tag, _tas, _trs, _phi, _r, _ds in (
        ("gate3", (0.02, 0.20), (0.02, 0.10, 0.40), R52.PHI_LIM_2, R52.R2, R52.DS),
        ("gate4", (0.02, 0.32), (0.01, 0.16), R52.PHI_LIM, R52.R, 0.01)):
    _g = _ft52.factorization_grid(R52.FLIGHT, R52.LO, R52.HI, _tas, _trs,
                                  surge=SurgeLimiter(spool="lp", phi_lim=_phi),
                                  r=_r, s_settle=R52.SETTLE, ds=_ds)
    put("D/fg/%s/max_residual" % _tag, _g["max_residual"])
    put("D/fg/%s/max_main_effect" % _tag, _g["max_main_effect"])
    putd("D/fg/%s/n_rows" % _tag, len(_g["rows"]))
    for _i, _ta in enumerate(_tas):
        put("D/fg/%s/credit_spread/%d" % (_tag, _i), _g["credit_spread"][_ta])
        for _j in range(len(_trs)):
            put("D/fg/%s/residual/%d/%d" % (_tag, _i, _j), _g["residual"][_i][_j])
    for _i, _row in enumerate(_g["rows"]):
        put52("D/fg/%s/row%d" % (_tag, _i), _row)

# ============================================================ E — the NO-ENGAGEMENT arms, ADDED
# § 5.18 finding 4: min `n_engaged` is 10 over every rung-49 floor cell and 2 over every rung-50
# `s_off` cell, so BOTH `NaN` arms are unreachable from the suites. A floor far BELOW the bare
# minimum can never bind; an `s_off` at the first march point releases before anything engages.
_MIN_PHI_LP_BARE = 0.735466        # `test_rung49.py:67`, the bare LP minimum at r=0.5
_DORMANT = _MIN_PHI_LP_BARE - 0.05
put49("E/surge_relief_dormant",
      _ft49.surge_relief(R49.FLIGHT, R49.LO, R49.HI,
                         SurgeLimiter(spool="lp", phi_lim=_DORMANT),
                         r=R49.R, s_settle=R49.SETTLE, ds=R49.DS))
put50("E/release_relief_dormant",
      _ft50.release_relief(R50.FLIGHT, R50.LO, R50.HI, R50.DS,
                           surge=SurgeLimiter(spool="lp", phi_lim=R50.PHI_LIM),
                           r=R50.R, s_settle=R50.SETTLE, ds=R50.DS))
# the TWO-SENTINEL ambiguity, emitted as its own keys so the claim is gated and not just prose
_dorm = _ft50.release_relief(R50.FLIGHT, R50.LO, R50.HI, R50.DS,
                             surge=SurgeLimiter(spool="lp", phi_lim=R50.PHI_LIM),
                             r=R50.R, s_settle=R50.SETTLE, ds=R50.DS)
putd("E/two_sentinels/n_engaged", _dorm["n_engaged"])
putd("E/two_sentinels/s_eng_is_nan", _dorm["s_eng"] != _dorm["s_eng"])
put("E/two_sentinels/deficit_at_release", _dorm["deficit_at_release"])

# ============================================================ F — `rate_sweep` INSIDE the window
# Step 3, injection I1: of the four `rate_sweep` rows the rung-51 suite produces, exactly two
# carry a live `tau_rel` and BOTH are contract 4's, whose claim is that `tau_rel` is inert there.
# So the function's one job is exercised only on cells chosen for inertness. This cell is inside
# the window at `r = 2.0`, where a dropped forwarding moves the PHYSICS.
for _i, _row in enumerate(_ft51.rate_sweep(R51.FLIGHT, R51.LO, R51.HI, 1.56,
                                           [None, 0.20, 0.40],
                                           surge=SurgeLimiter(spool="lp",
                                                              phi_lim=R51.PHI_LIM_2),
                                           r=R51.R2, s_settle=R51.SETTLE, ds=R51.DS)):
    put50("F/rate_sweep_live/%d" % _i, _row)

# ============================================================ G — the KNIFE-EDGE cells
# Step 2, finding 2: `release_weight` compares the ACCUMULATED march coordinate against `s_off`,
# and at `ds = 0.02` the accumulated `s` at `s_off = 0.20` is 0.19999999999999998 and at 0.26 is
# 0.25999999999999995 — so the leg stays armed one point LONGER than `k * ds` would keep it. A
# "cleaner" coordinate moves `n_engaged` 8 -> 7 and `s_rel` by a WHOLE CELL, and all 15 gates
# pass. These bits are the only instrument that holds it.
for _so in (0.20, 0.26):
    for _ds in (0.02, 0.01):
        _row = _ft50.release_relief(R50.FLIGHT, R50.LO, R50.HI, _so,
                                    surge=SurgeLimiter(spool="lp", phi_lim=R50.PHI_LIM),
                                    r=R50.R, s_settle=R50.SETTLE, ds=_ds)
        put50("G/knife/%s/%s" % (_so, _ds), _row)

# and the march coordinate ITSELF, at the indices that decide those two comparisons
_traj, _ = _ft50._fuel_ramp_march(R50.FLIGHT, R50.LO, R50.HI, R50.R, R50.SETTLE, 0.02)
for _k in (5, 8, 10, 13, 15, 22, 25):
    put("G/coord/ds002/%d" % _k, _traj[_k]["s"])
putd("G/coord/ds002/npts", len(_traj))

# ============================================================ H — the march LENGTHS
# § 5.18 finding 5b: every reader takes `nu_hp_end` off `lim[-1]`, so a one-point length
# difference would move it silently. 201 points at `s_end = 4.0`, 301 at 6.0, on all three
# marchers — emitted rather than asserted in prose.
_leg_h = SurgeLimiter(spool="lp", phi_lim=R50.PHI_LIM)
for _tag, _kw, _r, _settle in (("plain", {}, R50.R, R50.SETTLE),
                               ("forced", dict(surge=_leg_h, s_off=0.30), R50.R, R50.SETTLE),
                               ("faded", dict(surge=_leg_h, s_off=0.30, tau_rel=0.10),
                                R50.R, R50.SETTLE),
                               ("lagged", dict(surge=_leg_h, lag=AsymmetricLag(0.02, 0.10)),
                                R50.R, R50.SETTLE),
                               ("r2", dict(surge=_leg_h), 2.0, 4.0)):
    _t, _ = _ft50._fuel_ramp_march(R50.FLIGHT, R50.LO, R50.HI, _r, _settle, R50.DS, **_kw)
    putd("H/%s/npts" % _tag, len(_t))
    put("H/%s/s_end" % _tag, _t[-1]["s"])
    put("H/%s/nu_hp_end" % _tag, _t[-1]["nu_hp"])

# =========================================================================== emit
_out = open(OUT, "w", encoding="utf-8", newline="\n") if OUT else sys.stdout
_out.write("# slice U oracle (" + ARM + ") - rungs 49+50+51+52 readers - "
           "key\tu64 bits (or an integer)\trepr\n")
for _key, _bits, _text in ROWS:
    _out.write("%s\t%s\t%s\n" % (_key, _bits, _text))
if OUT:
    _out.close()
sys.stderr.write("[dump_release/%s] %d values\n" % (ARM, len(ROWS)))
