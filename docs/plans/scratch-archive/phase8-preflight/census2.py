import re, pathlib, collections
R = pathlib.Path(r"W:\Claude_projects\jet engine")
coll = collections.Counter()
for line in open(r"W:\temp\claude\phase8-preflight\collect.txt", encoding="utf-8"):
    if "::" in line:
        coll[line.split("::")[0].split("/")[-1][:-3]] += 1
rs = {}
for f in sorted((R/"rust/tests").glob("*.rs")):
    rs[f.stem] = len(re.findall(r"#\[test\]", f.read_text(encoding="utf-8")))
alias = {"test_stations":"rung1","test_polytropic":"rung2b","test_variable_cp":"rung3","test_reacting":"rung4","test_forkb":"rung5"}
short = []
tot_py = tot_rs = 0
for k, n in sorted(coll.items()):
    m = re.match(r"test_rung(\d+b?)$", k)
    tgt = alias.get(k) or (("rung"+m.group(1)) if m else None)
    r = rs.get(tgt)
    if tgt: tot_py += n; tot_rs += r or 0
    if r is None or r < n:
        short.append((k, n, tgt, r))
print("collected total", sum(coll.values()), "| rung-mapped py", tot_py, "rs", tot_rs)
for s in short: print("SHORT", s)
