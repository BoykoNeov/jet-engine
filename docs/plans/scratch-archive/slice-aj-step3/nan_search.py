import os, sys, time
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, os.path.join(REPO, "tests"))
from test_rung82 import FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, _rig, PI_LPC, PI_HPC, TT4, REAL, _cpg, V_MAX
from turbojet.engine import build_two_spool_turbojet
dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
m = _rig(dsg)
def sc(tf, r=0.35, pl=PHI_FUEL, tg=0.05, ds=0.005):
    s = m._threshold_scan(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=pl,
        phi_air=PHI_AIR, tau_f=tf, tau_gov=tg, tau_q=0.05, tau_s=0.05, r=r, s_settle=1.2,
        ds=ds, v_max=V_MAX, inc=False)
    print(r, tf, pl, tg, ds, s["kappa"], s["n_slope_excluded"], s["n_riding4"], s["n_fuel"], flush=True)
for r in (0.2, 0.7, 0.9):
    for tf in (0.02, 0.05, 0.2):
        sc(tf, r=r)
for pl in (0.745, 0.755):
    for tf in (0.02, 0.2):
        sc(tf, pl=pl)
for tg in (0.02, 0.2):
    for tf in (0.02, 0.2):
        sc(tf, tg=tg)
sc(0.05, r=0.25, ds=0.0025)
