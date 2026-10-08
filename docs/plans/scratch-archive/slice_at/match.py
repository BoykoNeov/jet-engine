"""Slice AT: auto-match every Python test function to its Rust port, by name tiers.

Writes W:/temp/claude/slice_at/match.tsv (one row per Python test function) and prints a census.
"""
import ast, os, re, sys, collections, json

ROOT = r"W:\Claude_projects\jet engine"
PYT = os.path.join(ROOT, "tests")
RST = os.path.join(ROOT, "rust", "tests")
OUT = r"W:\temp\claude\slice_at"

ALIAS = {"test_stations": "rung1", "test_polytropic": "rung2b", "test_variable_cp": "rung3",
         "test_reacting": "rung4", "test_forkb": "rung5", "test_validation": "rung1"}


def py_tests():
    rows = []
    for fn in sorted(os.listdir(PYT)):
        if not (fn.startswith("test_") and fn.endswith(".py")):
            continue
        mod = fn[:-3]
        src = open(os.path.join(PYT, fn), encoding="utf-8").read()
        tree = ast.parse(src)
        def visit(body, cls=None):
            for n in body:
                if isinstance(n, ast.ClassDef) and n.name.startswith("Test"):
                    visit(n.body, n.name)
                elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test_"):
                    marks = []
                    for d in n.decorator_list:
                        s = ast.unparse(d)
                        if "parametrize" in s: marks.append("param")
                        if "slow" in s: marks.append("slow")
                        if "skip" in s or "xfail" in s: marks.append("SKIP/XFAIL:" + s[:60])
                    rows.append(dict(file=mod, cls=cls or "", name=n.name, line=n.lineno, marks=",".join(marks)))
        visit(tree.body)
    return rows


def rust_tests():
    out = collections.defaultdict(list)   # stem -> [fn names]
    comments = {}
    for fn in sorted(os.listdir(RST)):
        if not fn.endswith(".rs"):
            continue
        src = open(os.path.join(RST, fn), encoding="utf-8").read()
        stem = fn[:-3]
        names = re.findall(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?fn\s+([A-Za-z0-9_]+)", src)
        out[stem] = names
        comments[stem] = src.lower()
    return out, comments


def norm(s):
    return re.sub(r"_+", "_", s.lower().removeprefix("test_")).strip("_")


GATE_PREFIX = re.compile(r"^(?:gate|g|p|t|test)\d+[a-z]?_|^(?:rung\d+[a-z]?_)")


def rust_keys(name):
    n = name.lower()
    keys = {n}
    m = GATE_PREFIX.sub("", n)
    keys.add(m)
    return keys


def main():
    pys = py_tests()
    rs, comments = rust_tests()
    all_rs = [(stem, n) for stem, ns in rs.items() for n in ns]
    print("python test fns:", len(pys), "rust #[test]:", len(all_rs))
    tiers = collections.Counter()
    rows = []
    for p in pys:
        own = ALIAS.get(p["file"], p["file"].removeprefix("test_"))
        pn = norm(p["name"])
        hit, tier = [], None
        # tier 1: exact (own file)
        for n in rs.get(own, []):
            if n.lower() == pn:
                hit.append(f"{own}::{n}"); tier = "EXACT"
        # tier 2: gate-prefixed / parametrize-split (own file)
        if not hit:
            for n in rs.get(own, []):
                ks = rust_keys(n)
                if pn in ks or any(k.startswith(pn + "_") for k in ks):
                    hit.append(f"{own}::{n}"); tier = "PREFIXED"
        # tier 3: same, elsewhere
        if not hit:
            for stem, n in all_rs:
                ks = rust_keys(n)
                if pn in ks or any(k.startswith(pn + "_") for k in ks):
                    hit.append(f"{stem}::{n}"); tier = "ELSEWHERE"
        # tier 3b: own-file rust fn whose name CONTAINS the python stem (a reworded port)
        if not hit:
            for n in rs.get(own, []):
                if pn in n.lower():
                    hit.append(f"{own}::{n}"); tier = "SUBSTR"
        # tier 4: python name cited in own rust file's text -> attribute to the next #[test] fn
        if not hit and own in comments and p["name"].lower() in comments[own]:
            t = comments[own]
            i = t.find(p["name"].lower())
            ctx = t[max(0, t.rfind("\n", 0, i - 200)):i + 400]
            if re.search(r"not (?:transcribed|ported)|deliberately|declared non|dropped|retired|vacu|"
                         r"compile error|unrepresentable|cannot be expressed|not a gate|omitted", ctx):
                tier = "DECLARED_NONPORT?"
            else:
                m = re.search(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([a-z0-9_]+)", t[i:])
                tier = "CITED_NEXT"
                if m: hit.append(f"{own}::{m.group(1)}")
        if not hit and tier is None and own in comments and pn in comments[own]:
            tier = "STEM_IN_TEXT"
        if not hit and tier is None:
            cited = [s for s, t in comments.items() if p["name"].lower() in t]
            if cited:
                tier = "CITED_ELSEWHERE"; hit = cited[:3]
        tier = tier or "UNMATCHED"
        tiers[tier] += 1
        rows.append({**p, "own": own, "own_exists": own in rs, "tier": tier, "hit": ";".join(hit[:4])})
    with open(os.path.join(OUT, "match.tsv"), "w", encoding="utf-8", newline="\n") as fh:
        keys = ["file", "cls", "name", "line", "marks", "own", "own_exists", "tier", "hit"]
        fh.write("\t".join(keys) + "\n")
        for r in rows:
            fh.write("\t".join(str(r[k]) for k in keys) + "\n")
    print(dict(tiers))
    by_file = collections.Counter(r["file"] for r in rows if r["tier"] in ("UNMATCHED", "CITED_OWN", "CITED_ELSEWHERE"))
    print("open by file:", by_file.most_common())
    # rust tests not claimed by any python row (own file only)
    claimed = set()
    for r in rows:
        for h in r["hit"].split(";"):
            if "::" in h: claimed.add(h)
    unclaimed = [f"{s}::{n}" for s, n in all_rs if f"{s}::{n}" not in claimed]
    json.dump(unclaimed, open(os.path.join(OUT, "rust_unclaimed.json"), "w"), indent=0)
    print("rust tests not claimed by a name match:", len(unclaimed))


main()
