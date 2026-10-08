# -*- coding: utf-8 -*-
"""PROBE 14 - the nest census WITHOUT the premise that broke probe 12.

Probe 12 censused calls to `_c_at` only, on the stated premise that it "is the only
method in the ladder that sets both fields from inside another method's live freeze".
THAT PREMISE WAS NEVER MEASURED AND IS FALSE: the runtime per-site counter found a
fifth nest through `_phi_at` (engine.py:20647), reached from `gauge_vs_device:20591`.

This writing derives the FREEZING-METHOD SET from the source instead of naming it, then
finds every call to any of them from inside a live block. It also records, per nest,
whether the callee receives the enclosing block's own (q, v) - because `_phi_at` does
NOT (it is handed `q + dq` / `q - dq`), which falsifies probe 12's stated reason even
though the conclusion survives on a different one.
"""
import ast, io, sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

E = r"W:/Claude_projects/jet engine/turbojet/engine.py"
SRC = open(E, encoding="utf-8").read()
TREE = ast.parse(SRC)
CLS = [(n.lineno, n.name, n.end_lineno) for n in TREE.body if isinstance(n, ast.ClassDef)]


def owner(ln):
    for a, nm, b in CLS:
        if a <= ln <= b:
            return nm
    return "?"


def state_set(stmt):
    """Return the assigned values if `stmt` sets the frozen-state pair, else None."""
    if not isinstance(stmt, ast.Assign) or len(stmt.targets) != 1:
        return None
    t = stmt.targets[0]
    if (isinstance(t, ast.Tuple) and len(t.elts) == 2
            and all(isinstance(e, ast.Attribute) for e in t.elts)
            and [e.attr for e in t.elts] == ["_b_state", "_v_state"]):
        return (tuple(ast.unparse(v) for v in stmt.value.elts)
                if isinstance(stmt.value, ast.Tuple) else ("?", "?"))
    return None


# --- STEP 1: DERIVE the set of methods that freeze in their own guard -----------------
FREEZERS = {}
for fn in ast.walk(TREE):
    if not isinstance(fn, ast.FunctionDef):
        continue
    sets = [s for s in ast.walk(fn) if state_set(s) not in (None,)
            and state_set(s) != ("None", "None")]
    clears = [s for s in ast.walk(fn) if state_set(s) == ("None", "None")]
    if sets and clears:
        # the guard's own params, so we can tell what the caller hands it
        FREEZERS[fn.name] = (fn.lineno, [a.arg for a in fn.args.args])

print("=== STEP 1 - methods that FREEZE and RESTORE in their own guard (DERIVED) ===")
for nm, (ln, args) in sorted(FREEZERS.items(), key=lambda kv: kv[1][0]):
    print(f"   engine.py:{ln:<7} {owner(ln)[:26]:<27}{nm}")
print(f"   -> {len(FREEZERS)} freezing methods. Probe 12 assumed ONE (`_c_at`).\n")


# --- STEP 2: every call to a freezer from inside a live block -------------------------
FOUND = []


def calls_in(node, open_set, fn):
    for c in ast.walk(node):
        if isinstance(c, ast.Call) and isinstance(c.func, ast.Attribute) \
           and c.func.attr in FREEZERS:
            FOUND.append(dict(line=c.lineno, callee=c.func.attr, fn=fn,
                              cls=owner(c.lineno), open_set=open_set,
                              args=[ast.unparse(a) for a in c.args]))


def run(body, open_set, fn):
    for st in body:
        s = state_set(st)
        if s is not None:
            open_set = None if s == ("None", "None") else (st.lineno, s)
            continue
        if isinstance(st, ast.Try):
            inner = run(st.body, open_set, fn)
            for h in st.handlers:
                run(h.body, inner, fn)
            run(st.orelse, inner, fn)
            cleared = any(state_set(x) == ("None", "None") for x in st.finalbody)
            run(st.finalbody, None if cleared else inner, fn)
            open_set = None if cleared else inner
            continue
        if isinstance(st, (ast.For, ast.While, ast.If, ast.With)):
            for f in ("test", "iter", "items"):
                if getattr(st, f, None) is not None:
                    calls_in(getattr(st, f), open_set, fn)
            after = run(st.body, open_set, fn)
            run(getattr(st, "orelse", []), open_set, fn)
            if isinstance(st, (ast.If, ast.With)):
                open_set = after
            continue
        if isinstance(st, ast.FunctionDef):
            run(st.body, None, st.name)
            continue
        calls_in(st, open_set, fn)
    return open_set


for fn in ast.walk(TREE):
    if isinstance(fn, ast.FunctionDef):
        run(fn.body, None, fn.name)

print("=== STEP 2 - every call to a freezer, with its enclosing freeze ===")
seen, nests = set(), []
for r in sorted(FOUND, key=lambda r: r["line"]):
    k = (r["line"], r["callee"])
    if k in seen:
        continue
    seen.add(k)
    o = r["open_set"]
    tail = tuple(r["args"][-2:]) if len(r["args"]) >= 2 else ()
    same = bool(o) and o[1] == tail
    if o:
        nests.append((r, same))
    print(f"  engine.py:{r['line']:<6} {r['cls'][:24]:<25}{r['fn'][:16]:<17}"
          f"-> {r['callee']:<12}{'NEST' if o else 'no nest':<9}"
          f"set@{o[0] if o else '-':<7}"
          f"{'SAME (q,v)' if same else ('DIFFERENT (q,v)' if o else '')}")

print(f"\n  NESTS: {len(nests)}")
print(f"  of which the callee receives the ENCLOSING (q, v): "
      f"{sum(1 for _, s in nests if s)}")
print(f"  of which it receives SOMETHING ELSE: {sum(1 for _, s in nests if not s)}")
for r, s in nests:
    if not s:
        print(f"     -> engine.py:{r['line']} {r['fn']} -> {r['callee']}"
              f"  args tail = {r['args'][-2:]}")
