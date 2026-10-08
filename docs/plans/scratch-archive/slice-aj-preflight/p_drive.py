"""Slice AJ pre-flight: drive every void code reachable cheaply, and record every round() argument.

`round` is shadowed in the ENGINE module's globals (module globals are consulted before builtins),
so every `round(...)` rungs 81-84 execute is recorded with its exact arguments. POSITIVE CONTROL:
the recorder is proved installed (nonzero records after the first edge_read) before anything else.
"""
import io, json, sys, time, builtins
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
ROOT = r"W:/Claude_projects/jet engine"
sys.path.insert(0, ROOT); sys.path.insert(0, ROOT + "/tests")
import turbojet.engine as E
import test_rung84 as T      # the suite's own rig and kwargs, not re-typed

REC = []
_r = builtins.round
def rec_round(x, *nd):
    out = _r(x, *nd)
    REC.append((x.hex() if isinstance(x, float) else repr(x), nd[0] if nd else None,
                out.hex() if isinstance(out, float) else repr(out)))
    return out
E.round = rec_round

design = T.build_two_spool_turbojet(T._cpg(), T.PI_LPC, T.PI_HPC, T.TT4, T.FLIGHT.p0,
                                    nozzle_convergent=True, **T.REAL)
m = T._rig(design)
t0 = time.time()
OUT = {}
# POSITIVE CONTROL + round args at three ds (the suite's three)
for ds in (0.005, 0.0025, 0.00125):
    for tau in (0.0197750, 0.0197875):
        rd = m.edge_read(tau, **T._kw(0.25, ds))
        OUT[f"edge_read/{ds}/{tau}"] = dict(edge=rd["edge"], edge_index=rd["edge_index"],
                                            on_grid=rd["edge_on_grid"], kappa=rd["kappa"],
                                            n_scored=rd["n_scored"], at_edge=rd["at_edge"])
    if ds == 0.005:
        assert REC, "the round() recorder never fired: instrument not wired"
        OUT["control_round_records_after_first_ds"] = len(REC)
# the round(x,6) of _threshold_scan at r = 0.35 (a second ramp)
s = m._scan(0.037, **T._kw(0.35))
OUT["scan/0.35/0.037/kappa"] = s["kappa"]
# ---- VOIDS -----------------------------------------------------------------------------------
V = {}
kw1 = T._kw(1.0)                          # r = 1.0: no four-loop window at all (rung 82's V1 test)
b = m._bisect(lambda s: s["n_fuel"] > 0, 0.004, 0.30, 0, **kw1)
V["bisect_V1_end"] = b.get("void")
rd1 = m.corrector_read(0.05, **kw1)
V["corrector_read_r1"] = dict(F=rd1["F"], g=rd1["g"], kappa_pure=rd1["kappa_pure"])
V["corrector_step_V4"] = m.corrector_step(rd1, 0.5)["void"]
V["corrector_secant_V4_start"] = m.corrector_secant(0.05, 0.06, cap=1, **kw1)["abort"]
V["lattice_count_V3"] = m.lattice_count(0.02, 0.021, **kw1)["void"]
V["staircase_number_V2"] = m.staircase_number(0.02, 0.021, **kw1)["void"]
V["root_class_bisect_void"] = m.root_class(n_bisect=0, **kw1)["void"]
kw = T._kw(0.25)
sec = m.corrector_secant(0.0198, 0.0198, cap=2, **kw)      # identical starts: g equal -> S2
V["corrector_secant_S2"] = sec["abort"]; V["S2_marches"] = sec["marches"]
V["staircase_number_V5"] = m.staircase_number(0.0198, 0.0198, **kw)["void"]
V["root_class_V6"] = m.root_class(n_bisect=0, eps=1.0, **kw)["void"]
V["corrector_step_V5"] = m.corrector_step(m.corrector_read(0.0198, **kw), 1.0)["void"]
# V3 with its SHIPPED bracket, the message as a returned value
b3 = m._bisect(lambda s: s["n_fuel"] > 0, 0.004, 0.30, 0, **dict(T._kw(0.35), phi_lim=0.760))
V["bisect_V3"] = b3.get("void")
OUT["voids"] = V
OUT["seconds"] = round(time.time() - t0, 1)
json.dump(dict(out=OUT, rec=REC), open(r"W:/temp/claude/slice-aj-preflight/p_drive.json", "w"), indent=1)
print(json.dumps(OUT, indent=1, default=str))
print("round records:", len(REC))
