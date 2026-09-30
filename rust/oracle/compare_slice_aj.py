"""Score plan § 5.34.6 (c)'s P-B and P-C: the reverse CPython run and the plain CPython golden vs PyPy.

    C:\\Python314\\python.exe rust/oracle/compare_slice_aj.py [<reverse_sq_cpython.tsv>]

Measured 2026-09-30: REVERSE 0 lines differ; PLAIN 422 (421 + the sum sentinel), all floats.
"""
import os
import sys

O = os.path.dirname(os.path.abspath(__file__))
REV = sys.argv[1] if len(sys.argv) > 1 else r"W:\temp\claude\slice-aj-step6\reverse_sq_cpython.tsv"


def load(p):
    with open(p, encoding="utf-8") as fh:
        return [tuple(l.rstrip("\n").split("\t", 1)) for l in fh if l.strip()]


py = load(O + r"\slice_aj_pypy.tsv")
for name, path in (("REVERSE", REV),
                   ("PLAIN", O + r"\slice_aj_cpython.tsv")):
    cp = load(path)
    print("=== %s: %d lines vs PyPy %d" % (name, len(cp), len(py)))
    assert [p for p, _ in cp] == [p for p, _ in py], "paths differ"
    d = [(k, py[k][0], py[k][1], cp[k][1]) for k in range(len(py)) if py[k][1] != cp[k][1]]
    print("differing lines:", len(d))
    kinds = {}
    for _, p, a, b in d:
        kinds.setdefault((a.split(":")[0], b.split(":")[0]), 0)
        kinds[(a.split(":")[0], b.split(":")[0])] += 1
    print("token kinds (pypy, cpython):", kinds)
    per = {}
    for _, p, a, b in d:
        r = p.split("@")[0].split(".")[0]
        per[r] = per.get(r, 0) + 1
    print("per reading:", per)
    for _, p, a, b in d:
        print("  %s  %s  %s" % (p, a, b))
