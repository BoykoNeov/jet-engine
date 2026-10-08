"""Slice AI step 4 probe: rung 80's four readers on tests/test_rung80.py's rig.

Emits py.json: every value as float.hex (None as null), each _split_march call's (sm, sm_air)
and the built machine's (lag_coord, phi_ref, sm_air) readback, rung 79's six class counters
around each reader, the rig's _sm_air after each reader, and a branch census."""
import json
import sys
import time

sys.path.insert(0, r"W:\Claude_projects\jet engine")
sys.path.insert(0, r"W:\Claude_projects\jet engine\tests")

from test_rung80 import FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, _rig  # noqa: E402
from test_rung80 import PI_LPC, PI_HPC, TT4, REAL, _cpg  # noqa: E402
from turbojet.engine import (build_two_spool_turbojet, SplitWallTransient as W,  # noqa: E402
                             StateCoordinateTransient as S)

NAMES = ("_coord_hits", "_coord_binds", "_coord_fb_phi", "_coord_fb_inc",
         "_coord_calls_phi", "_coord_calls_inc")


def snap():
    return [getattr(S, n) for n in NAMES]


def enc(x):
    if x is None:
        return None
    if isinstance(x, bool):
        return x
    if isinstance(x, int):
        return x
    if isinstance(x, float):
        return float(x).hex()
    if isinstance(x, str):
        return x
    if isinstance(x, (list, tuple)):
        return [enc(y) for y in x]
    if isinstance(x, dict):
        return {str(k): enc(v) for k, v in x.items()}
    raise TypeError(type(x))


dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                               nozzle_convergent=True, **REAL)

marches = []
orig_sm = W._split_march


def split_march(self, flight, Tt4_lo, Tt4_hi, Tt4_max, phi_lim, phi_air, coord, *a, **kw):
    ps = self.map_lp_design.phi_surge
    sm = phi_lim / ps - 1.0
    sm_air = None if phi_air is None else phi_air / ps - 1.0
    out = orig_sm(self, flight, Tt4_lo, Tt4_hi, Tt4_max, phi_lim, phi_air, coord, *a, **kw)
    m = out[0]
    marches.append(dict(coord=coord, phi_lim=enc(phi_lim), phi_air=enc(phi_air), sm=enc(sm),
                        sm_air=enc(sm_air), lag_coord=m._lag_coord, phi_ref=m._phi_ref,
                        m_sm_air=enc(m._sm_air), self_sm_air_inside=enc(self._sm_air),
                        npts=len(out[3])))
    return out


W._split_march = split_march

out = {"readers": {}}
t_all = time.time()
calls = [
    ("liveness", lambda m: m.split_liveness(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL)),
    ("arrest", lambda m: m.split_arrest(FLIGHT, LO, HI, TT4_MAX, phi_lim_lo=PHI_FUEL)),
    ("gains_clip", lambda m: m.split_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                           phi_airs=(None, 0.77, 0.80), coord="clip")),
    ("gains_demand", lambda m: m.split_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                             phi_airs=(None, 0.77, 0.80), coord="demand")),
    ("saturation", lambda m: m.split_saturation(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL)),
]
for name, fn in calls:
    m = _rig(dsg)
    k0 = len(marches)
    c0 = snap()
    t0 = time.time()
    r = fn(m)
    dt = time.time() - t0
    c1 = snap()
    out["readers"][name] = dict(result=enc(r), seconds=dt,
                                counters=[y - x for x, y in zip(c0, c1)],
                                rig_after=(m._lag_coord, m._phi_ref, enc(m._sm_air)),
                                marches=marches[k0:])
    print(name, "%.1f s" % dt, "counters", out["readers"][name]["counters"], flush=True)

out["seconds"] = time.time() - t_all
json.dump(out, open(r"W:\temp\claude\slice-ai-step4\py.json", "w"), indent=1)
print("TOTAL %.1f s" % out["seconds"], flush=True)
