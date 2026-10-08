import os, sys, time
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, os.path.join(REPO, "tests"))
from test_rung82 import FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, _rig, PI_LPC, PI_HPC, TT4, REAL, _cpg, V_MAX
from turbojet.engine import build_two_spool_turbojet
dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
m = _rig(dsg)
for r in (0.25, 0.35, 0.50):
    for tf in (0.004, 0.01, 0.02, 0.03, 0.05, 0.08, 0.12, 0.2, 0.3):
        t0 = time.time()
        s = m._threshold_scan(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL,
            phi_air=PHI_AIR, tau_f=tf, tau_gov=0.05, tau_q=0.05, tau_s=0.05, r=r, s_settle=1.2,
            ds=0.005, v_max=V_MAX, inc=False)
        print(r, tf, s["kappa"], s["n_slope_excluded"], s["n_riding4"], s["n_scored"], s["n_fuel"],
              s["h"], "%.2fs" % (time.time() - t0), flush=True)
