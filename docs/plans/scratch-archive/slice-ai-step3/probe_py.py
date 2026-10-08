"""Slice AI step 3 probe: rung 79's coord_march / coord_forced on tests/test_rung79.py's rig.

Every float as float.hex; rung 79's six class counters snapshotted around each reader AND around
each inner phase of coord_march (the two schedules, the phi march), so the port's counter
readback can be pinned where Python resets them and where it does not. The probe log is captured
by wrapping `_with_probe` and digested (FNV-1a 64 over the IEEE bits of every row, in order)."""
import json
import struct
import sys
import time

sys.path.insert(0, r"W:\Claude_projects\jet engine")
sys.path.insert(0, r"W:\Claude_projects\jet engine\tests")

from test_rung79 import FLIGHT, LO, HI, TT4_MAX, _rig  # noqa: E402
from test_rung79 import PI_LPC, PI_HPC, TT4, REAL, _cpg  # noqa: E402
from turbojet.engine import build_two_spool_turbojet, StateCoordinateTransient as S  # noqa: E402

NAMES = ("_coord_hits", "_coord_binds", "_coord_fb_phi", "_coord_fb_inc",
         "_coord_calls_phi", "_coord_calls_inc")


def snap():
    return [getattr(S, n) for n in NAMES]


def h(x):
    return None if x is None else float(x).hex()


def bits(x):
    return struct.unpack("<Q", struct.pack("<d", x))[0]


def fnv(words):
    v = 0xcbf29ce484222325
    for w in words:
        for i in range(8):
            v ^= (w >> (8 * i)) & 0xff
            v = (v * 0x100000001b3) & 0xffffffffffffffff
    return v


design = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)
out = {}

# ---- capture the log and the per-phase counters by wrapping the class methods -------------
captured = {}
orig_probe = S._with_probe
orig_accel = S.accel_for
phases = []


def with_probe(self, fn, *a, **kw):
    o, log = orig_probe(self, fn, *a, **kw)
    captured["log"] = list(log)
    return o, log


def accel_for(self, *a, **kw):
    c0 = snap()
    r = orig_accel(self, *a, **kw)
    phases.append(("accel_for", self._phi_ref, [y - x for x, y in zip(c0, snap())]))
    return r


S._with_probe = with_probe
S.accel_for = accel_for
t0 = time.time()
c0 = snap()
mr = _rig(design).coord_march(FLIGHT, LO, HI, TT4_MAX)
c1 = snap()
t_march = time.time() - t0
S._with_probe = orig_probe
S.accel_for = orig_accel

log = captured["log"]
words = []
for (a_c, p_p, p_i, ms, fb) in log:
    words += [bits(a_c), bits(p_p), bits(p_i), bits(ms), 1 if fb else 0]
out["march_counters_after"] = c1           # absolute: coord_march resets mid-way
out["march_counters_delta"] = [b - a for a, b in zip(c0, c1)]
out["phases"] = phases
out["log_n"] = len(log)
out["log_fnv"] = "0x%016x" % fnv(words)
out["log_first"] = [h(x) if isinstance(x, float) else x for x in log[0]]
out["log_last"] = [h(x) if isinstance(x, float) else x for x in log[-1]]
out["n_inf_accel"] = sum(1 for x in log if x[0] == float("inf"))
out["march"] = {k: (h(v) if isinstance(v, float) else v) for k, v in mr.items()}
out["t_march_s"] = t_march

c0 = snap()
t0 = time.time()
fo = _rig(design).coord_forced(FLIGHT, LO, HI, TT4_MAX)
out["t_forced_s"] = time.time() - t0
out["forced_counters"] = [b - a for a, b in zip(c0, snap())]
out["forced"] = dict(
    n=fo["n"], n_binding=fo["n_binding"], d_forced=h(fo["d_forced"]),
    d_forced_med=h(fo["d_forced_med"]), n_same_float=fo["n_same_float"],
    d_shipped=h(fo["d_shipped"]),
    rows=[{k: (h(v) if isinstance(v, float) else v) for k, v in r.items()} for r in fo["rows"]])

json.dump(out, open(r"W:\temp\claude\slice-ai-step3\py.json", "w"), indent=1)
print("march", {k: v for k, v in mr.items()})
print("march counters after/delta", out["march_counters_after"], out["march_counters_delta"])
print("phases", phases)
print("log", out["log_n"], out["log_fnv"], "inf accel", out["n_inf_accel"])
print("forced", {k: v for k, v in fo.items() if k != "rows"})
print("forced counters", out["forced_counters"])
print("times", t_march, out["t_forced_s"])
