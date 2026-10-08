"""Run dump_slice_ai.py's drive under a runtime builtins.sum census, BEFORE any golden is compared.

Every builtins.sum call, by the caller's file:line:function, whether any element is a float,
and the element count -- the derived set the CPython prediction is written from (AH's probe).
"""
import builtins, collections, json, runpy, sys
sys.path.insert(0, r"W:\Claude_projects\jet engine")

SUMS = collections.Counter()      # (site, has_float, n) -> calls
_real_sum = builtins.sum


def probe_sum(it, *a, **kw):
    xs = list(it)
    fr = sys._getframe(1)
    site = "%s:%d:%s" % (fr.f_code.co_filename.rsplit("\\", 1)[-1].rsplit("/", 1)[-1],
                         fr.f_lineno, fr.f_code.co_name)
    has_float = any(isinstance(x, float) for x in xs) or any(isinstance(x, float) for x in a)
    SUMS[(site, has_float, len(xs))] += 1
    return _real_sum(xs, *a, **kw)


builtins.sum = probe_sum
sys.argv = ["dump_slice_ai.py", r"W:\temp\claude\slice-ai-step6\probe_run.tsv"]
runpy.run_path(r"W:\Claude_projects\jet engine\rust\oracle\dump_slice_ai.py", run_name="__main__")

sites = collections.Counter()
for (s, hf, n), c in SUMS.items():
    if hf:
        sites[s] += c
out = {"sums_float": sorted("%s n=%d calls=%d" % (s, n, c)
                            for (s, hf, n), c in SUMS.items() if hf),
       "float_sites": dict(sites),
       "sums_nonfloat_sites": sorted({s for (s, hf, n) in SUMS if not hf})}
json.dump(out, open(r"W:\temp\claude\slice-ai-step6\probe_drive.json", "w"), indent=1)
print(json.dumps({"float_sites": out["float_sites"],
                  "nonfloat": out["sums_nonfloat_sites"]}, indent=1))
