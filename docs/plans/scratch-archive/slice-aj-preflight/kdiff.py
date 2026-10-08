import json, sys, glob, os
D = r"W:/temp/claude/slice-aj-preflight/out"
G = json.load(open(r"W:/temp/claude/slice-aj-preflight/ctl/tests/golden/numeric_fingerprint.json", encoding="utf-8"))
for k in ("r80", "r81", "r81m", "r82", "r82r", "r82t"):
    fc, fm = f"{D}/k_{k}_ctl.json", f"{D}/k_{k}_mut.json"
    if not (os.path.exists(fc) and os.path.exists(fm)):
        print(k, "pending"); continue
    c, m = json.load(open(fc)), json.load(open(fm))
    vc, vm = c["values"], m["values"]
    diff = [x for x in vc if vc[x] != vm.get(x)]
    gk = G["kernels"].get(k, {})
    print(f"{k}: ctl keys={len(vc)} mut keys={len(vm)} keysets_equal={set(vc)==set(vm)} differ={len(diff)} "
          f"| ctl.at_lever_in_r80={c['control']['at_lever_in_r80']} mut.at_lever_in_r80={m['control']['at_lever_in_r80']} "
          f"| golden keys={len(gk)} ctl-keyset==golden={set(vc)==set(gk)}")
    for x in diff[:5]:
        print("   ", x, vc[x], vm.get(x))
print("--- control dump (PyPy) vs shipped golden (CPython), exact ---")
for k in ("r80", "r81", "r81m", "r82", "r82r", "r82t"):
    fc = f"{D}/k_{k}_ctl.json"
    if not os.path.exists(fc):
        continue
    vc = json.load(open(fc))["values"]
    gk = G["kernels"][k]
    def genc(v):
        return ("f:" + v["f"]) if isinstance(v, dict) and "f" in v else repr(v)
    nd = [x for x in vc if vc[x] != genc(gk[x])]
    print(f"{k}: {len(nd)} of {len(vc)} differ from golden", nd[:3])
