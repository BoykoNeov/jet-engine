"""Slice AT matcher v2: cargo's own test list, pytest's own case counts, demoted weak tiers.

Outputs (W:/temp/claude/slice_at/):
  match2.tsv    one row per Python test function (1 203 expected)
  sheets/<file>.txt   per-file pairing sheet for every file with an OPEN row
"""
import ast, os, re, json, collections

ROOT = r"W:\Claude_projects\jet engine"
PYT = os.path.join(ROOT, "tests")
RST = os.path.join(ROOT, "rust", "tests")
OUT = r"W:\temp\claude\slice_at"

ALIAS = {"test_stations": "rung1", "test_polytropic": "rung2b", "test_variable_cp": "rung3",
         "test_reacting": "rung4", "test_forkb": "rung5", "test_validation": "rung1",
         "test_numeric_fingerprint": "fingerprint", "test_visuals_data": "visuals"}

per = json.load(open(os.path.join(OUT, "cargo_tests.json")))
rs = {k: [x.split("::")[-1] for x in v] for k, v in per.items() if os.path.exists(os.path.join(RST, k + ".rs"))}
src_lc = {k: open(os.path.join(RST, k + ".rs"), encoding="utf-8").read().lower() for k in rs}

# pytest case counts per function
cases = collections.Counter()
for l in open(os.path.join(OUT, "collect.txt"), encoding="utf-8"):
    l = l.strip()
    if "::" not in l:
        continue
    f, rest = l.split("::", 1)
    fn = rest.split("[")[0].split("::")[-1]
    cases[(os.path.basename(f)[:-3], fn)] += 1


def py_tests():
    rows = []
    for fn in sorted(os.listdir(PYT)):
        if not (fn.startswith("test_") and fn.endswith(".py")):
            continue
        mod = fn[:-3]
        tree = ast.parse(open(os.path.join(PYT, fn), encoding="utf-8").read())
        def visit(body, cls=None):
            for n in body:
                if isinstance(n, ast.ClassDef) and n.name.startswith("Test"):
                    visit(n.body, n.name)
                elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test_"):
                    marks = [m for d in n.decorator_list for m in ("param", "slow", "skip", "xfail")
                             if m in ast.unparse(d)]
                    rows.append(dict(file=mod, cls=cls or "", name=n.name, line=n.lineno,
                                     marks=",".join(sorted(set(marks))), cases=cases[(mod, n.name)]))
        visit(tree.body)
    return rows


def norm(s):
    return re.sub(r"_+", "_", s.lower().removeprefix("test_")).strip("_")


GP = re.compile(r"^(?:gate|g|p|t|test)\d+[a-z]?_|^rung\d+[a-z]?_")


def keys(n):
    n = n.lower()
    return {n, GP.sub("", n)}


def first_test_pos(t):
    m = re.search(r"#\[test\]", t)
    return m.start() if m else len(t)


pys = py_tests()
rows = []
for p in pys:
    own = ALIAS.get(p["file"], p["file"].removeprefix("test_"))
    pn = norm(p["name"])
    hit, tier = [], "UNMATCHED"
    for n in rs.get(own, []):
        if n.lower() == pn:
            hit.append(n); tier = "EXACT"
    if not hit:
        for n in rs.get(own, []):
            ks = keys(n)
            if pn in ks or any(k.startswith(pn + "_") for k in ks):
                hit.append(n); tier = "PREFIXED"
    if not hit:
        for n in rs.get(own, []):
            if pn in n.lower():
                hit.append(n); tier = "SUBSTR"
    if not hit and own in src_lc:
        t = src_lc[own]
        i = t.find(p["name"].lower())
        if i >= 0:
            tier = "CITED_HEADER" if i < first_test_pos(t) else "CITED_BODY"
    if tier == "UNMATCHED":
        for stem, ns in rs.items():
            for n in ns:
                ks = keys(n)
                if pn in ks or any(k.startswith(pn + "_") for k in ks):
                    hit.append(f"{stem}::{n}"); tier = "ELSEWHERE?"
    hit = [h if "::" in h else f"{own}::{h}" for h in hit]
    rows.append({**p, "own": own, "tier": tier, "hit": hit})

# ambiguity: rust fn claimed by >1 python row
claims = collections.Counter(h for r in rows for h in r["hit"])
for r in rows:
    flags = []
    if len(r["hit"]) > 1: flags.append("MULTI_RUST")
    if any(claims[h] > 1 for h in r["hit"]): flags.append("SHARED_RUST")
    if r["cases"] > 1: flags.append(f"GRID{r['cases']}")
    r["flags"] = ",".join(flags)

with open(os.path.join(OUT, "match2.tsv"), "w", encoding="utf-8", newline="\n") as fh:
    ks = ["file", "cls", "name", "line", "marks", "cases", "own", "tier", "flags", "hit"]
    fh.write("\t".join(ks) + "\n")
    for r in rows:
        fh.write("\t".join(";".join(r[k]) if k == "hit" else str(r[k]) for k in ks) + "\n")

STRONG = {"EXACT", "PREFIXED", "SUBSTR"}
print("python fns", len(rows), "collected cases", sum(r["cases"] for r in rows))
print(collections.Counter(r["tier"] for r in rows))
print("strong but flagged MULTI/SHARED:", sum(1 for r in rows if r["tier"] in STRONG and ("MULTI" in r["flags"] or "SHARED" in r["flags"])))
print("strong with grid>1:", sum(1 for r in rows if r["tier"] in STRONG and r["cases"] > 1))
open_rows = [r for r in rows if r["tier"] not in STRONG]
print("OPEN rows:", len(open_rows))
os.makedirs(os.path.join(OUT, "sheets"), exist_ok=True)
by = collections.defaultdict(list)
for r in open_rows:
    by[r["file"]].append(r)
for f, rr in sorted(by.items()):
    own = rr[0]["own"]
    unclaimed = [n for n in rs.get(own, []) if claims[f"{own}::{n}"] == 0]
    with open(os.path.join(OUT, "sheets", f + ".txt"), "w", encoding="utf-8") as fh:
        fh.write(f"# {f} -> {own}.rs   open={len(rr)} unclaimed_rust={len(unclaimed)}\n")
        for r in rr:
            fh.write(f"PY  {r['name']}  (line {r['line']}, cases {r['cases']}, {r['tier']}) {';'.join(r['hit'])}\n")
        for n in unclaimed:
            fh.write(f"RS  {n}\n")
print(sorted(((len(v), k) for k, v in by.items()), reverse=True))


# ---------------------------------------------------------------------------
# PROPOSALS for OPEN rows: fingerprint by kernel id; else greedy token-overlap
# pairing against the same file's UNCLAIMED rust fns. Each proposal carries both
# sides' first doc line so a hand-read can confirm or reject it.
# ---------------------------------------------------------------------------
STOP = {"test", "the", "a", "an", "is", "of", "to", "and", "at", "on", "in", "by", "its", "it", "not",
        "bit", "for", "gate", "contract", "with", "are", "be", "as", "rung"}


def toks(s):
    s = GP.sub("", s.lower().removeprefix("test_"))
    return {t for t in re.split(r"[_\d]+", s) if t and t not in STOP}


def py_doc(file, name):
    tree = ast.parse(open(os.path.join(PYT, file + ".py"), encoding="utf-8").read())
    for n in ast.walk(tree):
        if isinstance(n, ast.FunctionDef) and n.name == name:
            d = ast.get_docstring(n) or ""
            return " ".join(d.split())[:220]
    return ""


def rs_doc(own, fn):
    src = open(os.path.join(RST, own + ".rs"), encoding="utf-8").read()
    m = re.search(r"((?:[ \t]*///[^\n]*\n|[ \t]*#\[[^\n]*\n)*)[ \t]*fn\s+" + re.escape(fn) + r"\b", src)
    if not m:
        return ""
    d = " ".join(l.strip().lstrip("/").strip() for l in m.group(1).splitlines() if "///" in l)
    return d[:220]


prop = []
for f, rr in sorted(by.items()):
    own = rr[0]["own"]
    free = [n for n in rs.get(own, []) if claims[f"{own}::{n}"] == 0]
    if f == "test_numeric_fingerprint":
        for r in rr:
            m = re.match(r"test_golden_(?:fingerprint|kernel)_([A-Za-z0-9]+?)(?:_|$)", r["name"])
            k = f"k_{m.group(1).lower()}" if m else None
            if k in free:
                prop.append((r, k, 99)); free.remove(k)
            else:
                prop.append((r, None, 0))
        continue
    scored = sorted(((len(toks(r["name"]) & toks(n)) / max(1, len(toks(r["name"]) | toks(n))), i, n)
                     for i, r in enumerate(rr) for n in free), reverse=True)
    used_r, used_n, pick = set(), set(), {}
    for s, i, n in scored:
        if s <= 0 or i in used_r or n in used_n:
            continue
        used_r.add(i); used_n.add(n); pick[i] = (n, s)
    for i, r in enumerate(rr):
        n, s = pick.get(i, (None, 0))
        prop.append((r, n, round(s, 2)))

with open(os.path.join(OUT, "proposals.txt"), "w", encoding="utf-8") as fh:
    for r, n, s in prop:
        fh.write(f"=== {r['file']}::{r['name']}  [cases {r['cases']}, {r['tier']}]  ->  {n}  (score {s})\n")
        fh.write(f"  PY: {py_doc(r['file'], r['name'])}\n")
        if n:
            fh.write(f"  RS: {rs_doc(r['own'], n)}\n")
print("proposals", len(prop), "with a candidate", sum(1 for _, n, _ in prop if n),
      "score>=0.4", sum(1 for _, n, s in prop if n and s >= 0.4))


# ---------------------------------------------------------------------------
# HEADER TABLES: lines in a Rust test file naming a python `test_x` and a rust fn
# (`[`fn`]` link or `file.rs::fn`) on the SAME line — the porter's own mapping.
# ---------------------------------------------------------------------------
table = {}
for stem in rs:
    for ln, line in enumerate(open(os.path.join(RST, stem + ".rs"), encoding="utf-8")):
        pyn = re.findall(r"`(test_[A-Za-z0-9_]+)`", line)
        if len(pyn) != 1:
            continue
        tgt = re.findall(r"([a-z0-9_]+)\.rs::([A-Za-z0-9_]+)", line)
        links = re.findall(r"\[`([A-Za-z0-9_]+)`\]", line)
        out = [f"{s}::{f}" for s, f in tgt] + [f"{stem}::{f}" for f in links]
        out = [o for o in out if o.split("::")[1] in rs.get(o.split("::")[0], [])]
        if out:
            table.setdefault(pyn[0], []).append((stem, ln + 1, out, line.strip()[:160]))
json.dump(table, open(os.path.join(OUT, "header_tables.json"), "w"), indent=0)
n_open_resolved = sum(1 for r in open_rows if r["name"] in table)
print("header-table python names:", len(table), "open rows resolved by a table:", n_open_resolved)
