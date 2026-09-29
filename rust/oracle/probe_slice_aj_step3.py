"""SLICE AJ STEP 3's PROBE — rung 82's readers, every returned value, flattened for
`rust/tests/slice_aj_threshold.rs`.

Rung 82 (`ThresholdLawTransient`, `engine.py:22148`) is reader-only: `_scan_cells`, `_hats`,
`_threshold_scan`, `_scan`, `_bisect`, `threshold_law`, `_threshold_row`, `threshold_reference`,
`threshold_terms`. The Rust port lives in `rust/src/threshold_law.rs`. This probe runs the readers
on `tests/test_rung82.py`'s own rig and writes every value they return.

THE CALLS, and why each is in it:
  * `law`, `ref`, `terms` — `test_rung82.py`'s three module fixtures, verbatim.
  * `scan_ctl`, `scan_v1` — the two direct `_threshold_scan` calls the suite makes: rung 81's
    identity cell, and `r = 1.0`, where the window is EMPTY (every Option None, `kappa = []`).
  * `bisect_v3_hi`, `bisect_v3_lo` — `test_V3_censors_the_wall_on_both_sides`, verbatim: V3's
    `below` and `above`.
  * `bisect_v3_g` — V3 with `lo = 0.0123456789`. On the default bracket `%g` and Rust's `{}`
    print the SAME string, so only an argument like this one shows the message goes through `%g`.
  * `bisect_v1` — `_bisect` at `r = 1.0` on `[0.004, 0.05]`, where BOTH ends have an empty
    window. On the default bracket they do not: at `r = 1.0` the `0.30` end has 14 riding points,
    all fuel, so V1 fires off the low end alone and a bisection that tested the straddle BEFORE
    the window would return the same V1 there. Only both-ends-empty tells the two orders apart.
  * `law_fine` — the `ds_fine` leg the `law` fixture skips, and a VOID row (`r = 1.0`, V1).
  * `ref_void` — `threshold_reference`'s void shape (`r = 1.0`).
  * `terms_void` — UNSORTED `phi_lims` with one wall censored (V3): the sorted iteration beside
    the as-passed field, the void `at` shape, `n_void > 0`, and `p4 = p5 = None`.

OUTPUT: TSV, one line per value, in the dict's own key order —  path <TAB> token.
  token:  f:<16 hex digits, the IEEE bits>  f:nan (any NaN — its sign and payload are not
          visible to Python code, so they are not pinned)  i:<int>  b:0|1  s:<str>  n  (None)
          len:<n> before a list/tuple's items,  keys:<n> before a dict's items.

Run (PyPy, the repo venv, below-normal priority; ~10 min):
    .venv\\Scripts\\python.exe rust/oracle/probe_slice_aj_step3.py rust/oracle/slice_aj_step3_pypy.tsv
"""
import os
import struct
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, REPO)
sys.path.insert(0, os.path.join(REPO, "tests"))

from test_rung82 import (FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, _rig,  # noqa: E402
                         PI_LPC, PI_HPC, TT4, REAL, _cpg, RS, N_BISECT, R81, V_MAX)
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
KW = dict(phi_lim=PHI_FUEL, phi_air=PHI_AIR)
SCAN = dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL,
            phi_air=PHI_AIR, tau_f=0.05, tau_gov=0.05, tau_q=0.05, tau_s=0.05,
            s_settle=1.2, ds=0.005, v_max=V_MAX, inc=False)
V3 = dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_air=PHI_AIR,
          tau_gov=0.05, tau_q=0.05, tau_s=0.05, s_settle=1.2, v_max=V_MAX,
          inc=False, r=0.35, ds=0.005)


def fuel(s):
    return s["n_fuel"] > 0


CALLS = [
    ("law", lambda m: m.threshold_law(FLIGHT, LO, HI, TT4_MAX, rs=RS, n_bisect=N_BISECT,
                                      ds_fine=None, **KW)),
    ("ref", lambda m: m.threshold_reference(FLIGHT, LO, HI, TT4_MAX, n_bisect=N_BISECT, **KW)),
    ("terms", lambda m: m.threshold_terms(FLIGHT, LO, HI, TT4_MAX, n_bisect=N_BISECT,
                                          phi_lims=(0.745, 0.750, 0.755), **KW)),
    ("scan_ctl", lambda m: m._threshold_scan(**dict(SCAN, r=R81))),
    ("scan_v1", lambda m: m._threshold_scan(**dict(SCAN, r=1.0))),
    ("bisect_v3_hi", lambda m: m._bisect(fuel, 0.004, 0.30, 4, **dict(V3, phi_lim=0.760))),
    ("bisect_v3_lo", lambda m: m._bisect(fuel, 0.004, 0.30, 4, **dict(V3, phi_lim=0.740))),
    ("bisect_v3_g", lambda m: m._bisect(fuel, 0.0123456789, 0.30, 4, **dict(V3, phi_lim=0.760))),
    ("bisect_v1", lambda m: m._bisect(fuel, 0.004, 0.05, 4, **dict(V3, phi_lim=PHI_FUEL,
                                                                    r=1.0))),
    ("law_fine", lambda m: m.threshold_law(FLIGHT, LO, HI, TT4_MAX, rs=(0.25, 1.0),
                                           n_bisect=N_BISECT, ds_fine=0.0025, **KW)),
    ("ref_void", lambda m: m.threshold_reference(FLIGHT, LO, HI, TT4_MAX, r=1.0,
                                                 n_bisect=N_BISECT, **KW)),
    ("terms_void", lambda m: m.threshold_terms(FLIGHT, LO, HI, TT4_MAX, n_bisect=4,
                                               phi_lims=(0.760, 0.750), tau_govs=(0.05,),
                                               **KW)),
]

out = []
for name, fn in CALLS:
    m = _rig(dsg)
    t0 = time.time()
    r = fn(m)
    flat(name, r, out)
    # the rig after the reader: `_split_march`'s `_with_air` must have restored the knob
    flat(name + "@rig", dict(lag_coord=m._lag_coord, sm_air=m._sm_air), out)
    print("%-14s %6.1f s  %d lines" % (name, time.time() - t0, len(out)), flush=True)

with open(sys.argv[1], "w", newline="\n") as fh:
    for p, t in out:
        fh.write("%s\t%s\n" % (p, t))
print("wrote", len(out), "lines to", sys.argv[1], flush=True)
