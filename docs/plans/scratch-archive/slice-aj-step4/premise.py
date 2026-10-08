import os, sys, time
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, os.path.join(REPO, "tests"))
from test_rung84 import _rig, _kw, build_two_spool_turbojet, _cpg, PI_LPC, PI_HPC, TT4, FLIGHT, REAL
dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
m = _rig(dsg)
JL, JH = 0.0197750, 0.0197875
print("jump last == hi:", JL + (JH - JL) * 1 / (2 - 1.0) == JH, repr(JL + (JH - JL) * 1 / 1.0))
CL, CH = 0.037000, 0.037333
print("cross last == hi:", CL + (CH - CL) * 1 / 1.0 == CH, repr(CL + (CH - CL) * 1 / 1.0))
for ds in (0.005, 0.0025):
    t = time.time(); e = m.edge_read(0.03, **_kw(0.25, ds))
    print("ds", ds, "edge_read %.1fs" % (time.time() - t), "n_scored", e["n_scored"], "n_ride", e["n_ride"], "edge", e["edge"], flush=True)
t = time.time(); e = m.edge_read(0.05, **_kw(1.0))
print("r=1 edge_read %.1fs" % (time.time() - t), e["window_open"], e["summands"], flush=True)
# rung 83 F/g vs rung 84 F/g bit difference at the same read
for r, tau in ((0.25, 0.03), (0.35, 0.037), (0.5, 0.08), (0.25, 0.0197750), (0.25, 0.0197875)):
    a = m.corrector_read(tau, **_kw(r)); b = m.edge_read(tau, **_kw(r))
    print("r", r, "tau", tau, "F83==F84", a["F"] == b["F"], "g83==g84", a["g"] == b["g"], "h", a["h"] == b["h"], flush=True)
# secant from the open 0.30 end at r=1.0
for t0, t1 in ((0.30, 0.29), (0.30, 0.25), (0.28, 0.30)):
    t = time.time(); o = m.corrector_secant(t0, t1, cap=3, **_kw(1.0))
    print("secant r=1", t0, t1, o["abort"], [(x["tau"], x["g"]) for x in o["trace"]], "%.1fs" % (time.time() - t), flush=True)
