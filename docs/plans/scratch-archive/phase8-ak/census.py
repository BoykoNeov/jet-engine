import ast, collections, sys
src = open(r"W:\Claude_projects\jet engine\main.py", encoding="utf-8").read()
t = ast.parse(src)
specs = collections.Counter(); convs = collections.Counter(); bare = collections.Counter()
nested = 0
def spec_str(fs):
    out = []
    for v in fs.values:
        if isinstance(v, ast.Constant): out.append(v.value)
        else: out.append("{" + ast.unparse(v.value) + "}")
    return "".join(out)
for n in ast.walk(t):
    if isinstance(n, ast.FormattedValue):
        if n.conversion != -1: convs[chr(n.conversion)] += 1
        if n.format_spec is None:
            bare[ast.unparse(n.value)] += 1
        else:
            s = spec_str(n.format_spec)
            if "{" in s: nested += 1
            specs[s] += 1
print("conversions", dict(convs)); print("nested specs", nested)
print("DISTINCT SPECS", len(specs))
for s, c in sorted(specs.items(), key=lambda x: -x[1]): print(f"  {c:5d}  {s!r}")
print("BARE {x} count", sum(bare.values()), "distinct", len(bare))
for s, c in sorted(bare.items(), key=lambda x: -x[1]): print(f"  {c:4d}  {s}")
# print() call shapes
shapes = collections.Counter()
for n in ast.walk(t):
    if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == "print":
        kws = tuple(sorted(k.arg for k in n.keywords))
        shapes[(len(n.args), kws)] += 1
        if len(n.args) > 1 or kws: print("  PRINT", n.lineno, ast.unparse(n)[:150])
print("print shapes", dict(shapes))
ops = collections.Counter()
for n in ast.walk(t):
    if isinstance(n, ast.BinOp): ops[type(n.op).__name__] += 1
    if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id in ("round","int","str","repr","format","abs","len","sorted","zip","enumerate","divmod"):
        ops["call:"+n.func.id] += 1
print(dict(ops))
for n in ast.walk(t):
    if isinstance(n, ast.BinOp) and isinstance(n.op, (ast.Mod, ast.FloorDiv)):
        print("  MOD/FLOORDIV", n.lineno, ast.unparse(n)[:120])
    if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id in ("round","str","repr","format"):
        print("  CALL", n.lineno, ast.unparse(n)[:120])
    if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr in ("join","format","ljust","rjust","center"):
        print("  METH", n.lineno, ast.unparse(n)[:120])
