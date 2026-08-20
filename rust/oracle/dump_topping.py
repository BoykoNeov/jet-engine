"""SLICE T step 4 — THE ORACLE for rungs 46 + 47 + 48's readers, over the THREE suites' grids.

Steps 1-3 ported 31 gates and then measured what those gates can and cannot see. Three defects
survived every one of them, and this dump exists to hold exactly those three:

  * **`required` reads the APPLIED fuel instead of the SCHEDULE** in `_integrate_fuel_lagged`
    (step 2, injection 4) — moves 13 of 18 readings by up to 24 %, caught by 0 of rung 47's 9.
  * **the RK4 `g` weight, a `2` dropped from `k3g`** (step 2, injection 5) — 13 of 18, 0 of 9.
  * **the `fuel_removed` trapezoid losing its `0.5`** (step 3) — `fuel_removed` alone, exactly
    2x, caught by 0 of rung 48's 16 AND by nothing else in the project: every reader of that
    integral in either language is `> 0.0` or a pairwise `<`, both invariant under a positive
    scale factor. There is no bar to loosen; only a VALUE gate can hold it.

Rung 47's suite has no value content at all — four of its nine gates are bit-identities between
two runs of the same code and the other five are inequalities whose tightest margin is 2.19x. So
this file is the numbers underneath all three suites, and the lagged route's ONLY value coverage
in the crate beyond the two smoke cells slice S added for another reason.

**THE GRID IS THREE GRIDS, AND THEY ARE IMPORTED RATHER THAN COPIED.** Every prior dump in this
port copies the suite's constants verbatim into its own header, and slice S step 4 measured the
cost: a probe whose header claimed the suites' grids ran a cross-product of its own choosing and
four registered numbers died on it. Here the three test modules are IMPORTED and their
module-level constants read off them, so the grids cannot drift. They differ in ways that look
shareable and are not:

  rung 46   4 shapes, SETTLE 2.0, REDLINE 1480, **Gas.thermally_perfect()** on gates 3-6
  rung 47   the same 4 shapes, SETTLE 2.0, **CPG throughout**, a five-point tau sweep at r=0.5
  rung 48   **3** shapes (no `press/flow`), **SETTLE 4.0**, MARGINS + six one-off margins

Two cells are NOT reachable through a module constant, and one of them is the most valuable cell
in the slice: `test_rung46.py::test_the_lever_fast_ramp_switches_on_lp_relief` uses a LOCAL
`Tt4_max = 1440.0` (line 187) at r=0.15 — step 1 measured gate 6 as the ONLY gate carrying
`relief_lp`'s sign, and it lives at that redline. `test_rung47.py::
test_fast_ramp_lp_relief_eroded_by_lag_never_enhanced` uses a local `red = 1440.0` (line 234) with
a FOUR-point tau list where the r=0.5 sweep has five. Both are read out of the test source by eye
and cited at their sections.

**WHAT IS ADDED RATHER THAN PORTED, NAMED SO A SUPERSET CANNOT PASS AS A PORT:**

  * section C is the CROSS-PRODUCT 4 shapes x 6 taus. The suite runs tau=0.2 on all four shapes
    (gate 5) and the five-tau sweep on `flow/press` alone (gate 6); the other 15 cells are new.
  * `m = 0.60` at r=0.5 (section H) drives `n_engaged` to **0**, so `s_eng` is `float("nan")` —
    the arm `schedule_relief` has carried since rung 48 and which NO suite cell reaches (the
    lowest any of them drives it is 1, gate 12's m=0.78 at r=0.15). 0.60 is the suite's own
    dormant-schedule margin reached through a different reader, so the cell is adjacent rather
    than invented.
  * `m = 0.55` and `m = 0.02` (section H) bracket that boundary and finding 7's honest corner.
  * section N emits march LENGTHS. Neither `topping_relief` nor `schedule_relief` returns one,
    and slice S step 3 measured `npts` to be the only channel that witnesses the march bound.
  * section E emits the FULL engaged `(s, mf)` trace. Step 2's probe read its ENDS, and an error
    in the middle with correct ends survives that.

TWO ARMS:

  main      everything, under PyPy — the golden.
  cpython   the same under CPython 3.14, read as a DETECTOR with a measured sensitivity and
            never as coverage. The TPG sections (A, B) run NASA integrals and are expected to
            move; the CPG ones are not. NO count is registered here — five typed count bars in
            this port, five wrong — the Rust side tiers on what this arm actually produces.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_topping.py main    rust/oracle/topping_pypy.tsv
    C:\\Python314\\python.exe  rust/oracle/dump_topping.py cpython rust/oracle/topping_cpython.tsv
"""
import os
import struct
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(_HERE, "..", ".."))
sys.path.insert(0, os.path.join(_HERE, "..", "..", "tests"))

import test_rung46 as R46                                                 # noqa: E402
import test_rung47 as R47                                                 # noqa: E402
import test_rung48 as R48                                                 # noqa: E402

ARM = sys.argv[1] if len(sys.argv) > 1 else "main"
OUT = sys.argv[2] if len(sys.argv) > 2 else None
assert ARM in ("main", "cpython"), ARM

ROWS = []


def put(key, value):
    """A finite float key."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putn(key, value):
    """A float key that is ALLOWED to be NaN — `s_eng` and nothing else.

    Separate from `put` on purpose: the finiteness assert there is a real guard on every other
    key, and widening it would disarm it everywhere to reach one arm. What is gated is the BIT
    PATTERN, and the round-trip through this file is what needed checking rather than assuming:
    `struct.pack` gives `7ff8000000000000` on both interpreters and the Rust side parses that as
    a u64 like any other key, so nothing NaN-aware happens on either side of the comparison. An
    INFINITY here would still be a defect, so it is still refused."""
    v = float(value)
    assert abs(v) != float("inf"), f"{key} is infinite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    """A discrete key — counts and booleans. Negative values are written two's-complement, so a
    `-1` cannot crash the Rust reader's `u64` parse; the text column keeps the signed spelling."""
    n = int(n)
    ROWS.append((key, n if n >= 0 else n + (1 << 64), str(n)))


# ============================================================ the readers, key by key
# Every number comes out of the SHIPPED body. Nothing here recomputes a dumped value.

_TOPPING_FLOATS = ("Tt4_peak_bare", "Tt4_peak_top", "overshoot",
                   "min_phi_lp_bare", "min_phi_lp_top",
                   "min_phi_hp_bare", "min_phi_hp_top", "relief_lp", "relief_hp")
_EXC_FLOATS = ("ext_lp", "ext_hp", "s_lp", "s_hp", "min_phi_lp", "min_phi_hp",
               "Tt4_peak", "ratio")
_SCHED_FLOATS = ("margin", "r", "rho", "s_lp_bare", "s_hp_bare", "relief_lp", "relief_hp",
                 "min_phi_lp_bare", "min_phi_lp_lim", "min_phi_hp_bare", "min_phi_hp_lim",
                 "fuel_removed", "Tt4_peak_bare", "Tt4_peak_lim", "nu_hp_end", "nu_hp_end_bare")


def emit_topping(tag, row):
    """`topping_relief`'s 14 keys. `rho`/`r`/`Tt4_max` are echoed knobs and carried too — a row
    that echoes the wrong knob is a defect the derived keys would not show."""
    for k in ("rho", "r", "Tt4_max"):
        put(f"{tag}/{k}", row[k])
    putd(f"{tag}/tau_gov_is_set", 1 if row["tau_gov"] is not None else 0)
    if row["tau_gov"] is not None:
        put(f"{tag}/tau_gov", row["tau_gov"])
    for k in _TOPPING_FLOATS:
        put(f"{tag}/{k}", row[k])
    putd(f"{tag}/held", 1 if row["held"] else 0)


def emit_excursion(tag, exc):
    """`phi_excursion_fuel`'s 9 keys, read off the TOPPED / LIMITED march.

    Slice S's oracle gates this method on the BARE configuration only — its grid never arms
    `Tt4_max`, `tau_gov` or `accel` — so `npts`, `s_lp`, `s_hp`, `ext_*` and `ratio` under a live
    limiter are ungated anywhere in the crate. `ratio` is `inf` when `ext_hp` is 0; that has never
    happened on these cells and `put` refuses it if it starts to."""
    for k in _EXC_FLOATS:
        put(f"{tag}/{k}", exc[k])
    putd(f"{tag}/npts", exc["npts"])


def emit_schedule(tag, row):
    """`schedule_relief`'s 18 keys, `s_eng` included even when it is NaN."""
    for k in _SCHED_FLOATS:
        put(f"{tag}/{k}", row[k])
    putn(f"{tag}/s_eng", row["s_eng"])
    putd(f"{tag}/n_engaged", row["n_engaged"])


# ============================================================ A — rung 46, THERMALLY PERFECT
# test_rung46.py gates 3/4/5: the redline in the gap, all four shapes, on `Gas.thermally_perfect()`.
# The TPG gas is rung 46's and rung 46's alone — rungs 47 and 48 are CPG throughout — and it is
# why this section is two orders slower than the rest of the file and why the CPython arm tiers
# on it separately.
_tpg_design = R46._design(R46._tpg())

for _name, (_ml, _mh) in R46.SHAPES.items():
    _ft = R46.TwoSpoolFuelTransient(_tpg_design, R46.FLIGHT, 1.0,
                                    map_lp=_ml, map_hp=_mh, rho=1.0)
    _row = _ft.topping_relief(R46.FLIGHT, R46.LO, R46.HI, R46.REDLINE,
                              r=R46.R, s_settle=R46.SETTLE)
    emit_topping(f"A/{_name}", _row)
    _exc = _ft.phi_excursion_fuel(R46.FLIGHT, R46.LO, R46.HI, r=R46.R, s_settle=R46.SETTLE,
                                  Tt4_max=R46.REDLINE)
    emit_excursion(f"A/{_name}/top", _exc)

# ============================================================ B — rung 46's LEVER, TPG
# `test_rung46.py::test_the_lever_fast_ramp_switches_on_lp_relief` (line 187) — a LOCAL redline
# of 1440.0 on the `flow/press` shape at r in {0.5, 0.15}. Step 1 measured this as the ONE gate of
# seven that carries `relief_lp`'s SIGN: at moderate r the relief is EXACTLY 0.0, and a sign flip
# on an exact zero is invisible, so the fast-ramp cell is the only place the sign is testable.
_B_TT4_MAX = 1440.0                                    # test_rung46.py:187, a local not a constant
_ft_b = R46.TwoSpoolFuelTransient(_tpg_design, R46.FLIGHT, 1.0,
                                  map_lp=R46.LP_SHAPED, map_hp=R46.HP_SHAPED, rho=1.0)
for _rlab, _r in (("0.5", 0.5), ("0.15", 0.15)):
    _tag = f"B/r{_rlab}"
    _row = _ft_b.topping_relief(R46.FLIGHT, R46.LO, R46.HI, _B_TT4_MAX,
                                r=_r, s_settle=R46.SETTLE)
    emit_topping(_tag, _row)
    emit_excursion(f"{_tag}/top",
                   _ft_b.phi_excursion_fuel(R46.FLIGHT, R46.LO, R46.HI, r=_r,
                                            s_settle=R46.SETTLE, Tt4_max=_B_TT4_MAX))

# ============================================================ C — rung 47's LAGGED governor, CPG
# THE CELLS THIS FILE EXISTS FOR. `_integrate_fuel_lagged`'s only value coverage in the crate is
# slice S's two smoke cells, which run Tt4_max=1380, tau_gov=0.2, ds=0.05, s_end=1.0 and ONE map
# pair; rung 47's gates run REDLINE=1480, five taus, four shapes, ds=0.02 and a second ramp rate.
#
# DECLARED SUPERSET: the suite runs tau=0.2 on all four shapes (gate 5) and the five-tau sweep on
# `flow/press` only (gate 6). The other 15 cells here are ADDED. `tau_gov=None` is carried first
# on every shape because it is rung 46's instantaneous min-select — the reduce the lagged route
# must land on, now as VALUES rather than as gate 1's two-call bit-identity, which step 3 measured
# to be vacuous in Rust (one `FuelLimiters`, one march, compared with itself).
_cpg_design = R47._design(R47._cpg_gas())
# LABELLED rather than formatted. Every key in this file is spelled by a literal on BOTH sides,
# so a golden key and the Rust key that looks for it cannot drift apart through two languages'
# float `repr` — the port has lost a day to a key that formatted differently, and the failure
# mode (`NO GOLDEN`) is loud but uninformative.
_C_TAUS = (("none", None), ("0.05", 0.05), ("0.1", 0.1), ("0.2", 0.2), ("0.4", 0.4),
           ("0.8", 0.8))                                # test_rung47.py:213, plus the None reduce

for _name, (_ml, _mh) in R47.SHAPES.items():
    _ft = R47.TwoSpoolFuelTransient(_cpg_design, R47.FLIGHT, 1.0,
                                    map_lp=_ml, map_hp=_mh, rho=1.0)
    for _tlab, _tau in _C_TAUS:
        _tag = f"C/{_name}/tau{_tlab}"
        emit_topping(_tag, _ft.topping_relief(R47.FLIGHT, R47.LO, R47.HI, R47.REDLINE,
                                              r=R47.R, s_settle=R47.SETTLE, tau_gov=_tau))
        emit_excursion(f"{_tag}/top",
                       _ft.phi_excursion_fuel(R47.FLIGHT, R47.LO, R47.HI, r=R47.R,
                                              s_settle=R47.SETTLE, Tt4_max=R47.REDLINE,
                                              tau_gov=_tau))

# ============================================================ D — rung 47's FAST RAMP, CPG
# `test_rung47.py::test_fast_ramp_lp_relief_eroded_by_lag_never_enhanced` (line 234): a LOCAL
# `red = 1440.0` at r=0.15 with a FOUR-point tau list, not the five-point one section C sweeps.
# This is where `relief_lp` is non-zero, so it is the only place the lagged route's LP half has a
# sign at all.
_D_RED = 1440.0                                        # test_rung47.py:234, a local not a constant
_ft_d = R47.TwoSpoolFuelTransient(_cpg_design, R47.FLIGHT, 1.0,
                                  map_lp=R47.LP_SHAPED, map_hp=R47.HP_SHAPED, rho=1.0)
for _tlab, _tau in (("none", None), ("0.05", 0.05), ("0.1", 0.1), ("0.2", 0.2), ("0.4", 0.4)):
    _tag = f"D/tau{_tlab}"
    emit_topping(_tag, _ft_d.topping_relief(R47.FLIGHT, R47.LO, R47.HI, _D_RED,
                                            r=0.15, s_settle=R47.SETTLE, tau_gov=_tau))
    emit_excursion(f"{_tag}/top",
                   _ft_d.phi_excursion_fuel(R47.FLIGHT, R47.LO, R47.HI, r=0.15,
                                            s_settle=R47.SETTLE, Tt4_max=_D_RED, tau_gov=_tau))

# ============================================================ E — rung 47's COMMAND TRACE, CPG
# EVERY engaged `(s, mf)` pair, not the ends. Step 2's probe read the first and last of this trace;
# an error in the middle with correct ends survives that, and `monotone_nondecreasing` is a
# BOOLEAN over the whole window, so the suite's one reader of the middle is a predicate.
for _name, (_ml, _mh) in R47.SHAPES.items():
    _ft = R47.TwoSpoolFuelTransient(_cpg_design, R47.FLIGHT, 1.0,
                                    map_lp=_ml, map_hp=_mh, rho=1.0)
    _t = _ft.topping_command_trace(R47.FLIGHT, R47.LO, R47.HI, R47.REDLINE,
                                   r=R47.R, s_settle=R47.SETTLE)
    putd(f"E/{_name}/n_engaged", _t["n_engaged"])
    putd(f"E/{_name}/monotone", 1 if _t["monotone_nondecreasing"] else 0)
    put(f"E/{_name}/Tt4_max", _t["Tt4_max"])
    put(f"E/{_name}/r", _t["r"])
    for _i, (_s, _mf) in enumerate(_t["engaged"]):
        put(f"E/{_name}/s{_i}", _s)
        put(f"E/{_name}/mf{_i}", _mf)

# ============================================================ F — rung 48's engagement sweep, CPG
# `test_rung48.py`'s MARGINS on its OWN three shapes at SETTLE=4.0. Rung 48's shape set has three
# entries against rung 47's four (no `press/flow`) and its settle is 4.0 against 2.0, because the
# sweep reads a SETTLED `nu_hp_end`; reusing rung 47's would have widened the grid silently.
_F_LABELS = ("0.15", "0.25", "0.35", "0.42", "0.45", "0.48")
assert R48.MARGINS == (0.15, 0.25, 0.35, 0.42, 0.45, 0.48), R48.MARGINS
_ft_f = {}
for _name, (_ml, _mh) in R48.SHAPES.items():
    _ft_f[_name] = R48._ft(ml=_ml, mh=_mh)
    _rows = _ft_f[_name].engagement_sweep(R48.FLIGHT, R48.LO, R48.HI, R48.MARGINS,
                                          r=R48.R, s_settle=R48.SETTLE, ds=R48.DS)
    assert len(_rows) == len(R48.MARGINS), (len(_rows), len(R48.MARGINS))
    for _lab, _row in zip(_F_LABELS, _rows):
        emit_schedule(f"F/{_name}/m{_lab}", _row)

# ============================================================ G — rung 48's ONE-OFF cells, CPG
# The four gates that leave the MARGINS sweep: gate 12's coincident-minima ramp, gate 9b's slow
# ramp where the HP relief is exactly zero PAST both minima, gate 11's degenerate margin, and the
# min-select COMPOSITE where rungs 46/47's governor is armed on top of the leg.
for _lab, _row in zip(("0.60", "0.70", "0.78"),
                      _ft_f["flow/press"].engagement_sweep(
                          R48.FLIGHT, R48.LO, R48.HI, (0.60, 0.70, 0.78),
                          r=0.15, s_settle=R48.SETTLE, ds=R48.DS)):
    emit_schedule(f"G/fast/m{_lab}", _row)                    # gate 12, r=0.15

_acc_9b = _ft_f["flow/press"].accel_schedule(R48.FLIGHT, R48.LO, R48.HI, 0.20)
emit_schedule("G/slow/m0.2", _ft_f["flow/press"].schedule_relief(
    R48.FLIGHT, R48.LO, R48.HI, _acc_9b, r=2.0, s_settle=R48.SETTLE, ds=R48.DS))  # gate 9b, r=2.0

for _row in _ft_f["flow/press"].engagement_sweep(R48.FLIGHT, R48.LO, R48.HI, (0.05,),
                                                 r=R48.R, s_settle=R48.SETTLE, ds=R48.DS):
    emit_schedule("G/deg/m0.05", _row)                        # gate 11's degenerate boundary

# The COMPOSITE: the Wf/pt3 leg with the TIT governor armed on top, instantaneous and lagged.
# `schedule_relief`'s bare leg stays governor-free either way, so the differential still isolates
# the schedule — which is exactly the claim gate 3 makes structurally and never numerically.
_acc_c = _ft_f["flow/press"].accel_schedule(R48.FLIGHT, R48.LO, R48.HI, 0.25)
for _tag, _tt4max, _tau in (("gov", R48.REDLINE, None), ("govlag", R48.REDLINE, 0.2)):
    emit_schedule(f"G/comp/{_tag}", _ft_f["flow/press"].schedule_relief(
        R48.FLIGHT, R48.LO, R48.HI, _acc_c, r=R48.R, s_settle=R48.SETTLE, ds=R48.DS,
        Tt4_max=_tt4max, tau_gov=_tau))

# ============================================================ H — the ADDED margins, CPG
# `s_eng = eng[0] if eng else float("nan")` has been live code since rung 48 and DEAD on every
# suite cell. m=0.60 at r=0.5 drives `n_engaged` to 0 and fires it; 0.55 is the boundary the sweep
# measured; 0.02 is finding 7's honest corner, which COMPLETES rather than refusing — a Rust-side
# refusal there is a defect, and the march LENGTH in section N is what witnesses it.
for _lab, _row in zip(("0.02", "0.55", "0.60"),
                      _ft_f["flow/press"].engagement_sweep(
                          R48.FLIGHT, R48.LO, R48.HI, (0.02, 0.55, 0.60),
                          r=R48.R, s_settle=R48.SETTLE, ds=R48.DS)):
    emit_schedule(f"H/m{_lab}", _row)

# ============================================================ N — the march LENGTHS
# Neither reader returns one. Slice S step 3 measured `npts` to be the ONLY channel that witnesses
# the march bound — dropping the `r` from `r + s_settle` left `min_phi_lp` bit-identical at all
# four ramp rates while the lengths moved — and step 3 of THIS slice found the same hole from the
# other side: a `zip` over two trajectories reports a TRUNCATED march as an unmoved one.
_ft_n = R48._ft()
_acc_n = _ft_n.accel_schedule(R48.FLIGHT, R48.LO, R48.HI, 0.25)
_acc_deg = _ft_n.accel_schedule(R48.FLIGHT, R48.LO, R48.HI, 0.02)
for _tag, _kw in (("bare", {}),
                  ("gov", dict(Tt4_max=R48.REDLINE)),
                  ("govlag", dict(Tt4_max=R48.REDLINE, tau_gov=0.2)),
                  ("accel", dict(accel=_acc_n)),
                  ("accel_deg", dict(accel=_acc_deg)),
                  ("both", dict(Tt4_max=R48.REDLINE, accel=_acc_n))):
    _traj, _ = _ft_n._fuel_ramp_march(R48.FLIGHT, R48.LO, R48.HI, R48.R, R48.SETTLE, R48.DS,
                                      _kw.get("Tt4_max"), _kw.get("tau_gov"), _kw.get("accel"))
    putd(f"N/{_tag}/npts", len(_traj))
    put(f"N/{_tag}/s_end", _traj[-1]["s"])
    put(f"N/{_tag}/mf_end", _traj[-1]["mf"])
    put(f"N/{_tag}/mf_sched_end", _traj[-1]["mf_sched"])

# =========================================================================== emit
out = open(OUT, "w", encoding="utf-8", newline="\n") if OUT else sys.stdout
out.write("# slice T oracle (" + ARM + ") — rungs 46+47+48 readers — "
          "key\tu64 bits (or an integer)\trepr\n")
for key, bits, text in ROWS:
    out.write(f"{key}\t{bits}\t{text}\n")
if OUT:
    out.close()
sys.stderr.write(f"[dump_topping/{ARM}] {len(ROWS)} values\n")
