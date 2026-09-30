"""THE REVERSE RUN — CPython made to do PyPy's arithmetic in exactly two places, then the dumper.

    C:\\Python314\\python.exe rust/oracle/reverse_slice_aj.py W:/temp/claude/slice-aj-step6/reverse_sq_cpython.tsv

Plan § 5.34.6 (c) P-B. Run 2026-09-30 (3 090 s): byte-identical to `slice_aj_pypy.tsv`, 24 324 of
24 324 lines, scored by `compare_slice_aj.py`. The output is not committed: it IS the PyPy golden.

1. Every literal `X ** 2` in the `turbojet` package is rewritten `__sq__(X)` = `X * X` AT IMPORT
   (an AST transform in a meta-path loader; the .pyc cache is bypassed). PyPy spells a float
   square as a multiply; CPython calls libm `pow`, which misrounds some squares (26 of 1 291 982
   `eta_c_at` calls in one march — `probe_powrate.py`). Nothing is enumerated by hand: every
   site in the package is rewritten, and the count per module is printed.
2. `builtins.sum` is a naive left fold (PyPy's), installed before `turbojet` imports. CPython
   3.12+ compensates float sums.

Prediction (predictions.md P-B): the output is byte-identical to the PyPy golden — every line,
the `_interp.sum_probe` sentinel included (0.0 under the naive fold, as PyPy's). A surviving
difference names a mechanism that is neither.

Positive controls, asserted: the interpreter IS CPython; the rewrite touched > 0 sites in
`turbojet.engine`; `__sq__` was CALLED during the drive; `sum([1e16, 1.0, -1e16]) == 0.0`.
"""
import ast
import builtins
import importlib.abc
import importlib.machinery
import os
import platform
import runpy
import sys

print(sys.version.replace("\n", " "), flush=True)
assert platform.python_implementation() == "CPython", "the reverse run is a CPython run"

REPO = r"W:\Claude_projects\jet engine"
DUMPER = os.path.join(REPO, "rust", "oracle", "dump_slice_aj.py")
SITES = {}
CALLS = [0]


def _sq(x):
    CALLS[0] += 1
    return x * x


builtins.__sq__ = _sq


def _naive_sum(iterable, /, start=0):
    acc = start
    for x in iterable:
        acc = acc + x
    return acc


builtins.sum = _naive_sum
assert sum([1e16, 1.0, -1e16]) == 0.0, "the naive fold is not installed"


class _Sq(ast.NodeTransformer):
    def __init__(self):
        self.n = 0

    def visit_BinOp(self, node):
        self.generic_visit(node)
        if (isinstance(node.op, ast.Pow) and isinstance(node.right, ast.Constant)
                and type(node.right.value) in (int, float) and node.right.value == 2):
            self.n += 1
            return ast.copy_location(
                ast.Call(func=ast.Name(id="__sq__", ctx=ast.Load()), args=[node.left],
                         keywords=[]), node)
        return node


class _Loader(importlib.machinery.SourceFileLoader):
    def get_code(self, fullname):
        src = self.get_data(self.path)
        tree = ast.parse(src, filename=self.path)
        t = _Sq()
        tree = ast.fix_missing_locations(t.visit(tree))
        SITES[fullname] = t.n
        return compile(tree, self.path, "exec", dont_inherit=True)


class _Finder(importlib.abc.MetaPathFinder):
    def find_spec(self, name, path, target=None):
        if name != "turbojet" and not name.startswith("turbojet."):
            return None
        spec = importlib.machinery.PathFinder.find_spec(name, path)
        if spec is not None and isinstance(spec.loader, importlib.machinery.SourceFileLoader):
            spec.loader = _Loader(spec.loader.name, spec.loader.path)
        return spec


sys.meta_path.insert(0, _Finder())
sys.path.insert(0, REPO)
out = sys.argv[1]
assert "cpython" in os.path.basename(out).lower(), "name the output *cpython*"
sys.argv = [DUMPER, out]
runpy.run_path(DUMPER, run_name="__main__")

print("rewritten sites per module:", dict(sorted(SITES.items())), flush=True)
print("__sq__ calls during the drive:", CALLS[0], flush=True)
assert SITES.get("turbojet.engine", 0) > 0, "the rewrite never reached turbojet.engine"
assert CALLS[0] > 0, "no rewritten square ran"
