"""AH pre-flight probe 1: the CELL CENSUS for rungs 77/78.

A cell is a name that is OVERRIDDEN **and SUBSTITUTABLE** -- slice AC (xii)'s repaired
predicate, not `overridden at least once`.  Emitted, never transcribed.
"""
import ast, io, sys, collections
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

SRC = r"W:\Claude_projects\jet engine\turbojet\engine.py"
tree = ast.parse(open(SRC, encoding="utf-8").read())

classes = {}          # name -> (node, [bases])
for n in tree.body:
    if isinstance(n, ast.ClassDef):
        classes[n.name] = n

def methods(cn):
    return {m.name: m for m in classes[cn].body
            if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef))}

def bases(cn):
    out = []
    for b in classes[cn].bases:
        if isinstance(b, ast.Name) and b.id in classes:
            out.append(b.id)
    return out

def mro_up(cn):
    """all ancestors, breadth-first"""
    seen, q = [], list(bases(cn))
    while q:
        c = q.pop(0)
        if c in seen:
            continue
        seen.append(c)
        q.extend(bases(c))
    return seen

def sig(fn):
    a = fn.args
    pos = [p.arg for p in a.posonlyargs + a.args]
    kw  = [p.arg for p in a.kwonlyargs]
    nd  = len(a.defaults)
    required = pos[:len(pos)-nd] if nd else pos
    return pos, kw, required

TARGETS = ["StateCoordinateTransient", "SplitWallTransient"]

definers = collections.defaultdict(list)
for cn in classes:
    for m in methods(cn):
        definers[m].append(cn)

order = [c for c in classes]      # source order == ladder order here

print("=== AH: the two classes ===")
for t in TARGETS:
    ms = methods(t)
    print(f"{t}: {len(ms)} methods, bases={bases(t)}")

names = sorted(set(list(methods(TARGETS[0])) + list(methods(TARGETS[1]))))
print(f"\n=== {len(names)} distinct method names across the two classes ===")
print(f"{'name':34} {'definers':8} earliest            AH's job / substitutable?")
for nm in names:
    ds = [c for c in order if c in definers[nm]]
    job = []
    for t in TARGETS:
        if nm in methods(t):
            # is it an override of an ancestor's definition?
            anc = [a for a in mro_up(t) if nm in methods(a)]
            if anc:
                owner = anc[0]
                p1, k1, r1 = sig(methods(owner)[nm])
                p2, k2, r2 = sig(methods(t)[nm])
                if (p1, k1) == (p2, k2):
                    cls = "SAME"
                elif set(r1) == set(r2) and set(p1) <= set(p2) and set(k1) <= set(k2):
                    cls = "WIDENED"
                elif len(p1) == len(p2) and len(k1) == len(k2):
                    cls = "RENAMED?"
                else:
                    cls = "**INCOMPATIBLE**"
                job.append(f"{t[:4]}:SWAP<-{owner}[{cls}]")
            else:
                job.append(f"{t[:4]}:ADD")
    print(f"{nm:34} {len(ds):<8} {ds[0] if ds else '-':19} {' | '.join(job)}")
