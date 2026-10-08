# -*- coding: utf-8 -*-
"""Slice AI pre-flight: is `_with_coord`'s 74 -> 79 NAME REUSE observable BY VALUE?

Rung 74's `demand_gains` pins `m._lag_coord = "clip"` by ASSIGNMENT and then enters
`m._with_coord("demand", m._demand_gains_at, ...)` -- a DISPATCH (engine.py:18278). On a rung-79
machine that dispatch lands on rung 79's body, which writes `_phi_ref`, not `_lag_coord`.

Run `demand_gains` at the fingerprint's r74 settings on machines of rungs 74, 78, 79 and 80,
same everything, and diff. A CONTROL arm (78) must agree with 74 bit for bit, else the
difference is not attributable to the setter.
"""
import io, sys, time, os
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
sys.path.insert(0, r"W:/Claude_projects/jet engine")
sys.path.insert(0, r"W:/Claude_projects/jet engine/tests")
import test_numeric_fingerprint as T
from turbojet import engine as E

EVERY = int(os.environ.get("EVERY", "16"))
PHI = float(os.environ.get("PHI", "0.80"))
calls = {}
for cls in (E.DemandCoordinateTransient, E.StateCoordinateTransient):
    body = cls.__dict__["_with_coord"]

    def make(body, owner):
        def w(self, *a, **kw):
            k = (type(self).__name__, owner)
            calls[k] = calls.get(k, 0) + 1
            return body(self, *a, **kw)
        return w
    setattr(cls, "_with_coord", make(body, cls.__name__))


SEEN = []
_dga = E.DemandCoordinateTransient._demand_gains_at


def _spy(self, *a, **kw):
    SEEN.append((type(self).__name__, getattr(self, "_lag_coord", None),
                 getattr(self, "_phi_ref", "<no attr>")))
    return _dga(self, *a, **kw)


E.DemandCoordinateTransient._demand_gains_at = _spy
QSEEN = []
_qga = E.AppliedReferenceTransient.__dict__["_quad_gains_at"]   # rung 73's; every rung >= 73 resolves here


def _qspy(self, *a, **kw):
    QSEEN.append((type(self).__name__, getattr(self, "_lag_coord", None),
                  getattr(self, "_phi_ref", "<no attr>")))
    return _qga(self, *a, **kw)


for _c in [c for c in vars(E).values() if isinstance(c, type) and "_quad_gains_at" in c.__dict__]:
    print("   _quad_gains_at defined on", _c.__name__)
E.AppliedReferenceTransient._quad_gains_at = _qspy
INSCOPE = []
CTR = ("_coord_hits", "_coord_binds", "_coord_fb_phi", "_coord_fb_inc",
       "_coord_calls_phi", "_coord_calls_inc")


def ctr():
    return tuple(getattr(E.StateCoordinateTransient, k) for k in CTR)


def run(cls):
    SEEN.clear()
    c0 = ctr()
    m = T._s3_rig(cls, _lag_coord="demand", _ref_law="sched")
    t0 = time.time()
    d = m.demand_gains(T.FLIGHT, T._S3_LO, T._S3_HI, T._S3_TT4MAX, phi_lim=PHI,
                       taus=T._S3_TAUS, inc=False, r=T._S3_R, s_settle=T._S3_SETTLE,
                       ds=T._S3_DS, v_max=T._S3_VMAX, every=EVERY)
    c1 = ctr()
    print(f"   {cls.__name__}: rung-79 counters moved by "
          f"{dict((k, b - a) for k, a, b in zip(CTR, c0, c1))}")
    print(f"   (machine, _lag_coord, _phi_ref) inside the scope: {sorted(set(SEEN))} x{len(SEEN)}")
    from collections import Counter
    print(f"   ... at every _quad_gains_at entry: {dict(Counter(QSEEN))}")
    return d, time.time() - t0


def flat(o, p, out):
    if isinstance(o, dict):
        for k, v in o.items():
            flat(v, f"{p}/{k}", out)
    elif isinstance(o, (list, tuple)):
        out[p + "#n"] = len(o)
        for i, v in enumerate(o):
            flat(v, f"{p}/{i}", out)
    else:
        out[p] = o
    return out


res = {}
for cls in (E.DemandCoordinateTransient, E.ResidualGaugeTransient,
            E.StateCoordinateTransient, E.SplitWallTransient):
    d, dt = run(cls)
    res[cls.__name__] = flat(d, "", {})
    print(f"{cls.__name__:<28} {dt:6.1f}s  keys={len(res[cls.__name__])}")
print("with_coord dispatches (machine, body):", calls)

base = res["ResidualGaugeTransient"]   # the advisor: diff against 78, where the break is introduced
for nm, r in res.items():
    if nm == "ResidualGaugeTransient":
        continue
    keys = set(base) | set(r)
    diff = [k for k in sorted(keys) if base.get(k, "<absent>") != r.get(k, "<absent>")
            and not (isinstance(base.get(k), float) and isinstance(r.get(k), float)
                     and base[k] != base[k] and r[k] != r[k])]
    print(f"\n{nm}: {len(diff)} of {len(keys)} keys differ from rung 78")
    for k in diff[:25]:
        print(f"   {k:<40} r74={base.get(k, '<absent>')!r:<28} here={r.get(k, '<absent>')!r}")
