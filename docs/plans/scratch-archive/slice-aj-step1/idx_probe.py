"""Record the INDICES rung 81's `id(p)` round trip keeps, on test_rung81.py's rig, for the
riding4_idx gate. Mirrors authority_clock's loop (engine.py:21940-21946) exactly."""
import sys
sys.path.insert(0, r"W:\Claude_projects\jet engine")
sys.path.insert(0, r"W:\Claude_projects\jet engine\tests")
import test_rung81 as T
from turbojet.engine import build_two_spool_turbojet

design = build_two_spool_turbojet(T._cpg(), T.PI_LPC, T.PI_HPC, T.TT4, T.FLIGHT.p0,
                                  nozzle_convergent=True, **T.REAL)
for name, taus, coord in (("matched_demand", T.MATCHED, "demand"),
                          ("slow_fuel_demand", T.SLOW_FUEL, "demand"),
                          ("slow_fuel_clip", T.SLOW_FUEL, "clip")):
    m = T._rig(design)
    mm, surge, lag, traj = T._march(m, taus, coord)
    ride = m._riding4(traj, mm.bleed_lim.b_max)
    seen = {id(p) for p in ride}
    idx = [i for i, p in enumerate(traj) if id(p) in seen]
    assert len(set(id(p) for p in traj)) == len(traj), "traj repeats an object"
    interior = [i for i in idx if 0 < i < len(traj) - 1]
    print(name, len(traj), len(idx), len(interior), idx)
