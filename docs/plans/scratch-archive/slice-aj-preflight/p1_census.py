"""AJ pre-flight probe 1: cell census + sizing for rungs 81-84 (AC's repaired predicate)."""
import ast, io, sys, collections
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
SRC = r"W:\Claude_projects\jet engine\turbojet\engine.py"
TEXT = open(SRC, encoding="utf-8").read(); LINES = TEXT.splitlines()
tree = ast.parse(TEXT)
classes = {n.name: n for n in tree.body if isinstance(n, ast.ClassDef)}
order = list(classes)
def methods(cn): return {m.name: m for m in classes[cn].body if isinstance(m, ast.FunctionDef)}
def bases(cn): return [b.id for b in classes[cn].bases if isinstance(b, ast.Name) and b.id in classes]
def mro_up(cn):
    seen, q = [], list(bases(cn))
    while q:
        c = q.pop(0)
        if c in seen: continue
        seen.append(c); q.extend(bases(c))
    return seen
def sig(fn):
    a = fn.args; pos = [p.arg for p in a.posonlyargs + a.args]; kw = [p.arg for p in a.kwonlyargs]
    nd = len(a.defaults); return pos, kw, (pos[:len(pos)-nd] if nd else pos)
def deco(fn): return [ast.unparse(d) for d in fn.decorator_list]
T = ["AuthorityClockTransient", "ThresholdLawTransient", "CorrectorLawTransient", "StaircaseLawTransient"]
definers = collections.defaultdict(list)
for cn in classes:
    for m in methods(cn): definers[m].append(cn)
for t in T:
    print(f"\n=== {t} bases={bases(t)} ===")
    for nm, fn in methods(t).items():
        anc = [a for a in mro_up(t) if nm in methods(a)]
        later = [c for c in order if nm in methods(c) and order.index(c) > order.index(t)]
        if anc:
            p1,k1,r1 = sig(methods(anc[0])[nm]); p2,k2,r2 = sig(fn)
            cls = "SAME" if (p1,k1)==(p2,k2) else ("WIDENED" if set(r1)==set(r2) and set(p1)<=set(p2) and set(k1)<=set(k2) else ("RENAMED?" if len(p1)==len(p2) and len(k1)==len(k2) else "**INCOMPATIBLE**"))
            job = f"SWAP<-{anc[0]}[{cls}]"
        else:
            job = "new"
        print(f"  {fn.lineno:>6} {nm:22} {','.join(deco(fn)) or '-':14} {job:40} later={later} defs={len(definers[nm])}")
# sizing
def stats(names):
    tot = meth = body = 0
    for nm in names:
        c = classes[nm]; tot += c.end_lineno - c.lineno + 1
        meth += sum(1 for m in c.body if isinstance(m, ast.FunctionDef))
        doc = set()
        for node in ast.walk(c):
            if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
                if ast.get_docstring(node, clean=False) is not None:
                    e = node.body[0]; doc.update(range(e.lineno, e.end_lineno + 1))
        for i in range(c.lineno, c.end_lineno + 1):
            if i in doc: continue
            s = LINES[i-1].strip()
            if not s or s.startswith("#"): continue
            body += 1
    return tot, meth, body
print("\n=== SIZING (non-blank, non-comment, minus docstrings) ===")
for label, names in (("AG 75+76",["AntiWindupTransient","SensedCapTransient"]),("AH 77+78",["StiffnessLedgerTransient","ResidualGaugeTransient"]),
                     ("AI 79+80",["StateCoordinateTransient","SplitWallTransient"]),("AJ 81-84",T),
                     ("  81",[T[0]]),("  82",[T[1]]),("  83",[T[2]]),("  84",[T[3]])):
    print(f"  {label:10} total/methods/body = {stats(names)}")
# cross-class self.* calls from AJ methods into ancestors (which Rust fns they need)
print("\n=== self.<name>(...) calls from AJ classes, by definer ===")
ajm = set().union(*[methods(t) for t in T])
calls = collections.Counter()
for t in T:
    for node in ast.walk(classes[t]):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name) and node.func.value.id in ("self","m","mm","r","rig"):
            calls[(node.func.value.id, node.func.attr)] += 1
for (recv, nm), n in sorted(calls.items(), key=lambda x: x[0][1]):
    ds = [c for c in order if nm in methods(c)]
    print(f"  {recv}.{nm:26} x{n:<3} definers={ds[:2]}{'...' if len(ds)>2 else ''} n={len(ds)} {'(AJ-own)' if nm in ajm else ''}")
