"""Dump one fingerprint kernel's output EXACTLY (float.hex / repr), from a given package copy.
usage: kdump.py <root> <kernel> <out.json>"""
import importlib.util, json, math, os, sys
root, name, out = sys.argv[1], sys.argv[2], sys.argv[3]
sys.path.insert(0, root)
spec = importlib.util.spec_from_file_location("fp", os.path.join(root, "tests", "test_numeric_fingerprint.py"))
fp = importlib.util.module_from_spec(spec); spec.loader.exec_module(fp)
import turbojet.engine as E
assert os.path.normcase(os.path.abspath(E.__file__)).startswith(os.path.normcase(os.path.abspath(root))), E.__file__
ctl = {"engine": E.__file__, "at_lever_in_r80": "at_lever" in E.SplitWallTransient.__dict__}
res = fp.KERNELS[name]()
def enc(v):
    if isinstance(v, float):
        return "f:" + v.hex()
    return repr(v)
J = {"control": ctl, "values": {k: enc(v) for k, v in res.items()}}
open(out, "w", encoding="utf-8").write(json.dumps(J, indent=0, sort_keys=True))
print(name, len(res), ctl)
