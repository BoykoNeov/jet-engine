import os, re
T = r"W:\Claude_projects\jet engine\rust\tests"
alls = sorted(f[:-3] for f in os.listdir(T) if f.endswith(".rs"))
err = open(r"W:\temp\claude\phase8-ak\cargo_full.err", encoding="utf-8", errors="replace").read()
ran = re.findall(r"Running tests[\\/](\w+)\.rs", err)
print("started", len(ran), "of", len(alls), "| last started:", ran[-1])
rest = [t for t in alls if t not in ran[:-1]]
print("REMAINING", len(rest)); print(" ".join(rest))
open(r"W:\temp\claude\phase8-ak\remaining.txt", "w").write(" ".join("--test " + t for t in rest))
