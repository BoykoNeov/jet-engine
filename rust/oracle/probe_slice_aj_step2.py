"""SLICE AJ STEP 2's PROBE — rung 81's two readers, every returned value, flattened for
`rust/tests/slice_aj_clock.rs`.

Rung 81 (`AuthorityClockTransient`, `engine.py:21795`) is reader-only: `_central`,
`_criterion_at`, `authority_clock`, `_tau_f_inert`, `authority_mask`. The Rust port lives in
`rust/src/authority_clock.rs`. This probe runs the readers on `tests/test_rung81.py`'s own rig
and writes every value the two public readers return.

THE CALLS, and why each is in it:
  * `clock`, `mask` — `test_rung81.py`'s two module fixtures, verbatim (defaults otherwise).
  * `clock_latched` — `coords=("demand-latched",)`. A latched march records DEMAND-shaped points
    but `_criterion_at` takes its ELSE (clip-form) branch, because it branches on the coordinate
    STRING. The fixtures never run it, so without this call a port branching on the point's
    shape would pass.
  * `clock_single` — one `tau_f`, `coords=("clip",)`: `_tau_f_inert`'s `None` column (fewer than
    two rows) and a control loop that never meets `tau_f == 0.05` (an EMPTY `control_clip_shared`).
  * `mask_clip` — `coord="clip"`, `every=2`: the stride, and the mask in the other coordinate.

OUTPUT: TSV, one line per value, in the dict's own key order —  path <TAB> token.
  token:  f:<16 hex digits, the IEEE bits>  i:<int>  b:0|1  s:<str>  n  (None)
          len:<n> before a list/tuple's items,  keys:<n> before a dict's items.
A set is written as its sorted list. The Rust test writes the same lines from its own structs
and compares them one for one, so a missing field, an extra one, or a field out of Python's
order fails at its path.

Run (PyPy, the repo venv; ~3 min):
    .venv\\Scripts\\python.exe rust/oracle/probe_slice_aj_step2.py rust/oracle/slice_aj_step2_pypy.tsv
"""
import os
import struct
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, REPO)
sys.path.insert(0, os.path.join(REPO, "tests"))

from test_rung81 import (FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, _rig,  # noqa: E402
                         PI_LPC, PI_HPC, TT4, REAL, _cpg)
from turbojet.engine import build_two_spool_turbojet  # noqa: E402


def tok(x):
    if x is None:
        return "n"
    if isinstance(x, bool):
        return "b:%d" % int(x)
    if isinstance(x, int):
        return "i:%d" % x
    if isinstance(x, float):
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
CALLS = [
    ("clock", lambda m: m.authority_clock(FLIGHT, LO, HI, TT4_MAX, **KW)),
    ("mask", lambda m: m.authority_mask(FLIGHT, LO, HI, TT4_MAX, **KW)),
    ("clock_latched", lambda m: m.authority_clock(
        FLIGHT, LO, HI, TT4_MAX, tau_fs=(0.05, 0.20), tau_govs=(0.05,),
        coords=("demand-latched",), **KW)),
    ("clock_single", lambda m: m.authority_clock(
        FLIGHT, LO, HI, TT4_MAX, tau_fs=(0.20,), tau_govs=(0.05,), coords=("clip",), **KW)),
    ("mask_clip", lambda m: m.authority_mask(FLIGHT, LO, HI, TT4_MAX, coord="clip", every=2,
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
