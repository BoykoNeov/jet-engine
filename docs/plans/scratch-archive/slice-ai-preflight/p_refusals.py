# -*- coding: utf-8 -*-
"""Slice AI pre-flight: DRIVE each of rungs 79/80's seven refusals, or report it unreached.

Each drive is the cheapest ADMISSIBLE-or-direct call that should reach the assert; the result
is the message's first 70 chars (so the needle can be checked for SHARPNESS), or what happened
instead. A refusal is 'reachable' here only if the assert with that exact message fired.
"""
import io, sys, math, traceback
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
sys.path.insert(0, r"W:/Claude_projects/jet engine")
sys.path.insert(0, r"W:/Claude_projects/jet engine/tests")
import test_numeric_fingerprint as T
from turbojet import engine as E
from turbojet.engine import SurgeLimiter

SRC = open(r"W:/Claude_projects/jet engine/turbojet/engine.py", encoding="utf-8").read()


def rig(cls, **kw):
    return T._s3_rig(cls, _lag_coord="demand", _ref_law="sched", **kw)


def drive(label, fn):
    try:
        out = fn()
        print(f"  {label:<34} NO REFUSAL -> returned {type(out).__name__}")
    except AssertionError as e:
        msg = str(e)
        print(f"  {label:<34} REFUSED: {msg[:70]!r}")
    except Exception as e:
        print(f"  {label:<34} OTHER {type(e).__name__}: {str(e)[:90]!r}")


FL = T.FLIGHT
sm = T._S3_SM
print("=== rung 79 ===")
m = rig(E.StateCoordinateTransient)
m._phi_ref, m._gauge_k = "incidence", 2.0
drive(":20910 incidence x gauge", lambda: m._cap_fuel(FL, 1.0, 1.0, 0.01, None,
                                                      SurgeLimiter(spool="lp", phi_lim=0.8)))
m = rig(E.StateCoordinateTransient)
m._phi_ref, m._gauge_k = "demand", 2.0          # the THIRD value, the advisor's item 2
drive(":20910 via _phi_ref='demand'", lambda: m._cap_fuel(FL, 1.0, 1.0, 0.01, None,
                                                         SurgeLimiter(spool="lp", phi_lim=0.8)))
m = rig(E.StateCoordinateTransient)
drive(":21285 forced, binding, phi_lim=50", lambda: m._forced_cap(
    FL, 1.0, 1.0, 0.01, SurgeLimiter(spool="lp", phi_lim=50.0), "phi"))
drive(":21301 forced, slack, phi_lim=1e-3", lambda: m._forced_cap(
    FL, 1.0, 1.0, 0.01, SurgeLimiter(spool="lp", phi_lim=1e-3), "phi"))

print("=== rung 80 ===")
m = rig(E.SplitWallTransient)
TAU, TAU_S, VMAX, TT4 = 0.05, 0.05, T._S3_VMAX, T._S3_TT4MAX
drive(":21467 sm_air < sm", lambda: m._with_air(sm - 0.05, m._shared_rig, sm, TAU, TAU_S,
                                                VMAX, TT4))
nxt = math.nextafter(sm, 1.0)
drive(":21486 sm_air = sm + 1 ulp", lambda: m._with_air(nxt, m._shared_rig, sm, TAU, TAU_S,
                                                        VMAX, TT4))
# how many ulps above sm before the two BUILT walls separate?
for k in (1, 2, 4, 8, 16, 64, 256):
    x = sm
    for _ in range(k):
        x = math.nextafter(x, 1.0)
    try:
        m._with_air(x, m._shared_rig, sm, TAU, TAU_S, VMAX, TT4)
        print(f"     sm + {k:>3} ulp: split reached the plant")
        break
    except AssertionError as e:
        print(f"     sm + {k:>3} ulp: REFUSED ({str(e)[:40]!r})")
nov = rig(E.SplitWallTransient)
nov.bleed_lim = None
drive(":21551 _split_row, valve-less m", lambda: E.SplitWallTransient._split_row(
    nov, None, [dict(Tt4=1000.0, b=0.0, v=0.0, phi_lp=0.8, required_fuel=0.0,
                     required_gov=0.0)], 1000.0, 0.75, None, "demand"))
m = rig(E.SplitWallTransient)
drive(":21647 split_arrest, bad walls", lambda: m.split_arrest(FL, 1000.0, 1400.0, 1200.0,
                                                               phi_lim_lo=0.78))

print("\n=== needle sharpness: the suite's two needles ===")
for nd in ("REFUSED", "AIRFLOW wall sits AT or ABOVE"):
    print(f"  {nd!r}: {SRC.count(nd)} occurrences in engine.py")
