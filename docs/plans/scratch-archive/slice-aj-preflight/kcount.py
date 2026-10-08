"""Run one fingerprint kernel from a package copy with a COUNTER on every `at_lever` definer.
Answers: did this kernel ENTER the rig-building path (and, under the mutation, rung 79's rebuild)?
usage: kcount.py <root> <kernel> <out.json>"""
import collections, functools, importlib.util, json, os, sys
root, name, out = sys.argv[1], sys.argv[2], sys.argv[3]
sys.path.insert(0, root)
spec = importlib.util.spec_from_file_location("fp", os.path.join(root, "tests", "test_numeric_fingerprint.py"))
fp = importlib.util.module_from_spec(spec); spec.loader.exec_module(fp)
import turbojet.engine as E
assert os.path.normcase(os.path.abspath(E.__file__)).startswith(os.path.normcase(os.path.abspath(root)))
C = collections.Counter()
for cls in [c for c in E.StaircaseLawTransient.__mro__ if "at_lever" in c.__dict__]:
    fn = cls.__dict__["at_lever"]
    def mk(fn, owner):
        @functools.wraps(fn)
        def w(self, *a, **kw):
            r = fn(self, *a, **kw)
            C[f"{owner} | {type(self).__name__} | {type(r).__name__}"] += 1
            return r
        return w
    setattr(cls, "at_lever", mk(fn, cls.__name__))
# POSITIVE CONTROL: the wrapper fires
C.clear()
res = fp.KERNELS[name]()
J = {"engine": E.__file__, "at_lever_in_r80": "at_lever" in E.SplitWallTransient.__dict__,
     "keys": len(res), "at_lever": dict(C)}
open(out, "w", encoding="utf-8").write(json.dumps(J, indent=1))
print(name, J)
