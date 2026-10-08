"""grid.py — for every parametrized Python test, extract its literal parameter values and check
each one appears as a literal in the matched Rust fn body(ies). Unresolvable params -> HAND."""
import ast, os, re, json, collections

ROOT = r"W:\Claude_projects\jet engine"
OUT = r"W:\temp\claude\slice_at"
rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(OUT, "match2.tsv"), encoding="utf-8")]
hdr, rows = rows[0], rows[1:]
H = {k: i for i, k in enumerate(hdr)}


def rust_body(stem, fn):
    rs = open(os.path.join(ROOT, "rust", "tests", stem + ".rs"), encoding="utf-8").read().splitlines()
    for i, l in enumerate(rs):
        if re.match(r"\s*(pub\s+)?fn\s+" + re.escape(fn) + r"\b", l):
            depth, k, seen, out = 0, i, False, []
            while k < len(rs):
                out.append(rs[k]); depth += rs[k].count("{") - rs[k].count("}")
                if "{" in rs[k]: seen = True
                if seen and depth <= 0: break
                k += 1
            return "\n".join(out)
    return ""


def lit_forms(v):
    if isinstance(v, bool):
        return {str(v).lower()}
    if isinstance(v, (int, float)):
        f = float(v)
        forms = {repr(f), repr(f).rstrip("0").rstrip(".") if "." in repr(f) else repr(f)}
        if f == int(f):
            forms |= {str(int(f)) + ".0", str(int(f)) + "_000.0" if abs(f) >= 1000 and f % 1000 == 0 else str(int(f)),
                      f"{int(f):_}.0", str(int(f))}
        return forms
    if isinstance(v, str):
        return {v, v.lower()}
    return set()


def present(form, body):
    # a numeric literal must not be a substring of a longer number
    return re.search(r"(?<![\w.])" + re.escape(form) + r"(?![\d])", body) is not None


res = collections.Counter()
lines = []
for r in rows:
    if int(r[H["cases"]]) <= 1:
        continue
    pf, pt, hits = r[H["file"]], r[H["name"]], [h for h in r[H["hit"]].split(";") if "::" in h]
    tree = ast.parse(open(os.path.join(ROOT, "tests", pf + ".py"), encoding="utf-8").read())
    vals, opaque = [], []
    for n in ast.walk(tree):
        if isinstance(n, ast.FunctionDef) and n.name == pt:
            for d in n.decorator_list:
                if isinstance(d, ast.Call) and "parametrize" in ast.unparse(d.func):
                    try:
                        v = ast.literal_eval(d.args[1])
                        for x in v:
                            vals.extend(x if isinstance(x, (tuple, list)) else [x])
                    except Exception:
                        opaque.append(ast.unparse(d.args[1]))
    if not hits:
        verdict = "NO-RUST-MATCH"
    elif len(hits) == int(r[H["cases"]]):
        verdict = "SPLIT-1:1"
    else:
        body = "\n".join(rust_body(*h.split("::")) for h in hits)
        fl = set()
        for m in re.finditer(r"(?<![\w.])(\d[\d_]*(?:\.\d*)?(?:e[-+]?\d+)?)", body):
            try: fl.add(float(m.group(1).replace("_", "")))
            except ValueError: pass
        def ok(v):
            if isinstance(v, bool): return str(v).lower() in body
            if isinstance(v, (int, float)): return abs(float(v)) in fl
            if isinstance(v, str):
                if v in ("flow/press", "press/flow", "tilted", "steep", "flat-eta") and re.search(r"in (ALL_)?SHAPES|shapes\(\)", body):
                    return True
                return v.lower() in body.lower()
            return False
        # SHAPES-style opaque params: resolve a module-level dict literal's keys
        for o in list(opaque):
            m = re.fullmatch(r"list\((\w+)\)", o)
            if m:
                for n2 in tree.body:
                    if isinstance(n2, ast.Assign) and any(getattr(t, "id", None) == m.group(1) for t in n2.targets) and isinstance(n2.value, ast.Dict):
                        vals.extend(ast.literal_eval(k) for k in n2.value.keys)
                        opaque.remove(o)
        missing = [v for v in set(map(lambda x: x if not isinstance(x, list) else tuple(x), vals)) if not ok(v)]
        if opaque:
            verdict = "HAND(opaque:" + ",".join(opaque) + ")" + (f" missing={missing}" if missing else "")
        elif missing:
            verdict = f"HAND(missing={missing})"
        else:
            verdict = "LOOP-LITERALS-PRESENT"
    res[verdict.split("(")[0]] += 1
    lines.append(f"{pf}|{pt}|c{r[H['cases']]}|n_rust={len(hits)}|{verdict}")
open(os.path.join(OUT, "grid.txt"), "w", encoding="utf-8").write("\n".join(lines))
print(res)
for l in lines:
    if "LOOP-LITERALS" not in l and "SPLIT-1:1" not in l:
        print(l)
