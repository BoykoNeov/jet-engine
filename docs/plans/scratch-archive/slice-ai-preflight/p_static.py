# -*- coding: utf-8 -*-
"""Slice AI pre-flight: the static probes, adapted from slice AH's, over rungs 79/80."""
import ast, io, re, sys, collections
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

ROOT = r"W:/Claude_projects/jet engine"
E = ROOT + r"/turbojet/engine.py"
SRC = open(E, encoding="utf-8").read()
LINES = SRC.splitlines()
TREE = ast.parse(SRC)
BY = {n.name: n for n in TREE.body if isinstance(n, ast.ClassDef)}
CLS = [(n.lineno, n.name, n.end_lineno) for n in TREE.body if isinstance(n, ast.ClassDef)]
R79, R80 = BY["StateCoordinateTransient"], BY["SplitWallTransient"]
AI = (R79, R80)


def owner(ln):
    for a, nm, b in CLS:
        if a <= ln <= b:
            return nm
    return "?"


def meths(c):
    return {m.name: m for m in c.body if isinstance(m, ast.FunctionDef)}


# ---------------------------------------------------------------- PROBE 2/9: sizing
def stats(names):
    tot = meth = body = 0
    for nm in names:
        c = BY[nm]
        tot += c.end_lineno - c.lineno + 1
        meth += sum(1 for m in c.body if isinstance(m, ast.FunctionDef))
        doc = set()
        for node in ast.walk(c):
            if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
                d = ast.get_docstring(node, clean=False)
                if d is not None:
                    e = node.body[0]
                    doc.update(range(e.lineno, e.end_lineno + 1))
        for i in range(c.lineno, c.end_lineno + 1):
            if i in doc:
                continue
            s = LINES[i - 1].strip()
            if not s or s.startswith("#"):
                continue
            body += 1
    return tot, meth, body


print("=== SIZING (AH's rule: non-blank, non-comment, minus docstrings) ===")
for label, names in (("AC (70+71)", ["CrossSplitTransient", "FullSplitTransient"]),
                     ("AF (74)", ["DemandCoordinateTransient"]),
                     ("AG (75+76)", ["AntiWindupTransient", "SensedCapTransient"]),
                     ("AH (77+78)", ["StiffnessLedgerTransient", "ResidualGaugeTransient"]),
                     ("AI (79+80)", ["StateCoordinateTransient", "SplitWallTransient"]),
                     ("  79", ["StateCoordinateTransient"]), ("  80", ["SplitWallTransient"])):
    t, m, b = stats(names)
    print(f"  {label:<12} total={t:>5}  methods={m:>3}  body={b:>5}")

# ---------------------------------------------------------------- PROBE 4: refusals
print("\n=== REFUSALS (assert with message / bare / raise) ===")
for c in (BY["StiffnessLedgerTransient"], BY["ResidualGaugeTransient"]) + AI:
    am = [n for n in ast.walk(c) if isinstance(n, ast.Assert) and n.msg is not None]
    ab = [n for n in ast.walk(c) if isinstance(n, ast.Assert) and n.msg is None]
    rs = [n for n in ast.walk(c) if isinstance(n, ast.Raise)]
    print(f"  {c.name:<28} assert+msg={len(am)} bare={len(ab)} raise={len(rs)}")
    if c in AI:
        for n in am:
            fn = [f.name for f in ast.walk(c) if isinstance(f, ast.FunctionDef)
                  and f.lineno <= n.lineno <= f.end_lineno]
            head = ast.unparse(n.msg)[:90]
            print(f"     engine.py:{n.lineno:<6} in {fn[-1] if fn else '?':<18} {head}")

# ---------------------------------------------------------------- PROBE 7: later redefinitions
print("\n=== LATER CLASSES redefining an AI method name (is 'no new cell' durable?) ===")
ai_names = set(meths(R79)) | set(meths(R80))
after = [n for n in TREE.body if isinstance(n, ast.ClassDef) and n.lineno > R80.lineno]
print(f"  classes after rung 80: {[n.name for n in after]}")
hits = collections.defaultdict(list)
for c in after:
    for m in meths(c):
        if m in ai_names:
            hits[m].append(c.name)
for m, w in sorted(hits.items()):
    print(f"   {m:<22} redefined by {w}")

# ---------------------------------------------------------------- PROBE 8: at_lever carry chain
print("\n=== at_lever CARRY chain ===")
chain = ["SensedCapTransient", "StiffnessLedgerTransient", "ResidualGaugeTransient",
         "StateCoordinateTransient", "SplitWallTransient"] + [c.name for c in after]
prev = None
for nm in chain:
    m = meths(BY[nm]).get("at_lever")
    if m is None:
        print(f"  {nm:<28} NO at_lever override")
        continue
    out = []
    for node in ast.walk(m):
        if isinstance(node, ast.Assign):
            for t in node.targets:
                for tt in (t.elts if isinstance(t, ast.Tuple) else [t]):
                    if isinstance(tt, ast.Attribute) and isinstance(tt.value, ast.Name) \
                       and tt.value.id == "m":
                        out.append(tt.attr)
    new = [] if prev is None else [f for f in out if f not in prev]
    lost = [] if prev is None else [f for f in prev if f not in out]
    print(f"  engine.py:{m.lineno:<6} {nm:<28} carries={len(out)} NEW={new or '-'} LOST={lost or '-'}")
    prev = out

# ---------------------------------------------------------------- PROBE 10: arithmetic surface
print("\n=== ARITHMETIC SURFACE (sites listed for a HAND reading) ===")
RISK = {"sum", "max", "min", "sorted", "pow", "round", "abs", "fsum"}
for c in AI:
    hits = collections.defaultdict(list)
    for n in ast.walk(c):
        if isinstance(n, ast.Call):
            f = n.func
            nm = f.id if isinstance(f, ast.Name) else (f.attr if isinstance(f, ast.Attribute) else None)
            if nm in RISK:
                hits[nm].append(n.lineno)
    print(f"  -- {c.name}: " + ", ".join(f"{k} x{len(v)}" for k, v in sorted(hits.items())))
    for n in ast.walk(c):
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == "sum":
            print(f"     sum  @ engine.py:{n.lineno}: {ast.unparse(n)[:110]}")
        if isinstance(n, ast.BinOp) and isinstance(n.op, ast.Pow):
            print(f"     **   @ engine.py:{n.lineno}: {ast.unparse(n)[:110]}")
    for i in range(c.lineno, c.end_lineno + 1):
        s = LINES[i - 1]
        if re.search(r"max\((?!\[)[^,()]*(\([^()]*\))?[^,()]*,\s*1e-(9|12|30)\)", s):
            print(f"     fold @ engine.py:{i}: {s.strip()[:110]}")

# ---------------------------------------------------------------- PROBE 11: the suite
print("\n=== THE SUITE for rungs 79/80 ===")
for t in ("test_rung79.py", "test_rung80.py"):
    s = open(rf"{ROOT}/tests/{t}", encoding="utf-8").read()
    nd = re.findall(r'match=(?:r?)["\']([^"\']+)["\']', s)
    nt = len(re.findall(r"^def test_|^    def test_", s, re.M))
    ns = len(re.findall(r"pytest\.mark\.slow", s))
    nr = len(re.findall(r"pytest\.raises", s))
    print(f"  {t}: {len(s.splitlines())} lines, {nt} tests, {ns} slow marks, {nr} raises, "
          f"{len(nd)} needles")
    for x in nd:
        print(f"     needle {x!r}  matches in engine.py: {len(re.findall(re.escape(x), SRC))}")

# ---------------------------------------------------------------- CLASS-LEVEL (process-global) state
print("\n=== CLASS-LEVEL WRITES (process-global state) inside rungs 79/80 ===")
for c in AI:
    for n in ast.walk(c):
        tg = []
        if isinstance(n, ast.Assign):
            tg = n.targets
        elif isinstance(n, ast.AugAssign):
            tg = [n.target]
        for t in tg:
            for tt in (t.elts if isinstance(t, ast.Tuple) else [t]):
                if isinstance(tt, ast.Attribute) and isinstance(tt.value, ast.Name) \
                   and tt.value.id in BY:
                    fn = [f.name for f in ast.walk(c) if isinstance(f, ast.FunctionDef)
                          and f.lineno <= n.lineno <= f.end_lineno]
                    print(f"   engine.py:{n.lineno:<6} {fn[-1] if fn else '?':<16} "
                          f"{tt.value.id}.{tt.attr} {'+=' if isinstance(n, ast.AugAssign) else '='}")

# ---------------------------------------------------------------- _with_coord / fields
print("\n=== EVERY call of `_with_coord`, and every WRITE of `_lag_coord` / `_phi_ref` ===")
for n in ast.walk(TREE):
    if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr == "_with_coord":
        fn = [f for f in ast.walk(BY[owner(n.lineno)]) if isinstance(f, ast.FunctionDef)
              and f.lineno <= n.lineno <= f.end_lineno] if owner(n.lineno) in BY else []
        print(f"   CALL  engine.py:{n.lineno:<6} {owner(n.lineno):<28} {fn[-1].name if fn else '?':<18} "
              f"recv={ast.unparse(n.func.value)} arg0={ast.unparse(n.args[0]) if n.args else '?'}")
for n in ast.walk(TREE):
    if isinstance(n, (ast.Assign, ast.AugAssign)):
        tg = n.targets if isinstance(n, ast.Assign) else [n.target]
        for t in tg:
            for tt in (t.elts if isinstance(t, ast.Tuple) else [t]):
                if isinstance(tt, ast.Attribute) and tt.attr in ("_lag_coord", "_phi_ref", "_sm_air"):
                    print(f"   WRITE engine.py:{n.lineno:<6} {owner(n.lineno):<28} "
                          f"{ast.unparse(tt.value)}.{tt.attr}")

print("\n=== callers of `split_gains` / `_coord_march` / `coord_march` / `_split_march` ===")
for n in ast.walk(TREE):
    if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and \
       n.func.attr in ("split_gains", "_coord_march", "coord_march", "_split_march",
                       "_cap_march", "accel_for", "_gauge_points", "_riding4"):
        print(f"   engine.py:{n.lineno:<6} {owner(n.lineno):<28} -> {n.func.attr}")
