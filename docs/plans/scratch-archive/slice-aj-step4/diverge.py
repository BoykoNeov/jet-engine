import os, sys, struct
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, os.path.join(REPO, "tests"))
from test_rung84 import _rig, _kw, build_two_spool_turbojet, _cpg, PI_LPC, PI_HPC, TT4, FLIGHT, REAL
dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
hx = lambda x: "%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]

tau4 = 0.0190 + (0.0206 - 0.0190) * 4 / (9 - 1.0)
print("tau4", repr(tau4))
for trial in range(2):
    e = _rig(dsg).edge_read(tau4, **_kw(0.25))
    print("fresh edge_read tau4 trial", trial, [hx(v) for _, v in list(e["summands"].items())[28:30]])
# the same point as the 5th march on one rig, as staircase_scan runs it
m = _rig(dsg)
for i in range(5):
    e = m.edge_read(0.0190 + (0.0206 - 0.0190) * i / 8.0, **_kw(0.25))
print("5th on one rig", [hx(v) for _, v in list(e["summands"].items())[28:30]])
# rung 83: tau = 0.29 at r = 1.0, fresh vs second on a rig
print("fresh 0.29 F", hx(_rig(dsg).corrector_read(0.29, **_kw(1.0))["F"]))
m = _rig(dsg); m.corrector_read(0.30, **_kw(1.0))
print("after 0.30 F", hx(m.corrector_read(0.29, **_kw(1.0))["F"]))
