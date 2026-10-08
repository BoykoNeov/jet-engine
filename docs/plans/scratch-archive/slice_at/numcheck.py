"""For every ledger row the pair tier settled, compare the distinctive NUMBERS in the Python test
body with those in the paired Rust body. Rows where the Python carries numbers the Rust body lacks
are listed for a hand-read."""
import ast, json, os, re, sys

ROOT = r"W:\Claude_projects\jet engine"
OUT = r"W:\temp\claude\slice_at"
led = json.load(open(os.path.join(OUT, "ledger_draft.json")))
import sys as _s
TIER = _s.argv[1] if len(_s.argv) > 1 else "pair"
pairs = [x for x in led if x["src"].startswith(TIER) and len(x["rust"].split(";")) == 1 and x["rust"]]
print("pair rows:", len(pairs))

COMMON = {0.0, 1.0, 2.0, 0.5, 1e-9, 1e-12}


def nums(text):
    out = set()
    for m in re.finditer(r"(?<![\w.])(\d[\d_]*(?:\.\d*)?(?:[eE][-+]?\d+)?)", text):
        try:
            out.add(float(m.group(1).replace("_", "")))
        except ValueError:
            pass
    return out


def py_body(f, t):
    src = open(os.path.join(ROOT, "tests", f + ".py"), encoding="utf-8").read()
    for n in ast.walk(ast.parse(src)):
        if isinstance(n, ast.FunctionDef) and n.name == t:
            body = ast.get_source_segment(src, n)
            doc = ast.get_docstring(n) or ""
            return body.replace(doc, "") if doc else body
    return ""


def rs_body(stem, fn):
    rs = open(os.path.join(ROOT, "rust", "tests", stem + ".rs"), encoding="utf-8").read().splitlines()
    for i, l in enumerate(rs):
        if re.match(r"\s*(?:#\[test\]\s*)?(pub\s+)?fn\s+" + re.escape(fn) + r"\b", l):
            depth, k, seen, out = 0, i, False, []
            while k < len(rs):
                line = rs[k].split("//")[0]
                out.append(line); depth += line.count("{") - line.count("}")
                if "{" in line: seen = True
                if seen and depth <= 0: break
                k += 1
            return "\n".join(out)
    return ""


flag = []
for x in pairs:
    stem, fn = x["rust"].split("::")
    pb, rb = py_body(x["py_file"], x["py_test"]), rs_body(stem, fn)
    pn = {v for v in nums(pb) if v not in COMMON}
    rn = nums(rb)
    missing = sorted(v for v in pn if v not in rn)
    status = "OK" if not missing else "CHECK"
    if missing:
        flag.append((x["py_file"], x["py_test"], fn, missing))
    print(f"{status:5} {x['py_file'][5:]}::{x['py_test'][5:]} -> {fn}  py_nums={len(pn)} missing={missing[:8]}")
print("rows to hand-read:", len(flag))
json.dump(flag, open(os.path.join(OUT, "numcheck_flag.json"), "w"), indent=0)
