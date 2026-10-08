import sys, time
print(sys.version.replace("\n", " "), flush=True)
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, REPO + r"\tests")
from test_rung84 import FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, PI_LPC, PI_HPC, TT4, REAL, _cpg, LP, B, TAU, V_MAX, TAU_S, FLOOR
from turbojet.engine import (build_two_spool_turbojet, StaircaseLawTransient, BleedLimiter,
    StatorLimiter, StatorIncidenceLimiter)
D = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
def rig(inc):
    sm = 0.80 / FLOOR - 1.0
    m = StaircaseLawTransient(D, FLIGHT, 1.0, map_lp=LP, map_hp=__import__("test_rung84").HP, rho=1.0,
        bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
        stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc else None),
        stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m
def kw(r, inc):
    return dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL, phi_air=PHI_AIR,
                tau_gov=0.05, tau_q=0.05, tau_s=0.05, r=r, s_settle=1.2, ds=0.005, v_max=V_MAX, inc=inc)
for inc in (False, True):
    m = rig(inc)
    for r in (0.25, 0.35, 0.5):
        for tf in (0.004, 0.02, 0.05, 0.30):
            t0 = time.time()
            s = m._threshold_scan(tau_f=tf, **kw(r, inc))
            print("inc=%d r=%.2f tf=%.3f  n_fuel=%s n_riding4=%s h=%s kappa_pure=%s  %.1fs" % (
                inc, r, tf, s.get("n_fuel"), s.get("n_riding4"), s.get("h"), s.get("kappa_pure"), time.time()-t0), flush=True)
    t0 = time.time()
    c = m.authority_clock(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL, phi_air=PHI_AIR, tau_fs=(0.05, 0.20), tau_govs=(0.05,), inc=inc)
    print("clock keys", list(c)[:12], "%.1fs" % (time.time()-t0))
    print(" rows", [(r.get("coord"), r.get("tau_f"), r.get("n_riding4"), r.get("n_scored")) for r in c["rows"]], flush=True)
