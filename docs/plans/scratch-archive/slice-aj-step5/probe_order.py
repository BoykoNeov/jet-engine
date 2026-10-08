"""Slice AJ step 5 follow-up: the two void ORDERS no gate trips both halves of.
ENTRY CONTROL: prints sys.version; refuses a *pypy* output off PyPy."""
import io, json, platform, sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
print(sys.version, flush=True)
OUT_PATH = sys.argv[1]
if "pypy" in OUT_PATH.lower() and platform.python_implementation() != "PyPy":
    raise SystemExit("refusing: not PyPy")
ROOT = r"W:/Claude_projects/jet engine"
sys.path.insert(0, ROOT); sys.path.insert(0, ROOT + "/tests")
import test_rung84 as T
design = T.build_two_spool_turbojet(T._cpg(), T.PI_LPC, T.PI_HPC, T.TT4, T.FLIGHT.p0,
                                    nozzle_convergent=True, **T.REAL)
m = T._rig(design)
kw1 = T._kw(1.0)
V = {}
rd = m.corrector_read(0.05, **kw1)
st = m.corrector_step(rd, 1.0)                     # no F AND c = 1: both voids' conditions
V["step_both"] = dict(st, F=rd["F"], window_open=rd["window_open"])
rc = m.root_class(n_bisect=0, eps=1.0, **kw1)      # bisect voids AND b - a < eps
bi = m._bisect(lambda s: s["h"] is not None and s["h"] < 0.0, 0.004, 0.30, 0, **kw1)
V["root_both"] = dict(rc, bisect_void=bi.get("void"), width=bi["hi"] - bi["lo"])
json.dump(V, open(OUT_PATH, "w"), indent=1, default=repr)
print(json.dumps(V, indent=1, default=repr))
