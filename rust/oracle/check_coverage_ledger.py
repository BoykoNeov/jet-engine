"""Check rust/tests/coverage_ledger.tsv against the Python suite it ledgers (slice AT).

The Rust gate (`rust/tests/coverage_ledger.rs`) keeps the ledger honest AFTER the Python is gone:
counts, statuses, reasons, and that every cited Rust test exists. It cannot check the other side
— that the rows ARE the Python suite — because that side is deleted at slice AU. This script does,
once, while both exist:

  * the (file, function) rows equal the set of `def test_` functions found by AST in tests/, in
    the same order (module-level and `Test*`-class methods), and
  * each row's `py_cases` equals the number of IDs `pytest --collect-only` lists for it.

Usage (from the repo root, nothing is run, only collected):

    python -m pytest --collect-only -q -n 0 -p no:cacheprovider > collect.txt
    python rust/oracle/check_coverage_ledger.py collect.txt

It dies with the Python at slice AU, as the rest of rust/oracle/*.py does; its output does not need
to outlive it, because the Rust gate freezes the counts it confirms.
"""
import ast
import collections
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TESTS = os.path.join(ROOT, "tests")
LEDGER = os.path.join(ROOT, "rust", "tests", "coverage_ledger.tsv")


def ast_rows():
    rows = []
    for fn in sorted(os.listdir(TESTS)):
        if not (fn.startswith("test_") and fn.endswith(".py")):
            continue
        tree = ast.parse(open(os.path.join(TESTS, fn), encoding="utf-8").read())

        def visit(body):
            for n in body:
                if isinstance(n, ast.ClassDef) and n.name.startswith("Test"):
                    visit(n.body)
                elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test_"):
                    rows.append((fn[:-3], n.name))
        visit(tree.body)
    return rows


def collected(path):
    cases = collections.Counter()
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if "::" in line:
            f, rest = line.split("::", 1)
            cases[(os.path.basename(f)[:-3], rest.split("[")[0].split("::")[-1])] += 1
    return cases


def main(collect_txt):
    lines = open(LEDGER, encoding="utf-8").read().splitlines()
    led = [l.split("\t") for l in lines[1:]]
    want = ast_rows()
    got = [(r[0], r[1]) for r in led]
    assert got == want, f"ledger rows != AST rows: {len(got)} vs {len(want)}; first diff " \
        f"{next(((a, b) for a, b in zip(got, want) if a != b), None)}"
    cases = collected(collect_txt)
    bad = [(r[0], r[1], r[2], cases[(r[0], r[1])]) for r in led if int(r[2]) != cases[(r[0], r[1])]]
    assert not bad, f"py_cases disagree with pytest's collection: {bad[:5]}"
    assert sum(cases.values()) == sum(int(r[2]) for r in led)
    print(f"OK: {len(led)} rows = the AST's {len(want)} test functions; "
          f"{sum(cases.values())} collected cases, every row's count matches")


if __name__ == "__main__":
    main(sys.argv[1])
