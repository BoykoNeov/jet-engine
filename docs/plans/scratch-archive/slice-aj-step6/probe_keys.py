import sys
print(sys.version.replace("\n", " "))
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, REPO + r"\tests")
from test_rung84 import FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, PI_LPC, PI_HPC, TT4, REAL, _cpg, _rig
from turbojet.engine import build_two_spool_turbojet
m = _rig(build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL))
traj, ride, cells = m._scan_cells(FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, 0.02, 0.05, 0.05, 0.05, 0.25, 1.2, 0.005, 0.20, False)
print(len(traj), [(k, type(v).__name__) for k, v in traj[0].items()])
print(set(tuple(p.keys()) == tuple(traj[0].keys()) for p in traj))
print({p["v_regime"] for p in traj}, {p["branch"] for p in traj}, {p["authority"] for p in traj})
