import re, pathlib
R=pathlib.Path(r"W:\Claude_projects\jet engine")
m=(R/"main.py").read_text(encoding="utf-8")
src="\n".join(f.read_text(encoding="utf-8") for f in (R/"rust/src").glob("*.rs"))
def snake(n):
    n=n.lstrip("_"); return re.sub(r"(?<=[a-z0-9])([A-Z])", r"_\1", n).lower()
imports=re.findall(r"from turbojet\.\w+ import \(([^)]*)\)", m, re.S)
names=[]
for blk in imports:
    for x in blk.split(","):
        x=x.split("#")[0].strip()
        if x: names.append(x)
methods=sorted(set(re.findall(r"\.([a-z_][a-z0-9_]*)\(", m)))
skip=set("append format join items keys values get plot scatter annotate set_xlabel set_ylabel set_title legend grid tight_layout savefig subplots close use log exp sqrt isfinite index sort copy update replace split strip extend isclose floor ceil count pop upper lower ljust rjust center startswith endswith fabs hypot sum log10 tanh atan sin cos".split())
methods=[x for x in methods if x not in skip]
def status(n):
    if n[0].isupper() and not n.isupper():
        cands={n, n.replace("PDF","Pdf"), n.replace("PDF","Pdf").replace("NO","No")}
        for c in cands:
            if re.search(r"\b(struct|enum|type|trait)\s+%s\b"%re.escape(c), src): return "type"
        if re.search(r"\bfn\s+%s\b"%snake(n), src): return "fn"
        return "MISSING"
    s=snake(n); u=n.lstrip('_').upper()
    if re.search(r"\bpub\s+(const\s+)?fn\s+%s\b"%re.escape(s), src) or re.search(r"\bpub\s+const\s+%s\b"%re.escape(u), src): return "pub"
    if re.search(r"\bpub\([a-z]+\)\s+(const\s+)?fn\s+%s\b"%re.escape(s), src): return "pub(crate)"
    if re.search(r"\bfn\s+%s\b"%re.escape(s), src) or re.search(r"\bconst\s+%s\b"%re.escape(u), src): return "private"
    return "MISSING"
for label, xs in (("imported", names), ("methods", methods)):
    st={x:status(x) for x in xs}
    print(label, len(xs), {k:sum(1 for v in st.values() if v==k) for k in set(st.values())})
    print("  MISSING:", [x for x,v in st.items() if v=="MISSING"])
    print("  non-pub:", [(x,v) for x,v in st.items() if v in("private","pub(crate)")])
