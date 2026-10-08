"""show.py PYFILE PYTEST [RUSTFILE RUSTFN] [-n N] — print the Python test (decorators+body head)
and the Rust fn (doc + body head) side by side, for hand-reading ledger rows."""
import ast, os, re, sys

ROOT = r"W:\Claude_projects\jet engine"
args = [a for a in sys.argv[1:] if not a.startswith("-n")]
N = 40
for a in sys.argv[1:]:
    if a.startswith("-n"):
        N = int(a[2:])
pf, pt = args[0], args[1]
src = open(os.path.join(ROOT, "tests", pf + ".py"), encoding="utf-8").read().splitlines()
tree = ast.parse("\n".join(src))
for n in ast.walk(tree):
    if isinstance(n, ast.FunctionDef) and n.name == pt:
        start = min([d.lineno for d in n.decorator_list] + [n.lineno])
        end = n.end_lineno
        print(f"--- PY {pf}:{start}-{end}")
        for i in range(start - 1, min(end, start - 1 + N)):
            print(src[i])
        if end - start + 1 > N:
            print(f"    ... ({end - start + 1 - N} more lines)")
if len(args) >= 4:
    rf, rfn = args[2], args[3]
    rs = open(os.path.join(ROOT, "rust", "tests", rf + ".rs"), encoding="utf-8").read().splitlines()
    for i, l in enumerate(rs):
        if re.match(r"\s*(pub\s+)?fn\s+" + re.escape(rfn) + r"\b", l):
            j = i
            while j > 0 and (rs[j - 1].strip().startswith(("///", "#[", "//")) ):
                j -= 1
            print(f"--- RS {rf}:{j + 1}")
            depth, k, seen = 0, i, False
            out = rs[j:i]
            while k < len(rs):
                out.append(rs[k]); depth += rs[k].count("{") - rs[k].count("}")
                if "{" in rs[k]: seen = True
                if seen and depth <= 0: break
                k += 1
            for l in out[: N + (i - j)]:
                print(l)
            if len(out) > N + (i - j):
                print(f"    ... ({len(out) - N - (i - j)} more lines)")
            break
    else:
        print("RS fn not found")
