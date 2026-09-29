"""SLICE AJ STEP 4's PROBE — rungs 83 and 84's readers, every returned value, flattened for
`rust/tests/slice_aj_corrector.rs` (rung 83) and `rust/tests/slice_aj_staircase.rs` (rung 84).

Rung 83 (`CorrectorLawTransient`, `engine.py:22645`): `corrector_read`, `corrector_step`,
`residual_shape`, `corrector_secant` — ported in `rust/src/corrector_law.rs`. Rung 84
(`StaircaseLawTransient`, `engine.py:22809`): `edge_read`, `classify`, `staircase_scan`,
`lattice_count`, `staircase_number`, `root_class` — ported in `rust/src/staircase_law.rs`. Both are
reader-only; this probe runs them on `tests/test_rung84.py`'s rig (identical to
`tests/test_rung83.py`'s) and writes every value they return.

THE READINGS. Rung 84's suite costs ~1 200 s on PyPy, so nothing here replays a grid: each reader
gets the suite's own call where one exists, plus the cheapest call that reaches a branch the
suites never take. A reading whose name has no `@rig` line ran NO march — it is a pure function
of reads taken by an earlier reading, named in its comment.

  rung 83
  * `read_ctl` — `corrector_read(0.08)` at `r = 0.5`, the read `test_a_degenerate_slope_…` steps
    from.
  * `step_ok`, `step_v5` (off `read_ctl`, `c = 0.044` and `1.0`) — the suite's two steps.
  * `read_v1` — `r = 1.0`, the EMPTY window; `step_v4` off it (`c = 0.044`), never called by the
    suite, and the misnamed `V4: kappa impure`.
  * `shape_jump` — § 3.2's jump window at `n = 3` (the suite's `n = 2` plus the midpoint, whose two
    ends are bit-equal to the suite's): a sign change AND a same-sign pair, the `continue` the
    suite's one-change windows never take.
  * `shape_cross` — § 3.4's crossing, verbatim.
  * `shape_v1` — `[0.05, 0.30]` at `r = 1.0`: one end has no `g` (`n_kappa_impure = 1`), the
    other does.
  * `secant_ok`, `secant_jump` — § 4's two secants, verbatim.
  * `secant_iter_v4` — `(0.30, 0.29)` at `r = 1.0`, `cap = 3`: the `0.30` window is open (rung
    82 step 3's finding), and the first iterate lands in an empty one — `V4: kappa impure at an
    iterate`, which the pre-flight could not drive.
  * `secant_start_v4` — `(0.30, 0.05)` at `r = 1.0`: the FIRST start pushed, the SECOND aborts.
  * `secant_s2` — identical starts: `S2`, whose `%g` of the default `1e-12` is not Rust's `{}`.
  * `secant_clamp` — bracket `(0.02, 0.035)` at `r = 0.35`, starts `0.025`/`0.03`, `cap = 2`: the
    root (~0.0371) is outside, so the iterates CLAMP; the suite never clamps.
  rung 84
  * `edge_jump_lo`, `edge_jump_hi` — rung 83 § 3.2's two `tau`s at `r = 0.25`; `classify_jump`,
    `classify_rev` (off them, forward and REVERSED — `left` non-empty only reversed).
  * `edge_cross_lo`, `edge_cross_hi` — § 3.4's at `r = 0.35`; `classify_cross` off them.
  * `edge_v1` — `r = 1.0`; `classify_v1` (off it, twice: every Option None).
  * `scan_p4` — `test_p4`'s ladder, verbatim.
  * `lattice_p3`, `lattice_fine` — `test_p3`'s window at `ds = 0.005` and `0.0025`;
    `lattice_v3` — `r = 1.0`, the misnamed `V3: an edge off the march grid`.
  * `sn_ok` — ONE edge move (§ 3.2's pair) with `test_p5`'s spacing formula at `ds = 0.005`;
    `sn_bare` (`spacing = None`), `sn_zero` (`spacing = 0.0` — truthiness, not `is None`);
    `sn_v5` at `0.0198123456` (`%g`: `0.0198123`); `sn_v2` at `r = 1.0`.
  * `root_jump` — `test_p7`'s coarse call, verbatim; `root_cross` — `r = 0.35`, a CROSSING;
    `root_v1` — `r = 1.0`; `root_v6` — `eps = 0.123456789`, `n_bisect = 2` (`%g`: `0.123457`).

OUTPUT: TSV, one line per value, in the dict's own key order —  path <TAB> token.
  token:  f:<16 hex digits, the IEEE bits>  f:nan  i:<int, signed>  b:0|1  s:<str>  n  (None)
          len:<n> before a list/tuple's items,  keys:<n> before a dict's items.
  A dict with FLOAT keys (`summands`) is written as a LIST of `[key, value]` pairs — its key
  bits pinned, its insertion order kept, and no float spelled into a path.

Run (PyPy, the repo venv, below-normal priority; ~4 min — NOT through `cmd start`, see below):
    .venv\\Scripts\\python.exe rust/oracle/probe_slice_aj_step4.py rust/oracle/slice_aj_step4_pypy.tsv
"""
import os
import platform
import struct
import sys
import time

# THE ENTRY CONTROL. The file name says PyPy and, the first time, nothing checked it: a launch
# through `cmd /c start /b /wait "…\python.exe" "probe.py"` takes the quoted interpreter as the
# window TITLE and runs the script through the `.py` association — CPython — and that oracle
# differed from the Rust port at two points, costing an hour of root-cause work against the wrong
# interpreter (plan § 5.34.4). An output named `*pypy*` is now refused off PyPy.
print(sys.version, flush=True)
if len(sys.argv) > 1 and "pypy" in os.path.basename(sys.argv[1]).lower():
    assert platform.python_implementation() == "PyPy", (
        "this probe writes a PyPy oracle and is running on %s" % platform.python_implementation())

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, REPO)
sys.path.insert(0, os.path.join(REPO, "tests"))

from test_rung84 import (FLIGHT, _rig, _kw, PI_LPC, PI_HPC, TT4, REAL, _cpg,  # noqa: E402
                         BRACKET)
from turbojet.engine import build_two_spool_turbojet  # noqa: E402


def tok(x):
    if x is None:
        return "n"
    if isinstance(x, bool):
        return "b:%d" % int(x)
    if isinstance(x, int):
        return "i:%d" % x
    if isinstance(x, float):
        if x != x:
            return "f:nan"
        return "f:%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
    if isinstance(x, str):
        return "s:" + x
    raise TypeError((type(x), x))


def flat(path, x, out):
    if isinstance(x, set):
        x = sorted(x)
    # by the FIELD's name, not by its keys: an EMPTY `summands` has no key to test, and a
    # key-type test wrote it as `keys:0` (the first run's defect, caught at the first compare)
    if isinstance(x, dict) and (path.rsplit(".", 1)[-1] == "summands"
                                or any(not isinstance(k, str) for k in x)):
        x = list(x.items())
    if isinstance(x, (list, tuple)):
        out.append((path, "len:%d" % len(x)))
        for i, y in enumerate(x):
            flat("%s.%d" % (path, i), y, out)
    elif isinstance(x, dict):
        out.append((path, "keys:%d" % len(x)))
        for k, v in x.items():
            flat("%s.%s" % (path, k), v, out)
    else:
        out.append((path, tok(x)))


dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                               nozzle_convergent=True, **REAL)

JL, JH = 0.0197750, 0.0197875          # rung 83 § 3.2's jump, r = 0.25
CL, CH = 0.037000, 0.037333            # rung 83 § 3.4's crossing, r = 0.35
T_START = (BRACKET[0] * BRACKET[1]) ** 0.5
ROOT = 0.019753906249999998            # rung 82's 13-march bisected mid at r = 0.25
SPACING = 0.005 * (0.024 - 0.016) / 0.005938   # test_p5's formula, at ds = 0.005

# (name, runs_a_march, fn(rig, store) -> value); a derived reading reads `store`
CALLS = [
    # --- rung 83
    ("read_ctl", True, lambda m, st: m.corrector_read(0.08, **_kw(0.5))),
    ("step_ok", False, lambda m, st: m.corrector_step(st["read_ctl"], 0.044)),
    ("step_v5", False, lambda m, st: m.corrector_step(st["read_ctl"], 1.0)),
    ("read_v1", True, lambda m, st: m.corrector_read(0.05, **_kw(1.0))),
    ("step_v4", False, lambda m, st: m.corrector_step(st["read_v1"], 0.044)),
    ("shape_jump", True, lambda m, st: m.residual_shape(JL, JH, n=3, **_kw(0.25))),
    ("shape_cross", True, lambda m, st: m.residual_shape(CL, CH, n=2, **_kw(0.35))),
    ("shape_v1", True, lambda m, st: m.residual_shape(0.05, 0.30, n=2, **_kw(1.0))),
    ("secant_ok", True, lambda m, st: m.corrector_secant(T_START, 1.25 * T_START, cap=6,
                                                         bracket=BRACKET, **_kw(0.35))),
    ("secant_jump", True, lambda m, st: m.corrector_secant(ROOT, 1.25 * ROOT, cap=6,
                                                           bracket=BRACKET, **_kw(0.25))),
    ("secant_iter_v4", True, lambda m, st: m.corrector_secant(0.30, 0.29, cap=3, **_kw(1.0))),
    ("secant_start_v4", True, lambda m, st: m.corrector_secant(0.30, 0.05, **_kw(1.0))),
    ("secant_s2", True, lambda m, st: m.corrector_secant(0.03, 0.03, **_kw(0.35))),
    ("secant_clamp", True, lambda m, st: m.corrector_secant(0.025, 0.03, cap=2,
                                                            bracket=(0.02, 0.035),
                                                            **_kw(0.35))),
    # --- rung 84
    ("edge_jump_lo", True, lambda m, st: m.edge_read(JL, **_kw(0.25))),
    ("edge_jump_hi", True, lambda m, st: m.edge_read(JH, **_kw(0.25))),
    ("classify_jump", False, lambda m, st: m.classify(st["edge_jump_lo"], st["edge_jump_hi"])),
    ("classify_rev", False, lambda m, st: m.classify(st["edge_jump_hi"], st["edge_jump_lo"])),
    ("edge_cross_lo", True, lambda m, st: m.edge_read(CL, **_kw(0.35))),
    ("edge_cross_hi", True, lambda m, st: m.edge_read(CH, **_kw(0.35))),
    ("classify_cross", False, lambda m, st: m.classify(st["edge_cross_lo"],
                                                      st["edge_cross_hi"])),
    ("edge_v1", True, lambda m, st: m.edge_read(0.05, **_kw(1.0))),
    ("classify_v1", False, lambda m, st: m.classify(st["edge_v1"], st["edge_v1"])),
    ("scan_p4", True, lambda m, st: m.staircase_scan(0.0190, 0.0206, 9, **_kw(0.25))),
    ("lattice_p3", True, lambda m, st: m.lattice_count(0.016, 0.024, **_kw(0.25, 0.005))),
    ("lattice_fine", True, lambda m, st: m.lattice_count(0.016, 0.024, **_kw(0.25, 0.0025))),
    ("lattice_v3", True, lambda m, st: m.lattice_count(0.004, 0.05, **_kw(1.0))),
    ("sn_ok", True, lambda m, st: m.staircase_number(JL, JH, spacing=SPACING, **_kw(0.25))),
    ("sn_bare", True, lambda m, st: m.staircase_number(JL, JH, **_kw(0.25))),
    ("sn_zero", True, lambda m, st: m.staircase_number(JL, JH, spacing=0.0, **_kw(0.25))),
    ("sn_v5", True, lambda m, st: m.staircase_number(0.0198123456, 0.0198123456, **_kw(0.25))),
    ("sn_v2", True, lambda m, st: m.staircase_number(0.05, 0.30, **_kw(1.0))),
    ("root_jump", True, lambda m, st: m.root_class(bracket=BRACKET, **_kw(0.25, 0.005))),
    ("root_cross", True, lambda m, st: m.root_class(bracket=BRACKET, **_kw(0.35, 0.005))),
    ("root_v1", True, lambda m, st: m.root_class(bracket=BRACKET, **_kw(1.0))),
    ("root_v6", True, lambda m, st: m.root_class(bracket=BRACKET, n_bisect=2,
                                                 eps=0.123456789, **_kw(0.35))),
]

only = set(sys.argv[2:])
out, store = [], {}
for name, marches, fn in CALLS:
    if only and name not in only:
        continue
    m = _rig(dsg)
    t0 = time.time()
    r = fn(m, store)
    store[name] = r
    flat(name, r, out)
    if marches:
        # the rig after the reader: `_split_march`'s `_with_air` must have restored the knob
        flat(name + "@rig", dict(lag_coord=m._lag_coord, sm_air=m._sm_air), out)
    print("%-16s %6.1f s  %d lines" % (name, time.time() - t0, len(out)), flush=True)

with open(sys.argv[1], "w", newline="\n") as fh:
    for p, t in out:
        fh.write("%s\t%s\n" % (p, t))
print("wrote", len(out), "lines to", sys.argv[1], flush=True)
