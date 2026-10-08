import os, sys, struct
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, os.path.join(REPO, "tests"))
from test_rung84 import _rig, _kw, build_two_spool_turbojet, _cpg, PI_LPC, PI_HPC, TT4, FLIGHT, REAL
dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
hx = lambda x: "%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
KEYS = ["s", "nu_lp", "nu_hp", "Tt4", "f", "phi_lp", "mf", "mf_sched"]
out = open(sys.argv[1], "w", newline="\n")
for name, tau, r in (("a", 0.0190 + (0.0206 - 0.0190) * 4 / 8.0, 0.25), ("b", 0.29, 1.0)):
    kw = _kw(r); kw.pop("ds"); kw.pop("r")
    m = _rig(dsg)
    _, _, _, traj = m._split_march(kw["flight"], kw["Tt4_lo"], kw["Tt4_hi"], kw["Tt4_max"],
                                   kw["phi_lim"], kw["phi_air"], "demand",
                                   (tau, kw["tau_gov"], kw["tau_q"], kw["tau_s"]), r,
                                   kw["s_settle"], 0.005, kw["v_max"], kw["inc"])
    print(name, "keys", sorted(traj[0].keys()))
    for i, p in enumerate(traj):
        out.write("%s\t%d\t%s\n" % (name, i, " ".join(hx(p[k]) for k in KEYS)))
