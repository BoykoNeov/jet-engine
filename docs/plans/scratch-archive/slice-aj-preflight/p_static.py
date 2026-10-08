# -*- coding: utf-8 -*-
"""Slice AJ pre-flight: static arithmetic / void / class-write / string-format census, rungs 81-84."""
import ast, io, re, sys, collections
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
E = r"W:/Claude_projects/jet engine/turbojet/engine.py"
SRC = open(E, encoding="utf-8").read(); L = SRC.splitlines(); TREE = ast.parse(SRC)
BY = {n.name: n for n in TREE.body if isinstance(n, ast.ClassDef)}
AJ = [BY[n] for n in ("AuthorityClockTransient", "ThresholdLawTransient", "CorrectorLawTransient", "StaircaseLawTransient")]
def fn_of(c, ln):
    fs = [f.name for f in ast.walk(c) if isinstance(f, ast.FunctionDef) and f.lineno <= ln <= f.end_lineno]
    return fs[-1] if fs else "?"
def show(tag, c, n):
    print(f"  {tag:10} engine.py:{n.lineno:<6} {c.name[:9]:9} {fn_of(c, n.lineno):20} {ast.unparse(n)[:110]}")
print("=== round() calls ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == "round":
            show("round/%d" % len(n.args), c, n)
print("\n=== sum() calls (float vs count) ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == "sum":
            a = n.args[0]
            kind = "COUNT" if isinstance(a, ast.GeneratorExp) and isinstance(a.elt, ast.Constant) else "FLOAT?"
            show(kind, c, n)
print("\n=== min/max/sorted with key= or default= ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id in ("min", "max", "sorted"):
            kws = [k.arg for k in n.keywords]
            if "key" in kws:
                show(n.func.id + "+key", c, n)
print("\n=== % string formatting and f-strings in returned void strings ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.BinOp) and isinstance(n.op, ast.Mod) and isinstance(n.left, ast.Constant) and isinstance(n.left.value, str):
            show("%fmt", c, n)
        if isinstance(n, ast.JoinedStr):
            show("fstring", c, n)
print("\n=== void= / abort= string literals ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.Constant) and isinstance(n.value, str) and re.match(r"^(V\d|S\d)", n.value):
            print(f"  engine.py:{n.lineno:<6} {c.name[:9]:9} {fn_of(c, n.lineno):20} {n.value!r}")
print("\n=== assert / raise ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, (ast.Assert, ast.Raise)):
            show(type(n).__name__, c, n)
print("\n=== attribute WRITES (self.x = / m.x = / Cls.x = / type(self).x =) ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, (ast.Assign, ast.AugAssign)):
            ts = n.targets if isinstance(n, ast.Assign) else [n.target]
            for t in ts:
                for e in ast.walk(t):
                    if isinstance(e, ast.Attribute) and isinstance(e.ctx, ast.Store):
                        show("attrW", c, n)
print("\n=== getattr/setattr/type()/isinstance/id() ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id in ("getattr", "setattr", "type", "isinstance", "id", "hasattr", "int", "float", "bool"):
            show(n.func.id, c, n)
print("\n=== float('nan') / inf / exact == on floats (Compare Eq/NotEq) ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.Compare) and any(isinstance(o, (ast.Eq, ast.NotEq)) for o in n.ops):
            show("==/!=", c, n)
print("\n=== self._with_* / m._with_* scopes and inc usage ===")
for c in AJ:
    for n in ast.walk(c):
        if isinstance(n, ast.Attribute) and n.attr.startswith("_with_"):
            show("scope", c, n)
        if isinstance(n, ast.Name) and n.id == "inc" and isinstance(n.ctx, ast.Load):
            pass
