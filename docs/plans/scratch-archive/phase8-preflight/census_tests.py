import re, pathlib
R = pathlib.Path(r"W:\Claude_projects\jet engine")
py = {}
for f in sorted((R/"tests").glob("test_*.py")):
    t = f.read_text(encoding="utf-8")
    py[f.stem] = len(re.findall(r"^\s*def test_", t, re.M))
rs = {}
for f in sorted((R/"rust/tests").glob("*.rs")):
    t = f.read_text(encoding="utf-8")
    rs[f.stem] = len(re.findall(r"#\[test\]", t))
print("py files", len(py), "def test_", sum(py.values()))
print("rs files", len(rs), "#[test]", sum(rs.values()))
alias = {"test_stations":"rung1","test_polytropic":"rung2b","test_variable_cp":"rung3","test_reacting":"rung4","test_forkb":"rung5"}
for k, n in py.items():
    m = re.match(r"test_rung(\d+b?)$", k)
    tgt = alias.get(k) or (("rung"+m.group(1)) if m else None)
    r = rs.get(tgt) if tgt else None
    flag = "" if (r is not None and r >= n) else "  <--"
    print(f"{k:38s} py={n:3d}  {tgt or '-':10s} rs={r}{flag}")
