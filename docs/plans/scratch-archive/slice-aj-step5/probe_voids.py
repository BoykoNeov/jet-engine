"""Slice AJ step 5: re-drive the ten pre-flight voids (p_drive.py's inputs) + one extra arm,
recording every field the Rust gates will assert. ENTRY CONTROL: prints sys.version first and
refuses to write a *pypy* output off PyPy (slice AJ step 4's lesson)."""
import io, json, platform, sys, time
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
print(sys.version, flush=True)
OUT_PATH = sys.argv[1]
if "pypy" in OUT_PATH.lower() and platform.python_implementation() != "PyPy":
    raise SystemExit("refusing: %s names PyPy but this is %s" % (OUT_PATH, platform.python_implementation()))
ROOT = r"W:/Claude_projects/jet engine"
sys.path.insert(0, ROOT); sys.path.insert(0, ROOT + "/tests")
import test_rung84 as T

design = T.build_two_spool_turbojet(T._cpg(), T.PI_LPC, T.PI_HPC, T.TT4, T.FLIGHT.p0,
                                    nozzle_convergent=True, **T.REAL)
m = T._rig(design)
t0 = time.time()
V = {}
kw1 = T._kw(1.0)
key = lambda s: s["n_fuel"] > 0
b = m._bisect(key, 0.004, 0.30, 0, **kw1)
V["bisect_V1"] = dict(void=b.get("void"), keys=list(b), lo_open=b["at_lo"]["window_open"],
                      hi_open=b["at_hi"]["window_open"])
b3 = m._bisect(key, 0.004, 0.30, 0, **dict(T._kw(0.35), phi_lim=0.760))
V["bisect_V3"] = dict(void=b3.get("void"), keys=list(b3), below=b3["below"], above=b3["above"])
rd1 = m.corrector_read(0.05, **kw1)
st = m.corrector_step(rd1, 0.5)
V["step_V4"] = dict(st, keys=list(st))
sec = m.corrector_secant(0.05, 0.06, cap=1, **kw1)
V["secant_V4_start"] = dict(abort=sec["abort"], marches=sec["marches"], trace=sec["trace"],
                            tau=sec["tau"], final_g=sec["final_g"], converged=sec["converged"])
# EXTRA ARM: the first start fails, the second (0.30 at r = 1.0) is valid -> break vs continue
sec2 = m.corrector_secant(0.05, 0.30, cap=1, **kw1)
rd30 = m.corrector_read(0.30, **kw1)
V["secant_V4_first_of_valid"] = dict(abort=sec2["abort"], marches=sec2["marches"],
                                     trace=sec2["trace"], second_g=rd30["g"],
                                     second_window=rd30["window_open"])
kw = T._kw(0.25)
rdv5 = m.corrector_read(0.0198, **kw)
st5 = m.corrector_step(rdv5, 1.0)
V["step_V5"] = dict(st5, keys=list(st5), F=rdv5["F"])
s2 = m.corrector_secant(0.0198, 0.0198, cap=2, **kw)
V["secant_S2"] = dict(abort=s2["abort"], marches=s2["marches"], clamps=s2["clamps"],
                      converged=s2["converged"], tau=s2["tau"],
                      g=[x["g"].hex() for x in s2["trace"]])
lc = m.lattice_count(0.02, 0.021, **kw1)
V["lattice_V3"] = dict(lc, keys=list(lc))
sn2 = m.staircase_number(0.02, 0.021, **kw1)
V["sn_V2"] = dict(sn2, keys=list(sn2))
sn5 = m.staircase_number(0.0198, 0.0198, **kw)
V["sn_V5"] = dict(sn5, keys=list(sn5))
rc6 = m.root_class(n_bisect=0, eps=1.0, **kw)
V["root_V6"] = dict(rc6, keys=list(rc6))
rc1 = m.root_class(n_bisect=0, **kw1)
V["root_V1"] = dict(rc1, keys=list(rc1))
V["seconds"] = round(time.time() - t0, 1)
json.dump(V, open(OUT_PATH, "w"), indent=1, default=repr)
print(json.dumps(V, indent=1, default=repr))
