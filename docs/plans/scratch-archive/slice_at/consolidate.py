"""Merge matcher tiers, header tables, proposals and manual decisions into ONE ledger draft.
Prints what is still UNRESOLVED."""
import os, re, json, collections, sys
sys.path.insert(0, r"W:\temp\claude\slice_at")
from manual import M
import manual2  # noqa: F401  (registers into M)
import manual3  # noqa: F401
import manual4  # noqa: F401

OUT = r"W:\temp\claude\slice_at"
rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(OUT, "match2.tsv"), encoding="utf-8")]
hdr, rows = rows[0], rows[1:]
H = {k: i for i, k in enumerate(hdr)}
table = json.load(open(os.path.join(OUT, "header_tables.json")))
grid = {}
for l in open(os.path.join(OUT, "grid.txt"), encoding="utf-8"):
    f, t, c, n, v = l.rstrip("\n").split("|", 4)
    grid[(f, t)] = v
prop = {}
for b in open(os.path.join(OUT, "proposals.txt"), encoding="utf-8").read().split("=== ")[1:]:
    m = re.match(r"(\S+)::(\S+)\s+\[cases (\d+), (\S+)\]\s+->\s+(\S+)\s+\(score ([\d.]+)\)", b)
    prop[(m.group(1), m.group(2))] = (m.group(5), float(m.group(6)))

STRONG = {"EXACT", "PREFIXED", "SUBSTR"}
NONRUNG = {"test_usage_blocks": ("RETIRED", "", "binds engine.py's Usage: docstrings, deleted with it (plan § 8.1 (v))"),
           "test_rust_line_citations": ("RETIRED", "", "guards engine.py:N citation DRIFT; a deleted file cannot drift. Citations resolve at the python-final tag (plan § 8.1 (v))")}
led = []
for r in rows:
    f, t, own, cases = r[H["file"]], r[H["name"]], r[H["own"]], int(r[H["cases"]])
    hits = [h for h in r[H["hit"]].split(";") if h]
    g = grid.get((f, t), "")
    if (f, t) in M:
        st, rust, note = M[(f, t)]
        rust = ";".join(x if "::" in x else f"{own}::{x}" for x in rust.split(";") if x)
        src = "manual"
    elif f in NONRUNG:
        st, rust, note = NONRUNG[f]; src = "policy"
    elif r[H["tier"]] in STRONG and ("SHARED" not in r[H["flags"]] or r[H["tier"]] == "EXACT"):
        rust = ";".join(hits)
        if cases > 1:
            st = {"SPLIT-1:1": "SPLIT", "LOOP-LITERALS-PRESENT": "LOOPED"}.get(g.split("(")[0], "LOOPED?")
        else:
            st = "PORTED" if len(hits) == 1 else "SPLIT"
        note, src = "", "name:" + r[H["tier"]]
    elif t in table and any(e[0] == own for e in table[t]):
        e = [e for e in table[t] if e[0] == own][0]
        rust = ";".join(e[2]); st = "PORTED" if len(e[2]) == 1 else "SPLIT"
        note, src = f"{own}.rs:{e[1]} header table", "table"
    elif (f, t) in prop and prop[(f, t)][0] != "None" and prop[(f, t)][1] >= 0.5:
        rust = f"{own}::{prop[(f, t)][0]}"; st = "PORTED" if cases == 1 else "LOOPED?"
        note, src = "", f"pair:{prop[(f, t)][1]}"
    else:
        st, rust, note, src = "UNRESOLVED", ";".join(hits), "", "-"
        if (f, t) in prop:
            note = f"proposal {prop[(f, t)][0]} ({prop[(f, t)][1]})"
    if f == "test_numeric_fingerprint" and re.match(r"test_golden_(fingerprint|kernel)_", t) and st == "PORTED":
        st, note = "RE-ANCHORED", "compared to the CPython golden under the module's tolerance; Rust's own anchor frozen beside it (slice AS)"
    led.append(dict(py_file=f, py_test=t, cases=cases, status=st, rust=rust, note=note, src=src, flags=r[H["flags"]], grid=g))

json.dump(led, open(os.path.join(OUT, "ledger_draft.json"), "w"), indent=0)
print(collections.Counter(x["status"] for x in led))
print(collections.Counter(x["src"].split(":")[0] for x in led))
for x in led:
    if x["status"] in ("UNRESOLVED", "LOOPED?") or (x["src"].startswith("name") and "SHARED" in x["flags"]):
        print(f"{x['status']:10} {x['py_file'][5:]}::{x['py_test'][5:]} c{x['cases']} | {x['rust'][:90]} | {x['note'][:80]} | {x['flags']}")


# ---------------------------------------------------------------------------
# EXPORT the committed ledger (rust/tests/coverage_ledger.tsv).
# ---------------------------------------------------------------------------
DEFAULT_NOTE = {
    "LOOPED": "one Rust fn loops the parametrize cases (every case literal present in its body)",
    "SPLIT": "one Rust fn per pytest case / arm",
}
dst = r"W:\Claude_projects\jet engine\rust\tests\coverage_ledger.tsv"
with open(dst, "w", encoding="utf-8", newline="\n") as fh:
    fh.write("py_file\tpy_test\tpy_cases\tstatus\trust\tnote\n")
    for x in led:
        note = x["note"] or DEFAULT_NOTE.get(x["status"], "")
        for v in (x["rust"], note):
            assert "\t" not in v and "\n" not in v, (x, v)
        fh.write(f"{x['py_file']}\t{x['py_test']}\t{x['cases']}\t{x['status']}\t{x['rust']}\t{note}\n")
print("wrote", dst, len(led), "rows,", sum(x["cases"] for x in led), "cases")
