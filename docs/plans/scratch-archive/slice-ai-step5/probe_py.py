"""Slice AI step 5 probe: P5's ulp band at engine.py:21486 and J6's slack forced_cap success.
Run on the repo's PyPy venv. Prints bits; the Rust gates are written from this output."""
import math
import struct
import sys

sys.path.insert(0, r"W:\Claude_projects\jet engine")

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    StateCoordinateTransient, SplitWallTransient, SurgeLimiter, BleedLimiter, StatorLimiter,
    ResidualGaugeTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def bits(x):
    return "0x%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]


def cpg():
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=g, cp_t=cp, R_t=(g - 1.0) / g * cp, hPR=42.8e6)


design = build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, FLIGHT.p0, nozzle_convergent=True,
                                  **REAL)

# ---- P5: engine.py:21486's ulp band, on test_rung80.py's rig
sm = 0.80 / FLOOR - 1.0
m = SplitWallTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                       bleed_lim=BleedLimiter.from_margin(LP, 0.10, sm, tau=0.05),
                       stator_lim=StatorLimiter.from_margin(LP, 0.20, sm, tau=0.05))
m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
print("P5 sm", repr(sm), bits(sm))
x = sm
for k in range(0, 5):
    try:
        rig, surge, _ = m._with_air(x, m._shared_rig, sm, 0.05, 0.05, 0.20, 1200.0)
        w = rig._walls_of(rig, surge)
        print("P5 k=%d sm_air=%s BUILT phi_lim=%s %s phi_air=%s %s valve=%s stator=%s" % (
            k, bits(x), repr(w["phi_lim"]), bits(w["phi_lim"]), repr(w["phi_air"]),
            bits(w["phi_air"]), bits(w["phi_valve"]), bits(w["phi_stator"])))
    except AssertionError as e:
        print("P5 k=%d sm_air=%s REFUSED %s" % (k, bits(x), str(e)[:70]))
    print("   after: m._sm_air =", m._sm_air)
    x = math.nextafter(x, math.inf)

# ---- J6: forced_cap's SLACK arm, on a valve-off rung-79 rig at SLACK = (0.8, 0.9, 0.02)
c = StateCoordinateTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
surge = SurgeLimiter(spool="lp", phi_lim=0.80)
for st in ((0.8, 0.9, 0.02), (0.8, 0.9, 0.035)):
    a, h, ms = st
    g0 = c._phi_residual(FLIGHT, a, h, surge, "phi")(ms)
    print("J6 state", st, "Gs(ms) =", repr(g0), "slack" if g0 <= 0.0 else "binding")
    for coord in ("phi", "incidence"):
        wf = c._forced_cap(FLIGHT, a, h, ms, surge, coord)
        wp = c._phi_cap(FLIGHT, a, h, ms, surge, coord)
        print("J6   %-9s forced=%s %s  phi_cap=%s %s  same=%s" % (
            coord, repr(wf), bits(wf), repr(wp), bits(wp), wf == wp))

# ---- the two forced refusals, on the same valve-off rig at SLACK's (a, h, ms)
for lim in (50.0, 1e-3):
    s2 = SurgeLimiter(spool="lp", phi_lim=lim)
    for coord in ("phi", "incidence"):
        try:
            w = c._forced_cap(FLIGHT, 0.8, 0.9, 0.02, s2, coord)
            print("REF phi_lim=%g %s RETURNED %r" % (lim, coord, w))
        except AssertionError as e:
            print("REF phi_lim=%g %s REFUSED %s" % (lim, coord, str(e)))
